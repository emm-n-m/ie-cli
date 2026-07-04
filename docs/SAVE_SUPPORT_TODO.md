# Save File Support — Status & Next Steps

Status: **read support implemented for the BG family and validated against real BG2EE saves;
scoped PSTEE item write implemented for embedded party CRE inventories.**
`iecli save-list` and `iecli save-info` decode `GAM` v2.0/2.1/2.2 game state and the
`SAV` zlib archive. The format facts below were verified against a real save on this
machine (`000000007-Chapter 1 Start`); keep them as a reference for the remaining work.

The behavioral reference is Near Infinity and [IESDP](https://gibberlings3.github.io/iesdp/),
same as every other format here. The quality bar is `crates/ie-formats/src/are.rs`.

(Background: a first attempt, `save_gam.rs`/`save_sav.rs`, was reviewed and reverted
because it heuristically scanned bytes instead of parsing the formats and found zero
saves on a real install. See "Pitfalls" — they're documented so they don't recur.)

---

## What's implemented

- **Discovery** (`crates/ie-io/src/lib.rs`): finds save folders under both the install
  root (`save/`, `mpsave/`) and the OS user-profile location (Documents, including
  OneDrive redirection and localized folder names), with a `--saves-dir` override. Keys
  each save by folder, strips the numeric prefix for a display name. Verified: lists all
  real BG2EE saves on this machine with correct names.
- **GAM decode** (`crates/ie-formats/src/save.rs`, `parse_gam`): signature/version
  validation, header (game time, gold, reputation, current/main area, section
  offsets+counts, chapter), party-member NPC structs (CRE resref, name, area, position,
  kill stats), and GLOBAL variables. The header model now also exposes raw offsets for
  party inventory (`0x28`) and PSTEE tail sections observed at `0x68`, `0x6C`, and `0x78`;
  these are included in save-write offset repair. All header offsets confirmed against
  the IESDP GAM v2.0 page; NPC (`0x160`) and variable (`0x54`) struct offsets verified.
- **SAV decode** (`parse_sav`): walks the entry table, zlib-inflates each member, and
  validates inflated size against the declared uncompressed size. Lists the contained
  resources (e.g. `AR0602.are`, `WORLDMAP.wmp`, `DEFAULT.toh`).
- **CLI**: `save-list --game <path> [--saves-dir <path>] --format <text|json>` and
  `save-info --game <path> --save <selector> [--part all|gam|sav]`. Strrefs resolve
  through the existing `TlkResolver`.
- **Scoped item write** (`save-add-item`): adds one item entry to one party member's
  embedded CRE inside `BALDUR.gam`, repairs CRE/GAM offsets, and writes a copied save
  folder by default. `--in-place` writes `BALDUR.gam.bak` unless disabled. Auto slot
  selection is currently verified only for PST/PSTEE inventory cells, including the real
  `Quick-Save-4` protagonist slot table where slot 33 is the first empty cell. Regression
  coverage asserts byte-exactness outside the inserted item and declared offset fields,
  including the PSTEE tail offsets that sit near EOF.
- Bounds-checked reads (`checked_end`/`checked_table_end`), typed `SaveParseError`,
  unit + integration tests with valid fixtures, clippy-clean.

---

## Next steps

- **Classic PST / GAM v1.1** — classic (non-EE) Planescape: Torment uses GAM **V1.1**,
  which the current parser rejects cleanly (`unsupported GAM version`). It has a different
  header and NPC layout; add a version branch **if** classic-PST support is ever needed.
  Note: **PSTEE (Steam "Project P") saves are GAM `V2.0`** and already parse — verified on
  a real save (`…-Quick-Save-4`, six party members decoded). The EE build does *not* use
  V1.1, so the driving PSTEE use case is already covered. **[judgment, likely low priority]**
- **Env-gated real-install test** — codify the manual validation as an `IE_GAME_PATH`-gated
  smoke test, following the ARE/BCS pattern, plus a small committed real fixture. The
  current read-only PSTEE slot-table regression is gated by `IE_PSTEE_SAVES`. **[mechanical]**
- **Near Infinity visual check for PST inventory slots** — real-save data now confirms
  the chosen `Quick-Save-4` auto slot, but NI should still be used to visually confirm
  that the range maps to backpack inventory cells rather than quick/equipment slots.
  **[judgment]**
- **BG/IWD inventory slot maps** — `save-add-item --slot auto` intentionally rejects
  non-PST variants until their general-inventory cell ranges are verified against real
  CREs/Near Infinity. Explicit `--slot INDEX` remains available. **[judgment]**
- **Field naming** — `GameNpcJson.character_name` is actually the CRE *resref*; the display
  name is `character_name_long`. Consider renaming to `cre_resref` / `character_name` for
  JSON consumers. **[mechanical]**
- **Variable `type_flags`** — the bitfield decode yields `[]` for real saved globals
  (raw type is typically 0; the value lives in `int_value`). Confirm the GAM variable
  "type" semantics against Near Infinity and adjust or drop the decode. **[judgment]**
- **Save `verify`** — optional cross-checks (party CRE resrefs resolvable, current-area
  present, journal strrefs in range) once a workflow needs them. **[mechanical]**
- **Decode SAV members** — reuse the existing ARE/CRE parsers on inflated entries when a
  workflow needs the contents, not just the manifest. **[judgment]**

---

## Verified format facts (reference)

### Save location & layout

- EE saves live under the OS user profile, e.g.
  `…/Documents/Baldur's Gate II - Enhanced Edition/save/` (OneDrive-redirected and
  localized as `…/OneDrive/Έγγραφα/…` on this machine). Classic games keep saves under
  the install dir. There's also an `mpsave/` sibling.
- Each save is a **folder** (`000000007-Chapter 1 Start`) containing `BALDUR.gam`,
  `BALDUR.SAV`, and `PORTRT*.bmp`. **The save name is the folder name**, not stored inside
  the files. Quicksaves are folders named `…-Quick-Save`; there is no `.QSV` extension.

### GAM (`BALDUR.gam`)

- Signature `GAME`, version `V2.0`/`V2.1`/`V2.2` (BG/IWD, and **PSTEE** — verified `V2.0`).
  Only *classic* (non-EE) PST uses `V1.1` (not yet handled).
- Header: game time `0x08`, gold `0x18`, party NPC offset/count `0x20`/`0x24`, non-party
  `0x30`/`0x34`, globals `0x38`/`0x3C`, main area `0x40` (8), journal **count `0x4C`,
  offset `0x50`** (note the order), reputation×10 `0x54`, current area `0x58` (8), real
  seconds `0x74`. Additional V2.x offsets used by the writer: party inventory `0x28`,
  PSTEE tail offsets `0x68`, `0x6C`, and `0x78`. NPC struct `0x160` bytes, variable
  struct `0x54` bytes.

### SAV (`BALDUR.SAV`)

- Signature `SAV ` + `V1.0`. A zlib archive of many resources. Entry layout, repeating:
  `u32` filename length (incl. NUL), filename, `u32` uncompressed length, `u32` compressed
  length, then `compressed length` bytes of zlib data.

---

## Pitfalls from the reverted attempt (avoid these)

- Searching only `<game_root>/save` → nothing on EE (saves are under Documents) and no
  recursion into per-save folders.
- Reading GAM offset `0x08` (game time) as a string and calling it the save name.
- Emitting `header_words` / `detected_strings` — authoritative-looking soup with no meaning.
- Treating `.SAV` as plaintext (it's zlib).
- Inventing a `SaveFormat::Quick` / `.QSV`.
- Tests built on invalid signatures (`GAM ` instead of `GAME`), so green ≠ correct.

## References

- `crates/ie-formats/src/are.rs` — parsing/JSON quality bar.
- `AGENTS.md` — Near Infinity validation workflow, fixture conventions.
- `docs/LOCAL_GAME_PATHS.md` — local install paths for real validation.
- IESDP `GAM` v2.0 and `SAV` v1.0 format pages.
