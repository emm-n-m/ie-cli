# `iecli verify` — implementation plan (handoff)

This document is an **agent handoff plan**. An agent picking this up cold should be able to read it once, build, implement, test, verify against a real install, and open a PR without further input.

## Why this exists

Today an investigation into a heavily modded BG1EE install discovered that the dungeon entered from `AR4300` had a Travel region whose `destination_entrance` did not match any entrance in the destination ARE — the engine fired the transition but landed the player in a fallback position. The bug was found by hand: dump source ARE, dump destination ARE, eyeball entrance names, notice the mismatch.

That class of bug is mechanical to detect with the data already in `iecli dump` JSON. The investigation also surfaced 18 missing area-script references and several missing actor links across the same dungeon. All of these are **broken cross-resource references** that an automated check could surface install-wide in one pass.

`iecli verify` is that pass.

## Concrete example (acceptance test target)

User's BG1EE install at `C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate Enhanced Edition`. After running:

```bash
iecli verify --game "<install>" --source override --format json
```

…the output **must** include an entry equivalent to:

```json
{
  "resource": "ARR019.ARE",
  "issue": "phantom_entrance",
  "severity": "error",
  "path": "regions[0].destination_entrance",
  "expected_in": "AR1900.ARE",
  "expected_value": "Exit1903",
  "available_entrances": ["Exit1901", "Exit1902", "Exit1903", "Exit1904", "Exit1905", "Exit1906", "Exit1907"],
  "message": "destination_entrance 'Exit1903' is valid; ..."
}
```

Note: the user has already patched ARR019 in their override (entrance is now `Exit1903`, which exists). The agent will need to **temporarily restore the `.bak` to recreate the broken state** for the integration smoke (or use a synthetic ARE fixture). Restore steps below.

## CLI surface

```
iecli verify --game <PATH>
             [--source override|bif|auto]    # default: auto
             [--resource-type ARE]            # only ARE in v1; reject others
             [--severity error|warning]      # filter; default: both
             [--format text|json]            # default: text
             [--max-issues N]                # cap output, useful in massive installs
```

Notes:
- `--resource-type` is intentionally restrictive in v1. Future iterations add DLG/BCS/CRE.
- `--source override` is the most useful filter — verifies only mod content, ignores stock-game references that are vanilla-correct.
- Text format: one issue per line, severity-prefixed, sortable. JSON format: array of issue objects.

## Issue catalog (v1, ARE only)

All categories operate over a single in-memory pass per ARE plus a registry built in a first pass over all AREs in scope.

| `issue`                  | Severity | Detection |
|--------------------------|----------|-----------|
| `dead_link`              | error    | Travel region's `destination_area.exists == false` |
| `phantom_entrance`       | error    | Travel region's `destination_area.exists == true`, but `destination_entrance` not present in destination ARE's entrance-table names |
| `missing_area_script`    | warning  | `header.area_script.exists == false` |
| `missing_region_script`  | warning  | region's `region_script.exists == false` |
| `missing_actor_cre`      | warning  | actor's `cre_file.link.exists == false` (only when CRE resref is present) |
| `missing_actor_dialog`   | warning  | actor's `dialog.exists == false` |
| `missing_actor_script`   | warning  | any of `scripts.{override,general,class,race,default,specific}_script.exists == false` |
| `missing_key_item`       | warning  | region's `key_item.exists == false` |

**Severity rationale:** `dead_link` and `phantom_entrance` cause player-visible engine-fallback transitions, which are gameplay-breaking. The `missing_*` warnings are flagged because they indicate broken installs but are routinely benign in modded games (many AREs reference scripts that the mod author never shipped, with no functional impact). Don't escalate warnings to errors without a concrete signal.

Empty-string / NUL resrefs are **not** issues — they mean "no reference," which is valid.

## Implementation outline

### Where the code goes

- New file: `crates/ie-formats/src/verify.rs`
  - Public types: `VerifyIssue`, `VerifySeverity`, `VerifyCategory`, `VerifyOptions`, `EntranceRegistry`
  - Public fn: `pub fn build_entrance_registry(installation: &dyn AreaSource) -> EntranceRegistry`
  - Public fn: `pub fn verify_are(area: &AreaJson, registry: &EntranceRegistry) -> Vec<VerifyIssue>`
  - The `AreaSource` trait abstracts "give me parsed AREs" so unit tests can pass a synthetic source and the CLI can pass the real locator.
