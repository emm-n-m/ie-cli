# Spec: Add-Item-To-Save (CRE inventory write inside a GAM)

Status: **proposed** — first write feature for saved games. 2026-07-04.

Behavioral reference is Near Infinity + IESDP, same as every other format here.
Quality/structure bar: `crates/ie-formats/src/are.rs` (parsing) and the existing
scalar writer `patch_cre_scalars` in `crates/ie-formats/src/cre.rs` (write pattern).

---

## 1. Goal

Add a single item to a party member's inventory in an existing saved game, so a
player who lost a quest item (motivating case: the **Modron Cube**, `CUBE.ITM`, in
PSTEE) can restore it without an external editor.

This is a **variable-length structured write** — the thing README:53 lists as
not-yet-implemented and AGENTS.md:359 defers until reads are trustworthy. That
prerequisite is now met for the target formats (see §2), so this is a deliberate,
tightly-scoped first write feature, not a general writeback layer.

## 2. Prerequisites already in place (do not rebuild)

- **GAM read** — `parse_gam` (`crates/ie-formats/src/save.rs`) decodes GAM
  `V2.0/2.1/2.2`, including per-party-member `embedded_cre_offset` /
  `embedded_cre_size` and all header section offsets. Verified against the real
  target save: PSTEE (Steam "Project P") saves are **GAM `V2.0`**, header `0xB4`,
  NPC struct `0x160` — already handled. (Classic-PST GAM `V1.1` is *not* handled and
  is **out of scope**; the EE build does not use it.)
- **CRE read** — `parse_cre_with_variant` (`crates/ie-formats/src/cre.rs`) decodes CRE
  `V1.0` (header `0x2D4`), including the item table (`parse_items`, 20-byte entries)
  and the slot table (`parse_item_slots`). Item/slot JSON: `CreatureItemJson`,
  `CreatureItemSlotJson`.
- **Scalar CRE write** — `patch_cre_scalars` + `ByteEdit` (`cre.rs:322`) is the
  established fixed-offset write pattern (copy input, splice edits, byte-exact
  elsewhere). Reuse its style and error type shape (`CreaturePatchError`).
- **Save discovery / read** — `list_saves`, `resolve_save_folder`, `read_save_member`
  (`crates/ie-io/src/lib.rs`); saves resolve case-insensitively and honor `--saves-dir`.

Neither GAM V2.0 nor CRE V1.0 stores a file-length or checksum field in scope here, so
no checksum recompute is needed — the only invariant to maintain is **internal offset
correctness** after growing a section.

## 3. Scope

**In (v1):**
- Add **one** item (a 20-byte CRE item entry) to **one** party member's inventory in a
  save's `BALDUR.gam`.
- Target member selectable; default = first party member (the protagonist).
- Assign the new item to a specified or auto-selected **empty general-inventory slot**.
- Item fields settable: resref (required), charges 1/2/3, flags (default: identified),
  expiration (default 0). No stacking/merge with an existing entry.
- Non-destructive by default (write a copy); explicit `--in-place` makes a `.bak` first.

**Out (v1) — reject cleanly with a clear message:**
- Removing or editing existing items; stacking onto an existing entry.
- Non-party (stored) CREs, CREs inside `BALDUR.SAV` (zlib archive), container/store
  inventories.
- Standalone `.CRE` file editing *may* fall out for free (see §4.1) — expose it only if
  trivial; otherwise defer.
- GAM `V1.1` (classic PST), GAM versions other than V2.x.
- Party gold, journal, globals, or any non-inventory edit.

## 4. Algorithm

Two nested variable-length inserts sharing one rule:
**pick an insertion point, splice N bytes, then add N to every stored offset that
points at or past the insertion point.** N = 20 (`CRE_ITEM_SIZE`).

### 4.1 Core: `add_item_to_cre(cre: &[u8], item: NewItem, slot: SlotChoice) -> Result<Vec<u8>>`

Pure function on CRE bytes; no I/O. Steps:

1. Validate: `CRE ` signature, `len >= CRE_HEADER_SIZE` (0x2D4). Read header offsets:
   `known_spells_offset` (0x02A4-ish — read the exact offsets from `parse_cre_with_variant`,
   do not hardcode blindly), `spell_memorization_offset`, `memorized_spells_offset`,
   `item_slots_offset` (0x02B8), `items_offset` (0x02BC), `items_count` (0x02C0),
   `effects_offset`.
2. `insertion = items_offset + items_count * 20`. This is the end of the items block.
3. Build the 20-byte entry: resref[8] + expiration u16 + charges1 u16 + charges2 u16 +
   charges3 u16 + flags u32 (little-endian), matching `parse_items` field order exactly.
4. Splice the 20 bytes into the buffer at `insertion`.
5. `items_count += 1` (write back at 0x02C0).
6. For **every** section offset in the CRE header whose value `>= insertion`, add 20.
   Compute from actual values — do **not** assume section order. In practice
   `item_slots_offset` sits after the items block and gets +20; `effects_offset` and the
   spell sections precede it and do not. (If a real CRE ever has slots before items, the
   value-based rule still does the right thing.)
