---
name: trace-quest-timer
description: Trace how long until an Infinity Engine quest or companion event fires by finding the script timer that gates it, reporting the wait converted to real time and classified game-time (rest-skippable) vs real-time (must actually play). Use when the user asks "how long until X happens", "when does <companion> return/reappear", "how long before <quest> activates", or wants to know a script delay/cooldown.
---

# Trace IE Quest / Companion Timer

Answer "how long until event X?" for Infinity Engine quests and companions by
finding the `(Real)SetGlobalTimer` that gates the event, reading its duration,
and — critically — saying whether it's a **game-time** timer (resting and
fast-travel skip it) or a **real-time** timer (counts actual play seconds, not
skippable). The same constant means very different waits in each: `FOUR_HOURS`
= 1200, which is 4 game hours under `SetGlobalTimer` but ~20 real minutes under
`RealSetGlobalTimer`.

## Inputs

- The companion/NPC whose event you're timing, as a **resref prefix** (e.g.
  `HEXXAT`, `JAN`). The companion's DLG/BCS resources share this prefix.
- Game install path. Read `docs/LOCAL_GAME_PATHS.md` for the user's installs
  (BG2EE is the usual target for companion quests); don't probe Steam folders.

iecli must be built: from the repo root, `cargo build --release`. The script
defaults to `target/release/iecli.exe`.

This skill depends on the BCS action parser fix (commit "Fixed BCS bug",
3a262d9b) — timer durations come from `int_args[0]` of `SetGlobalTimer`
actions, which were mislabeled before that commit.

## Step 1 — Run the tracer

```bash
python .claude/skills/trace-quest-timer/trace_timer.py \
    --game "<BG2EE-path>" \
    --prefix HEXXAT
```

Filter to one timer when you know the event's global, e.g. `--timer cabrina`.
Add extra files that aren't under the prefix with `--resource LISSA.DLG`
(repeatable) — see "Chasing a cross-file check" below.

Per timer the report shows:
- A header label: `game-time`, `REAL-time`, or `MIXED` (data sets/checks it as
  both — a genuine scripting inconsistency worth calling out).
- **SET** lines: the `(Real)SetGlobalTimer` call, the source file, and the
  duration converted to days/hours (game) or minutes (real). Named GTIMES
  constants and raw integers both resolve — the script loads the authoritative
  constant map from the game's `GTIMES.IDS` at runtime.
- **CHECK** lines: each `(Real)GlobalTimer(Not)Expired`, with the sibling
  trigger conditions that gate it and the actions that fire when it expires
  (for DLG checks, the full state-trigger text; for BCS, `gated by: … -> …`).

## Step 2 — Read the answer off the report

1. Find the timer whose name matches the event (e.g. `OHH_cabrinatimer`,
   `JanDidPlot`, `OHH_hexxatcoronettimer`).
2. The **SET** duration is the wait. State it in human terms.
3. State the **time base**, because it changes player behavior:
   - game-time → "you can rest/fast-travel the wait away."
   - real-time → "you must actually play ~N minutes; resting won't help."
4. Add the gating conditions from the **CHECK** so the answer is complete —
   e.g. "and only fires while you're in a non-dungeon area, out of combat"
   (`AreaCheck`/`CombatCounter`), or "next time you enter <area>"
   (`Global("…","AR####",…)`). The action after `->` tells you what the event
   *is* (`CreateCreatureObjectOffset` = spawns an NPC, `Dialog`/`StartDialog…`
   = initiates a conversation).

## Reporting back to the user

Lead with the number and the time base in one sentence ("~7 game days — and
since it's a game-time timer, resting counts"). Then give the mechanism: where
it's set, where it fires, and the conditions. Narrative, not raw report dump.

Verified reference answers (use to sanity-check the skill still works):
- Jan returns from the Lissa plot: `JanDidPlot` = `FIVE_DAYS` = 5 game days (rest-skippable).
- Hexxat's quest activates at the Copper Coronet: `OHH_hexxatcoronettimer` = `FOUR_HOURS` value but **real-time** ≈ 20 min of play.
- Cabrina appears: `OHH_cabrinatimer` = 50400 = 7 game days, then spawns via `CreateCreatureObjectOffset` in a safe (non-dungeon) area.

## Chasing a cross-file check

A timer's SET usually lives in the companion's own files (found by `--prefix`),
which is enough to answer the duration. But the CHECK that fires the event can
live elsewhere — Jan's `JanDidPlot` is checked in `LISSA.DLG`, not a `JAN*`
file. If the report shows a SET but no CHECK (or you want the firing
conditions), dump the suspected file and re-run with `--resource THATFILE.DLG`,
or grep the override for the timer name (`.bcs` is binary, so dump it and grep
the JSON, not the file).

## When NOT to use

- "What does this whole script do?" — that's a full `iecli dump` read, not a timer trace.
- Non-timer gating (pure `Global("X","GLOBAL",N)` state gates with no timer) — there's no duration to report; trace the state flips instead.
- Single known constant ("what is FIVE_DAYS?") — it's in `GTIMES.IDS`; `iecli dump-raw --resource GTIMES.IDS` is faster than this skill.
