# Spec: Save Item-Write — Complete (BG / IWD / PST)

Status: **proposed.** 2026-07-04. Extends `docs/SPEC_SAVE_ITEM_WRITE.md`, which
specified and delivered the **PSTEE** slice (commit `8950c9c4`). This spec is about
making `save-add-item` correct and safe for the **whole Infinity Engine EE family**
(BGEE, BG2EE, IWDEE, PSTEE), not just PST.

Behavioral reference: Near Infinity (its `GamResource` / `CreResource` field maps are
the definitive layout source) + IESDP GAM/CRE pages. Quality bar: `are.rs`.

---

## 0. Read this first: why PST "working" does NOT mean the others work

The delivered code runs on any GAM `V2.0/2.1/2.2` save (`parse_gam` accepts all three),
but its correctness is **PST-specific in two places**, and running it elsewhere will
silently corrupt saves:

1. **`GAM_V2_HEADER_OFFSET_FIELDS = [0x20,0x28,0x30,0x38,0x50,0x68,0x6C,0x78]`** was
   reverse-engineered from one PSTEE save. `0x68/0x6C/0x78` are PST-specific section
   pointers. In BG2EE/BGEE that header region is the **familiar block** (resrefs + counts),
   not offsets — the fix-up would treat familiar bytes as offsets and shift them, and
   would likely miss BG/IWD's *real* section pointers. Same silent-corruption class we hit
   with PST's non-party table, in reverse.
2. **Inventory slot map** (`inventory_slot_range`) is PST-only by construction; `Standard`
   returns an error. PST's 57-slot layout (with tattoo slots) does not match BG/IWD.

There is currently **no hard gate**: `add_item_to_save_gam` accepts any variant, and the
explicit `--slot N` path bypasses the PST-only auto-slot check. A BGEE user passing
`--slot 30` today would hit the PST-tuned offset fix-up and corrupt the save. **First
deliverable is the safety gate (§4); everything else lifts it per-game as validated.**

## 1. Goal

`iecli save-add-item` adds one item to a chosen party member's embedded CRE inventory in
`BALDUR.gam`, correct and byte-safe, for **every** supported EE game — with per-game GAM
header layouts and inventory slot maps that are validated against authoritative sources
and real saves, never guessed.

## 2. What is game-agnostic vs game-specific

| Concern | Scope | Notes |
|---|---|---|
| CRE V1.0 item entry (20 bytes), append + item-table shift | **Agnostic** | Same across BG/IWD/PST; keep as-is |
| CRE header section-offset set (0x2A0/2A8/2B0/2B8/2C4) | **Agnostic** | CRE V1.0 header is uniform |
| Offset-shift invariant (splice N, shift all offsets ≥ insertion) | **Agnostic** | Proven; keep |
| Read NPC table location from *post-shift* buffer | **Agnostic** | The non-party crash fix; keep and generalize |
| Re-parse every embedded CRE after write | **Agnostic** | Runtime guard; keep and extend (§6) |
| **GAM header section-offset field set** | **Per game+version** | The high-risk item (§3) |
| **Inventory slot map** (backpack vs equipment/quiver) | **Per game** | §5 |
| GAM version support | V2.0/2.1/2.2 in; V1.1 (classic) out | as today |

## 3. Variant-aware GAM header layout (the crux)

Replace the single hardcoded array with a per-variant descriptor selected by
`(gam_version, game_family)`:

```rust
struct GamLayout {
    /// Header dword offsets that hold ABSOLUTE file offsets to sections and must be
    /// shifted when a section grows. Authoritative per game+version.
    section_offset_fields: &'static [usize],
    /// Inventory (backpack) slot index ranges safe for auto-placement.
    inventory_slots: &'static [Range<usize>],
}
```

### 3.1 `GameVariant` is too coarse

Today `GameVariant = { Standard, Pst }`. BG and IWD are both `Standard`, but must be
confirmed to share a GAM header layout before they share a descriptor. Expand the enum to
at least `{ Bg, Iwd, Pst }` (keep `Standard`→`Bg` default or make detection explicit), or
key the descriptor off `(version_string, family)` where family is derived in
`detect_game_variant` (`crates/ie-io/src/lib.rs`) from stable root files
(`baldur.exe`/`icewind.exe`/`torment.lua`, chitin contents). **Decide and document**
whether BGEE, BG2EE, and BGEE:SoD (V2.0 vs V2.1) share one descriptor or need splitting.

