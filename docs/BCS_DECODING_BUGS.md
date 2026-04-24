# BCS Decoding — Format Bugs Found on First Real-Install Use

**Status: all three bugs fixed.** This document is a historical record of format mismatches discovered when real BG:EE BCS files first hit the parser, and how each was diagnosed and resolved. The canonical BCS spec now lives in the parser itself: [`crates/ie-formats/src/bcs.rs`](../crates/ie-formats/src/bcs.rs). No open action items remain. If making further changes to the BCS parser, always validate against a real resource (e.g. `iecli dump-raw --resource IMOEN.BCS --output /tmp/imoen.bcs`) before trusting any external spec — even IESDP-derived specs contained inaccuracies that only surfaced against live data.

## Bug 1 — Object specifier is 12 integers, not 13 (FIXED)

**Status**: fixed in a follow-up session. Left here for the record.

Plan said 13 integers. Real files have **12**. Fix applied:

- `BcsObjectJson::raw` changed from `[i32; 13]` → `[i32; 12]`.
- `BcsObjectDecodedJson::extra_targets` changed from `Option<[i32; 5]>` → `Option<[i32; 4]>`.
- `parse_object` reads 12 ints.
- `decode_object` signature + `extra_targets` slice updated.
- Test fixtures updated to emit 12 zeros instead of 13, and the `"DV"` object line uses `1 2 3 4` (4 extras) instead of `1 2 3 4 5` (5).

After this fix, `parse_tr` succeeds on real input. The remaining bugs are in `parse_ac`.

## Bug 2 — Action record layout is reversed and has an extra leading integer (FIXED)

**Status**: fixed in a follow-up session. Left here for the record.

### What the plan said

```
AC
<opcode> <int1> <int2> <int3> <int4> "<str1>" "<str2>"
OB <object 1> OB
OB <object 2> OB
OB <object 3> OB
<point_x> <point_y>
AC
```

### What real BCS files contain

Example from `RASAAD.BCS` (confirmed identical in stock `IMOEN.BCS`), lines shown verbatim:

```
100AC                                          <- RE weight + AC opens
30OB                                           <- <leading_int> then first OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB                   <- object 1 payload + closing OB
OB                                             <- opening OB for object 2
0 0 0 0 0 0 0 0 0 0 0 0 ""OB                   <- object 2 payload + closing OB
OB                                             <- opening OB for object 3
0 0 0 0 0 0 0 0 0 0 0 0 ""OB                   <- object 3 payload + closing OB
98 0 0 0 0"GLOBALDORN_ROMANCE_FIGHT" "" AC     <- opcode + args + AC closes
```

Key differences from the plan:

1. **Opcode and string/int args are at the END of the action, not the start**. The `AC` closing tag is on the same line as the opcode + args.
2. **A leading integer (`30` above) appears immediately after `AC` opens**, before the first `OB`. Meaning unknown — possibly frame-delay, probability, or a BG2-specific field. Treat as an opaque `i32` for now; surface it as a `BcsActionJson::leading` or `header_int` field until semantics are confirmed.
3. **No explicit `point_x` `point_y` pair at the end** in this file. Either the point is omitted for some actions, or the leading int subsumes it. The test fixture synthesizes `1 2\nAC` after the objects; real BCS does not. Remove the point from the required parse path; verify on multiple samples whether point ever appears (actions like `MoveToPoint` need coordinates — find one in `BALDUR.BCS` or an area BCS to confirm).

### Fix applied

`parse_ac` now parses the real layout:

1. opening `AC`
2. opaque leading `i32`
3. three `OB` object payloads
4. opcode + four integer args + two string args
5. optional point pair if present
6. closing `AC`

The JSON shape now surfaces:

- `BcsActionJson::leading`
- `BcsActionJson::point: Option<BcsPointJson>`

This preserves the unknown leading integer instead of dropping it, and no longer requires coordinates for every action.

### Suggested fix outline

Restructure `parse_ac` roughly:

```rust
fn parse_ac(&mut self) -> Result<BcsActionJson, BcsParseError> {
    self.expect_tag(Tag::Ac)?;

    let leading = self.expect_int()?;  // unknown; surface as `header_int`

    self.expect_tag(Tag::Ob)?;
    let object_1 = self.parse_object()?;
    self.expect_tag(Tag::Ob)?;

    self.expect_tag(Tag::Ob)?;
    let object_2 = self.parse_object()?;
    self.expect_tag(Tag::Ob)?;

    self.expect_tag(Tag::Ob)?;
    let object_3 = self.parse_object()?;
    self.expect_tag(Tag::Ob)?;

    let opcode = self.expect_int()?;
    let int_args = [
        self.expect_int()?,
        self.expect_int()?,
        self.expect_int()?,
        self.expect_int()?,
    ];
    let string_args = [self.expect_string()?, self.expect_string()?];

    self.expect_tag(Tag::Ac)?;
    // ... construct BcsActionJson with opcode resolved via resolver.resolve_action(opcode)
}
```

Remove `point` from the struct (or make it `Option<BcsPointJson>` and document that it's only populated if present — but first figure out *when* it's present).

### Test fixtures must be rewritten

The synthetic BCS in `crates/ie-formats/src/bcs.rs` tests was constructed to match the plan's (wrong) layout, so tests passed against a parser encoding the same mistake. This has now been corrected: fixtures use the real action layout, including the leading integer and opcode-at-end form.

## Reproduction

