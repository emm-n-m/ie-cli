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
- basic `dialog.tlk` string lookup
- `chitin.key` parsing
- resource lookup from:
  - `override`
  - KEY-backed BIFF mappings

Not implemented yet:

- BIFF/BIF/BIFC byte extraction
- typed `ITM`, `SPL`, `CRE`, `STO`, or `DLG` decoding
- stable JSON export for decoded resources

## Workspace

The Rust workspace is organized into:

- `crates/ie-core`
- `crates/ie-io`
- `crates/ie-formats`
- `crates/ie-cli`

Near Infinity remains a behavioral reference for format validation, but the legacy Java snapshot is no longer vendored
in this repository. See [Near Infinity Reference](./docs/NEAR_INFINITY_REFERENCE.md) for the expected workflow.

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
iecli tlk --game /path/to/game --strref 1
```

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [Todo priorities](./TODO_PRIORITIES.md)
- [Testing notes](./docs/TESTING.md)

## Notes

This is a new tool. It is not a Near Infinity fork in product direction, even though the old Near Infinity codebase is
being used as a reference implementation for format and loader behavior.
