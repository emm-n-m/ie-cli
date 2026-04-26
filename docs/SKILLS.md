# Project Skills

This repo ships **Claude Code agent skills** in [`.claude/skills/`](../.claude/skills/). When you run Claude Code from the repo root, these skills auto-register and Claude invokes them on matching natural-language requests. Each skill's `SKILL.md` is the runtime instruction Claude reads; this document is the human-facing inventory — what skills exist, when they fire, and what they ship with.

Skills are part of `iecli`'s AI-native design (see [README](../README.md), [ROADMAP](../ROADMAP.md)). The parser produces JSON; skills wrap that JSON in narrative output that an IE modder can use without knowing how to script. New skills should follow the same pattern: parser produces JSON, skill produces narrative.

---

## diagnose-dialog

**File:** [`.claude/skills/diagnose-dialog.md`](../.claude/skills/diagnose-dialog.md) — single-file skill, no scripts.

**Purpose:** Given an NPC name or DLG resref, identify which trigger or variable is gating an expected dialog branch — the canonical "why does this NPC give a one-liner and exit" investigation.

**Triggers on questions like:**
- "Why does Quenash give me one line and leave?"
- "What variable controls this dialog option?"
- "Why is the expected branch missing?"

**Output:** A structured report identifying the intended state, its gating trigger, the actual fallback state, and (when traceable) which upstream NPC/dialog path sets the gating variable. Closes with `CLUAConsole:GetGlobal(...)` verification commands the user can paste into their game.

**Workflow** (summary; full version in the SKILL.md):
1. `iecli list --type dlg --name <partial>` to find the DLG (resrefs cap at 8 chars, so `quenash` → `QUENAS`).
2. `iecli dump --resource <RESREF>.DLG` to extract state triggers and transitions.
3. Classify each state by trigger shape (`Global(...)`, `True()` fallback, etc.) — the engine takes the first state whose trigger passes.
4. Trace the gating variable upstream by grepping `SetGlobal` actions in candidate DLGs.

---

## explore-dungeon

**Folder:** [`.claude/skills/explore-dungeon/`](../.claude/skills/explore-dungeon/)

**Purpose:** Walk an IE dungeon's area graph from a starting ARE by following Travel regions, describe each room (actors, traps, exits), highlight enemy-faction transitions, and flag orphaned area files installed but unreachable.

Before graph walking, prefer `iecli verify --source override --format json` to find install-wide ARE cross-resource breakage such as dead Travel links, phantom entrances, missing area scripts, missing actor CRE/dialog/script links, and missing key items.

**Triggers on questions like:**
- "Walk me through this dungeon starting from AR4300"
- "I think I missed a level — can you find it?"
- "Where does ARR017 connect to?"
- "Is ARR018 reachable from anywhere?"

**Files:**

| File | Purpose |
|------|---------|
| `SKILL.md`           | Runtime instructions Claude follows when invoking this skill |
| `walk_graph.py`      | Walks the area graph; reports reachability, edges, one-way exits, dead links, and ranked orphans |
| `describe_rooms.py`  | DFS traversal narrative; per-room WED/script/actor/trap details with auto-flagged faction transitions |

### `walk_graph.py`

Walks Travel regions BFS from `--start`, then enumerates Override-source AREs to find orphans (installed files no Travel region reaches).

```bash
python .claude/skills/explore-dungeon/walk_graph.py \
    --game "C:\path\to\game" \
    --start AR4300
```

| Flag           | Purpose                                                          |
|----------------|------------------------------------------------------------------|
| `--game`       | Game install directory (required)                                |
| `--start`      | Starting ARE resref, e.g. `AR4300` (required)                    |
| `--iecli`      | Path to iecli executable (default: `target/debug/iecli.exe`)     |
| `--max-depth`  | Limit BFS depth                                                  |
| `--json`       | Emit JSON instead of human-readable text                         |

