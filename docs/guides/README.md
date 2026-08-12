# Guides

Analysis written by the [project skills](../SKILLS.md) against real game installs. Where the rest of
`docs/` documents the tool, these document *games* — they are the tool's narrative output, and the
practical evidence that the parser-produces-JSON / skill-produces-narrative loop works end to end.

Two things to know before reading any of them:

- **They describe a specific install, not vanilla.** Every guide names the install and mods it was
  derived from. A modded install's numbers are the point — but they are not the stock game's numbers.
- **They are reproducible.** Each guide carries a methodology or "Reproducing" section with the
  commands that generated it, so the analysis can be re-run against a different install rather than
  trusted on faith.

| Guide | Subject | Produced with |
|---|---|---|
| [PST_STAT_PLAN.md](PST_STAT_PLAN.md) | Full stat plan for the Nameless One — creation spread, level-up priority, and a sequenced itinerary of permanent boosts, tuned for a balanced completionist run | `plan-stat-build`, `map-stat-gates` |
| [PST_STAT_PLAN_MOD_DELTA.md](PST_STAT_PLAN_MOD_DELTA.md) | Companion to the stat plan: what a newly-installed mod changes for stat planning, and why the vanilla plan still holds | `mod-diff`, `map-stat-gates`, `plan-stat-build` |
| [PST_CONVERSATION_BOONS.md](PST_CONVERSATION_BOONS.md) | Every dialogue that permanently raises a stat or pays out large XP, including companion boons — from a sweep of all 859 dialogues in the install | `map-stat-gates`, `iecli dump` |
| [PST_LAW_LEDGER.md](PST_LAW_LEDGER.md) | Save-aware cross-reference of every dialogue choice that moves the Law/Chaos axis, marking each as already taken, still available, or untrackable | `iecli save-info` + DLG sweep |
| [BLIZZARD_IN_BAATOR_NPC_REWARDS.md](BLIZZARD_IN_BAATOR_NPC_REWARDS.md) | Which NPCs in the Blizzard in Baator mod (PST:EE) grant big XP, permanent stat boons, or items, and exactly what unlocks each | mod `.d` sources reconciled against `iecli` CRE dumps |

All five currently cover Planescape: Torment, which is the project's [driving use case](../../ROADMAP.md#driving-use-case).
The skills behind them are not PST-only — `map-stat-gates` and `trace-quest-timer` work across BG/IWD/PST — so
BG and IWD guides are a matter of running them, not of new tooling.
