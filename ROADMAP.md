# Roadmap

This document captures the current direction of `iecli`, the next vertical slices of work, and the real-world use case driving priorities. It is intended for agents (and humans) picking up the project cold.

Stable guidance lives in [AGENTS.md](./AGENTS.md). The full tiered backlog lives in [TODO_PRIORITIES.md](./TODO_PRIORITIES.md). This file changes with project state.

## Vision

`iecli` aims to be the first Infinity Engine tool designed for **agent-assisted modding workflows**, not GUI-driven hand editing.

The differentiator is not "another Near Infinity" — Near Infinity is mature, open-source, and excellent at GUI inspection. The differentiator is:

- **Stable machine-readable JSON** as the primary output, not a debug afterthought.
- **CLI-first** so every capability is scriptable, pipeable, and callable from an agent loop.
- **Deterministic** output so diffs are meaningful and snapshot tests are possible.
- **Round-trippable** (eventually) so agents can read, reason, and write.

The target workflow:

```
iecli reads  →  agent reasons in JSON  →  agent emits WeiDU .tp2 + .d + patched resources  →  WeiDU installs
```

Near Infinity remains the right tool for last-mile manual sanity checks and for hand-editing when a human wants to. `iecli` fills the machine-readable gap in the ecosystem.

The repo now ships the second half of that loop as well: [agent skills](./docs/SKILLS.md) that carry a
question from natural language through `iecli` JSON to a narrative answer, and the [guides](./docs/guides/)
those skills produced against real installs. JSON is the interface; the skills are the proof it is the
right one.

## Status Snapshot

Current as of 2026-08-14.

### Done

- Workspace, crate structure, error types, shared JSON conventions.
- Installation discovery: game root validation, `chitin.key`, language folder, `dialog.tlk`.
- Typed `KEY` parsing.
- `BIFF` / `BIF` / `BIFC` raw reads.
- Override precedence, with `--source <auto|override|bif>` opt-out (`locate`, `dump`, `dump-raw`, `list`, `verify`).
- Normalized `resource_name` regardless of source. Override files match case-insensitively, so casing is the only thing the filesystem can vary; reporting the on-disk spelling made the same resource serialize differently per install (and, before the override index landed, differently per filesystem case-sensitivity). `source_path` still carries the exact on-disk spelling. This is a precondition for the JSON golden tests below.
- Resource enumeration: `list --type <T> --name <glob> --source <S> --format <text|json>`.
- `TLK` string resolution.
- Typed decoders + JSON export for: `ITM`, `SPL`, `CRE`, `STO`, `DLG`, `BCS`.
- DLG graph export via `dump --format dot|mermaid`, with `--max-label-len`, `--no-triggers`, `--no-actions`, `--strings`, and `--follow-extern[=DEPTH]`.
- `ARE` decoder + JSON export for the header, deferred-section offsets/counts, actors, entrances, and Travel-region links.
- Lazy IDS loading and opcode/name resolution for BCS decoding.
- Game-variant detection (`standard` / `iwd` / `pst`) from stable root files (`torment.lua`, `icewind.exe`, …) rather than folder names, carried on every resource's metadata and reported by `locate` as `game_variant`.
- Variant-aware effect decoding: PST uses its own opcode table, so `ITM`, `SPL`, and `CRE` effect names decode correctly on PSTEE instead of silently borrowing BG opcode meanings.
- Install-wide `verify` for ARE cross-resource integrity, with `--severity`, `--max-issues`, `--source`, and `--format text|json`. Categories: `dead_link`, `phantom_entrance`, `missing_area_script`, `missing_region_script`, `missing_actor_cre`, `missing_actor_dialog`, `missing_actor_script`, `missing_key_item`, `parse_error`.
- Override trust workflow via `override-diff`: shadow reports against BIFF and hash-based comparison against reference directories/files.
- Real-install smoke coverage for `ITM` and `SPL`; selected Near Infinity comparisons for `SPL`.
- Env-gated CLI smoke coverage for `BCS` and PSTEE `ARE`.
- Initial CRE scalar patch support for fixed-offset fields, with byte-exact copy-only behavior.
- Initial ARE region patch support for `regions.<selector>.destination_entrance` and `regions.<selector>.destination_area`, addressed by region name or 0-based index, byte-exact copy-only behavior.
- Save inspection: `save-list` (folder discovery across install root + user Documents) and `save-info` (`GAM` v2.0/2.1/2.2 header/party/globals decode, `SAV` zlib container manifest). Validated against real BG2EE saves; see [docs/SAVE_SUPPORT_TODO.md](./docs/SAVE_SUPPORT_TODO.md).
- Scoped PSTEE save mutation via `save-add-item`: append one item to a party member's embedded CRE, repair affected CRE/GAM offsets, copy the save folder by default, and hard-refuse unvalidated BG/IWD layouts.
- `dialog.tlk` append support via `tlk-append`, with in-place and copy-output workflows.
- Agent skill layer: six skills that turn `iecli` JSON into narrative answers, packaged twice — [`.claude/skills/`](./.claude/skills/) for Claude Code and [`skills/`](./skills/) for Codex. Inventory and per-script flags live in [docs/SKILLS.md](./docs/SKILLS.md).
- Research guides in [`docs/guides/`](./docs/guides/) produced by running that skill layer against real installs — the first user-facing output of the tool that is not JSON.
- `ie-cli` decomposed from one large `main.rs` into per-concern modules (`dialog_graph`, `override_diff`, `patch_input`, `resource_links`, `save_support`, `verify_command`), keeping argument handling and presentation out of the parsers.
- Exact-value output goldens for every decoded format plus `list`, `locate`, `verify`, `override-diff`, DLG DOT/Mermaid graphs, save listing, TLK lookup, and every human-readable text mode.

