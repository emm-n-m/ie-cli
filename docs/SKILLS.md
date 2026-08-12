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

## Skill Inventory

### diagnose-dialog

Find the DLG state, trigger, or global variable preventing an expected NPC dialog branch.
Trace the variable upstream when possible and finish with usable `CLUAConsole:GetGlobal`
verification commands.

Example prompts:

- “Why does Quenash give me one line and leave?”
- “What variable controls this dialog option?”
- “Why is the expected branch missing?”

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

Bundled helpers: `walk_graph.py`, `describe_rooms.py`.

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

Bundled helper: `trace_timer.py`.

Codex: [`skills/trace-quest-timer/`](../skills/trace-quest-timer/)

Claude: [`.claude/skills/trace-quest-timer/`](../.claude/skills/trace-quest-timer/)

## Adding or Updating a Skill

1. Keep each skill focused on one exploration job.
2. Put trigger wording in the `name` and `description` frontmatter.
3. Prefer narrative output; leave JSON extraction to `iecli`.
4. Store deterministic repeated analysis in bundled scripts.
5. Keep the Claude and Codex variants behaviorally aligned while adapting product-specific
   paths and context assumptions.
6. For Codex packages, regenerate `agents/openai.yaml` when metadata changes and run the
   skill creator's `quick_validate.py` before committing.
