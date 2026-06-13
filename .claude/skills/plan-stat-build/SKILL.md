---
name: plan-stat-build
description: Plan an Infinity Engine protagonist's stat build from the actual install — which attributes to set at creation, how to spend level-up points, and a sequenced itinerary of where to grab every permanent stat boost — by extracting the dialogue stat-gates and the permanent stat economy and synthesizing them against the run's goal. Use when the user asks how to assign/optimize stats, plan a character or run, min-max a playthrough, or "what stats should I pump". Tuned for PST; the gate half generalizes to BG/IWD.
---

# Plan Stat Build

Generative companion to the diagnostic skills: nothing is broken — the user wants to *play well*.
Produce a full-game stat plan grounded in the actual install (so it reflects mods, not just guides):
creation spread → level-up priority → a sequenced "grab this boost here" itinerary, optimized for the
stated goal.

The scripts do the mechanical extraction; **the agent does the synthesis** (resolving exclusivity and
weighing trade-offs). Read the gotchas below — they are the hard-won part.

## Inputs

- Game install path (auto-memory / `docs/LOCAL_GAME_PATHS.md`).
- Protagonist object — `Protagonist` (PST) or `Player1` (BG/IWD).
- The **goal** — clarify if unstated, it changes everything: *see-everything completionist* vs
  *combat power* vs *balanced*. Also the creation rules (point pool, per-stat cap) and intended class
  — ask the user; these come from the game/char-creation screen, not the files.

iecli must be built (`cargo build --release`). **Verify the PST opcode fix is in** before trusting
item effects: if Step 2's item-effect stats are only Wisdom/Strength (other stats blank), iecli is
mis-decoding PST opcodes — see `docs/BUG_pst_opcode_table.md`.

## Step 0 — trust layer (modded installs)

Know what the mod changed before planning against it. Run the `mod-diff` skill (or
`iecli override-diff`) and note any stat-granting DLG/ITM that differs from stock — those grants may
not behave as community guides describe.

## Step 1 — content map (what the gates are *worth*, then how much)

```bash
python .claude/skills/map-stat-gates/gate_payoffs.py   --game "<game-path>" --protagonist Protagonist  # value: lead here
python .claude/skills/map-stat-gates/gate_histogram.py --game "<game-path>" --protagonist Protagonist  # volume: secondary
```

Lead with payoffs — a stat that gates hundreds of *flavor* lines is still a dump (modded CON gated 20
replies, 0 of value). Note the **early-firing high-value gates** (e.g. a CHA/INT-gated Ravel payout):
those must be covered by *base* stat at creation, since you can't buff in time — they set creation floors.

## Step 2 — permanent stat economy (what you can gain, and where)

```bash
python .claude/skills/plan-stat-build/stat_economy.py --game "<game-path>" --protagonist Protagonist
# --no-spl to skip the spell scan if it's slow / empty
```

Prints, **RAW**: permanent dialogue grants (`PermanentStatChange`, by stat, with the granting DLG),
and ITM/SPL stat effects split into *permanent* (consumed) vs *equip* (tattoos, rings, while-worn).

## Step 3 — resolve exclusivity (the part scripts can't)

The Step-2 counts **overcount** — they sum mutually-exclusive choices. For each stat-granting DLG,
`iecli dump <DLG> --strings resolved` and read the tree: multiple `PermanentStatChange` under
different transitions of the *same state* are pick-one. Check cross-DLG exclusivity too. Compute the
**achievable max per stat in one playthrough**, not the naive sum.

## Synthesis — the gotchas to bake into the plan

1. **Gate-value ≠ combat-value (AD&D 2e).** Ability *bonuses* don't start until **16** (STR/CON) or
   **15** (DEX AC). A stat at 13 is a *dialogue-gate* value, not a combat one. Percentile Strength
   (18/xx) is **warrior-only** — a Mage's STR is near-useless in a fight. Never recommend pumping a
   physical stat "for combat" below its bonus threshold.
2. **Caps and overcap.** Stats cap (25 in PST). Late grants that would exceed the cap are wasted — so
   size creation investment so that *creation + reliable grants* lands at/under the cap, not over it.
   The stat with the most/latest recovery is the one to invest LESS at creation (it climbs for free);
   the stat that recovers little must be bought up front.
3. **Effective stat clears gates; acquisition timing limits it.** `CheckStat` reads the current
   (item-modified) stat, so a +N stat tattoo/buff equipped before a conversation passes a gate N above
   base. BUT most strong stat gear is progress-gated to mid/late game, so **early/mid walls still need
   base stat** — tattoos relax late checks, not creation. (PST has a +1/+2 tattoo for every stat, plus
   +3 class tattoos; some stats *also* have an XP or per-class wrinkle, e.g. PST Wisdom gives an XP
   bonus computed from *effective* WIS, rewarding early investment and the "buff WIS before quest
   turn-in" trick.)
4. **Class & milestone bonuses.** If the protagonist multiclasses (PST), the one-time
   specialization bonuses go to the *first* class to reach each level milestone — a real, plannable
   chunk of stats (e.g. PST Mage double-spec = +3 INT / +1 WIS). Account for them.
5. **Goal weighting.** Completionist → weight the gate histogram (INT/CHA/WIS-style talk stats) and
   the experiential payoffs; power → weight class mechanics and the bonus thresholds.

## Deliverable

Write `docs/<GAME>_STAT_PLAN.md` (+ a chat summary) with: a creation spread table, level-up priority,
the sequenced permanent-boost itinerary (early→late, with the DLG/quest for each and any cost), and an
honest caveats section (what's out of reach, mod deltas, redundant routes). `PST_STAT_PLAN.md` in this
repo is the worked reference.

## When NOT to use

- "Which stat does check X want?" — `iecli dump <DLG>` directly.
- Just the content map, no build — use `map-stat-gates`.
- A game with no `CheckStat` dialogue gating and no `PermanentStatChange` — there's little to plan.
