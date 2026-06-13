---
name: map-stat-gates
description: Map an Infinity Engine game's dialogue stat-checks (CheckStat) — both which thresholds gate the most content AND, more importantly, what those gated replies actually DO (stat boosts, items, quests, big XP vs. flavor). Use when the user asks which stats/values matter for dialogue, what stat checks a game or mod has, where a stat gates content, which stat gates the most *valuable* content, or wants the content-map half of a character build. Works for BG / IWD / PST.
---

# Map Stat Gates

Scan every dialogue in an IE install for `CheckStat*` checks on the protagonist. Two views:

1. **Payoffs (the one that matters)** — *what* each stat-gated reply does (a permanent stat boost,
   an item, a quest step, big XP — vs. flavor). **Counts mislead**: a stat can gate hundreds of lines
   that are almost all "snap the zombie's neck" flavor (e.g. modded CON gated 20 replies, *0* of value),
   while one WIS-gated line hands you 120k XP. Always lead with this.
2. **Volume** — how many gated branches sit at each threshold (the histogram). Useful for "how much
   content exists behind value N", but a weak proxy for value — read it *after* the payoffs.

Both are deterministic extraction; the agent judges value from the raw `action_text`/`journal_text`
the payoff view surfaces. (For the full build pipeline — permanent stat gains, gear, synthesis — use
the `plan-stat-build` skill, which calls these as its first step.)

## Inputs

- Game install path. Use the user's install from auto-memory (see `docs/LOCAL_GAME_PATHS.md`)
  unless they specify another.
- Protagonist object name — `Protagonist` (PST) or `Player1` (BG/IWD). The script matches either
  by default; pass `--protagonist any` to count checks on *all* objects.

iecli must be built: `cargo build --release` from the repo root (the script auto-prefers the
release binary; the DLG sweep is ~hundreds of dumps, so release is much faster than debug).

## Step 1 — payoffs (lead with this)

```bash
python .claude/skills/map-stat-gates/gate_payoffs.py \
    --game "<game-path>" --protagonist Protagonist     # or Player1 / any
# --high-only to drop flavor; --json for the full records (agent reasons on these)
```

Joins each `CheckStatGT`-gated reply to its `action_text` + `journal_text`, dedups, and tags a
**coarse** category as a sort key: `STAT_CORE` (PermanentStatChange to a core stat), `SKILL`
(PermanentStatChange to a thief skill), `COMPANION`, `ITEM`, `XP_BIG`/`XP_small`, `QUEST` (has a
journal entry), `STORY`, `TRAVEL`, `STATE` (sets a global — could be quest progress or trivial),
`FLAVOR` (nothing). Each record also carries the **full trigger** (so co-conditions / rough timing
are visible) and the **raw action** — *you* judge value from those, the category is only a sort.

## Step 2 — volume (secondary)

```bash
python .claude/skills/map-stat-gates/gate_histogram.py --game "<game-path>" --protagonist Protagonist
```

Per stat: the `CheckStatGT` histogram (`need >= V : N branches`) + a `CheckStatLT` summary. Tells you
*how much* content sits behind a value — interpret only after the payoffs.

## Interpreting

- **Lead with the high-value counts, not the totals.** "INT 794 gated / 57 high-value" and
  "CON 20 gated / 0 high-value" tell very different stories than the raw counts. A stat that gates
  thousands of flavor lines is still a dump stat.
- **Read the raw actions for the top hits** — a `GiveExperience(...,120000)` or a
  `PermanentStatChange` is a jackpot; a `SetGlobal("X_talked",1)` is nothing. The `STATE` bucket
  especially needs your eyes (quest-progress globals vs trivial flags).
- **Timing matters as much as value.** The full trigger often shows co-conditions (a chapter/quest
  global) that hint *when* a gate fires. A high-value gate that fires **early** (before you can boost
  the stat) must be covered by **base** stat at creation — see `plan-stat-build`.
- **`CheckStat` reads the *current* (item-modified) stat** — equipment/buffs clear a gate above base,
  so a threshold isn't strictly a base-stat requirement (except for early gates you can't yet buff).
- **High/low variants are usually mutually exclusive** — "see everything" means the up-gated (`GT`)
  side; the `LT` branches are flavor you forgo.

## Reporting back

Lead with: per stat, **how many gated replies actually pay off** and what the biggest payoffs *are*
(name them). Then which stats are dumps *by payoff* (not just low count). Flag the early-firing
high-value gates that force a creation floor.

## When NOT to use

- You want a full build plan (creation spread, level-ups, where to grab permanent boosts) — use
  `plan-stat-build`, which builds on this.
- A single specific check ("what does NPC X want?") — `iecli dump <DLG>` and read the trigger directly.
