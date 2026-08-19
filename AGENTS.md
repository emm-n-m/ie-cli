# IE-CLI Agent Guide

## Purpose

This document gives coding agents a strict working guide for building a CLI-first, Rust-based Infinity Engine resource inspector.

For current project status, active milestones, and the driving use case, see [ROADMAP.md](./ROADMAP.md). This guide is stable; the roadmap changes each session.

The first goal is a reliable parser and exporter that can turn installed game resources into stable, machine-readable output for:

- human inspection
- AI-assisted analysis
- diffing
- scripting
- future WeiDU patch generation workflows

[IESDP](https://gibberlings3.github.io/iesdp/) is the specification reference for format work.
When it is ambiguous, real game resources decide — not another implementation. See
[docs/FORMAT_REFERENCES.md](./docs/FORMAT_REFERENCES.md).

## Game Exploration Skills

This repository ships the same game-exploration skills to more than one coding agent, so
contributors can work with the agent of their choice. [`skills/`](./skills/) holds the
canonical Codex / Open Agent Skills packages; [`.claude/skills/`](./.claude/skills/) is a
generated Claude Code mirror of them. See [docs/SKILLS.md](./docs/SKILLS.md) for the full
inventory and the parity contract.

When a request matches one of the workflows below, read that skill's `SKILL.md` completely
before acting and follow its bundled scripts and reporting guidance:

- `diagnose-dialog` — explain a missing or fall-through NPC dialog branch
- `explore-dungeon` — walk an ARE graph and find broken, one-way, or orphaned areas
- `map-stat-gates` — inventory dialogue stat checks and rank their actual payoffs
- `mod-diff` — summarize resources added or changed by a mod
- `plan-stat-build` — synthesize an install-aware protagonist stat plan
- `trace-quest-timer` — find a quest timer's duration, time base, and firing conditions

Codex normally discovers repository skills through `.agents/skills`. In environments where
that directory is writable, point it at this repository's canonical packages with
`ln -s ../skills .agents/skills`. The explicit routing above is the fallback for managed
environments that mount `.agents` read-only. Claude Code discovers `.claude/skills/`
directly, with no setup.

Edit `skills/` only. After changing a skill, regenerate the mirror with
`python scripts/skill_parity.py --sync` and commit both trees; CI fails the build when they
drift apart.

## Product Direction

Build a tool that:

- reads Infinity Engine installations directly
- locates resources from `override`, `KEY`, and `BIF`
- resolves `dialog.tlk` string references
- exports selected resource types as stable JSON
- favors correctness, determinism, and inspectability over breadth

This tool should be useful even if it never grows a GUI.

## Non-Goals

Do not spend early effort on:

- GUI work
- in-place editing or writeback
- supporting every resource format
- broad architectural rewrites without a live milestone
- clever abstractions that are not needed by current formats

## Engineering Principles

- Language: Rust.
- CLI first.
- Parser core should be usable as a library.
- Output must be deterministic.
- Prefer explicit structs over untyped maps.
- Preserve raw values when possible.
- Do not silently discard unknown bytes or fields.
- Keep parsing, loading, and rendering separate.
- Ship narrow vertical slices instead of horizontal half-implementations.

## Repository Layout

The workspace is established; match it rather than proposing a new shape.

```text
crates/
  ie-core/        # shared types, errors, resource ids, common helpers
  ie-io/          # installation discovery, KEY/BIF/TLK/DLC loading
  ie-formats/     # format decoders: ITM, SPL, CRE, STO, DLG, BCS, ARE
  ie-cli/         # command-line frontend, one module per concern
skills/           # canonical agent skills (mirrored to .claude/skills/)
scripts/          # repo maintenance tooling
docs/
  formats/        # per-format notes
  decisions/      # <date>-<topic>.md decision records
  guides/         # narrative output produced by the skill layer
```

Tests live beside the crate they exercise, in `crates/<crate>/tests/`, with committed
goldens under `crates/ie-cli/tests/goldens/` and `crates/ie-formats/tests/goldens/`, and
real-resource expectations under `crates/ie-cli/tests/expectations/`. There is no
top-level `tests/` directory and no checked-in game data.

Only split a crate further when a boundary is real, not speculative.

## Core Domain Model

Favor a layered model:

1. `ResourceLocator`
   Resolves where a resource comes from.

2. `ResourceBytes`
   Raw bytes plus metadata:
   - source path
   - source kind: override, bif, loose file
   - resource type
   - resource name

3. `DecodedResource`
   Typed parsed resource.

4. `RenderableJson`
   Stable exported form used by CLI output.

Do not mix “bytes loading” with “field decoding” with “JSON presentation”.

## Resource Resolution Rules

Agents should implement and preserve these assumptions unless tests prove otherwise:

- override resources shadow container resources
- resolution order is `override` → KEY-backed BIFF → read-only DLC archives (`dlc/*.zip`),
  each DLC carrying a KEY of its own; `--source <auto|override|bif>` opts out
- lookup should be case-insensitive where engine behavior is case-insensitive
- **normalize `resource_name` in output, and keep the on-disk spelling in `source_path`.**
  Override files match case-insensitively, so casing is the only thing the filesystem can
  vary; emitting the on-disk spelling made the same resource serialize differently per
  install. The JSON goldens depend on the normalized form — do not "restore" raw spelling
  to `resource_name`.
- support game path plus explicit resource path workflows
- do not assume a single game variant; `game_variant` (`standard` / `iwd` / `pst`) is detected
  from stable root files, never folder names, and selects the effect-opcode table

## Output Rules

JSON must be designed for:

- stable diffs
- programmatic consumption
- AI prompts

Requirements:

- stable field names
- preserve raw numeric values
- include decoded labels when available
- model `strref` as both id and resolved text
- preserve resource references as resrefs, not only resolved filenames
- separate derived interpretations from raw source values

Example:

```json
{
  "resource_type": "ITM",
  "resource_name": "FOA",
  "version": "V1  ",
  "identified_name": {
    "strref": 12345,
    "text": "Flail of Ages"
  },
  "unidentified_name": {
    "strref": 12346,
    "text": "Flail"
  }
}
```

If a value is unknown, prefer:

```json
{
  "raw_value": 17,
  "decoded": null,
  "note": "unknown enum value"
}
```

over inventing semantics.

## IESDP Validation Workflow

For each newly supported format:

1. Pick 3-5 real sample resources from at least one installed game.
2. Work out the expected values from the relevant IESDP offset table.
3. Record key expected values:
   - version
   - header fields
   - counts
   - offsets
   - string refs
   - embedded effects/abilities if relevant
4. Parse the same resource in the new tool.
5. Compare both structured outputs.
6. Add a regression fixture and test.

Validation is complete when:

- header fields match
- counts and offsets are interpreted correctly
- exported strings resolve as expected
- nested structures are not truncated
- output remains stable across runs

## Milestones

Milestone tracking lives in [ROADMAP.md](./ROADMAP.md) — see its Status Snapshot for what is
done and Next Milestones for what is live. Do not restate project status here; this guide is
meant to stay stable while the roadmap moves.

## Definition Of Done For A Resource Type

A format is “done enough” for its current slice when:

- real game files parse without hand-editing
- header fields are covered by tests
- at least one nested structure is covered by tests
- `strref` fields resolve correctly if the format uses them
- CLI export is stable and non-debug
- at least one fixture documents an edge case

## Test Strategy

Every new format should add:

- unit tests for field decoding
- fixture-based tests using real binary resources
- one override-precedence test if relevant
- one JSON snapshot or golden-file style test

Fixture guidance:

- keep fixtures small when possible
- note the source game and resource name
- do not rely only on synthetic files
- use real-world weirdness early

If a test exists because a specific real resource behaves a certain way, name that resource and its game in the test comment or docs.

The suite already has more structure than the list above implies, and it is worth reading
before adding tests:

- [`docs/TESTING.md`](./docs/TESTING.md) — the default suite is self-contained; real-install
  tests are env-gated per installation (`IE_GAME_PATH`, `IE_BGEE_PATH`, `IE_IWDEE_PATH`,
  `IE_PSTEE_PATH`) and no-op when a path is unset. Never make CI depend on game data.
- [`docs/GOLDENS.md`](./docs/GOLDENS.md) — two tiers: exact-value goldens over synthetic
  fixtures and a synthetic install, which run in CI; and normalized JSON *shape* goldens
  checked against four real installs. Read a golden diff before committing it.

## Documentation Expectations

Add a short decision note when:

- a format detail is ambiguous
- a field name is inferred rather than directly specified
- real game files contradict IESDP, and the files win
- output structure choices may affect downstream tools

Suggested docs:

- `docs/formats/<type>.md`
- `docs/decisions/<date>-<topic>.md`

## Scope Discipline

Agents must resist these failure modes:

- adding support for a new format before finishing the current one
- redesigning crate boundaries mid-feature
- building write support before read support is trustworthy — writes now exist, but only
  as byte-exact fixed-offset patches; see the Write Support Framework in ROADMAP.md before
  extending them to variable-length sections
- creating large generic abstractions before the second concrete use
- mixing parser correctness with output prettification

Preferred rhythm:

1. one format
2. one parser
3. one JSON view
4. tests
5. CLI exposure

## Prompt Templates For Coding Agents

Use prompts like these when delegating or resuming work.

### Implement A Format

```text
Implement read-only support for <FORMAT>.

Scope:
- parse the header and core nested structures needed for useful inspection
- expose stable JSON export in the CLI
- add tests over synthetic fixtures for CI, plus env-gated real-install coverage

Constraints:
- do not add write support
- do not refactor unrelated crates
- preserve raw values where decoding is incomplete
- document any ambiguous field interpretation

Validation:
- decode at least 3 real resources and check the fields against the IESDP offset table
- note any discrepancies between IESDP and the real files
```

### Add CLI Support

```text
Add CLI support for dumping already-decoded <FORMAT> resources.

Requirements:
- accept game path plus resource name
- print stable JSON
- return actionable errors
- do not change parser semantics
```

### Investigate A Mismatch

```text
Investigate a parsing mismatch between this tool and the IESDP layout for <RESOURCE>.

Tasks:
- identify which field or offset differs
- inspect the relevant parser code path
- determine whether the bug is in loading, decoding, or rendering
- add or update a regression test
- keep the fix minimal
```

## Review Checklist For Agents

Before considering a task complete, verify:

- does the code parse real files, not just synthetic ones?
- is JSON output deterministic?
- are unknown values preserved rather than dropped?
- are parser errors specific enough to debug?
- does the CLI expose only supported fields, not accidental internals?
- are tests tied to real resource behavior — while still leaving `cargo test --workspace`
  passing with no game data and no env vars set?
- if a skill changed, does `python scripts/skill_parity.py --check` pass?

## CLI Surface

Twelve subcommands ship today. Extend this surface; do not reinvent it.

| Command | Purpose |
|---|---|
| `locate` | resolve a resource and report its source, path, and `game_variant` |
| `list` | enumerate resources by `--type` / `--name` glob / `--source` |
| `dump` | typed JSON for `ITM`, `SPL`, `CRE`, `STO`, `DLG`, `BCS`, `ARE`; DLG `--format dot\|mermaid` |
| `dump-raw` | raw bytes for any located resource |
| `patch` | Tier 1 fixed-offset writes (`CRE`/`CHR` scalars, `ARE` Travel regions) |
| `override-diff` | override trust: BIFF shadow report, or hash diff against a clean reference |
| `verify` | install-wide ARE cross-resource integrity |
| `tlk` / `tlk-append` | resolve a `strref`; append strings for local testing |
| `save-list` / `save-info` | discover save folders; decode `GAM` + the `SAV` container |
| `save-add-item` | scoped PSTEE save mutation, hard-gated on unvalidated layouts |

Run `iecli <command> --help` for the current flags. New read commands default to
`--format json`; keep any text mode a presentation layer over the same data.

## Good Defaults

- use explicit subcommands
- make errors human-readable
- prefer JSON to ad hoc text output
- add `--pretty` only as a presentation option
- keep machine-readable defaults stable over time

## If You Are Unsure

When blocked by uncertainty:

- preserve the raw field
- expose a minimal representation
- add a `todo` or `unknown_*` field
- document the uncertainty
- move forward without inventing false precision

## Final Rule

Do not chase completeness. Chase trustworthy extraction.

The project becomes valuable as soon as it can reliably expose a handful of important resource types in a form that humans and AI can reason about.