**Output sections:**
- **Reached areas** — sorted resref list with depth from start
- **Edges** — directed `A -> B` pairs with region name and entrance
- **One-way exits** — `A -> B` with no reciprocal `B -> A`
- **Dead links** — Travel regions whose `destination_area.exists` is `false`
- **Parse failures** — AREs that couldn't be dumped
- **Orphans** — Override AREs sharing a 3-char resref prefix with the reached set, ranked by **prefix popularity** (prefixes claimed by more reached areas come first). For a mod-namespaced dungeon like `ARR*`, the dungeon's own missing levels surface above unrelated vanilla overrides.

### `describe_rooms.py`

Walks the graph in DFS order (preserving Travel-region declaration order so side rooms appear inline), then prints a per-room narrative.

```bash
python .claude/skills/explore-dungeon/describe_rooms.py \
    --game "C:\path\to\game" \
    --start AR4300
```

Same flags as `walk_graph.py` except no `--json`.

**Per-room output:**
- WED tileset, area script (and whether it's actually installed), area-type flags
- Counts: actors, regions, containers, doors, spawn points, entrances, ambients, animations
- Travel exits with destination, entrance, bounding box, flags, region script, dead-link marker
- Trap/trigger regions with their region scripts (e.g., `GTWEB.BCS`, `GTFB.BCS`)
- Actor roster grouped by display name, plus a faction breakdown
- Named/unique actors (display names that don't match any common faction — usually mod-introduced bosses)
- Distinct actor scripts and any actors with dialogs

**Faction transitions** are flagged as `*** TRANSITION: ... ***` lines between rooms, triggered when:
- The dominant faction changes (most-numerous, ignoring `other`)
- A faction crossing 30% of room actors appears that wasn't above-threshold in the previous room
- A faction previously above 30% has disappeared

Faction classification is regex-based on actor display names. Current factions: `undead`, `drow`, `duergar`, `gnoll`, `spider`, `bugbear`, `orc`, `hobgoblin`, `ogre`, `basilisk`, `bandit`, `amnian`, `bat`, `slave`, `dog`, `miner`. Anything else lands in `other` (usually named bosses). Tweak the `FACTIONS` list in the script if a new mod introduces a creature family that should be a peer faction.

**Final pass:** WED-tileset reuse — flags rooms cloning the same tileset (e.g., `AR1401: ['ARR003', 'ARR015', 'ARR016', 'ARR018']` — the spider-room template).

### Patching broken exits

Once a broken Travel region is identified (dead link or phantom entrance), the skill instructs Claude to fix it via `iecli patch` rather than dropping back to NearInfinity or hex-edits. ARE region patching supports two fields:

- `regions.<selector>.destination_entrance` — 32-byte string, the entrance name in the destination ARE
- `regions.<selector>.destination_area` — 8-byte resref, the destination ARE itself

Selector is either the region's exact name (case-sensitive, NUL-truncated) or its 0-based index. Always back up first; warn the user that BG:EE caches loaded areas, so a fresh entry into the patched area is required to verify in-game. Full reasoning lives in the SKILL.md.

### Prerequisites

`iecli` must be built before either script runs:

```bash
cargo build
```

The scripts default to `target/debug/iecli.exe`. For release builds, pass `--iecli target/release/iecli.exe`.

---

## Adding a new skill

1. Decide single-file vs folder layout. Single-file (`.claude/skills/<name>.md`) is fine for skills that are pure prompt instructions. Folder layout (`.claude/skills/<name>/SKILL.md` + scripts) is needed when the skill ships executable artifacts.
2. SKILL.md frontmatter must include `name` and `description`. The `description` is what triggers Claude to invoke the skill — write it in terms of the natural-language questions a user might ask.
3. Keep skill output **narrative, not JSON** — the audience is IE modders, many of whom don't script. The JSON is the parser's job; the skill's job is to make the answer readable.
4. Add an entry to this document so the skill is discoverable when browsing the repo.
