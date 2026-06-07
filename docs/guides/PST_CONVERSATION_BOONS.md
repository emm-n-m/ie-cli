# Planescape: Torment — Conversation Boons & XP Guide

*How to talk your way into permanent stat gains and the game's biggest XP payouts.*

This guide catalogues every **dialogue** in Planescape that permanently raises one of the
Nameless One's stats, or hands the party / the Nameless One a large lump of experience —
plus the conversations where a **companion** earns boons of their own.

> **Source & accuracy.** Extracted directly from the *installed* game with
> `iecli` (sweep of all 859 dialogues; see [Reproducing](#reproducing-this-guide) at the
> end). This install is **modded** (`PST-UB-reloaded`, `PowerOfBelief` per `WeiDU.log`), so a
> few branches and numbers reflect those mods rather than vanilla PST:EE. Everything below is
> what *your* install actually does — which is the point.
>
> Trigger thresholds are stated as the **value you need**. The engine uses `CheckStatGT(…,N)`
> = "stat **strictly greater** than N", so a `GT 17` gate means you need **18+**.

---

## TL;DR — what your stats unlock

The conversation economy is overwhelmingly **Wisdom-gated**, with Intelligence and Charisma
opening specific doors:

| Stat | What conversations it unlocks |
|------|-------------------------------|
| **Wisdom** | The single most valuable stat. Gates Dak'kon's *Circles of Zerthimon* XP ladder, Ravel's "close your eyes" boon, the Incarnation merges, and Grace's riddle. High WIS = most XP and most extra dialogue, full stop. |
| **Intelligence** | Dak'kon's plate-puzzle XP (16+/18+), Grace's riddle, Ravel's reading-the-Art boon, the Practical Incarnation. |
| **Charisma** | Scales **Nordom's** party-XP payouts and his stat upgrades; one Vhailor release branch (21+). |
| **Strength / Dexterity / Con** | Mostly *received* as boons (Vhailor → STR, Annah → thief skills) rather than gating content. |

**Rule of thumb:** pump **Wisdom** first if you want to see — and be paid for — the most
conversation content. Then Intelligence. Charisma pays off specifically if you recruit Nordom.

---

# Part 1 — Companion conversations

The seven joinable companions: **Dak'kon, Morte, Annah, Fall-from-Grace, Nordom, Vhailor, Ignus.**

## Dak'kon — the Circles of Zerthimon  *(the single best repeatable XP engine)*

Dak'kon carries the **Unbroken Circle of Zerthimon**. Studying it and discussing its eight
Circles with him is the iconic "conversation = XP" loop in the game, and it is **Wisdom-and-
Intelligence gated** — which is the in-fiction reason a thoughtful Nameless One is rewarded.

**Reciting each Circle back to him** — Dak'kon asks *"What did you come to know?"*; each Circle
you've absorbed (tracked by `Dakkon_Teach`) pays escalating XP, but only if your Wisdom is high
enough to have understood it:

| Circle (`Dakkon_Teach`) | Wisdom needed | XP to TNO |
|---|---|---|
| 2 | 12+ | 300 |
| 4 | 12+ | 600 |
| 6 | 14+ | 900 |
| 8 | 15+ | 1,500 |
| 10 | 16+ | 3,000 |
| 12 | 18+ | 5,000 |

**Puzzling out the plates yourself** (the *Unbroken Circle* item dialogue) rewards Intelligence:

- INT 16+, after Circle 13 unlocked → **3,000**
- INT 18+, after Circle 16 → **6,000**

**Unlocking the deeper Circles:**

- Seventh Circle → **5,000** to TNO
- Eighth Circle → **10,000** to TNO **and 10,000 to Dak'kon**, plus a permanent boon to **Dak'kon: STR +1, DEX +2, CON +2**.

Plus scattered philosophy beats (his questions about *knowing*, the Fell, his slavery) worth
250–5,000 XP to TNO each. **Bottom line:** invest Wisdom early, exhaust Dak'kon's teaching tree
as your WIS climbs, and re-visit *"What did you come to know?"* each time you cross a WIS
breakpoint.

## Fall-from-Grace

- **The sensory-stone riddle** ("the tenth student") → **3,000 party XP**. You get the better
  answer line with **INT 15+** or **WIS 14+**, but the XP is granted either way.
- **The romance / "I do" beat** → **20,000 party XP** (two branches, 20k each — the single
  largest companion-conversation XP in her tree).
- **Deionarra's longing memory** → **2,000 to TNO** (shared one-time memory — see note below).
- Coming to terms with her past / the Ravel naming → **1,000 party** each.

## Morte

- **Freeing Morte from the Pillar of Skulls** (the recovered memory) → **12,000 to TNO**.
- **"Thanks for coming clean, Morte"** — when he finally tells you the truth → **12,000 party
  XP** *and* a permanent boon to **Morte: STR +4, DEX +2, CON +2** (his best upgrade — get it).
- "What were you DOING there?" (the hells) → **12,000 party**.
- The tattoo / "don't trust the skull" beat → **1,000 party**.

## Annah — mutual thief training

Annah can **train the Nameless One in thievery**, and you can train her back.

- **Full thief tutoring** → sets TNO's **Stealth → 17, Find/Remove Traps → 13, Pick Pockets →
  22, Open Locks → 18**, plus XP that *scales inversely with your Dexterity* (the clumsier you
  are, the more she teaches and the more XP):
  - DEX 12 or less → **2,500 XP**
  - DEX 13–15 → **3,125**
  - DEX 16–17 → **3,438**
  - DEX 18+ → **3,750**

  This is a huge deal for a **non-thief Nameless One**: it's a one-conversation jump to
  respectable thief skills.
- Incremental lessons (Stealth / Open Locks / Traps / Pick Pockets) → **+3 each + 1,000 XP**.
- **Training Annah back** → she gets **+3** to each skill **+1,000 XP** per lesson (gated on the
  `Annah_Training` global advancing), and later **Fire / Magic-Fire resistance +5**.
- Deionarra's longing memory → **2,000 to TNO** (shared one-time — see note).

## Nordom — the party-XP machine *(Charisma-scaled)*

Nordom hands out the **biggest repeatable party XP** of any companion — **36,000 a pop** across
joining, his quest, and self-reflection beats. The marquee branch tunes him up *and* pays you
more the higher your **Charisma**:

**"Strength. Speed. Power. Focus."** —
- CHR 18 or less → **36,000 party**, Nordom STR +1 / CON +1
- CHR 19–24 → **48,000 party**, Nordom STR +2 / CON +1 / AC improves
- CHR 25+ → **60,000 party**, Nordom STR +2 / CON +1 / Luck +1 / AC improves by 2

**Class-tailored orders** (each → 36,000 party) require the matching class **and** stat:
Fighter + STR 17+, Mage + INT 17+, or Thief + DEX 17+. Other beats let you reshape Nordom (trade
INT for CON/AC, raise DEX, etc.).

## Vhailor — the largest single boon in the game *(Law-scaled)*

- **Recalling the memory of Vhailor** → **60,000 to TNO**.
- **Vhailor's parting gift** (the *"Wh…"* branch) scales with how **lawful** you've been (the
  `Law` counter):
  - Law < 25 → **90,000 to TNO**, **STR +1, Damage Bonus +1**
  - Law 25–34 → **120,000**, **STR +2, Dmg +2**
  - Law 35+ → **150,000**, **STR +3, Dmg +3**

  Act lawfully across the game and this becomes the best stat-and-XP boon any companion gives.
- Various release / "let go" beats → **90,000 party / TNO** each.
- At the finale, charging Vhailor with your final justice gives **him** STR +3 and **2,000,000
  XP** (endgame).

## Ignus — fire resistance for a price

Ignus teaches fire magic, and (if you feed him pieces of yourself) trades **permanent fire
resistance for max HP**:

- Teaching beats / learning his story → **5,000 / 10,000 / 6,000 party XP**.
- "To learn, you must suffer" memory → **10,000 to TNO**.
- **Body-part teaching** (give him your eyeball, a strip of flesh, etc.):
  - **Resist Fire +3 / Resist Magic-Fire +3, Max HP −1**, +6,000 XP
  - **Resist Fire +5 / Resist Magic-Fire +5, Max HP −3**, +12,000 XP
  - handing over the eyeball / intestine → **+12,000 / +24,000 to TNO**

  A deliberate trade — you permanently lose a little max HP for standing fire immunity.

> **Shared one-time memory.** *Deionarra's Longing* (`Memory_Deionarra_Longing`) grants **2,000
> to TNO once**, triggered through whichever of Grace or Annah you happen to talk to first. Don't
> expect to collect it twice.

---

# Part 2 — Big XP & stat conversations (non-companions)

These aren't party members, but they're the **largest "talk and get paid" moments** in the game
and several grant permanent core-stat boons.

## Ravel Puzzlewell — the stat-and-XP jackpot

The night-hag's conversation is the richest single dialogue outside the endgame: it grants
**permanent Wisdom/Intelligence**, a max-HP bump, and XP in 90,000–180,000 chunks.

- **"Close your eyes and listen…"** scales with how much you've **flattered her**
  (`Ravel_Flatter`):
  - Flatter < 7 → **90,000 + WIS +1**
  - Flatter 7–10 → **120,000 + WIS +1**
  - Flatter 11+ → **180,000 + WIS +2**
- **Reading the Art** with her (the Mebbeth echo) → **WIS +1**, **INT +1**, or **both** (+90k–120k).
- **"Let her work"** (she plucks a hair, works her craft on you) → **Max HP +3**.
- Solving her riddle / surviving the encounter → **90,000–180,000 party** across several beats.

**Flatter Ravel, keep WIS/INT in the "she respects you" band, and milk every branch** before the
fight — this is the biggest permanent-stat conversation in the game.

## The Incarnations — Fortress of Regrets

Confronting (and merging with) your **past selves** at the endgame grants core stats + huge XP:

| Incarnation | Boon to TNO | XP |
|---|---|---|
| **Practical / First** (`DINCAR1`) | **INT +1, WIS +1** | 96,000 |
| **Paranoid / Warrior** (`DINCAR2`) | **STR +1, CON +1** | 64,000 |
| **Good Incarnation** (`DINCAR3`) | **WIS +1** | 32,000 |

Plus party XP for understanding the nature of the incarnations (the "are we all the first one?"
realization needs **INT 17+** → 96,000 party).

## The Bronze Sphere — the climactic memory

`BSPHERE` — recovering the final memory ("the first incarnation…") grants **2,000,000 XP to the
Nameless One**, the largest single payout in the game.

## Other large quest-completion payouts

These are **quest rewards delivered in dialogue** (party XP) rather than skill/stat boons, but
they're the heaviest lumps you'll bank — listed so you know which conversations not to skip:

| Conversation | Party XP | Who / what |
|---|---|---|
| `DTRIAS` | ~2,112,500 | Trias the Betrayer (endgame) |
| `DFHJULL` | ~770,000 | Fhjull Forked-Tongue |
| `DCRSSMTH` / `DKESTER` | ~543,750 each | Curst — the smith / Kester |
| `DMARQUEZ` | ~522,500 | Marquez |
| `DBURGHER` | ~475,000 | the Burgher (Curst) |
| `DEBB` / `DCRSCAPT` | ~450k / 400k | Curst questlines |
| `DLOTHAR` | ~360,000 | Lothar the Bone Lord |

*(Full ranked list lives in the scan output — regenerate with the command below.)*

---

## Build implications

1. **Wisdom is king.** It gates Dak'kon's Circle XP ladder, Ravel's WIS boon, the Incarnations,
   and Grace's riddle — and the permanent WIS *boons* (Ravel +2, Incarnations, Dak'kon-adjacent)
   compound, so early WIS investment snowballs.
