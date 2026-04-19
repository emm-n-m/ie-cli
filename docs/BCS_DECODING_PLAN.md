# BCS Decoding — Implementation Record

Completed on 2026-04-19. This document now serves as a historical implementation note and reference for follow-up validation work. It is no longer an active handoff plan.

## Goal

Add typed `BCS` decoding and JSON export to `iecli`, so `iecli dump --resource FOO.BCS` returns a human-readable script structure with opcode names (not numeric codes). This work is now implemented and unblocks trigger-graph analysis of NPC behavior (companion quests, area spawn scripts, global state machines).

The driving use case: diagnosing *"my quest isn't advancing"* problems where a variable is the wrong value. With decoded BCS we can answer "what trigger block bumps this global from N to N+1" — currently only answerable by opening the file in Near Infinity.

## Current state of the codebase

- Format decoders live in [crates/ie-formats/src/](../crates/ie-formats/src/). Existing decoders: `itm.rs`, `spl.rs`, `cre.rs`, `sto.rs`, `dlg.rs`. **Use `dlg.rs` and `sto.rs` as style templates** — they are the cleanest references for the expected shape (top-level JSON struct, `RawDecodedFlags`/`RawDecoded<T>` helpers, `parse_table` utility, unit test with a synthetic buffer).
- The dispatcher lives in [crates/ie-formats/src/lib.rs](../crates/ie-formats/src/lib.rs) and now routes `ResourceType::Bcs` to the real parser.
- `ResourceType::Bcs` is already recognized in [crates/ie-core/src/lib.rs](../crates/ie-core/src/lib.rs) — nothing to add there.
- StrRef resolution uses the `StrRefResolver` trait (see `crates/ie-core/src/lib.rs`). BCS does **not** reference strrefs directly, so no resolver is needed for BCS — but an **IDS resolver** is needed for opcode names.
- IDS resources (`TRIGGER.IDS`, `ACTION.IDS`, `OBJECT.IDS`, `GENDER.IDS`, etc.) ship in the game BIFs. Confirm with `iecli list --type ids`. They are plain text.
- CLI entry: [crates/ie-cli/src/main.rs](../crates/ie-cli/src/main.rs). The `dump` subcommand already routes to `decode_to_json`; once BCS is wired into `decode_to_json`, `dump` works for BCS automatically.

## Scope

### Delivered
1. **BCS parser** — consumes ASCII BCS, emits typed IR.
2. **IDS parser** — reads `TRIGGER.IDS` / `ACTION.IDS` / `OBJECT.IDS` (and any supporting IDS files) into lookup maps.
3. **IDS resolver trait** — passed into `parse_bcs` by the CLI, analogous to `StrRefResolver`. When absent (unit tests, synthetic inputs), decoded names fall back to `None` and the JSON shows the raw opcode.
4. **Resolver plumbing** — refactor `decode_to_json` so it can receive both TLK and IDS capabilities without special-casing BCS in the CLI. Prefer a small resolver bundle/context struct over adding more unrelated positional parameters.
5. **CLI wiring** — `dump --resource X.BCS` produces decoded JSON. CLI constructs an IDS resolver from the game installation and threads it through.
6. **Tests** — unit tests with a synthetic BCS buffer (no real-install dependency), covering: trigger/action opcode lookup, object specifier decoding, response-weight handling, nested block structure, negated triggers, malformed input.
7. **Real-install smoke coverage** — env-gated tests for 2-3 real BCS resources with structural assertions only (no committed copyrighted dumps).
8. **README update** — remove BCS from the "Not implemented yet" list; add to the typed JSON export list.

### Still out of scope / follow-ups
- `iecli ids <trigger|action|object> <N>` subcommand for spot-checking opcodes. Valuable but separable.
- Pretty-printing decoded BCS back to BAF source form.
- BAF → BCS compilation.

## BCS file format

BCS is **plain ASCII text** despite being called "compiled." Each block is delimited by a 2-letter tag on its own line (or occasionally inlined at end of line). The same tag opens and closes the block.

### Block hierarchy

