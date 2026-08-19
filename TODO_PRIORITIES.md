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
- Validate selected `SPL` outputs against the IESDP layout using real BG2EE resources.
- Add real-install `dump` regression coverage for validated `SPL` resources when `IE_GAME_PATH` is set.
- Add reusable env-gated factual expectations for stock BGEE `SW1H01.ITM` and `SPWI112.SPL`.

- Add JSON golden coverage for exported output: exact-value goldens over synthetic fixtures, plus normalized shape goldens against real installs. See [docs/GOLDENS.md](./docs/GOLDENS.md).

Remaining:

- Expand env-gated factual assertions for decoded `ITM` and `SPL` fields against more real
  resources. Raw game resources and full derived dumps are not committed; see
  [docs/REGRESSION_PLAN.md](./docs/REGRESSION_PLAN.md).
- Broaden real-resource expectation coverage for `ITM` and additional `SPL` resources.

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

Completed:

- Implement `CRE` parser.
- Implement `STO` parser.
- Add stable JSON export.
- Add env-gated factual expectations for stock BGEE `GORION.CRE` and `FRIEND.STO`.

Remaining:

- Add env-gated factual assertions for additional representative `CRE` and `STO` resources and
  other game families.
- Verify representative resources against the IESDP layout. Exact synthetic JSON goldens already pin both
  formats' exported shape and values.

Suggested verification resources:

- a joinable NPC creature
- a hostile creature
- a merchant store
- a special-purpose store

Exit criteria:

- tool can dump party-relevant creatures and stores
- exported strings resolve correctly

## P2: Dialogue

Completed:

- Design exported `DLG` representation.
- Parse states, transitions, and replies.
- Resolve associated strings.
- Preserve references and indices clearly.

Remaining:

- Add more real-install validation for representative dialogue files.
- Expand regression coverage for edge cases such as external-dialog references.

Important constraint:

- do not prematurely flatten dialogue into prose
- preserve graph structure

Exit criteria:

- a dialogue tree can be exported in a form suitable for AI analysis and diffing

## P2: Quality And Validation

Completed:

- Extend exact-value goldens to `dump --format dot|mermaid`, `save-list`, `tlk`, and every text (non-JSON) output mode. Every output named in this backlog item is now pinned.
- Add a reusable, provenance-bearing expectation harness for factual real-resource checks.
- Add deterministic adversarial-input no-panic coverage for every decoder in the normal test suite.

Remaining:

- Add coverage-guided fuzzing if deterministic adversarial cases and targeted truncation regressions
  expose insufficient paths. The dependency-free randomized harness currently covers every decoder.
- Improve CLI error messages.
- Add a compact JSON mode if demand justifies another presentation option. JSON is currently
  pretty-printed consistently.

Exit criteria:

- parser behavior is dependable on both happy-path and malformed data

## P2: Cross-Game Coverage

Current status:

- `BG2EE` has been used as the first real validation target.
- `PSTEE` is now a second real target: every dialogue in the install has been swept, plus a 2,313-resource mod, and PSTEE saves drive the scoped save-write path.
- `IWDEE` is validated: a whole-install sweep decoded 5,995 of 5,996 resources, and its stat-item anchors are pinned as tests.
- `BGEE` is validated: a whole-install sweep of a heavily modded install (16 WeiDU mods, 7,762 override files, SoD merged) decoded 10,965 of 10,966 resources.
- All four titles now have real-install coverage, and both install *shapes* do: the merged BGEE root above, plus an unmerged BGEE+SoD install whose DLC is still packed in `dlc/sod-dlc.zip`.

Completed:

- Test against BG2EE and PSTEE.
- Ensure discovery logic is not BG-only — game-variant detection keys off root files (`torment.lua`, `icewind.exe`, …) rather than folder names, since installs are routinely renamed.
- Document the first per-game quirk: PST uses a different effect-opcode table, now decoded per variant.
- Report the detected variant from `locate` as `game_variant`, so a misdetected install is visible instead of silently decoding as standard.