```bash
./target/release/iecli dump-raw \
  --game "<BG1EE path>" \
  --resource RASAAD.BCS \
  --output /tmp/rasaad.bcs

./target/release/iecli dump --game "<BG1EE path>" --resource RASAAD.BCS
# Current failure: "resource parsing failure: unexpected token at line 16, column 3: expected integer, found tag OB"
```

## Bug 3 — Trigger opcode names never resolve (actions do) (FIXED)

**Status**: fixed in a follow-up session. Left here for the record.

Actions in decoded `RASAAD.BCS` do get their `name` populated (e.g. `"leading": 134, "name": "SetGlobal"`, `"leading": 30, "name": "AddWayPoint"`). Triggers do not — every single trigger in the output has `"name": null`, despite `opcode` values that are clearly valid IE trigger codes (e.g. `16399` is `Global(S:Name*,S:Area*,I:Value*)`, `16448` is `GlobalTimerExpired(S:Name*,S:Area*)`, `16596` appears to be an AreaCheck variant, etc.).

Quick proof from `RASAAD.BCS` dump:

```bash
grep -B1 '"negated"' /c/tmp/rasaad_bcs.json | grep '"name":' | sort | uniq -c
# → 204  "name": null,
```

204 triggers, zero resolved.

### Likely causes (in order of probability)

1. **Wrong IDS file is being consulted.** The resolver may be loading `ACTION.IDS` for both action and trigger lookups, or be keyed on a single shared IDS file. The fix is to route trigger lookups specifically to `TRIGGER.IDS`.
2. **Signed vs unsigned opcode mismatch.** Negated triggers appear as negative opcodes in the raw BCS (`-16399`), but `TRIGGER.IDS` entries are unsigned. The resolver must look up by *absolute* value when `negated: true`. Verify this is done at the parse-layer call site, not pushed onto the resolver.
3. **Decimal vs hex parsing in `TRIGGER.IDS`.** Some IDS files store entries in hex (`0x400F Global`), others in decimal. If the IDS reader assumes one and this file is the other, every lookup silently misses.

### Root cause

The trigger-name failure was caused by the IDS reader rejecting BGEE's combined header form:

```text
IDS V1.0
```

`TRIGGER.IDS` in the user's install starts with that single-line header. The IDS parser only tolerated `IDS` and `V1.0` as separate lines, so loading `TRIGGER.IDS` failed and the lazy resolver cached `None` for the trigger table. Action names still resolved because `ACTION.IDS` in this install came from override and used a different header shape.

The fix was:

- teach `parse_ids_text` to ignore a combined `IDS V1.0` header line
- add a unit test for that header form
- keep trigger lookup keyed to `TRIGGER.IDS`
- keep trigger lookup using the absolute value of negative opcodes

While verifying the fix, the action-side `name` resolver was also corrected to resolve from the leading action opcode rather than the trailing integer field.

### How to verify the fix

```bash
./target/release/iecli dump --game "<BG1EE>" --resource RASAAD.BCS > /tmp/rasaad_bcs.json
grep -B1 '"negated"' /tmp/rasaad_bcs.json | grep '"name":' | sort | uniq -c
```

Expected result: names like `"Global"`, `"GlobalTimerExpired"`, `"AreaCheck"`, `"InParty"` appear with non-trivial counts. Zero occurrences of `"name": null` in trigger bodies.

Also spot-check a known block: the `RASAAD_PLOT` 1→2 transition block should show a trigger with `"name": "Global"` and string_args `["GLOBALRASAAD_PLOT", ""]` — not `"name": null`.

### Follow-up observations (no action required)

Earlier drafts of this bug speculated that the action `opcode` JSON field might be mislabeled and that the real action code lived in `leading`. On re-verification post-fix, this turned out to be wrong: `opcode` is the correct field holding the ACTION.IDS code (`SetGlobal` → `opcode: 30`, and names resolve from it), and `leading` is a separate opaque integer surfaced as-is per the parser in [`crates/ie-formats/src/bcs.rs`](../crates/ie-formats/src/bcs.rs). Field naming is fine as-is — no cleanup needed.

## Acceptance criteria for the fix

- [x] `iecli dump --resource RASAAD.BCS` on the user's BG:EE install completes successfully and produces JSON with `name` populated for the action opcodes (e.g. `"name": "SetGlobal"`, `"name": "GiveItem"`).
- [x] Unit tests updated to use real-BCS-shaped fixtures; old fixtures removed.
- [x] `iecli dump --resource IMOEN.BCS` and `iecli dump --resource BALDUR.BCS` both exit 0 — verified on the user's BG:EE install after the parser fixes.
- [x] The unknown `leading` int in the action header is surfaced in JSON (do not silently drop data).
- [x] Decode `RASAAD.BCS` and verify the output contains a block with a `Global(..., RASAAD_PLOT, 1)` trigger paired with a `SetGlobal(..., RASAAD_PLOT, ...)` action, all with fully resolved names. Verified as block 18: triggers include `Global`, `GlobalTimerExpired`, `AreaCheckObject`, `OR`, `Range`, `Dead` — all name-resolved. (Note: the block's action sets `RASAAD_PLOT` to 0 — a reset path. Forward advancement of `RASAAD_PLOT` lives in DLG `action_text`, not BCS, which matches what the dialog investigation already showed.)
- [x] (Bug 3) Trigger `name` fields are populated in `RASAAD.BCS` output: at minimum `Global`, `GlobalTimerExpired`, and one AreaCheck variant resolve. Verified — 204 triggers, 0 with `"name": null`; named opcodes include `Global`, `GlobalTimerExpired`, `AreaCheck`, `AreaCheckObject`, `InParty`, `Dead`, `OR`, etc.