```
SC                         <- script root (opens)
  CR                       <- condition-response pair (opens an IF/THEN)
    CO                     <- condition block (the IF part: AND-chain of triggers)
      TR                   <- single trigger record (opens)
        <trigger payload>
      TR                   <- trigger closes
      TR ... TR            <- more triggers (AND)
    CO                     <- condition closes
    RS                     <- response set (opens; contains weighted responses)
      RE                   <- single weighted response (opens)
        <weight int>
        AC                 <- action record (opens)
          <action payload>
          OB               <- object specifier for action target
            <object payload>
          OB               <- object closes
        AC                 <- action closes
        AC ... AC          <- more actions in this response
      RE                   <- response closes
      RE ... RE            <- alternative weighted responses
    RS                     <- response set closes
  CR                       <- condition-response closes
  CR ... CR                <- more condition-response pairs
SC                         <- script closes
```

Semantically: a script is a list of `IF (all triggers in CO) THEN (pick one RE by weight, execute its actions)` blocks.

### Trigger record (inside `TR ... TR`)

```
TR
<opcode:i32> <int_arg1:i32> <int_arg2:i32> <int_arg3:i32> <int_arg4:i32> "<string_arg1>" "<string_arg2>" OB
<object specifier: 13 integers + 1 quoted string> OB
TR
```

- First line: opcode (looked up in `TRIGGER.IDS`) + 4 integer args + 2 string args + an opening `OB` marker (sometimes inline at end of line with no leading space).
- Second line: the object specifier (target of the trigger, e.g. `Myself`, `Player1`, `[ENEMY]`) — 13 integers followed by a single quoted string, terminated by `OB`.
- Opcode sign: a **negative** opcode means the trigger is negated (`!Global(...)`).
- One trigger has **one** object specifier. Do not attempt to support multi-object triggers.

### Action record (inside `AC ... AC`)

```
AC
<opcode:i32> <int_arg1:i32> <int_arg2:i32> <int_arg3:i32> <int_arg4:i32> "<string_arg1>" "<string_arg2>"
OB
<object specifier 1>
OB
OB
<object specifier 2>
OB
OB
<object specifier 3>
OB
<point_x:i32> <point_y:i32>
AC
```

- Opcode looked up in `ACTION.IDS`.
- Actions always carry **3 object specifiers** (not 1 like triggers) plus a final point `(x, y)`. Most specifiers are zero-filled when unused.
- Integer args and string args are analogous to triggers.

### Object specifier payload

13 integers + 1 quoted string. Slots (per IESDP, EE variant):

| Index | Meaning          | IDS file     |
|-------|------------------|--------------|
| 0     | EA               | EA.IDS       |
| 1     | General          | GENERAL.IDS  |
| 2     | Race             | RACE.IDS     |
| 3     | Class            | CLASS.IDS    |
| 4     | Specific         | SPECIFIC.IDS |
| 5     | Gender           | GENDER.IDS   |
| 6     | Alignment        | ALIGN.IDS    |
| 7     | Identifier       | OBJECT.IDS   |
| 8–12  | IDS target slots | (optional)   |
| str   | Name / deathvar  | —            |

Zero values mean "any" / unused and should be suppressed in JSON output (do not emit `"ea": "ANYONE"` for every object; show only non-zero fields).

For slots `8..=12`, keep the raw integers unconditionally and document them as unresolved target slots unless a later validation pass proves stable semantics. Do not invent names for them in this milestone. If they are surfaced beyond `raw`, use a neutral field such as `extra_targets`.

### IDS file format

IDS files are plain text. Two common shapes in EE:

```
IDS
V1.0
<count or 0>
<value> <name>
<value> <name>
...
```

or:

```
<count>
<value> <name>
<value> <name>
...
```

Some IDS files use hexadecimal values (prefixed `0x`) and some use decimal. Some have a single-line header that is not a value mapping. **Parser must tolerate both**. Each entry is `<integer> <whitespace> <name>`; the name may contain parentheses and signature digits (`Global(S:Name*,S:Area*,I:Value*)`). Extract only the bare function name (`Global`) for the JSON output — strip the signature.

Negative/signed values in IDS are common (e.g. in OBJECT.IDS: `-1 LastAttackerOf(O:Object*)`). Treat values as signed 32-bit.

## Suggested code layout

```
crates/ie-io/src/ids.rs              (new — IDS file reader)
crates/ie-formats/src/bcs.rs         (new — BCS parser, matches dlg.rs style)
crates/ie-formats/src/lib.rs         (edit — register BCS arm)
crates/ie-core/src/lib.rs            (edit — add IdsResolver trait alongside StrRefResolver)
crates/ie-cli/src/main.rs            (edit — build IdsResolver for dump)
```