- `crates/ie-formats/src/lib.rs`: re-export `pub use verify::{...}`.
- `crates/ie-cli/src/main.rs`: new `Verify(VerifyArgs)` subcommand. Implementation drives the two passes via the existing `ResourceLocator`/`ResourceReader`/`decode_to_json`.

### Two-pass design

**Pass 1 — registry build.** Enumerate all ARE resources via `locator.list(ResourceListOptions { resource_type: Some("ARE"), source: ... })`. For each, parse and record `{resref → set<entrance_name>}`. Note: parse failures should produce a `parse_error` issue and be excluded from the registry.

**Pass 2 — per-area verify.** For each ARE in scope, call `verify_are(&parsed, &registry)`. Emit issues. The `dead_link`, `missing_*` checks just inspect the JSON's existing `exists: bool` flags. The `phantom_entrance` check uses the registry.

### Entrance comparison rules

Use byte-exact, NUL-truncated, **case-sensitive** match on entrance names. Region/entrance names in IE are conventionally case-sensitive. Don't trim or normalize beyond the existing `parse_char_array` behavior.

### Output paths

For each issue, the `path` field is a JSON-Path-ish breadcrumb: `regions[3].destination_entrance`, `actors[12].dialog`, `header.area_script`. Use 0-based indices.

## Test plan

### Unit tests in `verify.rs` (synthetic AREs only — no game install)

- `phantom_entrance_detected_when_destination_lacks_entrance` — two synthetic AREs; source has Travel→dest with entrance "Foo"; dest has only entrance "Bar"; expect 1 issue.
- `entrance_match_clean_pass` — same shape but matching entrance; expect 0 issues.
- `dead_link_detected_when_destination_does_not_exist` — Travel region pointing to a resref the registry doesn't contain; expect `dead_link`.
- `non_travel_regions_skipped` — Trigger/Info regions aren't checked even if they have garbage in the destination_entrance bytes.
- `missing_area_script_warning` — `header.area_script.exists=false` produces a warning.
- `missing_actor_dialog_warning` — actor with non-empty dialog resref but `exists=false` produces a warning.
- `missing_actor_dialog_no_issue_when_resref_empty` — actor with no dialog resref produces no issue.
- `severity_filter_drops_warnings_when_error_only` — option-level filter applied at the issue-emit boundary.

### Integration smoke (env-gated, opt-in)

`#[test] #[ignore]` or behind `IE_GAME_PATH` env (mirrors existing testing convention — see [docs/TESTING.md](../TESTING.md)). Test should:

1. Read `IE_GAME_PATH` (skip if unset).
2. Run `verify` programmatically (not via subprocess) over `--source override`.
3. Assert at least one `phantom_entrance` issue exists in the result, OR run against a synthetic ARE fixture if the user's install has been patched.

For the user's specific install: ARR019.ARE has been patched (working state). To recreate the broken state for verification:

```bash
# Restore broken state from backup the user kept:
cp "<install>/override/ArR019.ARE" "<install>/override/ArR019.ARE.user-bak"
cp "<install>/override/ArR019.ARE.bak" "<install>/override/ArR019.ARE"
# ... run verify, expect phantom_entrance issue ...
# Restore patched state:
cp "<install>/override/ArR019.ARE.user-bak" "<install>/override/ArR019.ARE"
rm "<install>/override/ArR019.ARE.user-bak"
```

If the agent doesn't want to touch the install, **prefer the synthetic fixture path** in unit tests and skip the integration smoke. The unit tests cover the logic; integration is for sanity, not coverage.

## Out of scope (do NOT add these in this PR)

- DLG state-graph cycle detection
- BCS trigger/action resolution checks
- CRE item/spell reference checks
- Cross-format chain checks (ARE → CRE → DLG)
- WeiDU-style fix suggestions
- Auto-patching from verify output
- More resource-type values for `--resource-type`
- Performance optimization (caching parsed AREs across runs, parallelism)

These are tracked as future iterations after v1 lands. Resist scope-bloat — the verify command should ship usefully with ARE alone and grow on demand.

