# Blizzard in Baator — NPC Reward & Requirement Map

*Which NPCs in the **Blizzard in Baator** mod (PST:EE, v1.0.2) hand out big XP or permanent
stat boons, and exactly what you need to unlock each.*

> **Source & method.** Extracted from the *installed* mod's own WeiDU **`.d` dialogue source**
> (`BlizzardinBaator/d/*.d`, 171 files), cross-referenced against the compiled `g-bb*.cre`
> creatures via `iecli`. Because the mod ships source, every reward below is read from the
> literal `DO~...~` action block and its `IF~...~` trigger — not inferred from a statistical
> sweep. The parser reconciles **exactly** against a raw grep of the sources: **437/437**
> `AddExperienceParty`, **94/94** `PermanentStatChange`, **219/219** `GiveItemCreate` captured.
> See [Reproducing](#reproducing) at the end.
>
> **Threshold convention.** The engine has three separate stat triggers (confirmed from
> `TRIGGER.IDS`): `CheckStatGT(…,N)` = **strictly greater** (so `GT 14` means you need **15+**),
> `CheckStatLT(…,N)` = strictly less, and plain `CheckStat(…,N)` = **exactly N**. Thresholds
> below are stated as the value you actually need.

---

## TL;DR

- **213 creatures, 170 with dialogue.** 87 dialogue files contain a reward action; **~530
  reward-bearing reply branches** in total.
- The **single biggest permanent boon** is **Dopilp**, the kuo-toa priest (`G-BBD010`):
  become his tribe's god for **+1 to ALL six stats, +20 max HP, and 100,000 XP** — and it is
  **once only** (see the correction below).
- **Most reward branches are Wisdom/Intelligence gated**, matching vanilla PST. The mod adds a
  real **Charisma** ladder (16–22) and a couple of unusual **exact-DEX** checks.
- Ignore `G-BBD000` — it is not an NPC. It is the mod's **standalone-campaign character
  builder** (grants millions of XP + full gear based on a "what class were you?" interview).
  It only fires in the mod's own new-game start, not in a continued PST playthrough.

---

## The headline NPCs

Ranked by best single payout. "Once" / "Repeatable" is read from the guarding global.

| NPC (dialogue) | Best payout | Requirement | Notes |
|---|---|---|---|
| **Dopilp**, kuo-toa priest (`G-BBD010`) | **+1 to all 6 stats, +20 HP, 100k XP** | Complete the "become their god" ritual | **Once** — all 3 ritual routes end in `SetGlobal("G-kuotoagod",1)`. Separately, eating the cannibal/bladeling feast pays **200k–300k XP**. |
| **Zegonz Vlaric**, crook-armed bartender (`G-BBD059`) | **120k XP + boosts Dak'kon's WIS/INT/CHR** | Dak'kon fully taught (`Dakkon_Teach=19`) **or** TNO **WIS 15+** | A Dak'kon-development conversation. WIS ≤ 14 gives a lesser 30k branch. Many Faction-flavored variants (Godsmen/Dustmen/Sensate/Anarchist) each pay 60k + a Dak'kon stat. |
| **Sauna Master** (`G-BBD157`) | **+1 CON** to TNO and each companion present, 60k XP | First sauna visit (`g-1stsauna < 1`) | **Once.** The 33 `PermanentStatChange` calls are one +1 CON fanned across every "who's in your party" combination — **not** 33 separate boosts. A separate branch buffs **Ignus** (STR/CON/DEX/INT). |
| **Ulfbrand Völgarsson** (`G-BBD156`) | **200k XP + +2 STR + 20 crush resist** | Pay him (`PartyGold ≥ 10,000`) | The "help me become stronger" branch. Also a **battleaxe** reward (100k) and two theft branches (below). |
| **Wotyrxil** (`G-BBD052`) | **160k XP** | Return Idra's / Callimarus's flute (`G-fluteIdra=1`) | Quest turn-in. A separate 50k branch needs **INT 14+**. |
| **Pale Woman / Belle** (`G-BBD050`) | **100k XP** ("Remember") | **WIS 21+** | Also: **INT 21+** *and* Mage → 60k; Kiaransalee-worship branches → 80k. A repeatable-looking 60k "I still have some questions" loop. |
| **Narthuul Hollow Maw** (`G-BBD161`) | **100k XP** | **WIS 21+ and INT 21+** | The hardest stat gate in the mod. |
| **Word Archon** (`G-BBD159`) | **64k XP** | **INT 16+** (32k tiers at INT 14/15) | A pure **INT-gated philosophy debate** — an XP dispenser scaling with Intelligence. |
| **Brill** (`G-BBD008`) | **+2 to one chosen stat**, up to 60k XP | Complete his "focus on Pain/Understanding/Power…" ritual | Wish-style pick-one (CON/INT/STR/WIS/CHR). One branch **costs −1 CON**. |
| **Fragile-Tail** (`G-BBD019`) | up to 80k XP across many branches | mix of **CHR 20+ / INT 16+ / STR 18+** | Several independent stat-gated payouts. |

Other 100k-XP quest turn-ins with no stat gate (just quest state): **Ikss'odes** (`G-BBD029`),
**Mawu** (`G-BBD042`, +2 CHR), **Æ** (`G-BBD066`, +1 INT, gear), **Raelis Shai** (`G-BBD054`),
**Beatha** (`G-BBD123`), **Elvra Syne** (`G-BBD167`), **Ayryn Farlight** (`G-BBD240`).

## Correction to the earlier delta note

[PST_STAT_PLAN_MOD_DELTA.md](PST_STAT_PLAN_MOD_DELTA.md) flagged Dopilp (`G-BBD010`) as
*"possibly +3 to all — ⚠ verify in play"*. **Resolved from source: it is +1 to all six, once.**
The three "Rise from the bloodied stone" / "What now…" routes (`F1cont`, `P1`, `W1`) are three
paths *into the same ceremony*; every one of them fires the identical grant block and sets
`G-kuotoagod=1`. There is no stacking and no 3-step escalation.

## The unusual requirements worth knowing

- **Exact-DEX theft (Ulfbrand, `G-BBD156`).** Stealing the earring uses `CheckStat` (exact
  equality), not `≥`: TNO needs **DEX exactly 25**, or signal **Annah at DEX exactly 24**.
  Annah's base DEX is 18, so this needs stacked DEX gear to hit the precise number — an easy
  branch to miss. Each pays 50k XP + the item.
- **`Dakkon_Teach=19` (Zegonz).** That global is the *fully-taught* end of Dak'kon's vanilla
  Circles-of-Zerthimon ladder (stock `DDAKKON.DLG` sets 1→3→5→7→9→11→13→16→19). So Zegonz's
  best branch rewards having finished Dak'kon's teaching — reachable in a normal game, but only
  if you actually complete the Circles.
- **WIS 21 / INT 21 (Pale Woman, Narthuul).** These sit above every vanilla PST gate; they're
  reachable only with the mod's own +3 stat gear stacked (see the delta doc's gear table).

