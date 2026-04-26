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

## Status Snapshot

Current as of 2026-04-25.

### Done

- Workspace, crate structure, error types, shared JSON conventions.
- Installation discovery: game root validation, `chitin.key`, language folder, `dialog.tlk`.
- Typed `KEY` parsing.
- `BIFF` / `BIF` / `BIFC` raw reads.
- Override precedence, with `--source <auto|override|bif>` opt-out (`locate`, `dump`, `dump-raw`).
- Resource enumeration: `list --type <T> --name <glob> --source <S> --format <text|json>`.
- `TLK` string resolution.
- Typed decoders + JSON export for: `ITM`, `SPL`, `CRE`, `STO`, `DLG`, `BCS`.
- Minimum viable `ARE` decoder + JSON export for header, deferred-section offsets/counts, and actor placement/link data.
- Lazy IDS loading and opcode/name resolution for BCS decoding.
- Real-install smoke coverage for `ITM` and `SPL`; selected Near Infinity comparisons for `SPL`.
- Env-gated CLI smoke coverage for `BCS` and PSTEE `ARE`.
- Initial CRE scalar patch support for fixed-offset fields, with byte-exact copy-only behavior.
- Initial ARE region patch support for `regions.<selector>.destination_entrance` and `regions.<selector>.destination_area`, addressed by region name or 0-based index, byte-exact copy-only behavior.

### Validated in real-world use

- Override-vs-BIFF comparison workflow used end-to-end to diagnose a modded-install bug (Kirinhale morale regression) in a single session.
- Read-extend → diagnose → patch loop used end-to-end to fix a broken Travel-region exit (ARR019 → AR1900 in a drow-mod dungeon): added Travel-region and entrance parsing to ARE, identified a destination-entrance name mismatch, repaired via `iecli patch`, verified in-game.
- `iecli verify --source override` now automates ARE cross-resource checks for dead Travel links, phantom entrances, and missing referenced scripts/actors/items.

### Not started

- `ARE`, `WED`, `TIS`, `BAM`, `MOS`, `2DA`.
- Write support at any tier (see below).
- JSON golden/snapshot tests for any decoded format.
- Cross-game validation beyond BG2EE (BGEE, PSTEE, IWDEE).

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

#### DLG Read + JSON Export

Implemented. `iecli dump --resource FOO.DLG` now exports structured dialogue graphs with states, transitions, script tables, and inline `strref` resolution.

Remaining follow-up:

- verify more real PSTEE dialogues against Near Infinity
- expand regression coverage for external-dialog references and edge cases

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
- expand ARE support only when a concrete workflow needs regions, doors, containers, spawn points, or ambients

## Next Milestones

### M7: CRE Write — Tier 1 (Scalar Edits)

**Why third:** enables the agent to template a new NPC by copying an existing Festhall CRE and scalar-editing a handful of fields (morale, stats, dialog resref, scripts, name strref).

**Scope:**

- `iecli patch --resource FOO.CRE --set morale=9 --set reputation=15 --output ./FOO.CRE`
- Accept a JSON patch file alternative: `--patch-json ./patches.json`.
- Scalar fields only — no variable-length section edits (no adding/removing effects, items, spells). That is Tier 2 and deferred until a use case demands it.
- Copy input bytes verbatim, overwrite only the specified byte ranges.
- Validate field name and value bounds before writing.

Current slice delivered:

- `iecli patch` for CRE/CHR resources.
- `--set field=value` and `--patch-json` inputs.
- Fixed-offset scalar coverage for names, morale, morale break, morale recovery time, reputation, dialog, and script resrefs.
- Unit tests for copy-only byte exactness, offset-limited edits, unknown fields, out-of-range numeric values, and invalid resrefs.

**Acceptance criteria:**

- Round-trip byte-exactness when no fields are set (copy-only).
- When fields are set, only the specified byte ranges differ from input.
- Unknown field name or out-of-range value produces a clear error before any write.
- Real installed game can load the patched CRE without error.
- Covers at least the fields exercised in the Kirinhale debug session (morale, morale_break, scripts, names).

### M8: TLK Append

**Why fourth:** every new dialogue line and NPC name requires a new strref, and strrefs only exist in `dialog.tlk`. Without append support, the WeiDU layer has to do it, which is fine for shipping but unhelpful for iterative agent-driven development.

**Scope:**

- `iecli tlk-append --game /path --text "..." --output-strref-to /tmp/strref.txt`
- Appends to `dialog.tlk` (or a working copy — see below) and returns the new strref.
- Option to write to a copy: `--tlk-out ./dialog-patched.tlk`.

**Acceptance criteria:**

- Real installed game loads the modified TLK and resolves the new strref correctly in-game.
- Original TLK untouched if `--tlk-out` is given.
- Handles the TLK format variant actually used by PSTEE correctly.

### Later

- `CRE` Tier 2 write (variable-section edits) — deferred until a concrete scenario requires it.
- WeiDU `.tp2` / `.d` emission from JSON diffs — this is Tier 3 write support and the genuinely novel capability. Defer until M6–M8 have been exercised end-to-end.
- JSON golden tests across all decoded formats.
- Cross-game validation (PSTEE, BGEE, IWDEE).

## Write Support Framework

Write support is framed in three tiers to avoid the "boil the ocean" failure mode:

- **Tier 1 — Scalar poke.** Edit known fields at known offsets. No offset/count recomputation. Byte-exact outside the edited field. Low risk. Covers ~80% of "one wrong byte" bugs. *This is the only tier planned in the active roadmap.*
- **Tier 2 — Structured write.** Full JSON-in, bytes-out for a format. Requires exhaustive parsing or explicit opaque-range preservation so unknown bytes survive round-trip. Real risk of silently producing engine-valid-but-wrong files. Test burden: byte-exact round-trip on every real fixture.
- **Tier 3 — Patch emission.** JSON diff → WeiDU `.tp2`/`.d` script. The novel ecosystem contribution, but multiplies Tier 2 across every format plus adds patch-syntax generation. Months of work, and only valuable after Tiers 1–2 have proven themselves on real workflows.

Current rule: **ship Tier 1 when a concrete need appears; do not start Tier 2 or Tier 3 speculatively.**

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
- **This file** — current vision, next milestones, write-support framework, driving use case. Revise each session.
