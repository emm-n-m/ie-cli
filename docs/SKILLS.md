# Project Skills

This repository ships parallel game-exploration skills for Claude Code and Codex:

- Claude Code packages: [`.claude/skills/`](../.claude/skills/)
- Codex/Open Agent Skills packages: [`skills/`](../skills/)

Both sets turn `iecli`'s deterministic JSON into narrative answers for IE players and
modders. The parser owns extraction; the skills own investigation workflows and readable
reporting.

Codex officially discovers repository skills from `.agents/skills`. This workspace mounts
`.agents` read-only, so the versioned packages live in `skills/` and AGENTS.md routes matching
requests there. In a normal checkout, enable native `$skill-name` discovery with:

```bash
mkdir -p .agents
ln -s ../skills .agents/skills
```

Codex follows symlinked skill directories. Restart Codex only if a new skill does not appear.

## At a glance

| Skill | Kind | Answers |
|---|---|---|
| [`diagnose-dialog`](#diagnose-dialog)     | diagnostic | Why is this dialog branch not firing? |
| [`explore-dungeon`](#explore-dungeon)     | diagnostic | Where does this area connect, and what's unreachable? |
| [`mod-diff`](#mod-diff)                   | triage     | What did this mod add, change, or break? |
| [`trace-quest-timer`](#trace-quest-timer) | diagnostic | How long until this quest/companion event fires? |
| [`map-stat-gates`](#map-stat-gates)       | analysis   | Which stat thresholds gate content, and is it worth anything? |
| [`plan-stat-build`](#plan-stat-build)     | generative | What stats should I take, and where do I grab every boost? |

Every skill shells out to `iecli`, so build first — `cargo build --release`. The sweeps run
hundreds of dumps and debug builds are much slower; the bundled scripts auto-prefer the release
binary and accept `--iecli <path>` to override. Install paths come from
[`LOCAL_GAME_PATHS.md`](LOCAL_GAME_PATHS.md).

**Script paths differ by tree.** Command examples below use the Claude layout
(`.claude/skills/<skill>/<script>.py`); the Codex equivalent is
`skills/<skill>/scripts/<script>.py`. Flags are identical.

## Skill Inventory

### diagnose-dialog

Find the DLG state, trigger, or global variable preventing an expected NPC dialog branch.
Trace the variable upstream when possible and finish with usable `CLUAConsole:GetGlobal`
verification commands.

Example prompts:

- “Why does Quenash give me one line and leave?”
- “What variable controls this dialog option?”
- “Why is the expected branch missing?”

Workflow (summary; the SKILL.md is authoritative):

1. `iecli list --type dlg --name <partial>` to find the DLG — resrefs cap at 8 chars, so
   `quenash` → `QUENAS`.
2. `iecli dump --resource <RESREF>.DLG` to extract state triggers and transitions.
3. Classify each state by trigger shape (`Global(...)`, `True()` fallback, …); the engine takes
   the first state whose trigger passes.
4. Trace the gating variable upstream by grepping `SetGlobal` actions in candidate DLGs.

No bundled scripts — this one is pure prompt instructions.

Codex: [`skills/diagnose-dialog/`](../skills/diagnose-dialog/)

Claude: [`.claude/skills/diagnose-dialog.md`](../.claude/skills/diagnose-dialog.md)

### explore-dungeon

Walk Travel regions from a starting ARE, describe rooms and actors, flag faction transitions,
and find dead links, phantom entrances, one-way exits, missing scripts, and likely orphaned
mod areas.

Example prompts:

- “Walk me through this dungeon starting from AR4300.”
- “I think I missed a level—can you find it?”
- “Is ARR018 reachable from anywhere?”

Before graph walking, prefer `iecli verify --source override --format json` to find install-wide
ARE breakage: dead Travel links, phantom entrances, missing area scripts, missing actor
CRE/dialog/script links, and missing key items.

Bundled helpers: `walk_graph.py`, `describe_rooms.py`.

```bash
python .claude/skills/explore-dungeon/walk_graph.py --game "<game-path>" --start AR4300
python .claude/skills/explore-dungeon/describe_rooms.py --game "<game-path>" --start AR4300
```

| Flag          | Purpose                                                      |
|---------------|--------------------------------------------------------------|
| `--game`      | Game install directory (required)                            |
| `--start`     | Starting ARE resref, e.g. `AR4300` (required)                |
| `--iecli`     | Path to iecli (default: `target/debug/iecli.exe`)            |
| `--max-depth` | Limit BFS depth                                              |
| `--json`      | `walk_graph.py` only: emit JSON instead of text              |

`walk_graph.py` reports **reached areas** with depth, **edges** (`A -> B` with region and
entrance), **one-way exits**, **dead links**, **parse failures**, and **orphans** — Override AREs
sharing a 3-char prefix with the reached set, ranked by prefix popularity so a mod-namespaced
dungeon's own missing levels surface above unrelated vanilla overrides.

`describe_rooms.py` walks in DFS order (preserving Travel-region declaration order, so side rooms
appear inline) and prints per room: WED tileset, area script and whether it is installed, section
counts, Travel exits with bounding boxes and dead-link markers, trap/trigger regions with their
scripts, the actor roster grouped by display name with a faction breakdown, named/unique actors,
and distinct actor scripts. `*** TRANSITION: ... ***` lines mark faction changes between rooms
(dominant faction changes, a faction crossing 30% appearing, or a previously-dominant one
vanishing). Faction classification is regex-based on display names — extend the `FACTIONS` list
when a mod introduces a new creature family. A final pass flags WED-tileset reuse across rooms.

**Patching broken exits:** once a dead link or phantom entrance is identified, fix it with
`iecli patch` rather than dropping to NearInfinity. Two ARE fields are supported —
`regions.<selector>.destination_entrance` (32-byte string) and
`regions.<selector>.destination_area` (8-byte resref) — where the selector is the region's exact
name (case-sensitive, NUL-truncated) or its 0-based index. Back up first, and note that the engine
caches loaded areas, so verifying in-game needs a fresh entry into the patched area.

Codex: [`skills/explore-dungeon/`](../skills/explore-dungeon/)

Claude: [`.claude/skills/explore-dungeon/`](../.claude/skills/explore-dungeon/)

### map-stat-gates

Scan all DLG resources for `CheckStat*` gates. Lead with the actual rewards behind those
checks—permanent boosts, items, quests, and XP—then use the threshold histogram as a
secondary measure of content volume.

Example prompts:

- “Which stats unlock valuable dialogue in this install?”
- “What stat checks does this mod add?”
- “Is CON actually worth raising for conversations?”

Bundled helpers: `gate_payoffs.py`, `gate_histogram.py`.

```bash
python .claude/skills/map-stat-gates/gate_payoffs.py   --game "<game-path>" --protagonist Protagonist
python .claude/skills/map-stat-gates/gate_histogram.py --game "<game-path>" --protagonist Protagonist
```

| Flag            | Purpose                                                                        |
|-----------------|--------------------------------------------------------------------------------|
| `--game`        | Game install directory (required)                                              |
| `--protagonist` | Object to match — `Protagonist` (PST) or `Player1` (BG/IWD); `any` = no filter |
| `--high-only`   | `gate_payoffs.py` only: drop flavor-tier payoffs                               |
| `--iecli`       | Path to iecli (default: release, then debug)                                   |
| `--json`        | Emit JSON instead of the text report                                           |

**Design note:** `gate_payoffs.py` classifies payoffs only coarsely (`STAT_CORE`, `SKILL`,
`COMPANION`, `ITEM`, `XP_BIG`/`XP_small`, `QUEST`, `STORY`, `TRAVEL`, `STATE`, `FLAVOR`) and
surfaces the raw trigger and action text alongside. The category is a sort key; the agent judges
value. Counts alone mislead — a stat can gate hundreds of flavor lines while one WIS gate hands
over 120k XP.

Codex: [`skills/map-stat-gates/`](../skills/map-stat-gates/)

Claude: [`.claude/skills/map-stat-gates/`](../.claude/skills/map-stat-gates/)

### mod-diff

Bucket override resources into mod-added, genuinely changed, and byte-identical shadows.
Optionally compare a live install with a clean binary reference directory or file.

Example prompts:

- “What did this mod add or change?”
- “Which vanilla resources did the mod overwrite?”
- “Scope this investigation to the mod's delta.”

Bundled helper: `mod_diff.py`.

Two modes:

- **Mode A — shadow report** (no reference). Override resources that also exist in a BIFF are
  shadows; `override_only` resources are mod additions. Buckets: **MOD-ADDED**, **MOD-CHANGED**,
  **benign shadows** (byte-identical re-ship).
- **Mode B — reference diff** (`--against <dir-or-file>`). Hash-compare the live override against
  a clean reference to get added / removed / changed.

```bash
python .claude/skills/mod-diff/mod_diff.py --game "<game-path>"
python .claude/skills/mod-diff/mod_diff.py --game "<modded>" --against "<clean-ref>"
```

| Flag        | Purpose                                          |
|-------------|--------------------------------------------------|
| `--game`    | Modded game install directory (required)         |
| `--against` | Clean reference directory or file (Mode B)       |
| `--type`    | Filter to one resource type, e.g. `DLG`          |
| `--iecli`   | Path to iecli                                    |
| `--json`    | Pass through the raw `override-diff` JSON        |

**MOD-CHANGED is the interesting bucket** — clobbered vanilla files are where stuck quests and
broken links come from. Note the limits: it reports *that* a resource changed, not which WeiDU
component did it, and compiled resources cannot be diffed against a mod's `.d`/`.baf` sources.

Codex: [`skills/mod-diff/`](../skills/mod-diff/)

Claude: [`.claude/skills/mod-diff/`](../.claude/skills/mod-diff/)

### plan-stat-build

Combine high-value dialogue gates, permanent stat grants, equipment effects, timing,
exclusivity, class mechanics, and the player's goal into a creation spread, level-up plan,
and sequenced boost itinerary. Tuned for PST; the gate analysis generalizes to BG and IWD.

Example prompts:

- “How should I assign my PST stats for a completionist run?”
- “What stats should I pump in this modded install?”
- “Make me a full permanent-boost itinerary.”

Bundled helper: `stat_economy.py`; it also uses the `map-stat-gates` helpers.

```bash
python .claude/skills/plan-stat-build/stat_economy.py --game "<game-path>" --protagonist Protagonist
```

| Flag            | Purpose                                             |
|-----------------|-----------------------------------------------------|
| `--game`        | Game install directory (required)                   |
| `--protagonist` | Object for `PermanentStatChange`; `any` = no filter |
| `--no-spl`      | Skip the SPL scan (faster)                          |
| `--iecli`       | Path to iecli                                       |
| `--json`        | Emit JSON instead of the text report                |

**Pipeline:** `mod-diff` (trust layer on modded installs) → `map-stat-gates` (what the gates are
worth) → `stat_economy.py` (what you can gain) → agent synthesis. The scripts are mechanical;
resolving mutually-exclusive grants, stat caps, and AD&D bonus thresholds is the agent's job.
Counts from `stat_economy.py` are **raw and overcount**, since several grants are alternative
branches of one conversation.

**Deliverable:** a written plan under [`guides/`](guides/); [`PST_STAT_PLAN.md`](guides/PST_STAT_PLAN.md)
is the worked reference.

Codex: [`skills/plan-stat-build/`](../skills/plan-stat-build/)

Claude: [`.claude/skills/plan-stat-build/`](../.claude/skills/plan-stat-build/)

### trace-quest-timer

Find `(Real)SetGlobalTimer` and `(Real)GlobalTimer(Not)Expired` calls across DLG and BCS
resources. Convert the duration and distinguish game-time waits, which resting and travel
advance, from real-time waits that require actual play.

Example prompts:

- “How long until Jan comes back?”
- “When does this companion quest activate?”
- “Can I rest through this timer?”

The distinction is the whole point: `FOUR_HOURS` = 1200 means 4 game hours under
`SetGlobalTimer` but roughly 20 minutes of real play under `RealSetGlobalTimer`.

Bundled helper: `trace_timer.py`.

```bash
python .claude/skills/trace-quest-timer/trace_timer.py --game "<BG2EE-path>" --prefix HEXXAT
```

| Flag         | Purpose                                                        |
|--------------|----------------------------------------------------------------|
| `--game`     | Game install directory (required)                              |
| `--prefix`   | NPC resref prefix to scan, e.g. `HEXXAT` (repeatable)          |
| `--resource` | Explicit extra resource to scan, e.g. `LISSA.DLG` (repeatable) |
| `--timer`    | Only report timers whose name contains this substring          |
| `--iecli`    | Path to iecli                                                  |
| `--json`     | Emit JSON instead of the narrative report                      |

Constant names resolve from the game's own `GTIMES.IDS` at runtime, with a small built-in fallback
map. Each timer is labelled `game-time`, `REAL-time`, or `MIXED` — set and checked as both, a real
scripting inconsistency worth surfacing. Depends on the BCS action-argument fix in commit
`3a262d9b`; before it, durations were read from the wrong argument.

Codex: [`skills/trace-quest-timer/`](../skills/trace-quest-timer/)

Claude: [`.claude/skills/trace-quest-timer/`](../.claude/skills/trace-quest-timer/)

## Guides produced by these skills

[`docs/guides/`](guides/) holds the narrative output of running these skills against real installs
— the first user-facing output of this project that is not JSON. Each guide names the install it
was derived from and how to reproduce it. See the [guides index](guides/README.md).

## Adding or Updating a Skill

1. Keep each skill focused on one exploration job.
2. Put trigger wording in the `name` and `description` frontmatter.
3. Prefer narrative output; leave JSON extraction to `iecli`.
4. Store deterministic repeated analysis in bundled scripts.
5. Keep the Claude and Codex variants behaviorally aligned while adapting product-specific
   paths and context assumptions.
6. For Codex packages, regenerate `agents/openai.yaml` when metadata changes and run the
   skill creator's `quick_validate.py` before committing.
7. Add the skill to the at-a-glance table and inventory above, so it is discoverable when
   browsing the repo rather than only at runtime.
