---
name: diagnose-dialog
description: Diagnose Infinity Engine NPC dialog trigger graphs via `iecli` — use when the user asks why an NPC gives a one-liner and exits, why an expected dialog option is missing, or what conditions gate a dialog branch. Produces a concrete answer about which variable or trigger is stopping the intended dialog path.
---

# Diagnose IE NPC Dialog

Given an NPC name or DLG resref, identify what trigger/variable is gating the expected dialog branch — and where the variable normally gets set.

## Inputs

The user provides an NPC name (e.g. "Quenash", "Rasaad") or a DLG resref. Be tolerant of partial names: **resrefs are max 8 characters**, so "quenash" → `QUENAS`, "shandalar" → `SHANDA`. Always try truncated variants if the obvious name misses.

The game install path is in the user's auto-memory (typically their BG:EE Steam install). Use that — do not ask the user for the path.

## Step 1 — Locate the DLG

```bash
./target/release/iecli list --game "<game-path>" --type dlg --name <partial> --format json
```

- If **multiple** DLGs match, list them and ask which one to investigate.
- If **none** match, try a shorter prefix or check the user's creature naming hints from the conversation.
- Note the `source_kind`: `Override` = modded; `Bif` = stock. This informs later comparison steps.

If the user hasn't already dumped the DLG, check for `*_dialog.txt` or `*_dialog.json` files in the working directory first — they may have one from a previous session.

## Step 2 — Dump the DLG

```bash
./target/release/iecli dump --game "<game-path>" --resource <RESREF>.DLG > /c/tmp/<resref>.json
```

If the override/stock question matters, dump both:
```bash
./target/release/iecli dump ... --source override > override.json
./target/release/iecli dump ... --source bif      > bif.json
```
Then diff.

## Step 3 — Analyze state triggers

Grep the `state_triggers` section and each state's `trigger_text` field. Classify each state:

| Trigger shape                                   | Role                                       |
|-------------------------------------------------|--------------------------------------------|
| `Global("X","GLOBAL",N)` / `Global("X","LOCALS",N)` | Quest-state gated — the "good" branch  |
| `NumberOfTimesTalkedTo(0)` + `AreaCheck(...)`   | First-meeting branch                       |
| `Dead(...)` / `ReactionGT(...)`                 | Context gate (plot-progress or reputation) |
| `True()` with a single `TerminatesDialog` transition | **The brush-off fallback** — a one-liner that ends dialog |

**Core pattern**: the engine tries states *in order*; the first with a passing trigger wins. When the user reports "one line then ends", the `True()` fallback is firing because every earlier `Global(...)`-gated state failed.

## Step 4 — Identify the gating variable(s)

From the intended (failing) state's trigger, extract:
- GLOBAL variable names (most common)
- LOCALS variable names (stored on the creature, harder to check from console)
- Area checks, Dead() checks, etc.

For each gating variable, the next question is: *what sets it, and under what conditions?*

## Step 5 — Trace upstream

If the gating variable is a global set by another NPC, find that NPC's DLG and grep its `action_text` fields:

```bash
./target/release/iecli dump --game "<game-path>" --resource <UPSTREAM>.DLG > /c/tmp/upstream.json
grep -n "SetGlobal.*\"<VarName>\"" /c/tmp/upstream.json
```

This reveals which transition path sets the variable — i.e. which conversation branch the player needed to take. That's the "missed hint" that often explains the symptom.

If you don't know the upstream NPC, this step can be deferred; often the user knows or can guess.

## Step 6 — Report

Structure the findings:

1. **Intended state**: quote its response text (the NPC line the user was expecting).
2. **Gating trigger**: the exact condition line, with the key variable called out.
3. **Actual state**: the fallback state's response text (what the user is seeing).
4. **Upstream** (if traced): which NPC and which dialog path sets the gating variable.
5. **Verification commands** — see below.

## Step 7 — In-game verification commands

On the user's BG:EE install, the console requires the full `CLUAConsole:` prefix. The short `C:` alias does **not** work. Always use:

```
CLUAConsole:GetGlobal("<VarName>","GLOBAL")
```

If the user asks to force-set (testing only, not intended fix):
```
CLUAConsole:SetGlobal("<VarName>","GLOBAL",<value>)
```

For LOCALS-namespace variables, console access is awkward — mention this and prefer checking an equivalent GLOBAL if one exists, or inferring state from visible side-effects.

## Interpretation — bug vs. intended

Before framing findings as a bug, consider the alternative: **the dialog branch may simply be gated behind a hint or conversation the player hasn't reached yet**. The output of this skill is useful either way — it tells the user exactly which upstream conversation/condition to pursue. Present findings neutrally; let the user decide whether it's stock behavior or a real bug.

## Pitfalls

- **Resref truncation**: always try the 8-char truncation (`quenash` → `QUENAS`). This is a frequent "no matches" cause.
- **Heavily modded override DLGs**: trigger sets may not match IESDP-typical patterns. When in doubt, dump both override and BIFF and diff.
- **Mod-added globals** with unusual prefixes (e.g. `X3...`, `RE1_...`) indicate mod content — not necessarily the bug, just a source identifier.
- **Terminate-only transitions without Global-gated states anywhere** means the DLG has no branching logic to fail — if the user sees a one-liner there, the issue is usually upstream (wrong CRE.dialog reference, wrong NPC, etc.), not a DLG-trigger issue.

## Example summary format

```
Intended state (state <N>): "<NPC line the user expected>"
  Gated by: Global("<VarName>","GLOBAL",<value>)

Actual state (state <M>): "<one-liner the user is seeing>"
  Trigger: True() — catches the fall-through

The variable "<VarName>" is set by <Upstream NPC> in their dialog at <which path>.
The player likely missed that branch.

Verify: CLUAConsole:GetGlobal("<VarName>","GLOBAL")
```
