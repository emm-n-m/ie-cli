# Project Skills

This repo ships **Claude Code agent skills** in [`.claude/skills/`](../.claude/skills/). When you run Claude Code from the repo root, these skills auto-register and Claude invokes them on matching natural-language requests. Each skill's `SKILL.md` is the runtime instruction Claude reads; this document is the human-facing inventory — what skills exist, when they fire, and what they ship with.

Skills are part of `iecli`'s AI-native design (see [README](../README.md), [ROADMAP](../ROADMAP.md)). The parser produces JSON; skills wrap that JSON in narrative output that an IE modder can use without knowing how to script. New skills should follow the same pattern: parser produces JSON, skill produces narrative.

## At a glance

| Skill | Kind | Answers |
|---|---|---|
| [`diagnose-dialog`](#diagnose-dialog)     | diagnostic | Why is this dialog branch not firing? |
| [`explore-dungeon`](#explore-dungeon)     | diagnostic | Where does this area connect, and what's unreachable? |
| [`mod-diff`](#mod-diff)                   | triage     | What did this mod add, change, or break? |
| [`trace-quest-timer`](#trace-quest-timer) | diagnostic | How long until this quest/companion event fires? |
| [`map-stat-gates`](#map-stat-gates)       | analysis   | Which stat thresholds gate content, and is it worth anything? |
| [`plan-stat-build`](#plan-stat-build)     | generative | What stats should I take, and where do I grab every boost? |

All skills shell out to `iecli`, so build first (`cargo build --release` — the sweeps run hundreds of dumps, and debug builds are much slower). Scripts auto-prefer the release binary and accept `--iecli <path>` to override. Install paths come from [`LOCAL_GAME_PATHS.md`](LOCAL_GAME_PATHS.md).

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

## map-stat-gates

**Folder:** [`.claude/skills/map-stat-gates/`](../.claude/skills/map-stat-gates/)

**Purpose:** Scan every DLG in an install for `CheckStat*` gates on the protagonist and report both *what the gated replies do* (payoffs) and *how much* content sits behind each threshold (volume). Works for BG / IWD / PST.

**Triggers on questions like:**
- "Which stats actually matter for dialogue in this game?"
- "What does INT 16 unlock?"
- "Which stat gates the most valuable content?"

**Files:**

| File | Purpose |
|------|---------|
| `SKILL.md`           | Runtime instructions, including how to read payoffs before volume |
| `gate_payoffs.py`    | Joins each stat-gated reply to its action/journal text and tags a coarse payoff category |
| `gate_histogram.py`  | Per-stat histogram of how many gated branches require each value |

```bash
python .claude/skills/map-stat-gates/gate_payoffs.py --game "<game-path>" --protagonist Protagonist
python .claude/skills/map-stat-gates/gate_histogram.py --game "<game-path>" --protagonist Protagonist
```

| Flag             | Purpose                                                                     |
|------------------|-----------------------------------------------------------------------------|
| `--game`         | Game install directory (required)                                            |
| `--protagonist`  | Object to match — `Protagonist` (PST) or `Player1` (BG/IWD); `any` = no filter |
| `--high-only`    | `gate_payoffs.py` only: drop flavor-tier payoffs                              |
| `--iecli`        | Path to iecli (default: release, then debug)                                  |
| `--json`         | Emit JSON instead of the text report                                          |

**Design note:** the scripts classify payoffs only coarsely (`STAT_CORE`, `SKILL`, `COMPANION`, `ITEM`, `XP_BIG`/`XP_small`, `QUEST`, `STORY`, `TRAVEL`, `STATE`, `FLAVOR`) and surface the raw trigger and action text alongside. The category is a sort key; the agent judges value. Counts alone mislead — a stat can gate hundreds of flavor lines while one WIS gate hands over 120k XP.

---

## mod-diff

**Folder:** [`.claude/skills/mod-diff/`](../.claude/skills/mod-diff/)

**Purpose:** Bucket `iecli override-diff` output by resource type so an investigation can be scoped to what a mod actually touched. The Step 0 before exploring new content or planning a run on a modded install.

**Triggers on questions like:**
- "What did this mod change?"
- "What resources did it add?"
- "Diff my modded install against vanilla."

**Files:**

| File | Purpose |
|------|---------|
| `SKILL.md`     | Runtime instructions, mode selection, and interpretation rules |
| `mod_diff.py`  | Runs `override-diff` and buckets results by resource type      |

**Two modes:**

- **Mode A — shadow report** (no reference). Override resources that also exist in a BIFF are shadows; `override_only` resources are mod additions. Output buckets: **MOD-ADDED**, **MOD-CHANGED**, **benign shadows** (byte-identical re-ship).
- **Mode B — reference diff** (`--against <dir-or-file>`). Hash-compare the live override against a clean reference to get added / removed / changed.

```bash
python .claude/skills/mod-diff/mod_diff.py --game "<game-path>"
python .claude/skills/mod-diff/mod_diff.py --game "<modded>" --against "<clean-ref>"
```

| Flag        | Purpose                                             |
|-------------|-----------------------------------------------------|
| `--game`    | Modded game install directory (required)            |
| `--against` | Clean reference directory or file (Mode B)          |
| `--type`    | Filter to one resource type, e.g. `DLG`             |
| `--iecli`   | Path to iecli                                       |
| `--json`    | Pass through the raw `override-diff` JSON           |

**MOD-CHANGED is the interesting bucket** — clobbered vanilla files are where stuck quests and broken links come from. Note the limits: it reports *that* a resource changed, not which WeiDU component did it, and compiled resources can't be diffed against a mod's `.d`/`.baf` sources.

---

## plan-stat-build

**Folder:** [`.claude/skills/plan-stat-build/`](../.claude/skills/plan-stat-build/)

**Purpose:** The generative counterpart to the diagnostic skills — produce a full-game stat plan from the actual install: creation spread, level-up priority, and a sequenced itinerary of where to grab each permanent boost. Tuned for PST; the gate half generalizes to BG/IWD.

**Triggers on questions like:**
- "What stats should I pump?"
- "Plan a completionist run for me."
- "How do I min-max this playthrough?"

**Files:**

| File | Purpose |
|------|---------|
| `SKILL.md`         | Runtime instructions, the pipeline, and the synthesis gotchas |
| `stat_economy.py`  | Catalogues permanent `PermanentStatChange` dialogue grants plus ITM/SPL stat effects, split permanent vs while-equipped |

```bash
python .claude/skills/plan-stat-build/stat_economy.py --game "<game-path>" --protagonist Protagonist
```

| Flag            | Purpose                                              |
|-----------------|------------------------------------------------------|
| `--game`        | Game install directory (required)                    |
| `--protagonist` | Object for `PermanentStatChange`; `any` = no filter  |
| `--no-spl`      | Skip the SPL scan (faster)                           |
| `--iecli`       | Path to iecli                                        |
| `--json`        | Emit JSON instead of the text report                 |

**Pipeline:** `mod-diff` (trust layer on modded installs) → `map-stat-gates` (what the gates are worth) → `stat_economy.py` (what you can gain) → agent synthesis. The scripts are mechanical; resolving mutually-exclusive grants, caps, and AD&D bonus thresholds is the agent's job. Counts from `stat_economy.py` are **raw and overcount** — several grants are alternative branches of one conversation.

**Deliverable:** a written plan under [`guides/`](guides/); [`PST_STAT_PLAN.md`](guides/PST_STAT_PLAN.md) is the worked reference.

---

## trace-quest-timer

**Folder:** [`.claude/skills/trace-quest-timer/`](../.claude/skills/trace-quest-timer/)

**Purpose:** Answer "how long until X happens" by finding the `(Real)SetGlobalTimer` that gates an event, converting its duration to real time, and classifying it **game-time** (rest and fast-travel skip it) vs **real-time** (must actually play). The same constant means very different waits in each: `FOUR_HOURS` = 1200 is 4 game hours under `SetGlobalTimer` but ~20 real minutes under `RealSetGlobalTimer`.

**Triggers on questions like:**
- "When does Hexxat's quest activate?"
- "How long before Jan comes back?"
- "What's the cooldown on this script?"

**Files:**

| File | Purpose |
|------|---------|
| `SKILL.md`        | Runtime instructions, including cross-file timer chasing |
| `trace_timer.py`  | Dumps matching DLG/BCS and pairs every timer SET with its CHECKs |

```bash
python .claude/skills/trace-quest-timer/trace_timer.py --game "<BG2EE-path>" --prefix HEXXAT
```

| Flag         | Purpose                                                       |
|--------------|---------------------------------------------------------------|
| `--game`     | Game install directory (required)                             |
| `--prefix`   | NPC resref prefix to scan, e.g. `HEXXAT` (repeatable)         |
| `--resource` | Explicit extra resource to scan, e.g. `LISSA.DLG` (repeatable) |
| `--timer`    | Only report timers whose name contains this substring          |
| `--iecli`    | Path to iecli                                                  |
| `--json`     | Emit JSON instead of the narrative report                      |

Constant names resolve from the game's own `GTIMES.IDS` at runtime, with a small built-in fallback map. Reports label each timer `game-time`, `REAL-time`, or `MIXED` (set and checked as both — a real scripting inconsistency worth surfacing). Depends on the BCS action-argument fix in commit `3a262d9b`; before it, timer durations were read from the wrong argument.

---

## Guides produced by these skills

[`docs/guides/`](guides/) holds the narrative output of running the skills against real installs — the first user-facing output of this project that isn't JSON. Each guide states the install it was derived from and how to reproduce it. See the [guides index](guides/README.md).

---

## Adding a new skill

1. Decide single-file vs folder layout. Single-file (`.claude/skills/<name>.md`) is fine for skills that are pure prompt instructions. Folder layout (`.claude/skills/<name>/SKILL.md` + scripts) is needed when the skill ships executable artifacts.
2. SKILL.md frontmatter must include `name` and `description`. The `description` is what triggers Claude to invoke the skill — write it in terms of the natural-language questions a user might ask.
3. Keep skill output **narrative, not JSON** — the audience is IE modders, many of whom don't script. The JSON is the parser's job; the skill's job is to make the answer readable.
4. Add an entry to this document so the skill is discoverable when browsing the repo.