### 3.2 How to derive each `section_offset_fields` list — do NOT guess

For each (game, version), the list MUST be produced by this procedure, not by pattern-
matching one save:

1. **Authoritative field map.** Read Near Infinity's `GamResource` for that version and
   the IESDP GAM v2.0/2.1/2.2 page. Enumerate every field documented as an *offset to* a
   section (party NPC, party inventory, non-party NPC, globals, journal, familiar, and any
   version/PST-specific sections). That set — and only that set — goes in the list.
2. **Confirm on ≥2 real saves per game.** For each listed field, verify its value is either
   0, `0xFFFFFFFF` (sentinel/none), or a valid in-file offset pointing at plausible section
   data. For every header dword NOT listed, verify it is never a section offset (counts,
   resrefs, reputation, familiar block, etc.).
3. **Grow-and-reparse proof.** After a test insert, walk EVERY section from its shifted
   offset and confirm it parses (§6). A wrong/missing entry surfaces here, not in prod.
4. **In-game load test** on a real save of that game.

Record the derived tables and their validation in `docs/decisions/`.

### 3.3 Invariants that stay (already correct)

- Insertion rule, at both CRE and GAM levels: splice 20 bytes, add 20 to every listed
  offset whose value is `>= insertion`.
- **Sentinel skip:** never shift a field whose value is `0xFFFFFFFF` (none) or `0`
  (absent). `checked_add` must remain the fail-safe backstop.
- **Read NPC table base offsets from the post-shift `output` buffer**, never the pre-edit
  bytes — a table located after the insertion has already moved (the non-party crash).
  This applies to any per-variant section table walked for internal-offset fixups too.

### 3.4 Sections with *internal* offsets

Party/non-party NPC structs carry `embedded_cre_offset` (handled). Verify per game whether
any variant-specific section (PST modron/maze, BG familiar block, party-inventory stash)
stores *internal* absolute offsets that also need shifting. If so, walk and fix them the
same way (post-shift buffer, value ≥ insertion). Journal entries and GLOBAL vars are
self-contained (strrefs/inline values) — no internal offsets.

## 4. Safety gate (first deliverable, ships before BG/IWD support)

In `add_item_to_save_gam`, hard-refuse any `(version, variant)` without a validated
`GamLayout`:

```
Err(InvalidWrite("save item write is validated for PSTEE only; \
BG/IWD support is not yet verified (see SPEC_SAVE_ITEM_WRITE_COMPLETE.md)"))
```

This must gate the **whole** write, not just auto-slot — closing the `--slot N` footgun.
Lift per-game as each `GamLayout` passes §3.2 + §6 + in-game test.

## 5. Per-game inventory slot maps

Slot index → meaning is fixed per game. Auto-placement must target only general-inventory
(backpack) cells; never equipment, weapon, quiver, quick-item, or the trailing
`selected_weapon*` markers.

- **PST (fix existing):** current range `20..40` is too narrow — real backpack cells run
  ~20..54 on a real protagonist CRE (empties observed at 46–54; markers at 55–56). Widen to
  the validated PST backpack range; keep excluding the last two marker slots.
- **BG (BGEE/BG2EE):** derive from NI/IESDP CRE slot map (~40 slots: equipped 0–1x, quick
  weapons, quiver, quick items, rings/amulet/etc., then inventory cells). Populate the
  inventory-cell range(s) and validate against a real BG CRE.
- **IWD (IWDEE):** same procedure; confirm whether it matches BG or differs.

Source each map from Near Infinity's slot definitions for that game and confirm against a
real member CRE (dump `item_slots`, check which indices hold backpack items vs equipment).
`--slot INDEX` stays available and is validated only for emptiness + bounds (game-agnostic).

## 6. Validation — strengthen the runtime guard to full structural re-walk

The current guard re-parses every embedded CRE (party + non-party). Extend it, per write,
to re-walk **every** section the game reads from its post-edit offset and confirm it
parses / has the expected size:

