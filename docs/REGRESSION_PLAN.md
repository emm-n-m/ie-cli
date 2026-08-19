# Regression Plan: Real Installation Verification

This document defines specific resources to test against real game installations,
what to verify for each, and how to check results against the IESDP layout.

## Environment Setup

Each supported game needs an environment variable pointing to its install root:

| Variable          | Game                                 | Example path                                                                   |
|-------------------|--------------------------------------|--------------------------------------------------------------------------------|
| `IE_GAME_PATH`    | BG2EE (primary target)               | `C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate II Enhanced Edition` |
| `IE_BGEE_PATH`    | BGEE                                 | `...\Baldur's Gate Enhanced Edition`                                           |
| `IE_PSTEE_PATH`   | PSTEE                                | `...\Planescape Torment Enhanced Edition`                                      |
| `IE_IWDEE_PATH`   | IWDEE                                | `...\Icewind Dale Enhanced Edition`                                            |
| `IE_BGEE_SOD_PATH`| BGEE+SoD with the DLC still **packed** | `...\Baldur's Gate Enhanced Edition` (one that has not run DlcMerger)         |

Every variable names the **install root** — the directory holding `chitin.key` — not a subdirectory.
`IE_BGEE_SOD_PATH` in particular points at the game root, *not* at its `dlc/` folder; it is passed
straight through as `--game`, so pointing it at `dlc/` fails with `missing chitin.key`.

Tests silently skip when the variable is unset, so CI passes without game data.

Small factual expectations live in
`crates/ie-cli/tests/expectations/real_resources.json` and are executed by the reusable
`real_expectations` integration test. Each case must state its provenance: which IESDP table or raw-byte
check produced the expected value, and which install it came from. Never claim a stronger
provenance than the one actually used. A few early cases carry Near Infinity provenance and are
left as recorded — that is history, not a template for new cases.

Two further variables regenerate goldens rather than select an install. They are write switches, not
inputs — set either one and the run rewrites checked-in expectations instead of asserting against
them, so never set them on a verification run:

| Variable               | Effect                                                                        |
|------------------------|-------------------------------------------------------------------------------|
| `UPDATE_GOLDENS`       | Rewrites the synthetic value goldens in `crates/ie-formats/tests/goldens/`     |
| `UPDATE_SHAPE_GOLDENS` | Unions observed shapes into `crates/ie-cli/tests/goldens/shape/`               |

See [GOLDENS.md](./GOLDENS.md) for what each tier pins and why real installs can pin shape but not
values.

---

## 1. Infrastructure Tests (ie-io)

These validate the loading pipeline independent of any format decoder.

### 1.1 Game Discovery

| Test case                          | Game   | What to verify                                                  |
|------------------------------------|--------|-----------------------------------------------------------------|
| Discover chitin.key                | BG2EE  | `GameInstallation::discover` succeeds                           |
| Discover language folder           | BG2EE  | `lang/en_US/` selected, `dialog.tlk` found                     |
| Discover chitin.key                | BGEE   | Same as above, confirms not BG2-specific                        |
| Discover chitin.key                | PSTEE  | Same, may have different language folder layout                  |
| Reject invalid path                | (any)  | Non-game directory returns `InvalidGamePath` or `MissingChitinKey` |

### 1.2 KEY Parsing

| Test case                              | Game   | What to verify                                         |
|----------------------------------------|--------|--------------------------------------------------------|
| Parse chitin.key                       | BG2EE  | Entry count > 0, BIFF list > 0                        |
| Lookup known ITM resource              | BG2EE  | `ACIDBL.ITM` resolves to `data/Items.bif`             |
| Lookup known SPL resource              | BG2EE  | `SPWI112.SPL` resolves to a valid BIFF path           |
| Lookup known DLG resource              | BG2EE  | `AERIE.DLG` resolves (likely from override)            |
| Lookup nonexistent resource            | BG2EE  | Returns `ResourceNotFound`                             |
| Cross-game KEY parse                   | BGEE   | Entry count > 0, at least one ITM locatable            |

### 1.3 Resource Reading (BIFF Extraction)