## Conventions worth knowing

- **JSON output is the primary output**, not a debug afterthought. Stable shape matters; downstream skills will parse it.
- **Determinism**: issues sorted by `(resource, path)` so output diffs are stable across runs.
- **No emoji or Unicode embellishment** in messages.
- **Error messages should be actionable** — say what's missing AND what's available where applicable (the `phantom_entrance` issue lists `available_entrances`, which is the fix-it hint).
- **Don't suggest NearInfinity** in error messages or docs. The project is CLI-first / AI-native ([README.md](../../README.md), [ROADMAP.md](../../ROADMAP.md)).
- Match existing parser/patch style in [crates/ie-formats/src/are.rs](../../crates/ie-formats/src/are.rs) — `thiserror`-based error enums, `Result<_, AreaParseError>` flavor, `parse_table` helper if you need it.
- `cargo fmt` and `cargo clippy --all-targets` clean before PR. Pre-existing clippy warnings in `crates/ie-formats/src/dlg.rs` are NOT yours to fix in this PR.

## Acceptance criteria

- [ ] New `iecli verify` subcommand registered with the documented flags.
- [ ] All 8 issue categories implemented per the catalog above.
- [ ] At least 8 unit tests in `verify.rs` covering the catalog and severity-filter behavior, all passing under `cargo test -p ie-formats`.
- [ ] At least 1 env-gated integration test, gated on `IE_GAME_PATH`, that exercises pass 1 + pass 2 against a real install.
- [ ] `cargo build --quiet` clean. `cargo test` clean. `cargo clippy --all-targets` introduces no new warnings.
- [ ] [README.md](../../README.md) `## Current Commands` section gets a `verify` example.
- [ ] [ROADMAP.md](../../ROADMAP.md) gets a one-line entry under "Validated in real-world use" once the integration smoke passes.
- [ ] [docs/SKILLS.md](../SKILLS.md) and [.claude/skills/explore-dungeon/SKILL.md](../../.claude/skills/explore-dungeon/SKILL.md) updated to point at `verify` as the recommended way to find broken cross-resource references.

## Local validation commands

```bash
# Build & unit tests:
cargo build
cargo test -p ie-formats

# Workspace-wide:
cargo test
cargo clippy --all-targets

# Smoke (env-gated):
$env:IE_GAME_PATH = "C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate Enhanced Edition"
cargo test -- --ignored

# Manual run against the user's install (override-only, JSON):
target/debug/iecli verify --game $env:IE_GAME_PATH --source override --format json
```

## PR shape

- Branch: `agent/iecli-verify` (matches the previous agent-handoff convention from PR #2 `agent/are-m6-coverage`).
- Title: `iecli verify: ARE cross-resource validation (v1)`
- Body: summary of what's covered, link back to this plan, list of issue categories, note that DLG/BCS/CRE are deferred.
- Push to `origin` on the GitHub remote (`git@github.com:emm-n-m/ie-cli.git`) and open a PR against `main`. The user reviews and merges.

## What's already in place (you don't need to add it)

The data needed for v1 verify is already in the JSON. Specifically:
- `header.area_script: ResourceLink` with `exists` flag — added long ago.
- `regions: Vec<AreaRegionJson>` with `destination_area: Option<ResourceLink>`, `destination_entrance: Option<String>`, `region_script: Option<ResourceLink>`, `key_item: Option<ResourceLink>` — added 2026-04-26 (see [crates/ie-formats/src/are.rs](../../crates/ie-formats/src/are.rs)).
- `entrances: Vec<AreaEntranceJson>` with `name`, `position`, `orientation` — added 2026-04-26.
- `actors[*].dialog`, `actors[*].cre_file.link`, `actors[*].scripts.*` — all carry `exists` flags via the existing `ResourceLink` machinery.

So the verify implementation is mostly walking existing JSON shape and cross-referencing by resref. **No new parser fields are required.** If you find yourself needing to add parsing, that's a sign you've drifted out of v1 scope.

## When in doubt

The decisive prior decision is [2026-04-24-are-read-link-enrichment.md](2026-04-24-are-read-link-enrichment.md) — same crate, same style. Read that to absorb the project's tone for ARE work before starting.
