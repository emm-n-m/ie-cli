# Bug: PST effect-opcode names are mis-decoded (wrong opcode table for the Torment engine)

## Summary
`iecli`'s effect-opcode→name decoder assumes a single numbering (standard/EE). **Planescape:
Torment numbers effect opcodes differently**, so for PSTEE resources the `opcode.decoded` field on
ITM/SPL/CRE effects is wrong — names are mislabeled or blank. Any analysis that reasons over decoded
effect *names* for PST (stat bonuses on items, spell effects, creature effects) is unreliable.

This was found while cataloguing PST tattoo stat bonuses: a name-based scan concluded "only Wisdom
tattoos affect stats," when in fact there is a full **+1/+2 tattoo set for all six attributes** — the
scan only matched WIS/STR because those two opcodes happen to line up between the two numberings.

## Where
Three **duplicated, hardcoded** tables, all the same EE-style numbering, none engine-aware:
- `decode_effect_opcode` in [crates/ie-formats/src/itm.rs:724](crates/ie-formats/src/itm.rs#L724)
- `decode_effect_opcode` in [crates/ie-formats/src/spl.rs:585](crates/ie-formats/src/spl.rs#L585)
- `decode_effect_opcode` in [crates/ie-formats/src/cre.rs:1242](crates/ie-formats/src/cre.rs#L1242)

The parse entry points (`parse_itm` / `parse_spl` / `parse_cre`) take `(bytes, resource_name,
resolver)` — **no engine/game variant is threaded in**, so the decoder cannot select a table.
Engine/game-name detection already exists, but only in the IO layer
([crates/ie-io/src/lib.rs:1121](crates/ie-io/src/lib.rs#L1121) recognizes "Planescape Torment -
Enhanced Edition").

## Evidence (ground truth)
Verified against the install via the stat tattoos, which *name* the stat they boost
(`TTSTR1/2` = Tattoo of Might / Greater Might, `TTDEX1/2` = Action, `TTCON1/2` = Health,
`TTINT1/2` = Insight/Revelation, `TTWIS1/2` = Spirit/Soul, `TTCHR1/2` = Presence; `TTAC*`, `TTMAXHP*`):

| PST opcode | Correct meaning | What iecli prints today |
|---:|---|---|
| 0  | AC bonus       | "Cure Condition" |
| 6  | **Charisma**   | "Drain Magic" |
| 10 | **Constitution** | *(blank — not in table)* |
| 15 | **Dexterity**  | *(blank — not in table)* |
| 18 | Max HP         | "Fatigue" |
| 19 | **Intelligence** | "Intoxication" |
| 44 | **Strength**   | "Protection: Acid (Lesser)" |
| 49 | **Wisdom**     | "Wisdom" ✓ (coincidental match) |

Only WIS lines up by luck. (Note: opcode 45 currently prints "Strength" — that's a *false positive*
for PST; PST's Strength is 44, and 45 means something else there.)

## Repro
Needs a PSTEE install; pass it via `IE_PSTEE_PATH` (the same var the existing PSTEE tests use —
[crates/ie-cli/tests/dump.rs:339](crates/ie-cli/tests/dump.rs#L339)):
```
iecli dump --game <PSTEE> --resource TTSTR2.ITM --format json
#   equip effect shows: "opcode": { "raw": 44, "decoded": "Protection: Acid (Lesser)" }
#   expected (PST):     Strength
iecli dump --game <PSTEE> --resource TTCHR2.ITM --format json
#   "raw": 6, "decoded": "Drain Magic"   →  expected: Charisma
```

## Fix shape

**Judgment calls (design first):**
1. Introduce an engine/game-variant enum (at minimum `Standard` vs `Pst`) and thread it from the IO
   layer — where the game is already detected — down into the format parsers (add a param to
   `parse_itm/spl/cre`, or carry it on a context struct). Decide where it should live so the same
   switch can be reused for other PST-divergent tables later.
2. Source the **complete** PST opcode table (IESDP's Torment column). The eight rows above are the
   verified anchors; the full table has hundreds of entries and other ranges differ too.

**Mechanical (low-risk once the above is decided):**
3. Collapse the three duplicate `decode_effect_opcode` copies into one shared decoder that takes the
   variant and selects the table. Keep the existing table as the `Standard` default so non-PST games
   are byte-for-byte unchanged.

## Acceptance
- For a PSTEE game: `TTSTR2`→Strength, `TTDEX2`→Dexterity, `TTCON2`→Constitution, `TTINT2`→
  Intelligence, `TTCHR2`→Charisma, `TTWIS2`→Wisdom, `TTAC2`→AC, `TTMAXHP2`→Max HP.
- Non-PST behavior unchanged: the existing `spl.rs` decode tests
  ([crates/ie-formats/src/spl.rs:747](crates/ie-formats/src/spl.rs#L747), e.g. `101 → "Immunity to
  effect"`) still pass under the default table.
- Add a PST-gated test (skips when `IE_PSTEE_PATH` is unset, like the other PSTEE tests) asserting
  the tattoo decodes above **against the real install** — not just a hand-built fixture, which could
  be made to pass with the wrong table.

## Non-goals
Don't try to make every PST opcode authoritative in one pass — landing the ability-score / AC / Max-HP
block plus de-duplicating the three tables is the win. If you don't transcribe all of IESDP, leave a
comment marking the PST table as partial.