| Test case                              | Game   | Resource          | What to verify                            |
|----------------------------------------|--------|-------------------|-------------------------------------------|
| Read from uncompressed BIF             | BG2EE  | `ACIDBL.ITM`      | Bytes start with `ITM `, length > 114     |
| Read from override                     | BG2EE  | `AERIE.DLG`       | Bytes start with `DLG ` (if in override)  |
| Read compressed BIF (BIFC)             | BG2EE  | (find a .cbf)     | Decompresses without error                |

### 1.4 TLK Resolution

| Test case                      | Game   | StrRef   | What to verify                                       |
|--------------------------------|--------|----------|------------------------------------------------------|
| Resolve strref 0               | BG2EE  | 0        | Returns empty or placeholder (game-dependent)        |
| Resolve strref 1               | BG2EE  | 1        | Returns non-empty string                             |
| Resolve known item name        | BG2EE  | (from ACIDBL.ITM header) | Matches the name stored at the ITM header strref |
| Out-of-range strref            | BG2EE  | 99999999 | Returns `StrRefOutOfRange`                           |

### 1.5 Packed DLC Mounting

| Test case | Game | What to verify |
|---|---|---|
| Discover and mount packed DLC | BGEE+SoD | `dlc/*.zip` is opened and invalid archives fail discovery rather than being skipped |
| Merge the DLC's own KEY | BGEE+SoD | `mod.key` inside the zip is parsed and merged; SoD's ~39 BIFs are indexed. Mounting the zip without this reaches only `override/` and `lang/` — 3 files out of 21,272 |
| Read DLC-backed CRE | BGEE+SoD | A `BD*` CRE listed from the packed DLC resolves, reads, and decodes as a CRE |
| Resolve DLC TLK extension | BGEE+SoD | strref `50000` resolves to `If I could, I would. But I can't, so...` and output names the selected zip entry |
| Compare packed and merged installs | BGEE+SoD | Resource and string answers agree with the DlcMerger-merged installation |

The first three checks are gated by `IE_BGEE_SOD_PATH` and skip when no packed install is available.

---

## 2. ITM Decoder Tests (ie-formats)

### 2.1 Resource Selection Rationale

Choose items that exercise different parser paths:

| Resource       | Why                                                      | Category          |
|----------------|----------------------------------------------------------|-------------------|
| `ACIDBL.ITM`   | Existing validated resource, simple weapon                | Baseline          |
| `SW1H01.ITM`   | Standard melee weapon (Long Sword), basic abilities       | Standard weapon   |
| `BOOT01.ITM`   | Non-weapon equipment, no combat abilities                 | Equipment         |
| `STAF09.ITM`   | Staff of the Magi, many effects and flags                 | Complex item      |
| `POTN08.ITM`   | Consumable, single-use item with effects                  | Consumable        |
| `AMUL14.ITM`   | Amulet with passive effects                               | Passive equipment |
| `RING06.ITM`   | Ring of the Princes +1, six equipped effects               | Continuous effect |
| `SCRL1B.ITM`   | Scroll, learnable spell reference                         | Scroll            |
| `MISC01.ITM`   | Gold piece, zero-ability item                             | Minimal item      |

### 2.2 Field Verification Matrix

For each ITM resource, verify these fields against the IESDP ITM layout:

| Field group         | What to check                                                         |
|---------------------|-----------------------------------------------------------------------|
| **Header**          | `signature`, `version`, `name` (resolved), `identified_name`         |
| **Flags**           | All item flags match (droppable, displayable, cursed, etc.)           |
| **Type**            | `item_type` raw + decoded label                                       |
| **Usability**       | Usability flags match the IESDP usability bit table                   |
| **Stats**           | `price`, `stack_size`, `weight`, `lore`, `enchantment`                |
| **Abilities**       | Count matches, each ability's `attack_type`, `target`, `dice_*`      |
| **Effects**         | Count matches, each effect's `opcode` (raw + decoded), `target`, timing, parameters |
| **String refs**     | All resolved strings match `dialog.tlk` at that strref                 |
| **Version-specific**| V1.1 fields present only when version is `V1.1`                      |

### 2.3 Edge Cases

| Test case                     | Resource        | What to verify                                 |
|-------------------------------|-----------------|-------------------------------------------------|
| Item with zero abilities      | `MISC01.ITM`    | Abilities array is empty, no panic              |
| Item with zero effects        | (find one)      | Effects array is empty                          |
| Item with many effects        | `STAF09.ITM`    | All effects parse, count matches NI             |
| Item with unresolved strrefs  | (if any exist)  | Graceful handling, `strref` field still present |

