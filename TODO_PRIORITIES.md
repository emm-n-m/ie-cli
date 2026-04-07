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

Completed:

- Create Cargo workspace.
- Create initial crates: `ie-core`, `ie-io`, `ie-formats`, `ie-cli`.
- Define shared error types.
- Define `ResRef`, `StrRef`, `ResourceType`, and source metadata structs.
- Add CLI skeleton with subcommands and basic argument parsing.
- Add repository docs:
  - `README.md`
  - `ARCHITECTURE.md`
  - `TODO_PRIORITIES.md`

Follow-up:

- Add contributing or development notes if needed.

Exit criteria:

- project builds
- tests run
- CLI prints help

## P1: Installation And Resource Access

Completed:

- Implement game path validation.
- Locate `chitin.key`.
- Locate language folder and `dialog.tlk`.
- Parse `KEY`.
- Read resources from `BIFF`, `BIF`, and `BIFC`.
- Resolve override precedence over container resources.
- Add a `locate` command.
- Add a working `dump-raw` command with explicit output path.
- Add archive-loading tests for:
  - `BIFF`
  - `BIF`
  - `BIFC`
  - override precedence
  - locator-not-found and truncated archive cases
- Parse `chitin.key` into a richer typed model instead of the current minimal index.
- Add broader fixture coverage for KEY lookup across more than one resource family.
- Validate `locate` and `dump-raw` against a real BG2EE installation for:
  - override-backed resources
  - KEY-backed BIFF mappings
  - BIFF-backed raw extraction

Exit criteria:

- tool can locate a named resource
- tool can read raw bytes from both override and BIF-backed resources
- override precedence is covered by tests

## P1: TLK Support

Completed:

- Parse `dialog.tlk`.
- Implement `StrRef` resolution.
- Add a `tlk` command.
- Validate TLK lookup against a real BG2EE installation.
- Add tests for:
  - valid string lookup
  - invalid/out-of-range string refs
  - empty strings

Exit criteria:

- tool can resolve string refs reliably
- parser errors are actionable

## P1: First Decoders

Completed:

- Implement `ITM` parser.
- Implement `SPL` parser.
- Add stable JSON export for both.
- Add CLI `dump` support for decoded resources.
- Add parser unit tests for both formats.
- Validate selected `SPL` outputs against Near Infinity using real BG2EE resources.
- Add real-install `dump` regression coverage for validated `SPL` resources when `IE_GAME_PATH` is set.

Remaining:

- Add fixture tests using real resources.
- Add JSON golden or snapshot coverage for exported output.
- Expand semantic validation of decoded `ITM` and `SPL` fields against more real resources.
- Broaden Near Infinity comparison coverage for `ITM` and additional `SPL` resources.

Suggested verification resources:

- iconic weapons
- one complex magical item
- one divine spell
- one arcane spell
- one spell with global effects
- one priest-only spell with class exclusion flags

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

Current status:

- `BG2EE` has been used as the first real validation target.

Remaining:

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

Current high-value follow-up issues:

1. Add real-resource fixture coverage and Near Infinity comparisons for `ITM` and `SPL`.
2. Add JSON golden or snapshot tests for decoded `ITM` and `SPL` output.
3. Implement `CRE` decoding and JSON export.
4. Implement `STO` decoding and JSON export.
5. Broaden real-install validation beyond `BG2EE`.

## Stop Conditions

Stop and document before proceeding if:

- output shape is unstable across runs
- parser behavior disagrees with real files and the reason is unclear
- the next step requires broad redesign unrelated to the active milestone
- a feature request would derail the first useful release
