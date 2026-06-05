# DLG Graph Export And Override Diff

Status: implemented.

This file started as a proposal. The features are now in `iecli`; the filename is
kept so existing links do not break. What follows is the current shipped behavior,
not a future design sketch.

---

## 1. `dump --format dot|mermaid` for DLG graphs

### Purpose

`iecli dump --resource FOO.DLG --format json` already exposes the state/transition
structure. The graph formats are presentation layers over the parsed `DialogJson`
model, intended to make branch shape, gating conditions, extern jumps, journals,
and cutscene transitions easier to trace.

### Commands

```bash
iecli dump --game <path> --resource FINMEL01.DLG --format dot
iecli dump --game <path> --resource FINMEL01.DLG --format mermaid
iecli dump --game <path> --resource FINMEL01.DLG --format dot --strings both
iecli dump --game <path> --resource FINMEL01.DLG --format mermaid --follow-extern=2
```

`dot` and `mermaid` are valid only for `DLG`. Any other resource type errors with:

```text
--format dot is only supported for DLG
```

The same rule applies to `mermaid`.

### Options

- `--max-label-len <N>`: truncate normalized labels after `N` characters. Default: `40`.
- `--no-triggers`: omit state and transition trigger text from labels.
- `--no-actions`: omit transition action text from labels.
- `--strings <resolved|strref|both>`: choose resolved TLK text, `#strref`, or both.
- `--follow-extern[=DEPTH]`: load externally-jumped DLGs and render a multi-file graph.
  Bare `--follow-extern` means depth `1`.

### Current graph mapping

| DLG element | Graph output |
|---|---|
| State `index` | Node `S{index}` in single-file graphs; `<DLG>_S{index}` in multi-file graphs |
| State label | `S{index}: <response text>` plus `if <trigger>` when triggers are enabled |
| Local transition | Edge to the next state |
| `terminates_dialog = true` | Edge to `END` |
| Extern jump | Edge to a dashed external node, or to the loaded target cluster when `--follow-extern` reached it |
| `player_text` | First line of the edge label when present |
| Transition `trigger_text` | Added as `if <trigger>`; in DOT output the edge is colored red |
| `journal_text` | Edge label includes `[journal]` |
| `action_text` | Included when actions are enabled; `StartCutScene*` calls are tagged as `[cutscene: ...]` or `[cutscene mode]` |

### Multi-file behavior

When `--follow-extern` is set:

- the root DLG is always included
- extern jumps are followed breadth-first up to the requested depth
- already-visited DLGs are deduplicated case-insensitively
- each loaded DLG renders in its own cluster/subgraph
- unresolved externs remain dashed external nodes

Important current behavior:

- extern DLGs are loaded through the same `--source <auto|override|bif>` selection as the root
- a missing extern read does not abort the graph; the edge stays external
- a present-but-unparseable extern prints a warning to `stderr` and is skipped, again degrading to an external node

### Output examples

DOT output is intended for Graphviz:

```bash
iecli dump --game <path> --resource FINMEL01.DLG --format dot > out.dot
dot -Tsvg out.dot > out.svg
```

Mermaid output is intended for Markdown renderers that support Mermaid:

```bash
iecli dump --game <path> --resource FINMEL01.DLG --format mermaid > out.mmd
```

### What this does not do

- It does not add new DLG parsing semantics; it renders already-decoded data.
- It does not promise full install-wide dialogue closure. `--follow-extern` is bounded and intentionally tolerant of missing/broken targets.
- It does not currently expose a separate flag for suppressing player text; if a transition has player text, it is part of the edge label.

---

## 2. `override-diff`

### Purpose

`override-diff` answers the trust question behind almost every modded-install
inspection session:

- is the live copy coming from `override` or BIFF?
- if `override` wins, is it materially different from stock?
- how does the live override set compare to a known reference directory or file?

This is intentionally about the bytes the game will actually read. It is not a
WeiDU provenance tool.

### Commands

```bash
iecli override-diff --game <path>
iecli override-diff --game <path> --type DLG --format json
iecli override-diff --game <path> --resource KIRINH.CRE
iecli override-diff --game <path> --against ./reference-dir --format json
iecli override-diff --game <path> --resource KIRINH.CRE --against ./KIRINH-stock.CRE --format json
```

### Mode A: shadow report

Without `--against`, `override-diff` scans `override` and compares it to the BIFF-set
resource inventory.

JSON shape:

```json
{
  "shadows": [
    {
      "resource": "FINMEL01.DLG",
      "in_override": true,
      "in_bif": true,
      "override_sha1": "…",
      "bif_sha1": "…",
      "identical": false
    }
  ],
  "override_only": ["FINBODH.CRE"],
  "counts": {
    "override_total": 12,
    "shadowing_bif": 5,
    "override_only": 7
  }
}
```

Meaning:

- `shadows`: resources present in both `override` and BIFF
- `identical: true`: an override copy exists but is byte-identical to stock
- `override_only`: resources present in `override` with no BIFF counterpart
- `counts.override_total`: total override resources examined after any filters

Text output is a sorted tab-separated report:

```text
resource	status	identical	override_sha1	bif_sha1
KIRINH.CRE	shadow	false	...	...
NEWCRE.CRE	override_only
counts	override_total=2	shadowing_bif=1	override_only=1
```

Supported filters:

- `--type <EXT>`: filter by resource type, for example `DLG` or `CRE`
- `--resource <NAME.EXT>`: restrict to a single resource

### Mode B: compare against a reference

With `--against`, the command switches to reference comparison.

#### `--against` points to a file

- `--resource <NAME.EXT>` is required
- the command reads the live override copy of that resource
- it compares SHA-1 against the reference file bytes
- output reports `status: "match"` or `status: "differ"`

Example JSON:

```json
{
  "resource": "KIRINH.CRE",
  "status": "match",
  "override_sha1": "…",
  "reference_sha1": "…"
}
```

#### `--against` points to a directory

- the command compares the live override set against files in the reference directory
- output reports `added`, `removed`, `changed`, and summary counts including `unchanged`
- `--type` and `--resource` apply here too

Example JSON:

```json
{
  "added": [
    { "resource": "ADDED.CRE", "sha1": "…" }
  ],
  "removed": [
    { "resource": "REMOVED.CRE", "sha1": "…" }
  ],
  "changed": [
    {
      "resource": "CHANGED.CRE",
      "override_sha1": "…",
      "reference_sha1": "…"
    }
  ],
  "counts": {
    "override_total": 3,
    "reference_total": 3,
    "added": 1,
    "removed": 1,
    "changed": 1,
    "unchanged": 1
  }
}
```

Current scope note: directory comparison scans the top-level files in the reference
directory. It is not recursive.

### What this does not do

- It does not attribute an override to a specific WeiDU component.
- It does not compare compiled resources against WeiDU source (`.d`, `.baf`) in a meaningful way.
- It does not treat BIFF as the live source in `--against file` mode; that mode compares the live override copy only.

### Intended use

Use `override-diff` early in any investigation where "which bytes are live?" matters:

- validating whether a DLG or CRE you are reading is the stock or overridden version
- checking whether an override was actually changed or just re-shipped unchanged
- comparing a modded override set to a clean extracted baseline
- comparing a patched resource in `override` to a saved reference binary