### Validated in real-world use

- Override-vs-BIFF comparison workflow used end-to-end to diagnose a modded-install bug (Kirinhale morale regression) in a single session.
- Read-extend → diagnose → patch loop used end-to-end to fix a broken Travel-region exit (ARR019 → AR1900 in a drow-mod dungeon): added Travel-region and entrance parsing to ARE, identified a destination-entrance name mismatch, repaired via `iecli patch`, verified in-game.
- `iecli verify --source override` now automates ARE cross-resource checks for dead Travel links, phantom entrances, and missing referenced scripts/actors/items.
- Whole-install PSTEE sweeps: all 859 dialogues dumped and mined for `CheckStat` gates and `PermanentStatChange` grants, plus ITM/SPL effect scans, producing the stat-plan, conversation-boon, and Law-axis guides. This is the largest read-path workload the parsers have carried, and the ITM/SPL half is what surfaced the PST effect-opcode gap fixed above.
- Whole-install IWDEE sweep: 5,995 of 5,996 resources decode across `ITM`, `SPL`, `CRE`, `ARE`, `DLG`, `BCS`, and `STO`. The single failure is `#BONECIR.SPL`, which ships with a corrupt signature byte (`SPL\x03`); repairing that byte parses the file cleanly, so it is bad shipped data rather than a parser gap.
- Whole-install BGEE sweep, the first validation target that is *heavily modded* rather than stock: 16 WeiDU mods, 7,762 override files, and SoD merged in by DlcMerger. 10,965 of 10,966 resources decode across the seven typed formats. The expectation going in was that hand-written mod resources would break the parsers where uniform Bioware data had not; that was wrong, and usefully so — every one of the 7,762 override files decoded, and the only failure was first-party. See the sweep findings below.
- Fresh-mod triage on a 2,313-resource PST mod (Blizzard in Baator) via `override-diff`, establishing that the mod is additive rather than a vanilla rewrite before any deeper investigation ran.
- Save-aware analysis: `save-info` globals joined against DLG trackers to tell "already taken" from "still available" in the Law ledger.

### Remaining or deferred

