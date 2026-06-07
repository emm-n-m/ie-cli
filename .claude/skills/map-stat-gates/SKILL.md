---
name: map-stat-gates
description: Histogram an Infinity Engine game's dialogue stat-checks (CheckStat) by required value, per stat, to reveal which attribute thresholds unlock the most content. Use when the user asks which stats/values matter for dialogue, what stat checks a game or mod has, where a stat gates content, or wants the content-map half of a character build. Works for BG / IWD / PST.
---

# Map Stat Gates

Scan every dialogue in an IE install for `CheckStat*` checks on the protagonist and report,
per stat, **how many gated branches require each value** — i.e. which attribute thresholds open
the most content. This is the deterministic content-map; the agent interprets which breakpoints
are worth hitting. (For the full build pipeline — permanent stat gains, gear, synthesis — use the
`plan-stat-build` skill, which calls this script as its first step.)

## Inputs

- Game install path. Use the user's install from auto-memory (see `docs/LOCAL_GAME_PATHS.md`)
  unless they specify another.
- Protagonist object name — `Protagonist` (PST) or `Player1` (BG/IWD). The script matches either
  by default; pass `--protagonist any` to count checks on *all* objects.

iecli must be built: `cargo build --release` from the repo root (the script auto-prefers the
release binary; the DLG sweep is ~hundreds of dumps, so release is much faster than debug).

## Step — run the histogram

```bash
python .claude/skills/map-stat-gates/gate_histogram.py \
    --game "<game-path>" \
    --protagonist Protagonist          # or Player1 / any
# add --json for machine-readable output
```

Per stat it prints the up-gated (`CheckStatGT`) histogram — `need >= V : N branches` — sorted by
required value, plus a low-stat (`CheckStatLT`) summary.

## Interpreting the output

- **Breakpoints are the big jumps.** A row like `need >= 15 : 229` means 229 gated branches open at
  that value — a wall worth hitting. Small tail rows (`need >= 21 : 6`) are low-yield.
- **The count is *distinct gated trigger conditions*, not every player reply** — a good proxy for
  "how much content sits behind this threshold," not an exact branch count.
- **High-stat and low-stat checks are usually mutually exclusive** — a line often has a `CheckStatGT`
  variant *and* a `CheckStatLT` variant of the same beat; you only ever trigger one. So "see
  everything" means hitting the **up-gated** thresholds (the GT histogram); the low-stat branches are
  mostly the flavor you forgo, not extra content to chase.
- **`CheckStat` reads the *current* (item-modified) stat** — equipment/buffs let you clear a gate
  above your base value. So a threshold isn't strictly a base-stat requirement (the `plan-stat-build`
  skill leans on this).

## Reporting back

Summarize: the top breakpoint per stat, which stats gate the most total content (sum of branches),
and which stats barely gate anything (dump candidates). Flag any stat whose checks cluster at one
value (a single must-hit threshold) vs. spread across many (escalating investment).

## When NOT to use

- You want a full build plan (creation spread, level-ups, where to grab permanent boosts) — use
  `plan-stat-build`, which builds on this.
- A single specific check ("what does NPC X want?") — `iecli dump <DLG>` and read the trigger directly.