## Requirement histogram (reward-bearing branches only)

Across the ~76 reward branches that sit behind a stat check:

| Requirement | # reward branches |
|---|---|
| INT ≥ 16 | 23 |
| WIS ≥ 15 | 10 |
| INT ≥ 19 | 4 |
| WIS ≥ 16 | 4 |
| DEX ≥ 15 | 4 |
| INT ≥ 14 | 3 |
| INT ≥ 18 | 3 |
| CHR ≥ 20 | 3 |
| INT/WIS ≥ 21 | 2 each |
| CHR 16 / DEX =24 / DEX =25 / STR ≥ 18 / … | 1–2 each |

**Takeaway:** the reward economy leans **Intelligence** (the single densest gate, INT ≥ 16),
then **Wisdom**, with **Charisma** opening a genuine high tier (16–22) the base game lacks.
This matches the whole-dialogue histogram in the delta doc, but here it's the *valuable*
branches only, not flavor lines.

## Reproducing

```bash
PST="/mnt/c/Program Files (x86)/Steam/steamapps/common/Planescape Torment Enhanced Edition"
MOD="$PST/BlizzardinBaator"

# 1. Reward actions across the mod's dialogue source
grep -ohiE '\b(AddExperienceParty[A-Za-z]*|PermanentStatChange|ChangeStat|GiveItemCreate|GivePartyGold)\b' \
  "$MOD"/d/*.d | tr a-z A-Z | sort | uniq -c | sort -rn

# 2. Map a reward dialogue G-BBDnnn back to its creature (1:1 with g-bbnnn.cre)
iecli dump --game "$PST" --resource g-bb010.CRE --format json | \
  python3 -c 'import sys,json; h=json.load(sys.stdin)["header"]; print(h["long_name"]["text"], h["dialog"])'

# 3. Confirm CheckStat vs CheckStatGT/LT semantics
iecli dump-raw --game "$PST" --resource TRIGGER.IDS --output /tmp/trigger.ids
grep -i checkstat /tmp/trigger.ids     # 0x4044 CheckStat (=), 0x4045 GT, 0x4046 LT
```

The full extractor (`parse_rewards.py`) joins multi-line `~...~` action blocks before
line-parsing — several `DO~...~` grants span physical lines, and a naive line grep silently
undercounts them (it missed 50 of 94 `PermanentStatChange` calls until fixed).

*Generated 2026-07-10 from the installed Blizzard in Baator v1.0.2 source. Companion to
[PST_STAT_PLAN_MOD_DELTA.md](PST_STAT_PLAN_MOD_DELTA.md) and
[PST_CONVERSATION_BOONS.md](PST_CONVERSATION_BOONS.md).*