- Additional resource families: `WED`, `TIS`, `BAM`, `MOS`, `2DA`, and deeper `ARE` sections beyond the currently needed actors, entrances, and regions.
- Classic PST save support (`GAM` v1.1). PSTEE uses `GAM` v2.0 and is already covered by the current read path and the scoped item-write path.
- BG/BG2/IWD support for `save-add-item`; these variants remain hard-gated until their GAM layouts and inventory slot maps are validated.
- Broad cross-game validation. All four Enhanced Edition titles now have whole-install or substantial real-install coverage. What is missing is no longer a title but an install *shape*: every target so far has been a single merged game root.
- DLC archive mounting. `dlc/sod-dlc.zip` is a complete game-root overlay — its own `data/*.BIF`, `override/`, and `lang/<locale>/dialog.tlk` — and nothing in `ie-io` or `ie-core` reads zips. The BGEE sweep did cover SoD content only because DlcMerger had flattened it into the game root. On a default Steam BGEE+SoD install every SoD resource resolves as not-found *silently*, and SoD strrefs would resolve against the base-game TLK. Silent under-coverage is the worst failure shape for a tool that sells trustworthy extraction, so this outranks the remaining format work.
- `WMP` (worldmap) parsing, without which `verify` cannot tell a live broken exit from an unreachable leftover on BG-family installs. See the BGEE sweep findings.
- A known-issue baseline for `verify` on stock BGEE, since stock-install-verifies-clean does not generalize off IWDEE.
- `verify` beyond ARE. Other resource types are rejected rather than partially checked.
- General structured resource serialization and WeiDU patch emission.

## Driving Use Case

The project is a **tooling project** whose creative motivation is adding two custom NPCs to Planescape: Torment (Civic Festhall) with companion interjections.

PSTEE is chosen deliberately because it is mod-light: destructive iteration is cheap (Steam reinstall ~5 min), mod conflicts don't muddy signal, and the stock install is a reliable baseline.

The MVP is scoped to **maximum tooling coverage, minimum content**:

- 1 NPC (not 2) placed in AR0202
- 1 companion interject (not 4), from Fall-From-Grace (densest dialogue graph)
- 1 state-flag-gated trigger
- Packaged as a WeiDU installer

Shipping this MVP touches every format that matters: `CRE` (new + template), `DLG` (new + patch), `ARE` (actor placement), `BCS` (triggers), `TLK` (strings), plus the WeiDU install layer. Every subsequent NPC/interject after MVP is pure content scaling with zero new tooling risk.

Priorities below are ordered against this use case.

### Recently Completed

#### BGEE Validation Sweep

BGEE was the last unvalidated title. A whole-install sweep decoded 10,965 of 10,966 resources
(ITM 2294, SPL 1858, CRE 2811, ARE 601, DLG 1668, BCS 1578, STO 156), and `verify` ran across all
601 areas. Four things came out of it.

**A misfiled resource, not a corrupt one.** `CDDETECT.SPL` is indexed as a spell in `chitin.key`,
but `data/PATCH20.BIF` holds a complete, coherent `ITM V1` at that locator — valid signature, sane
offsets, `IMISC0` icon. Nothing is damaged; the index simply disagrees with the payload. This is a
different failure from IWDEE's `#BONECIR.SPL`, where one signature byte was corrupt, and the two are
only distinguishable because the error reports what it found as well as what it expected. That is
the second time the expected/found message has paid for itself.

**A resref with a space in the middle.** Stock BGEE ships `MONKTU 8.DLG` in `data/DIALOG.BIF`.
Resrefs are 8 bytes and space-padded, but here the padding falls mid-name. Lookup and `dump` handle
it correctly; `list --format text` did not, because it printed the resref bare, making one row
indistinguishable from two whitespace-separated fields. The first consumer to split that output —
the sweep harness itself — misaligned from that row onward and reported 2,444 phantom failures. Text
mode now quotes resrefs that are not purely graphic ASCII and leaves ordinary ones bare; `--format
json` was and remains the machine interface.

**A stock install is not always a clean one.** `verify` reports 359 issues, of which 10 are errors —
and 6 of those 10 are in BIF-backed shipped data (`AR2622`, `AR2624`, `AR2637`, `AR2638`, `AR4201`,
`PH0001`), not in any mod's override. `AR2621.ARE`, `ARC014Z.ARE`, and `FW0123.ARE` are absent from
the KEY, and mods only add resources, so a clean BGEE reports the same. Some are plain data typos:
`AR2637` points at `Exit2621` in `AR2638`, which offers `Exit2321` and `Exit2623`. This contradicts
the premise behind `verify_reports_no_issues_for_stock_iwdee_when_iwdee_game_path_is_set` — stock
IWDEE verifying clean is a fact about IWDEE, not a property of stock installs, and the analogous
BGEE assertion would fail on shipped data. A BGEE baseline needs a known-issue list rather than
`assert_empty`.

