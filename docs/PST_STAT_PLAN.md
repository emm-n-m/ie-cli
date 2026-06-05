# Planescape: Torment — Stat Plan for The Nameless One

**Goal:** *balanced completionist* — see/experience as much content as possible (dialogue,
memories, the true ending) while staying combat-viable enough to never get hard-stuck.

**Derived from your actual install** (`PSTEE = …\Project P`) using `iecli`, cross-checked
against community mechanics. Everything below is reproducible — see [Methodology](#methodology--data-provenance).

> **Mod-delta flags** (your install vs. stock): of the 11 dialogues that permanently change a
> stat, **10 are byte-identical to the stock game files**; only **`DVHAIL` (Vhailor's Vision,
> the +3 STR grant) is mod-altered** in your override. The whole install has just 5 override-only
> DLGs, none of which touch stats. So this plan is effectively vanilla, with Vhailor's grant as
> the one thing to verify in-game.

---

## TL;DR

**Creation spread (base 9 each, 21 points to distribute, max 18/stat):**

| STR | DEX | CON | INT | WIS | CHA |
|:---:|:---:|:---:|:---:|:---:|:---:|
| **9** | **13** | **9** | **15** | **16** | **13** |
| +0 | +4 | +0 | +6 | +7 | +4 | → **21 pts** |

*WIS 16 lands exactly on the 25 cap with no waste. For max early-XP, run **WIS 18 / DEX 11**
instead and accept ~2 wasted WIS at the endgame — see [the WIS dilemma](#wis-early-xp-vs-the-25-cap-the-one-genuine-creation-dilemma).*

**Double-specialize as Mage** (the single most important class decision — see
[Class plan](#class-plan--double-specialize-as-mage)) and put early level-up points into **WIS**
(XP) and **DEX → 15/16** (AC); **leave STR alone** — in 2e it gives no combat bonus below 16 and a
Mage can't get percentile STR, so it's a dialogue-gate stat only and recovers on its own. End-of-game
projection (no cheats, no Wish): **WIS 25 · INT ~20 · CHA ~16 · STR ~14 · DEX ~16 · CON ~12**.

**Why this shape:** dump the two stats that barely gate content *and* recover well in-game
(**STR**, +5 in-game; **CON**, ~2 dialogue gates total and TNO is functionally immortal), and
buy at creation the four walls you *can't* recover into in time (**INT 15, CHA 13, WIS 16, DEX 13**).
Mage double-spec (+3 INT, +1 WIS) then carries INT past every meaningful gate.

---

## The two things that decide everything

### 1. What each stat actually gates (content map)

Scanning all **859 dialogues** for `CheckStatGT(Protagonist, …)` checks and bucketing by the
value you must reach to pass. Higher number in a cell = more dialogue branches that open at that
threshold (this is "how much content am I buying with this point"):

| need ≥ | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 21 | 24/25 | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **INT** | 2 | 70 | 116 | 50 | **229** | 70 | 14 | 13 | 6 | — | **589** |
| **CHA** | 72 | 24 | **181** | 27 | 43 | 50 | 1 | 2 | 3 | 3 | **451** |
| **WIS** | — | 8 | **110** | 28 | 36 | 39 | 6 | 6 | 4 | 1 | **243** |
| **DEX** | — | 6 | **58** | 5 | 1 | 6 | 1 | 7 | — | — | **95** |
| **STR** | 7 | 19 | **37** | 2 | — | 9 | 1 | — | 1 | — | **81** |
| **CON** | — | — | — | — | — | — | — | — | — | — | **~2** |

Headline breakpoints: **INT 15** (229 branches — the single biggest stat gate in the game),
**CHA 13** (181), **WIS 13** (110 — the biggest single *discrete* jump), then a second tier at
**16** for all three mental stats. STR/DEX concentrate at **13**. CON is a pure dump stat.

### 2. How much you can recover in-game (the permanent stat economy)

PST permanently changes stats through dialogue actions: `PermanentStatChange(Protagonist, …)`.
Extracted from every dialogue, mutual-exclusivity resolved by reading each tree. **Reliable**
totals for a balanced run (no class-locked sources):

| Stat | In-game gain | Sources (when) |
|---|:---:|---|
| **WIS** | **+8** | O / Hive tavern +1 *(early)*; Sarossa +1 *(mid, Godsmen)*; Ravel +3 *(late)*; Practical Incarnation +1 *(late)*; Good Incarnation +2 *(late)* |
| **STR** | **+5** | Vhailor +3 *(late, Law ≥ 35)* ⚠mod-altered; Paranoid Incarnation +1 *(late)*; Sarossa +1 *(mid)* |
| **CHA** | **+3** | Vivian +1 *(mid, Scent quest)*; Sebastion +2 *(mid, Grosuk quest)* |
| **INT** | **+2** | Ravel/Mebbeth +1 *(late)*; Practical Incarnation +1 *(late)* |
| **CON** | **+2** | Paranoid Incarnation +1 *(late)*; Sarossa +1 *(mid)* |
| **DEX** | **+1** | Sarossa +1 *(mid, fiddly — see itinerary)* |

Plus **~12 shared ability points** from level-ups (one per *new character level* across TNO's
three classes), and **one Wish Scroll** (+2 to a single stat of your choice, *if* you're a Mage
with INT ≥ 18 at level 12+).

**And the class-specialization milestone bonuses — the lever that fixes INT.** These are
**one-time** and tied to *whichever class reaches the milestone first* (you do **not** collect them
per-class):

| Milestone | Fighter | **Mage** | Thief |
|---|---|---|---|
| **Level 7** (first class to 7) | +1 STR | **+1 INT** | +1 DEX |
| **Level 12** (first class to 12) | +1 STR, +1 CON, +3 HP | **+2 INT, +1 WIS, +5 Lore** | +2 DEX, +1 Luck |

You get exactly **one** L7 bonus and **one** L12 bonus. Making **Mage the first class to reach both
7 and 12** ("double-specialize") banks **+3 INT and +1 WIS** — which is why it's the recommended
path for a content build: dialogue only recovers INT +2, so the Mage bonus more than doubles your
INT growth and lets you reach the high-INT tiers (even the 21 outlier) that were otherwise out of
range. To secure it: switch TNO to **Mage early** (learn it from Mebbeth in Ragpicker's Square) and
keep him a Mage through level 12 — don't let Fighter or Thief hit 7 or 12 first.

**The key asymmetry:**
- **WIS** rewards *early* investment (its +2.5%-XP-per-point-above-12 bonus compounds over the
  whole game) **and** has the most + most-reliable recovery (+8) — but the recovery is almost all
  *late* (Ravel's maze, Fortress of Regrets), so it does nothing for the XP snowball. WIS is also
  **never lowered** by any choice. It's also the stat that benefits *continuously* from an equipped
  +2 [tattoo](#tattoos--a-late-game-2-amplifier-for-any-stat) (gates **and** XP).
  → invest a *moderate* amount at creation (16), wear a WIS tattoo early for XP, climb to the 25 cap for free.
- **INT/CHA** gate the *most* content but recover *least* from dialogue (+2/+3) and *late*. Their
  big walls fire in mid-game Sigil before any recovery arrives. → you must **arrive** at the
  threshold, not recover into it. Buy at creation. (**INT is rescued by Mage double-spec: +3 INT,
  +1 WIS** — see above — which is why this build commits to Mage.)
- **STR** is the least content-gating stat **and** recovers best (+5) **and** has gear. → dump at
  creation, rebuild with early level-up points + late grants.
- **CON** gates ~2 branches total and TNO resurrects on death. → hard dump.
- **DEX** is the awkward one: a big wall at 13 (58 branches) but almost no recovery (+1). → buy
  the 13 at creation; it's the one "physical" stat you can't defer.

---

## Recommended build

### Character creation (21 points)

```
STR  9   ← dump; 2e gives no combat bonus <16 & no percentile STR for a Mage, so it's dialogue-only. Recovers +5 on its own
DEX 13   ← buy the 58-branch wall now; it barely recovers
CON  9   ← hard dump; ~2 gates, TNO is immortal
INT 15   ← the 229-branch wall; must be present, recovers only +2/late
WIS 16   ← lands exactly on the 25 cap with zero waste; push to 17–18 for more early XP (see below)
CHA 13   ← the 181-branch cliff; recovers +3 to 16
```

This sits **every spent point exactly on a breakpoint** and catches four of the five major walls
at turn one. (Total stat points 75 = 54 base + 21.)

**Power-variant** (if you'll reliably dump your first level-up into DEX): run **INT 16 / DEX 12**
instead — banks the 70-branch INT-16 tier early (it's mostly unrecoverable in time), then a single
early level-up restores DEX to 13.

### WIS: early XP vs. the 25 cap (the one genuine creation dilemma)

WIS is the only stat where "just max it" backfires. Two opposing pressures:

- **Push it up early —** its +2.5%-XP-per-point bonus *compounds* over the whole game, and the
  faster leveling hands you **~1–2 extra level-up ability points** (early WIS partially pays for
  itself).
- **…but not too high —** reliable non-creation WIS is **+9** (O +1 early · Sarossa +1 mid ·
  Mage-spec +1 · Ravel +3 late · **Fortress +3 dead last**), and the cap is **25**. The Fortress
  incarnation grants land at the very end, so anything that puts you over 25 there is **wasted** —
  and wasted nearly for free, since late WIS gives no XP (game's over) and opens no new gates.

| Creation WIS | after Ravel | after Fortress | overcap |
|:---:|:---:|:---:|:---:|
| **16** | 22 | **25** (exact) | 0 |
| **17** | 23 | 26 | 1 |
| **18** | 24 | 27 | 2 |

So each creation point above 16 buys **+2.5% XP for most of the run** and wastes **~1 WIS** at the
very end. Within the 21-point budget the points come from **DEX** (the INT/CHA walls are
untouchable) — and the XP-driven bonus ability points refund that DEX almost exactly:

- **WIS 16 / DEX 13** — clean, zero waste, both on breakpoints. **The recommended default.**
- **WIS 18 / DEX 11** — the "max-XP" gambit: more XP from turn one (→ ~1–2 bonus points back into
  DEX), at the cost of ~2 WIS overcapped in the Fortress. Still a *marginal* option — note that
  tattoos **don't** rescue it (the WIS tattoos are gated mid/late, so they can't supply the *early*
  XP this buys), but a +2 WIS tattoo does cover mid/late WIS XP. So WIS 18 nets only a bit of
  *early-only* XP for the DEX cost. Take it only if you want the smoothest early curve; otherwise 16.

(There's no benefit to creation WIS **above 18** — it's the creation cap anyway, and you'd just
overcap more.)

### Tattoos — a late-game +2 amplifier for any stat

Tattoos are TNO-exclusive gear, and your install has a **complete +1/+2 set for every attribute**:

| Stat | +1 | +2 ("Greater") | | Stat | +1 | +2 ("Greater") |
|---|---|---|---|---|---|---|
| **STR** | Tattoo of Might | Greater Might | | **INT** | Tattoo of Insight | Revelation |
| **DEX** | Tattoo of Action | Greater Action | | **WIS** | Tattoo of the Spirit | the Soul |
| **CON** | Tattoo of Health | Greater Health | | **CHA** | Tattoo of Presence | Greater Presence |

(Also +1/+2 Max-HP and AC tattoos. The companion tattoos — Annah/Ignus/Skull/Ravel's Kiss — *cost*
−1 WIS for their other perks.)

> **iecli caveat (found while writing this):** PST uses a **different effect-opcode table** than
> Baldur's Gate, and iecli ships only the BG table — so it mislabels PST stat effects (PST's real
> ability opcodes are STR 44 · DEX 15 · CON 10 · INT 19 · WIS 49 · CHA 6; iecli reads 49/44 as
> WIS/STR by luck and blanks the rest). An earlier pass that filtered on iecli's *decoded names*
> wrongly concluded "WIS-only tattoos." These values were re-derived by reading raw opcodes. *(This
> is a real iecli gap — a PST opcode table is worth filing.)*

Two mechanics make these matter a lot:
- **Dialogue `CheckStat` gates read your *current* (item-modified) stat** — so **any single gate is
  reachable at base + 2** by equipping that stat's Greater tattoo before the conversation.
- **The WIS XP bonus uses effective WIS** — the standard "equip/buff WIS before a quest turn-in"
  trick; a +2 WIS tattoo at turn-ins is **+5% XP** on those awards.

**What this does to the plan — less than you'd hope early, a lot late.** Your install has **3+
tattoo slots**, so by the endgame you can stack **+2 on three different stats at once**. *But the
meaningful stat tattoos are progress-gated to mid/late game* (e.g. the Tattoo of the Redeemer needs
you to reason with Trias — right before the Fortress of Regrets). They therefore **do not help with
the early/mid-game walls** (INT 15, CHA 13), which is exactly when those checks fire. So:

- **Keep the creation floors as written** — you must clear the early/mid walls with *base* stat,
  because you won't own the tattoos yet. Tattoos do **not** let you safely shave creation.
- **Late game, stack +2 on your three highest-value stats.** With base + Mage-spec + dialogue grants
  already high, three +2 tattoos push **effective INT to ~22** (clears the INT-21 tier outright),
  top DEX/STR for the late physical checks, etc.
- **Wear a +2 WIS tattoo from whenever you first get one until base WIS nears 25** (XP at quest
  turn-ins + WIS gates), then free that slot — past 25 it just overcaps.
- Still out of reach: the **CHA 21/24/25** legendary checks (base ~16 + 2 = 18 < 21, and several fire
  before you'd have the tattoo). Everything else is clearable by endgame.

### Level-up points (~12 over the game), in priority order

> **2e reality check (this build is a Mage):** ability bonuses don't start until **16** (STR/CON) or
> **15** (DEX AC). So a stat sitting at 13 gives **no combat benefit** — it's a *dialogue-gate* value
> only. And percentile Strength (18/xx) is **warrior-only**, so a Mage's STR caps at a flat +1/+2 on
> a weapon you won't swing. Don't spend level-up points chasing combat numbers that 2e won't pay out;
> spend them where they convert.

1. **WIS — *early* points (if you stayed at creation 16).** A couple in the first third of the game
   buy XP for the rest of it (same early-XP/late-overcap trade as the
   [WIS dilemma](#wis-early-xp-vs-the-25-cap-the-one-genuine-creation-dilemma)). Once WIS is within
   ~3 of 25, stop — Ravel/Fortress grants finish it and the rest overcaps.
2. **DEX 13 → 15/16** — the *real* survivability lever for a squishy Mage: 2e AC bonus starts at 15
   (DEX 16 = −2 AC). This is what "early combat points" actually means here — **not STR**. Optional,
   since TNO resurrects on death; take it for comfort.
3. **INT** mostly takes care of itself: creation 15 + Mage double-spec (+3) + dialogue (+2) → 20.
   Spend here only to bridge to a high-INT line you're about to hit before Mage L12 lands.
4. **Not STR, not CHA.** STR stays 9 and drifts to ~14–18 on its own (Vhailor +3, grants, a STR
   tattoo) — fine for the *late* STR lines; the early intimidation lines are the only casualty, and
   they're minor. CHA needs nothing past the +3 grants → 16 (+2 tattoo → 18); the higher CHA checks
   are redundant/flavor (see caveats). Touch either **only** if you specifically want a dialogue gate
   they cover, and even then a STR/CHA tattoo equipped before the line is cheaper than base points.

### Class plan — double-specialize as Mage

This build **commits to Mage**, and the *timing* matters because the specialization bonuses go to
the **first** class to reach each milestone:

- TNO starts as a Fighter. **Learn the Mage class from Mebbeth (Ragpicker's Square) and switch
  early — before Fighter ever reaches level 7** — then stay a Mage straight through level 12. That
  makes Mage your L7 *and* L12 specialist → **+3 INT, +1 WIS, +5 Lore**.
- Don't grind Fighter or Thief to 7/12 first, or you'll spend your one L7 (and/or L12) bonus on the
  wrong class (Fighter L12 is +1 STR/+1 CON — a stat you're intentionally dumping).
- Early squishiness is a non-issue: PST's opening (Mortuary/Hive) is trivial, Morte and Dakkon tank,
  and TNO resurrects on death anyway. If you want a survivability cushion, your first level-up
  **ability** points (separate from class XP) go into **DEX** for AC (15+) — *not* STR, which 2e
  won't reward on a Mage.
- Mage is also the strongest endgame class and the only one that can use the Wish Scroll, so there's
  no late-game trade-off. (If you'd rather keep early STR, taking Fighter L7 +1 STR then Mage L12
  +2 INT/+1 WIS is a defensible split — but you lose 1 INT vs. full Mage double-spec.)

---

## Sequenced "grab-this-here" itinerary

> Pick the **stat-optimal** reply even when it's the "evil"-flavored one (e.g. Mebbeth/Ravel's
> *"Because I need power"*) — it banks both WIS and INT. None of these have lasting karma cost.

**Early — The Hive / Ragpicker's Square**
- **Switch to Mage ASAP** (learn it from **Mebbeth**, Ragpicker's Square) — before Fighter hits
  level 7 — and stay Mage through level 12 to lock the **+3 INT / +1 WIS** double-spec. This is the
  highest-value single action in the whole plan.
- **`O`, the Hive tavern** → **+1 WIS**. Path: reach the "secrets of existence" exchange, then
  *"That was… indescribable."*

**Mid — Sigil**
- **Sarossa, the Great Foundry** (*join the Godsmen first*) → **+1 WIS** (Truth *"Yes"* at the
  Godsmen-gated branch). ⚠ Her early branches can *trade away* STR/DEX/CON if you attack/lie —
  apologize cleanly and don't take the loss-then-restore detours.
- **Vivian, Hive brothel** (Scent quest) → **+1 CHA** (linear, no cost).
- **Sebastion** (Grosuk / abishai quest) → **+2 CHA** (the two +2 paths are mutually exclusive —
  you get +2 once, not +4).

**Late — Ravel's Maze**
- **Ravel Puzzlewell**:
  - Mebbeth-teaching: choose **"Because I need power"** → **+1 WIS & +1 INT** (one transition).
  - Multiverse scene with **`Ravel_Flatter` > 10** (flatter/charm her throughout — high CHA helps
    reach those replies) → **+2 WIS** (vs +1 if you flattered less).
  - *"Let her work"* when she stitches you → **+3 max HP**.

**Late — Fortress of Regrets** (you absorb **all three** incarnations in sequence — *not* pick-one)
- **Practical Incarnation** → **+1 INT & +1 WIS** (win the merge: WIS > 20 *or* INT > 20).
- **Paranoid Incarnation** → **+1 STR & +1 CON** (peaceful or forced — same reward).
- **Good Incarnation** → **+2 WIS**.
- **Vhailor's Vision** (*Law ≥ 35*) → **+3 STR (+3 damage)**. ⚠ **This is the one mod-altered
  dialogue in your install** — confirm the grant behaves as described when you get there.

**Optional**
- **Wish Scroll** (Mage, INT ≥ 18, level 12+) → **+2 to one stat**. Spend on **WIS** (→ 25) or
  **INT**. It's a genuine biffed item in your files but its location is uncertain (possibly rare/
  unused) — don't build the plan around it.

---

## Honest caveats

- **You cannot literally see every branch in one playthrough.** Many transitions have a *high-stat*
  and a *low-stat* variant of the same line — you'll only ever trigger one. "Completionist" here
  means catching the *up-gated* branches, which carry the rewards/lore; the low-stat versions are
  mostly flavor.
- **What's still out of reach — and why it doesn't matter:** the **CHA 21/24/25** checks. There are
  only three sources, all late: **the Transcendent One** (`DTRANS`, the finale — CHA 24), **Nordom**
  (`DNORDOM` — CHA 25, a companion-reaction line), and **Vhailor** (`DVHAIL` — CHA 21). CHA tops out
  ~16 base + 2 tattoo = 18 here. **Crucially, the finale is not CHA-locked:** the Transcendent One
  offers *parallel* routes — WIS>20/>23, INT>20, **or** CHA>23 — to the same resolution, and this
  build hits **WIS 25** and **INT ~22**, clearing the WIS and INT routes outright. So the CHA-24 path
  is a redundant alternative, not exclusive content; what you actually forgo is a handful of flavor
  lines (a Nordom reaction, a Vhailor persuasion alt). Nothing in the ending is lost.
- The **CHA 18** checks (Kesai-Serris, the Pillar of Skulls) *are* reachable: base 16 + a Greater
  Presence tattoo = 18. And **INT 21** is reachable (Mage double-spec → ~20, +tattoo/level-up → 22).
  WIS's 21/24 tiers are covered (you hit 25).
- **Alignment nudges:** Vhailor's +3 STR needs **Lawful** (Law ≥ 35); Sarossa's WIS needs
  **Godsmen** membership. Mild role-play steering, not hard requirements.

---

## Methodology / data provenance

All install-specific numbers were produced with `iecli` against `…\Project P`:

- **Content map** — dumped all 859 `DLG`, parsed `CheckStatGT(Protagonist,N,STAT)` out of state/
  transition trigger text, histogrammed required value (`N+1`) per stat.
- **Stat economy** — scanned every dialogue's `action` text for
  `PermanentStatChange(Protagonist,STAT,RAISE|LOWER,N)`; the 11 grant dialogues were re-dumped with
  resolved strings and each tree read to resolve mutual exclusivity and conditions. (Items/spells
  were also scanned — they only carry *while-equipped* stat effects, never permanent raises.)
- **Tattoos** — all 65 `TT*` items dumped. A **complete +1/+2 set exists for every attribute**
  (Might/Action/Health/Insight/Spirit/Presence at +1, the "Greater"/Soul/Revelation versions at +2),
  read from **raw opcodes** after discovering iecli mis-decodes PST effects: it uses the BG opcode
  table, but PST's ability opcodes are STR 44 · DEX 15 · CON 10 · INT 19 · WIS 49 · CHA 6. The
  *gates-read-current-stat* and *XP-uses-effective-WIS* behaviors are standard PST mechanics
  (community-documented), not something iecli measures.
- **Trust layer** — `iecli override-diff --type DLG` confirmed 10/11 grant dialogues are
  byte-identical to stock biff; **`DVHAIL` differs** (mod-altered); 5 override-only DLGs total,
  none stat-related.
- **Mechanics** (creation pool, level-up cadence, WIS XP table, class-specialization milestone
  bonuses) — community sources + your own install data (base 9, 21 to distribute) + your in-game
  confirmation of the level-7/12 specialization values. (Spec bonuses live in the engine/2DA tables,
  not the DLG/ITM resources iecli currently parses, so they're not install-verified here.)

*Generated 2026-06-05.*
