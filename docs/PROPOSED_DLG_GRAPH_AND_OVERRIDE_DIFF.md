# Proposed: DLG branch-graph export & override-diff

Two proposed commands, motivated by manually tracing the Ascension Melissan finale
(branching dialogue + battle logic shattered across ~14 `.d` files and several `.baf`
scripts, where the deciding condition is a single trigger string buried in one DLG
transition, and where "is this even the live version?" depends on the WeiDU install
order). Both are thin layers over data iecli already extracts.

---

## 1. `dump --format dot` — DLG state/transition graph

### Why
`finmel01.dlg` reasoning today means reading a flat JSON state list and reconstructing
the fork structure by hand. The branch data is *already* in `DialogJson`
(`crates/ie-formats/src/dlg.rs`): every `DialogTransitionJson` carries
`next_dialog`, `next_state_index`, `trigger_text`, `action_text`, `player_text`,
`journal_text`, and `terminates_dialog`. Rendering it as a graph is presentation only —
no new parsing.

### CLI
```
iecli dump --game <path> --resource FINMEL01.DLG --format dot
iecli dump --game <path> --resource FINMEL01.DLG --format mermaid   # for inline markdown
```
- Add `Dot` and `Mermaid` to the dump format enum. These are valid **only for DLG**;
  for any other resource type, error: `--format dot is only supported for DLG`.
  (Keep `save-info`'s format enum separate so `dot` can't leak there.)
- Output goes to stdout. Render with `dot -Tsvg out.dot` or paste mermaid into markdown.

### Graph mapping
| DLG element | Graph element |
|---|---|
| State `index` | Node `S{index}`; box shape; label = `index` + truncated `response_text` + (if present) the **state `trigger_text`** as the gating condition |
| Transition → same DLG (`next_state_index=j`, `next_dialog` empty/self) | Edge `S{i} → S{j}` |
| Transition → other DLG (`next_dialog != this resource`) | Edge `S{i} → EXT_{dlg}_{state}`; external nodes styled distinctly (dashed, different fill) — these are the `EXTERN`/cross-file jumps (e.g. `finmel01 → irenic2`) |
| `terminates_dialog = true` | Edge to a single terminal `END` node |
| Transition `trigger_text` non-empty | Edge label includes the condition; conditional edges colored (e.g. red) so gated paths stand out at a glance |
| `player_text` | Prepended to edge label (truncated) — the reply the player picks |
| `journal_text` present | Edge marked (e.g. 📓 / `[journal]`) |
| `action_text` contains `StartCutScene`/`StartCutSceneMode` | Edge tagged `[cutscene: <name>]` — high-signal for finale tracing |

### Options
- `--max-label-len N` (default ~40) — truncate resolved strings.
- `--no-triggers` / `--no-actions` — drop those from labels for a cleaner topology view.
- `--strings {resolved,strref,both}` (default `resolved`) — reuse existing strref resolution.
- **Phase 2 — `--follow-extern[=DEPTH]`**: when a transition jumps to another DLG, load
  that DLG too and render a multi-file graph (bounded by depth, cycle-safe, dedup nodes/edges).
  This is the feature that would have rendered the whole
  `finmel01 → irenic2 → finbodh → finsol01` web in one pass. Cross-file nodes grouped in
  Graphviz `subgraph cluster_<dlg>` boxes.

### Notes
- States can loop; emit each node/edge once, no recursion on the single-file path.
- Pure function `dialog_json_to_dot(&DialogJson, &Opts) -> String` in `ie-formats`,
  unit-testable against a small hand-built `DialogJson` (no install needed).

---

## 2. `override-diff` — what's actually live / what got clobbered

### Why
Every analysis on a modded install carries the unstated risk that a later WeiDU component
overwrote the resource you're reading. iecli's resolver already distinguishes
`override` from BIFF (`SourceArg::{Auto,Override,Bif}`, `ResolverBundle`), so it already
knows when `override` shadows the stock biffed copy. Surfacing that is the trust layer.

### Mode A — shadow report (no reference needed)
```
iecli override-diff --game <path> [--type DLG] [--format json]
```
Enumerate `override`; for each resource also resolvable in a BIFF, report it as a
**shadow** (override wins → live differs from stock):
```jsonc
{
  "shadows": [
    { "resource": "FINMEL01.DLG", "in_override": true, "in_bif": true,
      "override_sha1": "…", "bif_sha1": "…", "identical": false }
  ],
  "override_only": [ "FINBODH.CRE", … ],   // added by a mod, no stock version
  "counts": { "override_total": N, "shadowing_bif": M, "override_only": K }
}
```
- `identical: true` (hashes match) flags harmless shadows (a mod re-shipped a byte-identical
  file) vs. real overrides.
- `--type` filters by resource type; text format prints a sorted table.

### Mode B — diff against a reference
```
iecli override-diff --game <path> --against <dir-or-file>
```
Byte/hash-compare the live `override` resource(s) against a reference set — e.g. a clean
install's biff-extracted copies, or a mod's shipped binaries. Reports `added` /
`removed` / `changed` (by SHA-1), each annotated with resolved resource type. For a single
`--resource X --against file`, just prints `match` / `differ` + both hashes.

### Scope / honesty
- This answers **"is what I'm reading the live copy, and did something override the stock
  resource?"** — which is the question that actually bites during analysis.
- It does **not** attribute a shadow to a specific WeiDU component. True per-component
  attribution needs parsing WeiDU's `.DEBUG`/backup metadata and is a separate, larger
  effort — note as a future stretch, don't promise it here.
- Compiled resources (`.baf`→`.bcs`, `.d`→`.dlg`) can't be byte-diffed against mod *source*;
  Mode B is meaningful only against already-compiled reference binaries.

### Reuse
- Enumeration: existing `list` / `ResourceListOptions` path.
- Resolution & source selection: existing `ResolverBundle` / `ResourceSource`.
- Only genuinely new piece: hashing + the set-difference reporter.