The remaining 349 issues are warnings, 286 of them `missing_area_script`: BG1 areas routinely name an
area script after themselves and ship no such BCS. The references genuinely dangle, and the engine
tolerates it. Severity already separates these from the 10 errors, which is what keeps the signal
visible.

**Reachability needs the worldmap.** Dumping all 601 areas and building an inbound Travel-region map
shows `AR2622` reachable from `AR2623` and `AR2637`/`AR2638` from each other, so those dangling exits
are live rather than cut content. But 53 areas have zero inbound Travel links, and in BG1 that mostly
means worldmap entry, not unreachability. `iecli` has no `WMP` parser, so inbound Travel-region count
is a weak proxy and cannot currently tell "broken exit a player will hit" from "unreachable leftover".

#### DLG Read + Graph Export

Implemented. `iecli dump --resource FOO.DLG` now exports structured dialogue JSON with states, transitions, script tables, and inline `strref` resolution, plus graph views via `--format dot` and `--format mermaid`.

Current graph slice includes:

- single-file graph rendering for DLG resources
- optional multi-file extern following via `--follow-extern[=DEPTH]`
- label controls via `--max-label-len`, `--no-triggers`, `--no-actions`, and `--strings <resolved|strref|both>`
- explicit rejection for non-DLG graph export requests

Remaining follow-up:

- verify more real PSTEE dialogues against Near Infinity
- expand regression coverage for external-dialog references and edge cases

#### Override Diff

Implemented. `iecli override-diff` now reports which override resources shadow BIFF content, distinguishes byte-identical shadows from real overrides, and can hash-compare the live override set against a reference directory or a single reference file.

Remaining follow-up:

- validate the workflow on more real modded installs beyond the Kirinhale session
- decide whether later documentation should grow a dedicated WeiDU-attribution note, without promising component-level provenance in the command itself

#### BCS Read + JSON Export

Implemented. `iecli dump --resource FOO.BCS` now exports condition/response blocks with decoded trigger/action names, weighted responses, object specifiers, and line/column-aware parse errors.

Remaining follow-up:

- broaden real-install validation beyond the current smoke coverage
- compare representative scripts against Near Infinity and encode findings in assertions
- decide whether later script-adjacent tooling should add pretty-printing or cross-reference output

#### ARE Read + JSON Export

Implemented. `iecli dump --resource AR0202.ARE` exports stable area JSON with actor placement coordinates, dialog/script/CRE links, and CRE display-name enrichment when a linked creature can be resolved.

Validated locally against PSTEE `AR0202.ARE` and `AR0500.ARE`.

Remaining follow-up:

- compare selected area actor fields against Near Infinity
- expand ARE support only when a concrete workflow needs additional region fields, doors, containers, spawn points, or ambients

#### Scalar Patching, TLK Append, and Save Item Write

Implemented:

- `iecli patch` for fixed-offset CRE/CHR fields and selected ARE Travel-region fields, with `--set` and `--patch-json` inputs and byte-exactness checks outside declared edits.
- `iecli tlk-append` for one-string append, either in place or to a copy, with an optional strref output file for scripting.
- `iecli save-add-item` for one PSTEE party member's embedded CRE. The command performs a targeted variable-length insertion, repairs known offsets, validates the result, and is copy-first unless `--in-place` is requested.

Remaining validation:

- confirm the PSTEE inventory-slot range visually in Near Infinity
- add a committed real save fixture or broader env-gated real-save coverage
- keep BG/BG2/IWD save writes disabled until each layout and slot map satisfies the validation gate in [docs/SPEC_SAVE_ITEM_WRITE_COMPLETE.md](./docs/SPEC_SAVE_ITEM_WRITE_COMPLETE.md)

#### Install Verification

