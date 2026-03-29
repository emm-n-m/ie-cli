# Near Infinity Rewrite Agent Guide

## Purpose

This document gives coding agents a strict working guide for building a CLI-first, Rust-based Infinity Engine resource inspector inspired by Near Infinity.

The first goal is not a full Near Infinity replacement. The first goal is a reliable parser and exporter that can turn installed game resources into stable, machine-readable output for:

- human inspection
- AI-assisted analysis
- diffing
- scripting
- future WeiDU patch generation workflows

Near Infinity is the behavioral reference when its behavior is clear and intentional.

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
- a full Near Infinity clone

## First-Phase Deliverable

The first release-quality milestone should support:

- game installation discovery
- `KEY` parsing
- `BIF` resource access
- override precedence
- `TLK` string resolution
- JSON export for:
  - `ITM`
  - `SPL`
  - `CRE`
  - `STO`

After that:

- `DLG`
- `BCS`
- additional resource families

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

## Suggested Repository Layout

Use a layout close to this unless the repo already established another one:

```text
crates/
  ie-core/        # shared types, errors, resource ids, common helpers
  ie-io/          # installation discovery, KEY/BIF/TLK loading
  ie-formats/     # format-specific decoders: ITM, SPL, CRE, STO, later DLG/BCS
  ie-cli/         # command-line frontend
tests/
  fixtures/
    bg2ee/
    bgee/
    pstee/
docs/
  formats/
  decisions/
```

If the repo is still empty, start with fewer crates:

- `ie-core`
- `ie-formats`
- `ie-cli`

Only split further when a boundary is real, not speculative.

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
- lookup should be case-insensitive where engine behavior is case-insensitive
- preserve original resref spelling in output when available
- support game path plus explicit resource path workflows
- do not assume a single game variant

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

## Validation Philosophy

Near Infinity is the first comparison target, not the only authority.

When behavior is unclear:

1. inspect the relevant Near Infinity code path
2. check IESDP or other primary format references
3. compare real game files
4. encode the decision in a test
5. document any remaining ambiguity

If Near Infinity appears to have format-specific heuristics, preserve that behavior only when:

- it is clearly intentional
- real files depend on it
- the project documents it

## Near Infinity Validation Workflow

For each newly supported format:

1. Pick 3-5 real sample resources from at least one installed game.
2. Inspect the same resources in Near Infinity.
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

### Milestone 0: Skeleton

- create Cargo workspace
- define error types
- define resource id / resref types
- define shared JSON output conventions
- add basic CLI command scaffolding

### Milestone 1: Installation Access

- locate game root
- locate `chitin.key`
- locate `dialog.tlk` and language folder
- read `override`
- implement resource lookup precedence

### Milestone 2: Container Parsing

- parse `KEY`
- locate `BIF` entries
- read raw resources from `BIF`
- expose a `locate` and `dump-raw` command

### Milestone 3: TLK

- parse `dialog.tlk`
- resolve `strref`
- expose a `tlk` command
- include resolved text in parsed JSON where applicable

### Milestone 4: Item and Spell Family

- parse `ITM`
- parse `SPL`
- export stable JSON
- add real fixtures and tests

### Milestone 5: Creature and Store Family

- parse `CRE`
- parse `STO`
- export stable JSON
- add real fixtures and tests

### Milestone 6: Dialogue

- parse `DLG`
- resolve state and response strings
- preserve transitions and references
- provide a readable exported form

### Milestone 7: Scripts

- add `BCS` support only after representation is agreed
- keep raw and interpreted forms separate if needed

## Definition Of Done For A Resource Type

A format is “done enough” for the current milestone when:

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

If a test exists because NI behaves a certain way, say so in the test comment or docs.

## Documentation Expectations

Add a short decision note when:

- a format detail is ambiguous
- a field name is inferred rather than directly specified
- a Near Infinity behavior is mirrored for compatibility
- output structure choices may affect downstream tools

Suggested docs:

- `docs/formats/<type>.md`
- `docs/decisions/<date>-<topic>.md`

## Scope Discipline

Agents must resist these failure modes:

- adding support for a new format before finishing the current one
- redesigning crate boundaries mid-feature
- building write support before read support is trustworthy
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
- add fixture-based tests using real game resources

Constraints:
- do not add write support
- do not refactor unrelated crates
- preserve raw values where decoding is incomplete
- document any ambiguous field interpretation

Validation:
- compare behavior against Near Infinity for at least 3 real resources
- note any discrepancies
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
Investigate a parsing mismatch between this tool and Near Infinity for <RESOURCE>.

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
- are tests tied to real resource behavior?

## Suggested CLI Surface

Commands worth targeting first:

```bash
iecli locate --game /path/to/game --resource VICONI.CRE
iecli dump-raw --game /path/to/game --resource FOA.ITM
iecli tlk --game /path/to/game --strref 12345
iecli dump --game /path/to/game --resource FOA.ITM --format json
iecli dump --game /path/to/game --resource VICONI.CRE --format json
iecli dump --game /path/to/game --resource IMOEN2.CRE --format json
```

Later:

```bash
iecli dump --game /path/to/game --resource DPLAYER3.DLG --format json
iecli diff --game /path/to/game --resource VICONI.CRE --other ./override/VICONI.CRE
```

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