7. Resolve the slot (§4.3), then write the new item's index (`old items_count`) as a
   u16 into the chosen slot cell inside the (now shifted) item-slots table.
8. Return the new bytes.

Because this is a pure `&[u8] -> Vec<u8>` transform, the same function edits a standalone
`.CRE`; the save path just wraps it (§4.2).

### 4.2 Container: `add_item_to_save_gam(gam: &[u8], member: MemberSel, item, slot) -> Result<Vec<u8>>`

1. `parse_gam` to locate the target member's NPC struct and its
   `embedded_cre_offset` (E) + `embedded_cre_size` (S).
2. Extract `cre = gam[E .. E+S]`; call `add_item_to_cre`. New size `S' = S + 20`.
3. Splice 20 bytes into `gam` at `E + S` (end of that CRE blob) — i.e. replace the CRE
   region with the returned bytes.
4. Write the member's `embedded_cre_size = S'` back into its NPC struct.
5. `gam_insertion = E + S`. For **every** offset stored in the GAM that is at or after
   `gam_insertion`,
   add 20:
   - Header section offsets, including the fields currently exposed by `parse_gam` and
     additional V2.x section pointers such as party inventory (`0x28`) and the PSTEE tail
     offsets observed at `0x68`, `0x6C`, and `0x78`.
   - **Every** NPC struct's `embedded_cre_offset` (party *and* non-party) whose value
     is at or after `gam_insertion` — the CRE blobs of later members shift down.
   - The edited member's own `embedded_cre_offset` (= E) is `<= gam_insertion`, so it does
     **not** change; only its size did.
6. Return new GAM bytes.

The offset-shift set is the crux; get it wrong and the save loads with a corrupted party,
globals, or PSTEE tail section. Enumerate every known GAM V2 header offset field and decide
inclusion by the value comparison — a table-driven pass, not ad hoc. Do not limit the writer
to fields currently surfaced in the JSON reader.

### 4.3 Slot assignment — the one real judgment call

Item slots are a fixed array of u16 indices into the items table; `0xFFFF` = empty
(`parse_item_slots`, `cre.rs:940`). **Slot index carries fixed semantic meaning**
(helmet, armor, weapon slots, quiver, rings, then general-inventory cells, then the two
trailing `selected_weapon` / `selected_weapon_ability` markers). Dropping a quest item
into a weapon or equipment slot is wrong.

Requirements:
- `SlotChoice::Index(n)` — use exactly slot `n`; error if it is not currently empty.
- `SlotChoice::AutoInventory` (default) — pick the first empty slot that is a
  **general-inventory** cell: exclude equipment slots and the trailing two marker slots
  (`slot_name` already flags `selected_weapon*`, `cre.rs:978`).
- The general-inventory slot **range differs by game/variant** (PST has a different slot
  layout and count than BG). Add a small per-`GameVariant` slot-map (inventory-cell index
  range) rather than guessing. Populate PST first from IESDP/Near Infinity and validate
  against a real PST CRE; leave BG/IWD as a documented follow-up if not verified.
- If no empty inventory slot exists, error clearly (inventory full) — do **not** silently
  overwrite.

## 5. CLI surface

New subcommand (mirror `save-info` arg style; separate command, matching project idiom):

```
iecli save-add-item \
  --game <GAME> --save <SELECTOR> [--saves-dir <DIR>] \
  --item <RESREF>                # e.g. CUBE  (the .ITM resref)
  [--member <INDEX|CRE_RESREF>]  # default: first party member (protagonist)
  [--slot <auto|INDEX>]          # default: auto (first empty inventory cell)
  [--charges <N>] [--charges2 <N>] [--charges3 <N>]
  [--flags <identified|none|RAW_U32>]   # default: identified
  [--in-place | --output <DIR>]  # default: --output required (non-destructive)
  [--backup]                     # with --in-place: write <save>.bak first (default on)