Implemented. `iecli verify --game <path> --source override --format json` walks every ARE in the install, builds an entrance registry from all areas, and reports cross-resource breakage: dead Travel links, entrances named but absent in the destination area, and missing area/region scripts, actor CRE/dialog/script links, and key items. `--severity`, `--max-issues`, and `--format text|json` shape the output; unparseable areas surface as `parse_error` issues rather than aborting the run.

Validated against a stock IWDEE install, which now reports zero issues. Getting there fixed two false-positive classes: entrance names were compared case-sensitively (stock IWDEE ships `Fr3501` against a declared `FR3501`, and the transitions work in game), and a `NONE` resref in a script or dialog slot means "empty" rather than a missing resource (IWDEE has 1, PSTEE has 92). A stock install is the strongest oracle available for this command — whatever the shipped game does is by definition not breakage.

Remaining follow-up:

- extend beyond ARE only when a workflow needs it; other `--resource-type` values are rejected today
- decide whether warnings should gain per-category suppression once a large modded install produces enough noise to justify it

#### Game-Variant Awareness

Implemented. Installations are classified as `standard`, `iwd`, or `pst` from stable root files rather than folder names, since install directories are routinely renamed. The variant rides on `ResourceMetadata` and drives effect-opcode decoding, so PST effects decode against the PST table instead of the BG one.

`locate` reports the detected variant as `game_variant`, so a misdetected install (a repackaged PST without `torment.lua`/`torment.exe` at the root) can be caught before its effect names are trusted. Verified against the real BG2EE, PSTEE, and IWDEE installs, which report `standard`, `pst`, and `iwd` respectively.

`iwd` sharing the standard opcode table is now confirmed rather than assumed. IWDEE items whose own names and descriptions state the stat they grant decode correctly under the standard table — "Manual of Bodily Health" as Constitution, "Tome of Clear Thought" as Intelligence, "Gauntlets of Elven Might" as Strength 18 — and those anchors are pinned as tests, so a wrong table would surface as a name/label disagreement rather than a silent mislabel.

Remaining follow-up:

- extend variant awareness to any other decode path where PST/IWD diverge from BG, as concrete mismatches are found
- extend the opcode tables themselves: 28,885 of 70,696 opcode instances in IWDEE resolve to a name today. Unnamed opcodes surface as raw values with `decoded: null` rather than wrong names, so this is coverage, not correctness. The most common unnamed ones there are 233, 206, 324, 267, and 83.

#### Override Lookup Performance

Fixed. Override resolution rescanned the entire override directory, with a `stat` per entry, on every lookup that was not an exact-case hit — including every resource that lives in a BIF, since override is checked first. The cost scaled with override size, and stock PSTEE ships 3,398 override files while BG2EE ships 2,997.

Each override directory is now indexed once on first use:

| Workload | Before | After |
| --- | --- | --- |
| PSTEE `dump` of one area (9 actors) | 60s | 0.5s |
| PSTEE `verify` (251 areas) | >10min (timed out) | 14s |
| BG2EE `verify --source override` | >80min | 2min |
| `cargo test --workspace` with all three real installs | >80min | 2min |

This was invisible until now precisely because it was slow: the env-gated `verify` smoke test never finished, so its result had never been seen, and PSTEE `verify` was impractical on the game the driving use case targets.

Remaining follow-up:

- creature link enrichment still reads and fully decodes a CRE per actor to extract two strrefs, with no memoization across areas. Now that the override scan is gone this is the next cost if area-heavy workloads need to get faster.

#### Agent Skill Layer and Research Guides

Implemented, and the project's clearest demonstration of the vision: parser emits JSON, skill wraps it in narrative an IE modder can act on without scripting.

Six skills ship — `diagnose-dialog`, `explore-dungeon`, `map-stat-gates`, `plan-stat-build`, `mod-diff`, and `trace-quest-timer` — in two behaviorally aligned packagings: [`.claude/skills/`](./.claude/skills/) for Claude Code and [`skills/`](./skills/) for Codex. Each is documented with its scripts and flags in [docs/SKILLS.md](./docs/SKILLS.md).