- party NPCs + their CREs (done), non-party NPCs + their CREs (done)
- GLOBAL variables (count × 0x54 fits within file)
- journal entries (count × entry-size fits)
- party inventory section if present (BG)
- each variant-specific section pointer resolves to in-bounds data
- **byte-exactness:** everything outside the spliced 20 bytes and the declared shifted
  fields is identical to input (promote the test-only `assert_byte_exact_after_insert`
  idea into a runtime debug-assert or a `--verify` path).

A wrong offset list then fails the write loudly instead of shipping a broken save — the
lesson from the non-party bug.

## 7. Testing matrix

Per game (PST already done; add BG, IWD):

- **Unit:** hand-built GAM fixture with party + non-party CREs + that game's
  variant-specific sections, asserting each listed offset shifts and byte-exactness holds.
- **Env-gated real-install** (`IE_GAME_PATH` per game): copy a real save, add a known item,
  full structural re-walk (§6), assert item present + all sections parse.
- **Near Infinity cross-check:** open the edited CRE/GAM, confirm no structural error.
- **In-game load** on a real save of that game — the acceptance test.

Keep the existing PST regression test (non-party creature after insertion) as the template.

## 8. Scope

**In:** BGEE, BG2EE, IWDEE, PSTEE; GAM V2.0/2.1/2.2 + embedded CRE V1.0; one item to one
party member; auto backpack slot per validated game map or explicit `--slot`.

**Out (unchanged):** classic GAM V1.1; stored/SAV CREs; container/store inventories; item
removal/edit/stacking; shared party-inventory stash as a *target* (may be a shift concern
per §3.4, but not an add target in v1); non-inventory edits.

## 9. Deliverables

1. Safety gate (§4) — first, standalone, with a test.
2. `GamLayout` descriptor + per-`(version,family)` selection; `GameVariant`/detection
   expanded as needed (§3.1).
3. Validated `section_offset_fields` for BG and IWD (§3.2) + decision notes with sources.
4. Per-game inventory slot maps (§5), PST range widened.
5. Full structural re-walk validation (§6).
6. Per-game tests (§7).
7. Update `docs/SAVE_SUPPORT_TODO.md` and supersede the scope note in
   `docs/SPEC_SAVE_ITEM_WRITE.md`.

---

## 10. Ready-to-paste coding-agent prompt

```text
Extend save-add-item from PSTEE-only to the full EE family (BGEE, BG2EE, IWDEE, PSTEE).
Read docs/SPEC_SAVE_ITEM_WRITE_COMPLETE.md first — it is authoritative — and note the
delivered PST slice in commit 8950c9c4 as the validated baseline.

Order of work:
1. Safety gate FIRST: add_item_to_save_gam must hard-refuse any (GAM version, game family)
   that lacks a validated layout — gating the whole write, not just auto-slot. Ship + test.
2. Replace GAM_V2_HEADER_OFFSET_FIELDS with a per-variant GamLayout {section_offset_fields,
   inventory_slots}, selected by (gam_version, game_family). Expand GameVariant/detection if
   BG and IWD prove to differ.
3. Derive each game's section_offset_fields from Near Infinity's GamResource + IESDP — NOT
   by pattern-matching one save. Confirm on >=2 real saves, prove with a grow-and-reparse
   structural re-walk, and load-test in-game. Record tables + sources in docs/decisions/.
4. Per-game inventory slot maps from NI slot definitions, validated against a real member
   CRE. Widen the too-narrow PST range (20..40 -> validated backpack range).
5. Strengthen the runtime guard to re-walk EVERY section from its shifted offset (not just
   CREs) + byte-exactness, so a wrong offset list fails loudly.

Keep these proven invariants: splice-and-shift-offsets>=insertion at CRE and GAM levels;
read NPC table base offsets from the post-shift buffer (the non-party crash fix); skip
0xFFFFFFFF/0 sentinels; checked_add as backstop.

Validation per game: unit fixture (party + non-party + variant sections), IE_GAME_PATH-
gated real-save test with full structural re-walk, Near Infinity cross-check, in-game load.
Lift the safety gate for a game only after all four pass. Do not touch BALDUR.SAV.
```
