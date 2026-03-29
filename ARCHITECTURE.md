# Architecture

## Objective

Build a Rust-based, CLI-first Infinity Engine inspection tool that can:

- locate installed game resources
- read resources from `override`, `KEY`, and `BIF`
- resolve `dialog.tlk` strings
- decode selected binary formats
- export stable JSON for humans, scripts, and AI agents

The design should optimize for correctness, testability, and gradual expansion across Infinity Engine games such as BGEE, BG2EE, EET, IWD, and PST.

## System Shape

The system should be divided into a few clear layers:

1. Environment discovery
2. Resource lookup and byte loading
3. Format decoding
4. Text resolution
5. Serialization and CLI presentation

Each layer should have a narrow responsibility.

## Recommended Workspace Layout

```text
crates/
  ie-core/
  ie-io/
  ie-formats/
  ie-cli/
tests/
  fixtures/
docs/
```

If the project is still small, `ie-core` and `ie-io` can begin in the same crate and be split later.

## Crate Responsibilities

### `ie-core`

Holds shared domain types:

- `GamePath`
- `ResourceType`
- `ResRef`
- `StrRef`
- shared error types
- common binary parsing helpers
- source metadata types

This crate should stay small and dependency-light.

### `ie-io`

Responsible for reading the game installation:

- locate `chitin.key`
- locate language folder and `dialog.tlk`
- resolve override precedence
- parse `KEY`
- locate and read from `BIF`
- return raw resource bytes with metadata

This crate should not know format-specific details like the structure of `CRE` or `ITM`.

### `ie-formats`

Responsible for typed decoding of resource bytes:

- `ITM`
- `SPL`
- `CRE`
- `STO`
- later `DLG`
- later `BCS`

Each format should have:

- a raw parser
- typed structs
- JSON-ready export structs or serializers
- format-specific tests

### `ie-cli`

Responsible for user-facing commands:

- `locate`
- `dump-raw`
- `dump`
- `tlk`
- later `diff`

It should not contain parsing logic beyond argument handling and output formatting.

## Data Flow

Expected request path:

1. CLI receives `--game` and `--resource`
2. loader resolves where the resource lives
3. raw bytes are returned with metadata
4. format decoder parses bytes into typed structures
5. TLK resolver enriches `strref` fields if needed
6. serializer emits stable JSON

Keep these steps separately testable.

## Core Abstractions

### `ResourceLocator`

Purpose:

- resolve the winning source for a resource
- report whether the source is `override`, `bif`, or loose file

Expected output:

- resource name
- resource type
- physical source
- path or archive location

### `ResourceReader`

Purpose:

- read bytes from the resolved source

Should return:

- raw bytes
- source metadata

### `FormatDecoder`

Purpose:

- turn bytes into typed resources

Avoid giant enum-driven parser functions. Prefer per-format modules with a shared trait only if it helps, for example:

```rust
trait DecodeResource {
    type Output;
    fn decode(bytes: &[u8]) -> Result<Self::Output, DecodeError>;
}
```

### `TlkResolver`

Purpose:

- resolve `StrRef` into localized text

Keep resolution separate from parsing. Parsed resources should still be meaningful without live text resolution.

## Output Model

Use explicit exported structures rather than serializing internal parser structs directly.

Reason:

- internal structs often reflect offsets and parsing concerns
- exported structs should reflect inspection concerns
- the public JSON shape should remain stable even if parser internals evolve

Recommended output shape:

- raw ids preserved
- decoded labels where available
- nested structures named clearly
- unknown values kept visible

## Binary Parsing Strategy

Use a parsing style that makes offsets and field widths obvious.

Requirements:

- little-endian helpers
- fixed-width string/resref helpers
- strict bounds checking
- clear errors on truncated input

Avoid “parse by side effect” code that mutates many cursors implicitly.

## Resource-Type Strategy

Implement one format at a time.

Suggested order:

1. `ITM`
2. `SPL`
3. `CRE`
4. `STO`
5. `DLG`
6. `BCS`

Rationale:

- `ITM` and `SPL` are structurally rich but still manageable
- `CRE` and `STO` unlock practical modding and party-analysis use cases
- `DLG` is high value but larger in scope
- `BCS` should wait until its exported representation is agreed

## String Resolution

String references should be modeled as explicit value objects, not plain integers.

Suggested model:

```rust
struct ResolvedStrRef {
    strref: u32,
    text: Option<String>,
}
```

This avoids losing the original id and makes unresolved strings explicit.

## Game Variants

Do not hardcode assumptions for a single title.

The design should tolerate:

- different resource populations
- different language layouts
- title-specific quirks
- Enhanced Edition vs older layout differences when relevant

Variant-specific behavior should live in documented decision points, not spread through the codebase.

## Error Model

Errors should be actionable.

Prefer:

- `resource not found`
- `unsupported resource type`
- `invalid KEY entry offset`
- `truncated CRE header`

Over vague generic failures.

Preserve enough context to debug:

- game path
- resource name
- source kind
- format type
- failing field or offset if known

## Testing Strategy

Tests should exist at three levels:

### Unit tests

For:

- fixed-width readers
- endian helpers
- resref parsing
- TLK field parsing

### Fixture tests

For:

- real resources from installed games
- parser correctness
- JSON stability

### Integration tests

For:

- full command execution against fixture installs
- override precedence
- TLK resolution in exported JSON

## Fixture Policy

Prefer real resources over synthetic-only inputs.

Each fixture should document:

- source game
- original resource name
- why it exists
- what edge or behavior it covers

Use small fixtures where possible, but do not avoid weird real-world files.

## Near Infinity As Reference

Near Infinity should be used as:

- a format reference
- a behavior oracle for edge cases
- a comparison target

It should not dictate architecture.

If a parser rewrite is cleaner and still behaviorally compatible, prefer the cleaner rewrite.

## Serialization Rules

Default to JSON for all machine-readable output.

Requirements:

- deterministic field ordering where feasible
- no debug-only internals
- stable naming
- support compact and pretty output

Do not leak parser offsets unless they are intentionally exposed.

## CLI Design

Prefer explicit subcommands.

Minimal command surface:

- `locate`
- `dump-raw`
- `dump`
- `tlk`

Later:

- `diff`
- `list`
- `search`

CLI flags should be stable and unsurprising.

## Extension Points

Future growth should be possible in these directions:

- new resource formats
- diffing
- patch planning support
- machine-readable cross-reference graphs
- optional writeback tooling
- optional GUI or TUI frontends

Do not build those now, but avoid blocking them accidentally.

## Architectural Boundaries To Preserve

- `ie-cli` must not parse binary formats directly
- `ie-io` must not know game-mechanics semantics of `ITM` or `CRE`
- `ie-formats` must not depend on terminal output concerns
- text resolution must remain swappable

## Initial Success Criteria

The architecture is working if the project can reliably:

- locate `VICONI.CRE`
- dump an `ITM` as stable JSON
- resolve `strref` text from `dialog.tlk`
- prove override precedence with a test
- compare output meaningfully against Near Infinity

That is enough to make the project useful before it becomes large.