Their output is captured in [`docs/guides/`](./docs/guides/): a PST stat plan, conversation-boon catalogue, Law-axis ledger, mod delta, and a reward map for the Blizzard in Baator mod. Each guide states its provenance and reproduction command, so a reader can re-derive it against their own install.

Remaining follow-up:

- keep guides marked with the install they were derived from; they describe a specific modded install, not vanilla
- generalize the PST-tuned skills to BG/IWD only when a real question demands it

## Next Milestones

### Selection pending

No new implementation milestone has been selected as of 2026-08-12. Work since the last snapshot has been read-path breadth (PST variant support, install verification, the skill layer and guides) rather than a new tooling milestone. The next planning decision should choose among the following already identified work, rather than starting all of it:

- pay down validation debt with real fixtures, Near Infinity comparisons, and stable JSON snapshots for existing formats
- exercise the one-NPC/one-interjection PSTEE MVP end to end, using WeiDU for installation rather than adding speculative general-purpose writers
- validate and enable `save-add-item` for one additional game family
- close the IWDEE gap: validate discovery, decoding, and the `iwd` variant path against a real install

Until that decision is made, do not add another resource family or broaden write support.

## Write Support Framework

Write support is framed in three tiers to avoid the "boil the ocean" failure mode:

- **Tier 1 — Scalar poke.** Edit known fields at known offsets. No offset/count recomputation. Byte-exact outside the edited field. Low risk. This is implemented for selected CRE/CHR and ARE fields.
- **Tier 2 — Structured write.** Full JSON-in, bytes-out for a format. Requires exhaustive parsing or explicit opaque-range preservation so unknown bytes survive round-trip. Real risk of silently producing engine-valid-but-wrong files. Test burden: byte-exact round-trip on every real fixture.
- **Tier 3 — Patch emission.** JSON diff → WeiDU `.tp2`/`.d` script. The novel ecosystem contribution, but multiplies Tier 2 across every format plus adds patch-syntax generation. Months of work, and only valuable after Tiers 1–2 have proven themselves on real workflows.

`save-add-item` is a deliberate exception between Tier 1 and general Tier 2: it performs one narrowly specified variable-length insertion for a concrete PSTEE workflow, preserves untouched bytes, repairs an explicit offset set, and hard-gates unvalidated game families. It is not a general CRE or GAM serializer and should not be described as one.

Current rule: **ship scalar edits or narrowly scoped mutations only for a concrete workflow and with byte-preservation tests, explicit layout validation, and a documented safety gate. Do not start general Tier 2 serialization or Tier 3 patch emission speculatively.**

## Agent-Assisted Development Loop

The project itself practices what it preaches. The intended loop for future work:

1. A real use case (driving use case above, or a user-reported issue) exposes friction.
2. The friction is written up as a GitHub issue with motivation, proposal, and acceptance criteria — or captured directly in this roadmap if it aligns with an active milestone.
3. A coding agent picks up the issue (webhook-triggered, scheduled, or manual `/loop`), implements it with tests, opens a PR.
4. The PR is reviewed against the issue's acceptance criteria.
5. The next real use case exercises the new capability, surfaces the next friction.

This file is a durable artifact for that loop. Keep it current. Old completed milestones can move to a "Done" section or into commit history; fresh priorities go to the top of "Next Milestones."

## Pointer Summary

- **[AGENTS.md](./AGENTS.md)** — engineering principles, output rules, validation workflow, prompt templates. Stable.
- **[TODO_PRIORITIES.md](./TODO_PRIORITIES.md)** — full P0–P3 backlog with completion status. Tactical.
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** — crate layout, domain model.
- **[README.md](./README.md)** — user-facing intro and CLI surface.
- **[docs/SKILLS.md](./docs/SKILLS.md)** — inventory of the shipped Claude Code skills, their triggers, scripts, and flags.
- **[docs/guides/](./docs/guides/)** — analysis guides produced by running those skills against real installs.
- **[docs/PARSER_COVERAGE.md](./docs/PARSER_COVERAGE.md)** — per-format matrix of what is decoded, deferred, or left raw.
- **[docs/TESTING.md](./docs/TESTING.md)** — env-gated real-install test setup.
- **This file** — current vision, next milestones, write-support framework, driving use case. Revise each session.