- Validate IWDEE with a whole-install decode sweep and opcode anchors, and confirm the `iwd` variant can keep sharing the standard opcode table.
- Validate BGEE with a whole-install decode sweep plus an install-wide `verify` pass, on a heavily modded install rather than a stock one.
- Mount DLC archives (`dlc/*.zip`) read-only, so a default Steam BGEE+SoD install stops silently under-reporting. Specified in [docs/SPEC_DLC_MOUNTING.md](./docs/SPEC_DLC_MOUNTING.md) and verified against a real unmerged install. The load-bearing correction came from that install: a DLC carries **its own KEY** (SoD's `mod.key`, indexing 40 BIFs and 21,269 resources), so it is a second index to merge, not a path-resolution overlay — the merged reference install had hidden this, and the original spec drew the opposite conclusion from it.

Remaining:

- Keep the stock-BGEE `verify` known-issue baseline current across game patches. The committed
  env-gated test covers a clean Steam BGEE+SoD root with SoD packed in `dlc/sod-dlc.zip`.
- Parse `WMP` so `verify` can distinguish a live broken exit from an unreachable leftover. 53 BGEE areas have no inbound Travel region, but in BG1 that usually means worldmap entry rather than unreachability.
- Continue extending the effect-opcode tables and remeasure IWDEE coverage. The last sweep resolved
  41%; common save modifiers and cast/learn/protection-from-spell opcodes were added afterward.
  Unnamed opcodes emit `decoded: null`, so this is coverage rather than a correctness risk.
- Keep documenting per-game quirks as they surface. Known so far: PST uses its own opcode numbering; IWDEE ships `#BONECIR.SPL` with a corrupt signature byte; BGEE indexes `CDDETECT` as `.SPL` over ITM payload bytes and ships `MONKTU 8.DLG`, a resref whose padding space falls mid-name; stock areas rely on case-insensitive entrance names and use `NONE` as an empty script/dialog slot; SoD's `mod.key` and the base `chitin.key` name 17 identical BIF paths that are 17 different files, so BIF resolution must stay scoped to the KEY that named the entry.

Exit criteria:

- tool works on more than one Infinity Engine title (met: BG2EE + PSTEE + IWDEE + BGEE)
- each supported title has real-install coverage rather than assumed compatibility (met)
- each supported *install shape* has real coverage rather than assumed compatibility (met: merged and packed-DLC roots both validated against real installs)

## P2: Areas, Verification, And Scoped Writes

Completed:

- Implement `ARE` parsing for the header, actors, entrances, and Travel regions, with deferred-section offsets/counts preserved.
- Add DLG graph export (`dump --format dot|mermaid`) with extern following and label controls.
- Add `override-diff` for shadow reports and reference comparison.
- Add `verify` for install-wide ARE cross-resource integrity (dead links, phantom entrances, missing scripts/actors/items).
- Add Tier 1 scalar patching for fixed-offset `CRE`/`CHR` fields and `ARE` Travel-region destinations.
- Add `tlk-append` for single-string appends, in place or to a copy.
- Add save inspection (`save-list`, `save-info`) and the scoped PSTEE `save-add-item` write.

Remaining:

- Extend `verify` beyond ARE only when a workflow needs it.
- Expand `ARE` into doors, containers, spawn points, or ambients only on concrete demand.
- Classic PST `GAM` v1.1 saves, and BG/IWD `save-add-item` once each layout and slot map clears the gate in `docs/SPEC_SAVE_ITEM_WRITE_COMPLETE.md`.

Exit criteria:

- an area-level breakage can be found, diagnosed, and repaired without leaving the CLI (met for the Travel-region case)

## P3: Script And Diff Support

Completed:

- Add resource comparison via `override-diff` (shadow and reference modes). A general decoded-resource `diff` is still open.

Remaining:

- Add `DLG` follow-up improvements if the first model is too raw.
- Add `BCS` follow-up improvements once more real-world validation identifies gaps.
- Add `diff` command for decoded resources (JSON-level, not just hash-level).
- Add machine-readable cross-reference output.

Exit criteria:

- resource comparison and script-adjacent inspection are practical

## P3: Ecosystem And Workflow

Completed:

- Add examples for AI-assisted usage — six Claude Code skills in `.claude/skills/`, inventoried in `docs/SKILLS.md`.
- Add worked outputs of those workflows in `docs/guides/`, each with its own reproduction commands.
- Add command recipes for common workflows: inspect item, inspect NPC, compare override vs base resource, resolve dialogue strings (README "Current Commands", plus the skill workflows).

Remaining:

- Add sample prompts for generating WeiDU patches from exported JSON.
- Generalize the PST-tuned skills (`plan-stat-build`, and the PST specifics inside `map-stat-gates`) to BG/IWD when a real question demands it.

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

1. Broaden the initial BGEE factual expectation matrix across BG2EE, PSTEE, and IWDEE when those
   installs are available.
2. Add real-resource expectations with stated provenance for representative `ITM`, `CRE`, `STO`, `DLG`, `BCS`, and
   `ARE` resources; record the findings in the existing expectation manifest.
3. Add more real external-dialog edges and script constructs beyond the BGEE `ALATOS`/`BALDUR`
   anchors.

Validation debt is now the dominant backlog theme: read coverage has run far ahead of fixtures and
factual assertions against independently inspected resources. Items 1–4 predate the PSTEE sweeps
and remain open despite that workload.

## Stop Conditions

Stop and document before proceeding if:

- output shape is unstable across runs
- parser behavior disagrees with real files and the reason is unclear
- the next step requires broad redesign unrelated to the active milestone
- a feature request would derail the first useful release
