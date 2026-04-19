# iecli

`iecli` is a CLI-first, Rust-based Infinity Engine inspection tool.

The project is aimed at trustworthy extraction rather than GUI editing. The goal is to read installed game resources,
resolve them through the same archive and override rules the games use, and export stable machine-readable output that
is useful for:

- inspection
- scripting
- diffing
- AI-assisted analysis
- future patch-generation workflows

## Status

The Rust rewrite is in progress. The current workspace already supports:

- game root validation via `chitin.key`
- Enhanced Edition `lang/<locale>/dialog.tlk` discovery
- `dialog.tlk` string lookup with out-of-range handling
- typed `chitin.key` parsing
- resource lookup from:
  - `override`
  - KEY-backed BIFF mappings
- byte extraction from:
  - `BIFF`
  - `BIF`
  - `BIFC`
- `dump-raw` export for located resources
- source-selectable lookup for `locate`, `dump`, and `dump-raw` via `--source`
- resource enumeration with `list`
- typed JSON export for:
  - `ITM`
  - `SPL`
  - `CRE`
  - `STO`
  - `DLG`
  - `BCS`

Not implemented yet:

- real-resource fixture coverage for decoded formats
- broad Near Infinity comparison coverage for `ITM` and `SPL`
- JSON golden/snapshot coverage for decoded formats

Current validation for decoded formats already includes:

- real-install `dump` smoke coverage for `ITM` and `SPL`
- manual Near Infinity comparison for selected BG2EE `SPL` resources
- env-gated real-install regression tests for validated `SPL` resources

## Workspace

The Rust workspace is organized into:

- `crates/ie-core`
- `crates/ie-io`
- `crates/ie-formats`
- `crates/ie-cli`

Near Infinity remains a behavioral reference for format validation, but the legacy Java snapshot is no longer vendored
in this repository. See [Near Infinity Reference](./docs/NEAR_INFINITY_REFERENCE.md) for the expected workflow.

## Acknowledgement

This tool has been developed with Near Infinity as an important reference for:

- resource-loading behavior
- binary format interpretation
- validation of parser output against real game resources

`iecli` is a separate Rust implementation with a CLI-first product direction. It is not a Near Infinity fork, but it
does explicitly use Near Infinity as a comparison target when behavior needs to match established engine-facing tooling.

## Build

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Show CLI help:

```bash
cargo run -p iecli -- --help
```

## Current Commands

```bash
iecli locate --game /path/to/game --resource ACIDBL.ITM
iecli locate --game /path/to/game --resource KIRINH.CRE --source bif
iecli dump-raw --game /path/to/game --resource ACIDBL.ITM --output ./ACIDBL.ITM
iecli dump-raw --game /path/to/game --resource KIRINH.CRE --source bif --output ./KIRINH-stock.CRE
iecli dump --game /path/to/game --resource ACIDBL.ITM --format json
iecli dump --game /path/to/game --resource SPWI112.SPL --format json
iecli dump --game /path/to/game --resource BALDUR.BCS --format json
iecli list --game /path/to/game --type CRE --name "kirin*"
iecli list --game /path/to/game --type ITM --source override --format json
iecli tlk --game /path/to/game --strref 1
```

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [Todo priorities](./TODO_PRIORITIES.md)
- [Testing notes](./docs/TESTING.md)

## Notes

This is a new tool with its own architecture and scope. Near Infinity remains part of the validation workflow, but the
Rust workspace does not depend on the legacy Java source being present in this repository.