```

Behavior:
- Default is non-destructive: copy the entire save **folder** to `--output` and edit the
  copy's `BALDUR.gam` (a save is a folder of `BALDUR.gam` + `BALDUR.SAV` + portraits;
  editing only the GAM in a folder-copy keeps the save loadable).
- `--in-place` edits the real save folder's `BALDUR.gam`, writing `BALDUR.gam.bak` first
  unless `--backup=false`.
- Optionally validate `--item` resolves in the install (locate `<resref>.ITM`) and warn
  if not — but allow it (mods/edge cases).
- Print a summary: member, slot chosen, item resref/name (resolve via TLK), new
  `items_count`, byte delta.

## 6. Safety & correctness gates (mandatory)

1. **Backup**: never mutate the only copy of a save without a `.bak` (or folder copy).
2. **Round-trip assertion** (in code, before writing output): re-parse the edited GAM,
   then the edited embedded CRE; assert (a) the new item appears at the expected slot with
   the requested fields, (b) `items_count` incremented by exactly 1, (c) every *other*
   party member's CRE re-parses cleanly and its item list is byte-identical to before,
   (d) globals/journal counts unchanged. Fail the write if any assertion trips.
3. **Byte-exactness elsewhere**: outside the spliced 20 bytes and the adjusted offset
   fields, output must be byte-identical to input (same discipline as `patch_cre_scalars`).
4. **In-game validation**: the human loads the edited save in PSTEE and confirms the
   Modron Cube is in inventory and the party/areas are intact. This is the acceptance test
   the automated ones cannot fully replace.

## 7. Testing plan

Follow the ARE/BCS env-gated pattern (`IE_GAME_PATH`, plus a PST-specific gate).

- **Unit (pure, committed fixtures)**: hand-built minimal CRE with a known items/slots
  layout → `add_item_to_cre` → assert offsets shifted, item present, slot set, bytes
  outside the edit unchanged. Include a case where items block is followed by the slot
  table (normal) and a synthetic case exercising the `>= insertion` boundary.
- **GAM container unit**: minimal 2-member GAM fixture → add item to member 0 → assert
  member 1's `embedded_cre_offset` shifted by 20, header offsets shifted, member 0 size
  +20, both CREs re-parse.
- **Env-gated real-install regression** (`IE_GAME_PATH` = PSTEE + saves dir): copy a real
  save to a temp dir, add `CUBE` to the protagonist, re-parse, assert the Cube is present
  and all six party members re-parse. Use the real motivating save (`…-Quick-Save-4`) as
  the reference. Gate + temp-dir isolation like the existing `save.rs`/`tlk_append` tests.
- **Near Infinity cross-check**: open the edited save's CRE in NI, confirm the item and
  slot render correctly and NI reports no structural error. Record in the PR.

## 8. Risks / edge cases to handle explicitly

- **Wrong offset in the shift set** → corrupted save. Mitigation: table-driven offset
  pass + round-trip re-parse gate (§6.2). Highest-risk item; review carefully.
- **Slot semantics per variant** (§4.3) — the main judgment call; validate PST slot range
  against a real CRE before trusting `AutoInventory`.
- **Two-handed / equipped-weapon marker slots** — never auto-target them.
- **Item not in install** (typo'd resref) — warn, don't hard-fail (mods).
- **Save is a folder, not a file** — operate on `BALDUR.gam` within it; keep `BALDUR.SAV`
  and portraits untouched so the save stays loadable.
- **Endianness**: all IE integers are little-endian; reuse existing `parse_u16/u32` and
  `to_le_bytes` helpers.
- **Do not** touch the `BALDUR.SAV` zlib archive — inventory lives in `BALDUR.gam`.

## 9. Deliverables

1. `add_item_to_cre` + `add_item_to_save_gam` in `crates/ie-formats/src/save.rs` (or a new
   `save_write.rs`), with `NewItem` / `SlotChoice` / `MemberSel` types and a typed error.
2. Per-`GameVariant` inventory-slot map (PST populated + validated).
3. `save-add-item` CLI subcommand in `crates/ie-cli/src/main.rs`, wired like `patch`.
4. Tests per §7. `docs/SAVE_SUPPORT_TODO.md` updated to mark item-write as delivered and
   move slot-map coverage for BG/IWD to follow-ups.
5. Short decision note under `docs/decisions/2026-07-04-save-item-write.md`.

---

## 10. Ready-to-paste coding-agent prompt

```text
Implement scoped write support: add a single item to a party member's inventory in a
saved game's BALDUR.gam. Read docs/SPEC_SAVE_ITEM_WRITE.md first — it is authoritative.

Scope (v1): GAM V2.0 container + embedded CRE V1.0, PSTEE validated. Add one 20-byte CRE
item entry, assign it to an empty general-inventory slot, fix up all CRE-internal and
GAM-container offsets. Non-destructive by default; --in-place makes a .bak.

Reuse, do not rebuild:
- parse_gam / parse_cre_with_variant for reads (crates/ie-formats/src/save.rs, cre.rs)
- patch_cre_scalars + ByteEdit as the write-style reference (cre.rs:322)
- list_saves / read_save_member / --saves-dir plumbing (crates/ie-io/src/lib.rs)

Core rule (both nesting levels): choose insertion point, splice 20 bytes, add 20 to every
stored offset whose value points at/after the insertion point. Decide inclusion by value
comparison, never by assumed section order.

Constraints:
- byte-exact outside the spliced bytes and adjusted offsets
- never mutate a save without a backup
- reject out-of-scope inputs (V1.1 GAM, stored/SAV CREs, stacking, item removal) clearly
- per-GameVariant inventory-slot map; validate the PST inventory range against a real CRE
  before trusting slot auto-selection — never auto-target equipment or selected_weapon slots

Validation (all required):
- pure unit tests on hand-built CRE + 2-member GAM fixtures (offset shifts, byte-exactness,
  boundary cases)
- IE_GAME_PATH-gated real-install test: copy a real PSTEE save, add CUBE to the protagonist,
  re-parse, assert the Modron Cube is present and all party members re-parse
- in-code round-trip assertion gating every write (spec §6.2)
- cross-check one edited CRE in Near Infinity; note results in the PR

Acceptance: on the real PSTEE save (…-Quick-Save-4), `iecli save-add-item --item CUBE`
produces a save that loads in-game with the Modron Cube in inventory and party/areas intact.
```