`FileBackedIdsResolver` should live in `ie-io`, next to the IDS reader and `ResourceLocator` integration. `ie-cli` should only construct it and pass it into the format layer.

## Proposed traits / types

### `IdsResolver` trait (in `ie-core`)

```rust
pub trait IdsResolver {
    fn resolve_trigger(&self, opcode: i32) -> Option<String>;
    fn resolve_action(&self, opcode: i32) -> Option<String>;
    fn resolve_ids(&self, file: &str, value: i32) -> Option<String>;
}
```

- `resolve_ids("EA", 2)` → `Some("PC")` — generic lookup for object specifier fields.
- Accept signed inputs; negative opcodes (negated triggers) should be looked up by absolute value at the parser layer, not in the resolver.
- When no resolver is threaded through, `parse_bcs` must still succeed and populate `name: None` + `raw: <opcode>`.

### Resolver bundle

The current `decode_to_json` signature only accepts `Option<&dyn StrRefResolver>`. BCS is the first format that needs a second resolver family, so update the boundary explicitly instead of bolting on BCS-specific CLI logic.

Preferred shape:

```rust
pub struct ResolverBundle<'a> {
    pub strref: Option<&'a dyn StrRefResolver>,
    pub ids: Option<&'a dyn IdsResolver>,
}
```

Then update `decode_to_json` and per-format entry points to take either `&ResolverBundle` or the relevant optional members. Existing formats should continue using only `strref`; BCS should use only `ids`.

### BCS JSON shape

Follow the `dlg.rs` pattern. Sketch:

```rust
pub struct BcsJson {
    pub resource_type: String,
    pub resource_name: String,
    pub blocks: Vec<BcsConditionResponseJson>,
}

pub struct BcsConditionResponseJson {
    pub triggers: Vec<BcsTriggerJson>,    // the AND-chain from CO
    pub responses: Vec<BcsResponseJson>,  // from RS
}

pub struct BcsTriggerJson {
    pub opcode: i32,                       // raw (signed; negative = negated)
    pub name: Option<String>,              // from TRIGGER.IDS, without signature
    pub negated: bool,
    pub int_args: [i32; 4],
    pub string_args: [String; 2],
    pub object: BcsObjectJson,
}

pub struct BcsActionJson {
    pub opcode: i32,
    pub name: Option<String>,
    pub int_args: [i32; 4],
    pub string_args: [String; 2],
    pub objects: [BcsObjectJson; 3],
    pub point: (i32, i32),
}

pub struct BcsResponseJson {
    pub weight: i32,
    pub actions: Vec<BcsActionJson>,
}

pub struct BcsObjectJson {
    pub raw: [i32; 13],
    pub name: Option<String>,              // the quoted trailing string (deathvar / resref)
    pub decoded: BcsObjectDecodedJson,     // only non-zero fields populated
}

pub struct BcsObjectDecodedJson {
    pub ea: Option<String>,
    pub general: Option<String>,
    pub race: Option<String>,
    pub class: Option<String>,
    pub specific: Option<String>,
    pub gender: Option<String>,
    pub alignment: Option<String>,
    pub identifier: Option<String>,
    pub extra_targets: Option<[i32; 5]>,
}
```

Include both raw and decoded at every level so the output is useful without a resolver and diffable between installs. `extra_targets` should only be populated when any of slots `8..=12` are non-zero.

## Parsing strategy

The BCS tokenizer is simple: split on whitespace, but preserve quoted strings as single tokens (strings may contain spaces). Rough plan:

1. **Tokenize** — one pass producing `Vec<Token>` where `Token` is either `Tag(&str)` (2-letter block tag), `Int(i32)`, or `Str(String)`.
2. **Parse recursively** — `parse_sc`, `parse_cr`, `parse_co`, `parse_tr`, `parse_rs`, `parse_re`, `parse_ac`, `parse_ob`. Each consumes its opening tag, parses its payload, consumes its closing tag, returns the struct.
3. **Object specifier lookup** — during `parse_tr` / `parse_ac`, use the resolver (if present) to populate decoded fields. Suppress zero slots.

Each token should also carry byte offset and line/column information. This is not optional busywork: textual BCS without source positions yields poor parser diagnostics and makes malformed-script debugging unnecessarily slow.

### Gotchas

