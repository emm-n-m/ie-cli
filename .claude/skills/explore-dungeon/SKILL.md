---
name: explore-dungeon
description: Walk an Infinity Engine dungeon's area graph from a starting ARE by following Travel regions, describe each room (actors, traps, exits), highlight enemy-faction transitions, and flag orphaned area files installed but unreachable. Use when the user asks to explore a dungeon's levels, link area exits, find a missing level, trace where an area connects to, or check whether a mod-installed ARE is reachable.
---

# Explore IE Dungeon

Given a starting ARE, walk the area graph by following Travel regions, describe each room, and surface:

- Reachability tree (which areas connect to which, and how)
- One-way exits and dead links (destination ARE doesn't exist in install)
- **Orphans** — mod-installed AREs sharing the dungeon's resref prefix that no Travel region reaches (the usual "missed level" suspects)
- **Faction transitions** — rooms where the dominant enemy roster shifts (duergar → drow, etc.)
- WED tileset reuse (rooms cloning the same template)
- Missing area scripts (referenced but not installed)

## Inputs

- Starting ARE resref (e.g., `AR4300`). The user may give a level name — ask for the resref if unclear. Resrefs are max 8 chars.
- Game install path. Use the user's BG:EE Steam install from auto-memory unless they specify another.

iecli must be built. From the repo root: `cargo build`. The scripts default to `target/debug/iecli.exe`.

## Step 1 — Walk the graph

```bash
python .claude/skills/explore-dungeon/walk_graph.py \
    --game "<game-path>" \
    --start <RESREF>
```

Output sections:
- **Reached areas** — sorted resref list
- **Edges** — directed `A -> B` pairs
- **One-way exits** — `A -> B` with no reciprocal `B -> A`
- **Dead links** — Travel regions whose `destination_area.exists` is false
- **Orphans** — Override AREs sharing a 3-char prefix with the reached set, but never reached. These are the prime suspects for a "missed level."

If the user is hunting for a missing level, the orphan list is the first thing to look at.

## Step 2 — Describe the rooms

```bash
python .claude/skills/explore-dungeon/describe_rooms.py \
    --game "<game-path>" \
    --start <RESREF>
```

Walks in DFS order with side rooms inline. Per room: WED, area-script status, area-type flags, exit list, trap regions, actor tally with named-boss callouts, container/spawn counts.

Above each room, the script emits a **TRANSITION** line whenever the dominant enemy faction changes from the previous room (e.g., `duergar -> drow`), or when a major faction (≥30% of the room) appears or disappears. This is the answer to "I missed the duergar→drow transition" — it gets flagged automatically.

## Reporting back to the user

After running, summarize:

- The graph shape (linear chain? branching? hub-and-spoke?)
- Each faction-transition the script flagged, with the room number
- Any orphans, with a one-line note on each (run `iecli dump` on them to confirm they look like a real level — actors, exits — vs an empty stub)
- Any one-way Travel regions, dead links, missing area scripts
- WED reuse patterns (e.g., "AR1401 reused 4× as spider-room template")

If the user asks for an orphan-check fix, the patch is usually obvious from the orphan's own exits: the orphan's outgoing Travel regions tell you which two adjacent rooms in the chain were *meant* to wire through it.

## When NOT to use

- Single-area inspection — `iecli dump --resource <ARE>` directly is faster.
- Non-area resources (CRE, DLG, BCS) — those have their own dump flows.
- Vanilla areas with hundreds of AREs in BIF and no specific dungeon scope — the orphan filter will be unhelpful unless the dungeon uses a distinct resref prefix.