---

## 3. SPL Decoder Tests (ie-formats)

### 3.1 Resource Selection Rationale

| Resource       | Why                                                      | Category            |
|----------------|----------------------------------------------------------|---------------------|
| `SPWI112.SPL`  | Already validated against NI (Magic Missile)              | Baseline arcane     |
| `SPWI401.SPL`  | Already validated (Confusion)                             | Validated arcane    |
| `SPWI913.SPL`  | Already validated (high-level spell)                      | Validated arcane    |
| `SPPR211.SPL`  | Already validated (Silence 15' Radius, priest)            | Validated divine    |
| `SPDR301.SPL`  | Already validated (druid spell)                           | Validated druid     |
| `SPWI304.SPL`  | Fireball, multiple projectile targets                     | Area effect         |
| `SPPR101.SPL`  | Cure Light Wounds, single target healing                  | Healing             |
| `SPPR718.SPL`  | Nature's Beauty, level 7 druid, class exclusion flags     | Exclusion flags     |
| `SPWI220.SPL`  | Know Alignment, divination                                | Divination school   |
| `SPCL212.SPL`  | Innate ability (Lay On Hands or similar)                  | Innate/special      |
| `SPWI613.SPL`  | Improved Haste, many level-scaled effect blocks           | Complex mechanics   |

### 3.2 Field Verification Matrix

For each SPL resource, verify these fields against the IESDP SPL layout:

| Field group          | What to check                                                        |
|----------------------|----------------------------------------------------------------------|
| **Header**           | `signature`, `version`, `name` (resolved), `identified_name`        |
| **Spell metadata**   | `spell_type` (raw + decoded), `spell_level`, `school`, `form`        |
| **Exclusion flags**  | Raw value + all decoded flag names match NI                          |
| **Extended headers** | Count matches, each header's `spell_form`, `target`, `range`, `casting_time`, `dice_*` |
| **Feature blocks**   | Count matches, each effect's `opcode` (raw + decoded), parameters, duration, probability |
| **String refs**       | All resolved strings match NI's displayed text                      |
| **Projectile**       | Projectile index matches NI                                          |

### 3.3 Edge Cases

| Test case                        | Resource         | What to verify                                |
|----------------------------------|------------------|-----------------------------------------------|
| Spell with no extended headers   | (if any exist)   | Empty array, no panic                         |
| Spell with many feature blocks   | `SPWI913.SPL`    | All features parse, count matches NI          |
| Innate ability (non-wizard/priest)| `SPCL212.SPL`   | `spell_type` decoded correctly                |
| Spell with exclusion flags set   | `SPPR718.SPL`    | Each flag bit matches NI's display            |
| Level 0 or unusual level         | (if any exist)   | Level field parses correctly                  |

---

## 4. CLI Integration Tests (ie-cli)

### 4.1 Command: `locate`

| Test case                   | Resource         | Expected result                              |
|-----------------------------|------------------|----------------------------------------------|
| Locate BIF-backed item      | `ACIDBL.ITM`     | Source is BIFF, path contains `Items.bif`    |
| Locate override resource    | `AERIE.DLG`      | Source is override                           |
| Locate nonexistent resource | `ZZZZZ.ITM`      | Non-zero exit code, error message            |

### 4.2 Command: `dump-raw`

| Test case                   | Resource         | What to verify                               |
|-----------------------------|------------------|----------------------------------------------|
| Dump raw ITM                | `ACIDBL.ITM`     | Written file starts with `ITM V1`, byte count in JSON |
| Dump raw SPL                | `SPWI112.SPL`    | Written file starts with `SPL V1`, byte count in JSON |
| Dump raw to existing path   | any              | Overwrites cleanly                           |

### 4.3 Command: `dump --format json`

| Test case                          | Resource         | What to verify                          |
|------------------------------------|------------------|-----------------------------------------|
| Dump ITM JSON                      | `ACIDBL.ITM`     | Valid JSON, has `header`, `abilities`    |
| Dump SPL JSON                      | `SPWI112.SPL`    | Valid JSON, has `header`, `extended_headers` |
| Dump ITM with effects              | `STAF09.ITM`     | `effects` array present and non-empty   |
| Dump SPL priest spell              | `SPPR211.SPL`    | `spell_type` is priest/divine           |
| Dump DLG JSON                      | `AERIE.DLG`      | Valid JSON, has `states` and script tables |
| JSON is deterministic              | `ACIDBL.ITM`     | Two runs produce byte-identical output  |

### 4.4 Command: `tlk`

| Test case                 | StrRef   | What to verify                              |
|---------------------------|----------|---------------------------------------------|
| Resolve valid strref      | 1        | Non-empty text output                       |
| Resolve strref 0          | 0        | Returns without error                       |
| Resolve out-of-range      | 99999999 | Error message, non-zero exit                |

---

## 5. Verification Strategy (No Committed Game Data)

Infinity Engine game resources are protected IP. This project must not commit
game data — including full JSON dumps — to the public repository.

All regression coverage uses one of three approaches, layered by specificity:

### 5.1 Committed Value Assertions (primary approach)

Individual field assertions encoded directly in Rust tests. These are small
factual constants used for verification, not a reproduction of the work.

This is the existing pattern in `ie-cli/tests/dump.rs`:

```rust
assert_eq!(stdout["header"]["spell_level"], 4);
assert_eq!(stdout["header"]["school"]["decoded"], "Enchanter");
assert_eq!(stdout["feature_blocks"][0]["opcode"]["decoded"], "Confusion");
```

Each NI comparison session produces a set of `assert_eq!` lines that become
permanent automated checks. This is where the bulk of regression value lives.

### 5.2 Structural / Shape Assertions (supplement)

Assert structure and counts without referencing copyrighted content:

```rust
// Array lengths
assert_eq!(stdout["extended_headers"].as_array().map(Vec::len), Some(4));

// Required keys present
assert!(stdout["header"]["flags"]["raw"].is_number());
assert!(stdout["header"]["flags"]["decoded"].is_array());

// All strrefs resolved to non-empty strings
let name = stdout["header"]["name"]["resolved"].as_str().unwrap();
assert!(!name.is_empty());

// No null opcode labels (parser completeness)
for fb in stdout["feature_blocks"].as_array().unwrap() {
    assert!(fb["opcode"]["decoded"].is_string());
}
```

These catch structural regressions (missing fields, broken arrays, unresolved
strrefs) without embedding game text.

### 5.3 Local-Only Golden Files (developer convenience)

Developers with a game installation can generate and keep full JSON snapshots
locally for side-by-side diffing during development. These are gitignored.

```bash
# Generate
cargo run -p iecli -- dump --game "$IE_GAME_PATH" --resource SPWI401.SPL --format json \
  > tests/golden/SPWI401.SPL.json

# Diff after a parser change
cargo run -p iecli -- dump --game "$IE_GAME_PATH" --resource SPWI401.SPL --format json \
  | diff tests/golden/SPWI401.SPL.json -
```

The `tests/golden/` directory must be gitignored. These files are a development
aid, not a test artifact. They are never committed.

### 5.4 What NOT to commit

- Full JSON output of any game resource
- Raw or extracted binary resource files
- TLK string dumps
- Any file whose content is derived from game data beyond individual factual
  constants used in assertions

---

## 6. Cross-Game Verification

Once additional game paths are available, run a reduced test matrix on each game
to catch game-specific assumptions.

### 6.1 Per-Game Smoke Tests

| Game   | Variable         | Smoke resources                                            |
|--------|------------------|------------------------------------------------------------|
| BG2EE  | `IE_GAME_PATH`   | Full matrix above                                          |
| BGEE   | `IE_BGEE_PATH`   | 1 ITM (`SW1H01.ITM`), 1 SPL (`SPWI112.SPL`), 1 TLK lookup |
| PSTEE  | `IE_PSTEE_PATH`  | 1 ITM, 1 SPL (pick PSTEE-specific resources)               |
| IWDEE  | `IE_IWDEE_PATH`  | 1 ITM, 1 SPL (pick IWDEE-specific resources)               |

### 6.2 Cross-Game Issues to Watch For

- Language folder layout differences (PSTEE may differ)
- V1.0 vs V1.1 format version distribution
- Game-specific resource types or flags not present in BG2EE
- Different BIFF archive structures or compression
- TLK string table size and encoding quirks

---

## 7. Verification Procedure

When verifying a resource:

1. Open the relevant IESDP page for the format and version and note the offset table.
2. Dump the resource with `iecli dump --game <path> --resource <RESREF>.<EXT>`.
3. For each field group in the verification matrix:
   - Read the expected value from the IESDP offset table, or from the raw bytes at that offset
     (`iecli dump-raw` piped through a hex viewer) when IESDP is ambiguous.
   - Compare against the JSON field.
   - Note any discrepancy with an explanation.
4. Pay special attention to:
   - Flag decoding: iecli emits the raw value *and* a decoded array; both must be right.
   - String resolution: the strref and the resolved text are separate assertions.
   - Effect opcodes: the table is variant-specific — PST uses its own, so check
     `game_variant` before trusting an opcode name.
   - Field widths: a field IESDP lists as one byte must not be read as two, and the
     surrounding fields must still land correctly.
5. Acceptable differences from IESDP:
   - Label wording, as long as the meaning is the same.
   - Field ordering and nesting in JSON.
6. Unacceptable:
   - Missing fields that IESDP documents and real files populate.
   - Wrong numeric values or flag bits.
   - Wrong string resolution.
   - Silently dropped unknown bytes.
7. Record the outcome as a case in `crates/ie-cli/tests/expectations/real_resources.json`
   with provenance naming the IESDP table and the install.

When real files disagree with IESDP, the files win — write a note in `docs/decisions/`.

---

## 8. Running the Tests

### Full suite (no real install needed)

```bash
cargo test
```

### With BG2EE real-install tests

```powershell
$env:IE_GAME_PATH = "C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate II Enhanced Edition"
cargo test
```

### Single resource verification

```bash
cargo run -p iecli -- dump --game "$IE_GAME_PATH" --resource STAF09.ITM --format json > actual.json
# Compare actual.json against NI, then encode findings as assert_eq! lines
```

### Run only real-install tests

```bash
cargo test --test dump --test dump_raw
```

---

## 9. Progression Checklist

Track which resources have been fully verified. Mark each cell when done.

### ITM Resources

| Resource       | Parsed | Fields checked vs NI | Asserts committed | Notes |
|----------------|--------|----------------------|-------------------|-------|
| `ACIDBL.ITM`   | yes    | partial              | no                |       |
| `SW1H01.ITM`   | yes    | IESDP/raw bytes      | yes               | BGEE; exposed and fixed byte-width requirement bug |
| `BOOT01.ITM`   |        |                      |                   |       |
| `STAF09.ITM`   |        |                      |                   |       |
| `POTN08.ITM`   |        |                      |                   |       |
| `AMUL14.ITM`   |        |                      |                   |       |
| `RING06.ITM`   | yes    | IESDP/raw bytes      | yes               | BGEE equipped-effect case |
| `SCRL1B.ITM`   | yes    | IESDP/raw bytes      | yes               | BGEE cast/learn ability case |
| `MISC01.ITM`   |        |                      |                   |       |

### SPL Resources

| Resource       | Parsed | Fields checked vs NI | Asserts committed | Notes |
|----------------|--------|----------------------|-------------------|-------|
| `SPWI112.SPL`  | yes    | yes                  | partial           |       |
| `SPWI401.SPL`  | yes    | yes                  | yes               |       |
| `SPWI913.SPL`  | yes    | yes                  | no                |       |
| `SPPR211.SPL`  | yes    | yes                  | yes               |       |
| `SPDR301.SPL`  | yes    | yes                  | no                |       |
| `SPWI304.SPL`  |        |                      |                   |       |
| `SPPR101.SPL`  |        |                      |                   |       |
| `SPPR718.SPL`  |        |                      |                   |       |
| `SPWI220.SPL`  |        |                      |                   |       |
| `SPCL212.SPL`  |        |                      |                   |       |
| `SPWI613.SPL`  | yes    | IESDP/raw bytes      | yes               | BGEE 9 headers / 81 effects |

### Cross-Game

| Game   | Discovery | KEY parse | 1 ITM | 1 SPL | TLK | Notes |
|--------|-----------|-----------|-------|-------|-----|-------|
| BG2EE  | yes       | yes       | yes   | yes   | yes |       |
| BGEE   | yes       | yes       | yes   | yes   | yes | stock packed-DLC install |
| PSTEE  |           |           |       |       |     |       |
| IWDEE  |           |           |       |       |     |       |