- Tags can appear **inline at the end of a line** — e.g. `""OB` closes an object with an empty string name. The tokenizer must handle this cleanly.
- **Some scripts have `OB` immediately followed by another `OB`** when an action doesn't use a slot. Don't assume a newline between adjacent tags.
- **Negative opcodes** for triggers: store `negated: true` and resolve name by absolute value. Action opcodes don't use this convention.
- **Action point `(x, y)`** is at the end of the action, *after* the three objects, **not** inside any OB. Don't conflate it with object payload.
- **Weight on RE** is an integer immediately after `RE` opening. Do not mistake it for an opcode.

### Error model

Return typed parse errors with source position where possible:

```rust
pub struct BcsSourcePos {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
}
```

Examples of good failures:

- `unexpected token "AC" while parsing trigger object at line 42, column 1`
- `unterminated quoted string at line 10, column 17`
- `expected closing TR before end of file`

## Test plan

Mirror `dlg.rs`: one integration-ish unit test that builds a synthetic BCS string in-memory and verifies the parse. Cover:

1. **Single IF/THEN**: one trigger, one response, one action, both resolvers present. Assert `name` is populated.
2. **Negated trigger**: negative opcode → `negated: true`, `name` resolved from absolute value.
3. **Multiple triggers in one CO**: AND-chain preserved in order.
4. **Weighted responses**: two `RE` blocks under one `RS`, each with its weight.
5. **No resolver**: parse succeeds with `name: None` everywhere.
6. **Object specifier**: non-zero slot emits decoded field; zero slot stays `None`.
7. **Malformed**: truncated `TR` without closing tag returns a typed parse error (not panic).

Use the style from `dlg.rs` tests: `struct NullResolver; impl IdsResolver for NullResolver { ... }` for the no-resolver case and a small `TestResolver` with hard-coded entries for the positive cases.

IDS parsing needs its own tests in `ie-io` — minimal: parse a synthetic IDS string with both header shapes (with and without `IDS V1.0` line), hex and decimal entries, signatures stripped correctly.

Add one env-gated smoke test module for real installs:

1. `IMOEN.BCS` or another small stock script parses successfully and returns at least one block.
2. `BALDUR.BCS` or another large script parses successfully and resolves at least one known trigger/action name.
3. One query-oriented assertion proves the output shape is useful, e.g. at least one block contains an action named `SetGlobal`.

Keep these tests structural. Do not commit full JSON dumps or long copyrighted script text; follow [docs/REGRESSION_PLAN.md](REGRESSION_PLAN.md).

## CLI wiring

In [crates/ie-cli/src/main.rs](../crates/ie-cli/src/main.rs), the `dump` handler constructs a `StrRefResolver` from the installation's `dialog.tlk`. Do the analogous thing for IDS:

1. Build a cache struct `FileBackedIdsResolver` that lazily loads IDS files from the game installation on first lookup (via the existing `ResourceLocator`).
2. Introduce `ResolverBundle` in `ie-core` and thread that bundle into `decode_to_json`.
3. In `dump`, construct TLK and IDS resolvers independently. Do not require `dialog.tlk` to exist just to dump a `BCS`.

Prefer **lazy loading** — TRIGGER.IDS alone is ~4000 entries. Only load files that are actually queried during a given parse.

## Completion status

- [x] `cargo test --workspace` passes, including unit tests in `ie-io::ids` and `ie-formats::bcs`.
- [x] Env-gated real-install smoke coverage exists for BCS when the relevant game path variable is set.
- [x] `cargo build`/`cargo test` wiring for `iecli dump --resource *.BCS` is in place.
- [x] `README.md` moved `BCS` into the typed JSON export list.
- [x] Shared resolver plumbing (`ResolverBundle`, `IdsResolver`, lazy file-backed IDS loading) is implemented.

Remaining follow-up:

- broaden real-install validation beyond the current smoke checks
- compare representative scripts against Near Infinity and encode the findings in assertions
- decide whether script-adjacent follow-up work should add pretty-printing, diffing, or cross-reference graph output

## Reference resources for smoke-testing

- `RASAAD.BCS` (override on the user's BG:EE install) — dense, heavily modded, will exercise many opcodes.
- `IMOEN.BCS` (BIF, stock) — small and predictable; a good minimal sanity check.
- `BALDUR.BCS` (BIF, stock) — the global script; large and touches many IDS files.

After implementation, running the tool against `RASAAD.BCS` and reading the JSON should produce a concrete answer to: *"Which block in RASAAD.BCS advances `RASAAD_PLOT` from 1 to 2, and what are its trigger conditions?"* That is the headline test of success.
