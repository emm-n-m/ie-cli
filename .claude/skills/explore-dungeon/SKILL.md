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

## Patching broken Travel regions

Two common breakage modes that walk_graph and describe_rooms can surface:

1. **Dead link** — a Travel region's `destination_area.exists` is `false`. Either the destination ARE was never installed (mod failure) or the resref is misspelled.
2. **Phantom entrance** — a Travel region targets a `destination_entrance` that does not exist in the destination ARE's entrance table. The engine still fires the transition but lands the player at a fallback position (often (0,0) or off the walkable map). To check: dump the destination ARE and grep its `entrances[].name` for the value the source region wants.

For both, the fix is `iecli patch` against the **source** ARE (the one with the broken region) — never the destination, especially if the destination is a vanilla area:

```bash
# Repoint a Travel region's destination_entrance.
iecli patch --game "<game-path>" \
    --resource <SOURCE>.ARE \
    --set "regions.<region-name>.destination_entrance=<existing-entrance-in-dest>" \
    --output "<source-override-path>"

# Repoint a Travel region's destination_area resref (less common).
iecli patch --game "<game-path>" \
    --resource <SOURCE>.ARE \
    --set "regions.<region-name>.destination_area=<NEW_RESREF>" \
    --output "<source-override-path>"
```

The selector after `regions.` can be either the region's exact name (NUL-terminated, case-sensitive) or its 0-based index. Names are usually unique, but if the same name appears multiple times (rare but possible), iecli errors with `AmbiguousSelector` and you switch to the index form.

**Always back up before patching.** The pattern is `cp <override>/<RESOURCE>.ARE <override>/<RESOURCE>.ARE.bak` first, then patch with the override path itself as the `--output`. Document the backup location in your reply so the user can revert.

**After patching:** in BG:EE, a save where the patched area was already loaded does NOT pick up structural changes. Tell the user to either (a) load a save from before they entered the area, (b) exit-and-re-enter the area in the same session, or (c) full game restart. Reload-from-save where the player is currently in the patched area will not pick up the change.

When choosing a destination entrance among multiple existing options, prefer the one whose position matches the visual continuity of the source ARE's WED tileset. If `<SOURCE>` reuses `AR1903` (a vanilla bandit-camp hut) as its WED, the natural choice is `Exit1903` in the destination — the player feels like they "walked through the hut and out the front door."

## When NOT to use

- Single-area inspection — `iecli dump --resource <ARE>` directly is faster.
- Non-area resources (CRE, DLG, BCS) — those have their own dump flows.
- Vanilla areas with hundreds of AREs in BIF and no specific dungeon scope — the orphan filter will be unhelpful unless the dungeon uses a distinct resref prefix.
