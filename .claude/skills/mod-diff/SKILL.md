---
name: mod-diff
description: Summarize what a mod changed in an Infinity Engine install — which resources it added, which stock files it overrode, and (against a clean reference) exactly what differs — bucketed by resource type, as the day-one map for investigating a fresh mod. Use when the user asks what a mod changed/added/touched/broke, wants to diff a modded install against vanilla, scope an investigation to a mod's delta, or find mod-added resources.
---

# Mod Diff

Thin layer over `iecli override-diff` that buckets the result by resource type, so you can **scope an
investigation to what the mod actually touched** instead of re-sweeping the whole game. This is the
natural Step 0 before exploring new content (`explore-dungeon`) or planning a run on a modded install
(`plan-stat-build`).

## Inputs

- Modded game install path (auto-memory / `docs/LOCAL_GAME_PATHS.md`).
- *(Mode B, optional)* a **clean reference** — a directory of vanilla biff-extracted resources, or the
  mod's shipped binaries. Without it you get the shadow report (Mode A).

iecli must be built (`cargo build --release`).

## Two modes

**Mode A — shadow report (no reference).** Every `override` resource that also exists in a BIFF is a
*shadow*: the live game differs from stock. `override_only` resources are mod additions with no stock
version. Hash comparison flags benign shadows (a byte-identical re-ship) vs. real changes.

```bash
python .claude/skills/mod-diff/mod_diff.py --game "<game-path>"            # whole install
python .claude/skills/mod-diff/mod_diff.py --game "<game-path>" --type DLG # one type
```

Output buckets: **MOD-ADDED** (override-only), **MOD-CHANGED** (override differs from stock biff),
**benign shadows** (identical re-ship — ignore).

**Mode B — reference diff.** Byte/hash compare the live override against a clean reference →
added / removed / changed. This is the precise "what did the mod do" map for a fresh mod.

```bash
python .claude/skills/mod-diff/mod_diff.py --game "<modded>" --against "<clean-ref-dir-or-file>"
```

Add `--json` to either to pass the raw `override-diff` output through.

## Interpreting

- **MOD-ADDED / added** → brand-new content. Hand DLG/ARE here to `explore-dungeon` or read directly;
  these are what the mod introduces.
- **MOD-CHANGED / changed** → stock files the mod rewrote. **These are the prime suspects for the
  diagnostic "what did the mod break"** — a clobbered vanilla DLG/CRE/ARE is where stuck quests and
  broken links come from. Diff/read each.
- **benign / identical** → ignore; the mod re-shipped a byte-identical file.

## Caveats

- Mode A only knows override-vs-stock-biff *within one install*; it cannot tell you what changed
  relative to an *older* version of the same mod — that needs Mode B against the prior binaries.
- Compiled resources (`.bcs` from `.baf`, `.dlg` from `.d`) can't be byte-diffed against a mod's
  *source*; Mode B is meaningful only against already-compiled reference binaries.
- It surfaces *that* a resource changed, not *which WeiDU component* did it — component attribution is
  a separate, larger effort.

## When NOT to use

- A clean (unmodded) install — there's nothing to diff.
- You already know the resource and just want its contents — `iecli dump <resource>` directly.
