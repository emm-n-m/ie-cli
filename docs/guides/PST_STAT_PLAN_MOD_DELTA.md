# PST Stat Plan — New Mod Delta

Companion to [PST_STAT_PLAN.md](PST_STAT_PLAN.md): what the newly-installed mod changes for stat
planning. Produced by re-running the `mod-diff` / `map-stat-gates` / `plan-stat-build` skills against
the modded `Project P` install and diffing against the vanilla baseline.

## Trust layer — the mod is additive

`mod-diff` shows the mod added **2,313 new resources** (+174 DLG, +134 ITM, +42 SPL, +253 CRE,
+43 ARE; namespaces `G-BB*`, `ARG-*`, `Q!*`) but **changed almost no vanilla files** (MOD-CHANGED DLG
77→78). So **the vanilla plan still holds** — this is new content bolted on top, not a rewrite. The
creation spread, class plan, and vanilla itinerary in the base doc are unaffected.

## What the gates changed (content map)

The mod adds stat-gated dialogue across the board; breakpoints didn't move, but two things shift:

| Stat | Vanilla | Modded | Δ | Note |
|---|---:|---:|---:|---|
| INT | 589 | 787 | +198 | **16 tier 70→129, 18 tier 13→42** — pushing INT past 15 pays more |
| CHR | 451 | 600 | +149 | **opens a real 16–22 ladder** (16: 50→102; new 18/19/20/22 content) |
| WIS | 243 | 365 | +122 | proportional; 13 still the cliff |
| STR | 81 | 136 | +55 | proportional, still bottom-tier |
| DEX | 95 | 144 | +49 | proportional |
| **CON** | ~2 | **20** | **+18** | count rose, but **payoff = 0** (see below) — still a dump |

⚠ **Counts are a weak proxy — payoff matters.** A payoff pass (classifying each gated *reply* by its
`action_text`/`journal_text`) shows most gated lines are flavor/state-setting, not rewards:

| Stat | gated replies | high-value (stat/item/quest/big-XP) |
|---|---:|---:|
| INT | 794 | 57 |
| CHR | 502 | 47 |
| WIS | 365 | 32 (incl. the most stat-boosts) |
| DEX | 144 | 22 (mostly thief-training) |
| STR | 135 | **1** |
| CON | 20 | **0** |

So **CON's count jump is hollow — all 20 gates are flavor, zero payoff; it stays a dump.** STR is the
same (1 of 135). The high INT/CHA *counts* are real but ~⅔ flavor; the *valuable* high-tier content is
thinner than the histogram implied. Biggest single payoff in the mod: **`G-BBD059` — 120,000 XP + a
permanent stat change, gated at WIS ≥ 15** (free for a WIS-max build).

## New stat boosters

### New dialogue grants (`PermanentStatChange`, mod namespace)

| Dialogue | Grant | Notes |
|---|---|---|
| `G-BBD010` | **+1 to ALL six** | 3 grant-states ("Rise from the bloodied stone"); a 3-step ritual or revisit-repeatable — **possibly +3 to all**. ⚠ verify in play |
| `G-BBD008` | **+2 to one chosen stat** (CON/INT/STR/WIS/CHR) | wish-style pick-one; one branch costs −1 CON |
| `G-BBD157`, `G-BBD016` | +1 CON | |
| `G-BBD042` | +2 CHR | |
| `G-BBD156` | +2 STR | |
| `G-BBD066` | +1 INT | |
| `G-BBD208` | +1 CON / −1 CHR | a trade |
| `DANNAH` / `DDAKKON` / `DMORTE` | +1 CON each | companion grants (8 guarded branches each → ~+1; ⚠ confirm) |

The CON theme is deliberate — the mod that *adds CON gates* also *adds CON grants*.

### New stat gear (equip — the headline, aimed at the new high gates)

| Item | Effect | Slot/kind |
|---|---|---|
| **Mint Earring** (`G-BBI113`) | **+3 CHA** | earring |
| **Cerebral Parasite Earring** (`G-BBI105`) | **+3 INT**, +1 WIS, −1 CON | earring |
| **`QDTTPR2`** (quest tattoo) | **+3 WIS** | tattoo |
| **Anulus Demonis** (`G-BBI070`) | +3 STR, +3 DEX, +3 CON, **−5 CHA** | ring (physical build) |
| **Flame of Order** (`G-BBI026`) | +1 WIS, +2 CHA | weapon (wielded) |
| Philosopher's Eye (`G-BBI022`) | +1 WIS, +1 INT, −1 DEX | eyes |
| Mask of Sonne (`G-BBI079`) | +1 CHA | |
| Soego's Holy Symbol (`G-BBI013`) | +2 DEX, −1 CHA | |
| Agraval's Molar (`G-BBI073`) | +1 CON, −1 CHA | |

These stack across *different* slots (earring + tattoo + eyes + ring + weapon), so late-game
**effective** INT/CHA/WIS can climb well past base — e.g. CHA: base + grants + Greater Presence tattoo
(+2) + Mint Earring (+3) + Flame of Order (+2) reaches into the **new 18–22 CHR ladder**. Usability:
the earrings/tattoo read as broadly equippable; **confirm TNO can wear each in play.**

### Not for you
- **Candy of Might** (`G-BBI071`): +4 STR / +2 DEX / +2 CON, but very restrictive usability + "for
  Morte" in the name → **Morte-only**, not a TNO booster.

### Vanilla items the opcode fix revealed (were mislabeled last session)
The PST opcode fix means non-WIS item stats now decode, surfacing **vanilla** permanent consumables
the original plan missed: **Gordian Knot +2 CHA / −1 WIS**, **Tear of Salieru-Dei +1 CON**,
**Xachariah's Heart +1 DEX**. Fold these into the vanilla itinerary too.

## How the build shifts

1. **CON stays a dump (corrected).** The count jump was hollow — its 20 gates pay out nothing. It'll
   *passively* drift up from the new CON grants/gear if you grab them, but there's no content reason to
   *invest* a point in it. Same as vanilla.
2. **Aim INT/CHA higher.** The denser 16/18 (INT) and 16–22 (CHA) content, plus the +3 INT / +3 CHA
   earrings, mean the late game can clear gates the vanilla plan wrote off. Keep the same creation
   floors (the early walls are unchanged), but plan to **stack the new stat gear late** for the high
   tiers rather than treating 16–18 as the ceiling.
3. **Creation spread unchanged.** Early/mid walls are still INT 15 / CHA 13 / WIS 16 / DEX 13, and the
   boosters/gear are mid-late, so they relax the *ceiling*, not the *floor* — same as vanilla tattoos.

## Verify in play
- `G-BBD010`: is "+1 all" once, a 3-step ritual (+3 all), or revisit-repeatable? Big if it stacks.
- TNO usability of the new earrings/tattoo (Mint, Cerebral Parasite, `QDTTPR2`).
- Exclusivity of the companion CON grants (the 8 guarded branches per companion).

*Generated 2026-06-13 by re-running the stat skills on the modded install. One bug fixed en route: the
skill scripts decoded iecli output as cp1252 and crashed on the mod's non-cp1252 item strings — now
forced to UTF-8.*