2. **Intelligence second** — Dak'kon's plate puzzle, Ravel's reading-the-Art, the Practical
   Incarnation, and several "understand the truth" beats.
3. **Charisma if you take Nordom** — it directly multiplies his (large, repeatable) party-XP
   payouts and his stat upgrades.
4. **Don't over-invest base DEX** if you plan to recruit **Annah** — she'll tutor a clumsy
   Nameless One into solid thief skills *and* pay more XP for doing it.
5. **Recover every companion's core memory** — Morte (Pillar), Vhailor, Ignus, Grace (Deionarra):
   each is a one-time 10k–60k+ chunk plus, often, a permanent boon to that companion.

---

## Reproducing this guide

Everything here came from a deterministic sweep of the install — no GUI, no guessing:

```bash
# 1. dump + scan all dialogues for protagonist/party XP and PermanentStatChange
python C:\tmp\scan_boons.py        # caches dumps to C:\tmp\pst_dlg, writes C:\tmp\pst_boons.json
# 2. re-slice the cache by grant target (TNO vs each companion)
python C:\tmp\scan2.py
```

The scanners shell out to `iecli dump --resource <DLG> --format json` and match three action
patterns in each transition's `action_text`:

- `GiveExperience(Protagonist, N)` — XP to the Nameless One
- `AddexperienceParty(N)` — XP to the whole party
- `PermanentStatChange(Protagonist, STAT, RAISE|LOWER, N)` — permanent stat change

Each hit carries the NPC line, the player reply, and the `trigger_text` gate — which is how the
Wisdom/Intelligence/Charisma thresholds above were recovered.

> Want it as a sequenced "do this conversation at this point" itinerary instead of a catalogue?
> That's the `plan-stat-build` skill's job — it folds these grants into a full creation +
> level-up + route plan.
