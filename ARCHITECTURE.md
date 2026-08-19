# Architecture

This document states design intent and the boundaries to preserve. It is written in "should" form and
stays stable; for what is actually built today, see [ROADMAP.md](./ROADMAP.md) and
[docs/PARSER_COVERAGE.md](./docs/PARSER_COVERAGE.md).

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

## Workspace Layout

```text
crates/
  ie-core/        # shared types, errors, resource ids
  ie-io/          # installation discovery, KEY/BIF/TLK/DLC loading
  ie-formats/     # typed decoders and scoped write paths
  ie-cli/         # command frontend, one module per concern
skills/           # canonical agent skills (mirrored to .claude/skills/)
scripts/          # repo maintenance tooling
docs/
```

Tests live in `crates/<crate>/tests/`, with committed goldens under `tests/goldens/` and
real-resource expectations under `crates/ie-cli/tests/expectations/`. There is no top-level
`tests/` directory and no committed game data. The four crates are already split; keep the
boundaries rather than merging or re-cutting them mid-feature.

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
- `DLG`
- `BCS`
- `ARE`
- `CHR` (a wrapper around a complete embedded `CRE`)
- `GAM` / `SAV` (save containers)

Each format should have:

- a raw parser
- typed structs
- JSON-ready export structs or serializers
- format-specific tests

Cross-cutting modules also live here where they operate on decoded resources rather than bytes:
`effects` (variant-aware opcode tables) and `verify` (ARE cross-resource integrity checks).

Scoped write paths (`patch_cre_scalars`, `patch_chr_scalars`, `patch_are_scalars`,
`add_item_to_save_gam`) belong to this
crate too. They must preserve every byte outside the declared edit, repair only an explicit offset
set, and refuse layouts they have not been validated against.

### `ie-cli`

Responsible for user-facing commands:

- `locate`
- `dump-raw`
- `dump`
- `list`
- `tlk`, `tlk-append`
- `patch`
- `override-diff`
- `verify`
- `save-list`, `save-info`, `save-add-item`

It should not contain parsing logic beyond argument handling and output formatting. As the command
surface grew, `main.rs` was split into per-concern modules (`dialog_graph`, `override_diff`,
`patch_input`, `resource_links`, `save_support`, `verify_command`); new commands should follow that
shape rather than accreting in `main.rs`.

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

Implemented shape: `GameVariant` (`standard` / `iwd` / `pst`) is detected once at discovery from stable
root files rather than folder names, rides on `ResourceMetadata`, and is consumed at the specific points
that diverge — currently effect-opcode decoding and PST inventory slot maps. Add new variant branches at
that granularity; do not thread per-game conditionals through parsers wholesale.

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

Tests exist at four levels. See [docs/TESTING.md](./docs/TESTING.md) for how to run them and
[docs/GOLDENS.md](./docs/GOLDENS.md) for the two golden tiers.

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

- full command execution against a synthetic install
- override precedence
- TLK resolution in exported JSON

### Golden and expectation tests

For:

- exact-value goldens over synthetic fixtures, covering every decoded format and every
  human-readable output mode, running in CI
- normalized JSON *shape* goldens checked against real installs, which pin structure without
  committing values
- real-resource expectations with stated provenance
- deterministic adversarial-input coverage proving no decoder panics

## Fixture Policy

No game data is committed. `cargo test --workspace` must pass on a clean checkout with no
installs and no environment variables set, so the default suite runs on synthetic fixtures and a
synthetic installation. Real installs are reached only through env-gated tests that no-op when
their variable is unset — see [docs/TESTING.md](./docs/TESTING.md).

Real-resource facts are recorded as expectations rather than checked-in files. Each case should
document:

- source game and install (its provenance)
- original resource name
- why it exists
- what edge or behavior it covers

Keep expectations to individual non-localized facts, never whole dumps. Do not avoid weird
real-world files — reach them through an env-gated test.

## Format Authority

[IESDP](https://gibberlings3.github.io/iesdp/) is the layout authority: where each field sits,
how wide it is, what its bits and enums mean. It is not a data source — what any particular
resource contains can only be read from the file, so expected values always come from real
resources across several installs. When a real file's layout disagrees with IESDP, the file wins.
Neither dictates architecture: decode from the spec, then shape the output for this tool's
consumers.

See [docs/FORMAT_REFERENCES.md](./docs/FORMAT_REFERENCES.md) for the workflow.

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

Minimal command surface (all shipped):

- `locate`
- `dump-raw`
- `dump`
- `tlk`
- `list`

Later:

- `diff` for decoded resources
- `search`

Shared flags should stay shared: `--game` and `--resource` mean the same thing everywhere, and
`--source auto|override|bif` / `--skip-override` apply uniformly to lookup-based commands.

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

## Success Criteria

The architecture is working if the project can reliably:

- locate `VICONI.CRE`
- dump an `ITM` as stable JSON
- resolve `strref` text from `dialog.tlk`
- prove override precedence with a test
- pin that output with a real-resource expectation that states its provenance

That is enough to make the project useful before it becomes large.
