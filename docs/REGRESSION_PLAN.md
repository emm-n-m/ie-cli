# Regression Plan: Real Installation Verification

This document defines specific resources to test against real game installations,
what to verify for each, and how to compare results against Near Infinity.

## Environment Setup

Each supported game needs an environment variable pointing to its install root:

| Variable          | Game                                 | Example path                                                                   |
|-------------------|--------------------------------------|--------------------------------------------------------------------------------|
| `IE_GAME_PATH`    | BG2EE (primary target)               | `C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate II Enhanced Edition` |
| `IE_BGEE_PATH`    | BGEE                                 | `...\Baldur's Gate Enhanced Edition`                                           |
| `IE_PSTEE_PATH`   | PSTEE                                | `...\Planescape Torment Enhanced Edition`                                      |
| `IE_IWDEE_PATH`   | IWDEE                                | `...\Icewind Dale Enhanced Edition`                                            |

Tests silently skip when the variable is unset, so CI passes without game data.

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
| Resolve known item name        | BG2EE  | (from ACIDBL.ITM header) | Matches Near Infinity's displayed name |
| Out-of-range strref            | BG2EE  | 99999999 | Returns `StrRefOutOfRange`                           |

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
| `RING06.ITM`   | Ring of Free Action, continuous effects                   | Continuous effect |
| `SCRL1B.ITM`   | Scroll, learnable spell reference                         | Scroll            |
| `MISC01.ITM`   | Gold piece, zero-ability item                             | Minimal item      |

### 2.2 Field Verification Matrix

For each ITM resource, verify these fields against Near Infinity:

| Field group         | What to check                                                         |
|---------------------|-----------------------------------------------------------------------|
| **Header**          | `signature`, `version`, `name` (resolved), `identified_name`         |
| **Flags**           | All item flags match (droppable, displayable, cursed, etc.)           |
| **Type**            | `item_type` raw + decoded label                                       |
| **Usability**       | Usability flags match Near Infinity's checkboxes                      |
| **Stats**           | `price`, `stack_size`, `weight`, `lore`, `enchantment`                |
| **Abilities**       | Count matches, each ability's `attack_type`, `target`, `dice_*`      |
| **Effects**         | Count matches, each effect's `opcode` (raw + decoded), `target`, timing, parameters |
| **String refs**     | All resolved strings match Near Infinity's displayed text             |
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
| `SPWI613.SPL`  | Contingency, complex targeting                            | Complex mechanics   |

### 3.2 Field Verification Matrix

For each SPL resource, verify these fields against Near Infinity:

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

## 7. Near Infinity Comparison Procedure

When verifying a resource against Near Infinity:

1. Open Near Infinity pointed at the same game installation.
2. Navigate to the resource (e.g., Items > ACIDBL).
3. For each field group in the verification matrix:
   - Record the NI value.
   - Run `iecli dump` and compare the JSON field.
   - Note any discrepancies with an explanation.
4. Pay special attention to:
   - Flag decoding: NI shows named checkboxes; iecli shows raw + decoded array.
   - String resolution: NI shows inline text; iecli shows strref + resolved text.
   - Effect opcodes: NI uses its own label set; iecli labels may differ in wording.
5. Acceptable differences:
   - Label wording (e.g., "Two-handed" vs "two_handed") as long as the meaning is the same.
   - Field ordering in JSON vs NI's panel layout.
6. Unacceptable differences:
   - Missing fields that NI shows.
   - Wrong numeric values.
   - Missing or wrong flag bits.
   - Wrong string resolution.

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
| `SW1H01.ITM`   |        |                      |                   |       |
| `BOOT01.ITM`   |        |                      |                   |       |
| `STAF09.ITM`   |        |                      |                   |       |
| `POTN08.ITM`   |        |                      |                   |       |
| `AMUL14.ITM`   |        |                      |                   |       |
| `RING06.ITM`   |        |                      |                   |       |
| `SCRL1B.ITM`   |        |                      |                   |       |
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
| `SPWI613.SPL`  |        |                      |                   |       |

### Cross-Game

| Game   | Discovery | KEY parse | 1 ITM | 1 SPL | TLK | Notes |
|--------|-----------|-----------|-------|-------|-----|-------|
| BG2EE  | yes       | yes       | yes   | yes   | yes |       |
| BGEE   |           |           |       |       |     |       |
| PSTEE  |           |           |       |       |     |       |
| IWDEE  |           |           |       |       |     |       |
