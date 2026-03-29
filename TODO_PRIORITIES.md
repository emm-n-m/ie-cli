# TODO Priorities

## Priority Model

Use these levels:

- `P0`: blocks the project from being useful at all
- `P1`: core functionality needed for the first useful release
- `P2`: high-value follow-up after the first useful release
- `P3`: worthwhile later work

Within each priority:

- finish vertical slices
- prefer correctness over breadth
- do not start too many formats at once

## P0: Bootstrap

- Create Cargo workspace.
- Create initial crates: `ie-core`, `ie-formats`, `ie-cli`.
- Define shared error types.
- Define `ResRef`, `StrRef`, `ResourceType`, and source metadata structs.
- Add CLI skeleton with subcommands and basic argument parsing.
- Add repository docs:
  - `README.md`
  - `ARCHITECTURE.md`
  - `NEAR_INFINITY_AGENT.md`
  - contributing or development notes if needed

Exit criteria:

- project builds
- tests run
- CLI prints help

## P1: Installation And Resource Access

- Implement game path validation.
- Locate `chitin.key`.
- Locate language folder and `dialog.tlk`.
- Parse `KEY`.
- Read resources from `BIF`.
- Resolve override precedence over container resources.
- Add a `locate` command.
- Add a `dump-raw` command.
- Add fixture tests for resource lookup.

Exit criteria:

- tool can locate a named resource
- tool can read raw bytes from both override and BIF-backed resources
- override precedence is covered by tests

## P1: TLK Support

- Parse `dialog.tlk`.
- Implement `StrRef` resolution.
- Add a `tlk` command.
- Add tests for:
  - valid string lookup
  - invalid/out-of-range string refs
  - empty strings

Exit criteria:

- tool can resolve string refs reliably
- parser errors are actionable

## P1: First Decoders

- Implement `ITM` parser.
- Implement `SPL` parser.
- Add stable JSON export for both.
- Add fixture tests using real resources.
- Compare selected outputs against Near Infinity.

Suggested verification resources:

- iconic weapons
- one complex magical item
- one divine spell
- one arcane spell

Exit criteria:

- `dump` works for `ITM` and `SPL`
- exported JSON is stable enough for snapshot tests

## P1: Practical Formats

- Implement `CRE` parser.
- Implement `STO` parser.
- Add stable JSON export.
- Add fixture and snapshot tests.
- Verify several real resources against Near Infinity.

Suggested verification resources:

- a joinable NPC creature
- a hostile creature
- a merchant store
- a special-purpose store

Exit criteria:

- tool can dump party-relevant creatures and stores
- exported strings resolve correctly

## P2: Dialogue

- Design exported `DLG` representation.
- Parse states, transitions, and replies.
- Resolve associated strings.
- Preserve references and indices clearly.
- Add tests on real dialogue files.

Important constraint:

- do not prematurely flatten dialogue into prose
- preserve graph structure

Exit criteria:

- a dialogue tree can be exported in a form suitable for AI analysis and diffing

## P2: Quality And Validation

- Add comparison harness for checking selected resources against Near Infinity expectations.
- Add JSON golden tests.
- Add fuzz or truncated-input tests for critical parsers.
- Improve CLI error messages.
- Add `--pretty` and compact JSON modes if not already present.

Exit criteria:

- parser behavior is dependable on both happy-path and malformed data

## P2: Cross-Game Coverage

- Test against at least:
  - BGEE
  - BG2EE or EET
  - PSTEE
- Document observed per-game quirks.
- Ensure discovery logic is not BG-only.

Exit criteria:

- tool works on more than one Infinity Engine title

## P3: Script And Diff Support

- Add `DLG` follow-up improvements if the first model is too raw.
- Add `BCS` parsing once representation is agreed.
- Add `diff` command for decoded resources.
- Add machine-readable cross-reference output.

Exit criteria:

- resource comparison and script-adjacent inspection are practical

## P3: Ecosystem And Workflow

- Add examples for AI-assisted usage.
- Add sample prompts for generating WeiDU patches from exported JSON.
- Add command recipes for common workflows:
  - inspect item
  - inspect NPC
  - compare override vs base resource
  - resolve dialogue strings

Exit criteria:

- tool is easy to use as part of a modding and analysis pipeline

## Backlog Rules

- Do not add a new format unless the current one has:
  - parser tests
  - real fixture coverage
  - stable CLI output
- Do not add write support before read support is trusted.
- Do not optimize performance before correctness and output shape are stable.
- Do not refactor crate boundaries without a concrete pain point.

## Suggested First Issues

If the repo needs a starter backlog, begin with these:

1. Initialize Cargo workspace and CLI scaffold.
2. Implement `ResRef`, `StrRef`, and common error types.
3. Parse `chitin.key` and print summary stats.
4. Implement resource lookup with override precedence.
5. Parse `dialog.tlk` and add `tlk` command.
6. Implement `ITM` decoding and JSON export.
7. Implement `SPL` decoding and JSON export.
8. Implement `CRE` decoding and JSON export.

## Stop Conditions

Stop and document before proceeding if:

- output shape is unstable across runs
- parser behavior disagrees with real files and the reason is unclear
- the next step requires broad redesign unrelated to the active milestone
- a feature request would derail the first useful release
