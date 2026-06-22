# PST Lawful / Chaotic (Law-axis) Interaction Ledger

Save-aware cross-reference of every dialogue choice that changes your **Law** score.
Generated from save `000000001-Quick-Save-4` — **LAW = 27**, GOOD = 61 (as of 2026-06-21).

Legend:  ✅ done (tracker set in save) · ⬜ available (not yet taken) · ❔ untracked (direct increment, can't tell)
`+N` = lawful (toward Law/Vhailor) · `-N` = chaotic · *(g±N)* = same choice also shifts Good.
Vhailor axe tiers: Law 14 → +1 · Law 25–34 → +2 · **Law 35+ → +3**.

> **Read the "available" totals as a CEILING, not a forecast.** Each number is the sum of
> unset lawful trackers, counted once per `IncrementGlobalOnceEx` tracker. Two real caveats it
> can't see: (1) **Mutual exclusivity** — within one conversation the lawful replies are usually
> alternative answers to the *same* question, so you get the best one, not their sum (e.g. Ravel's
> beauty question: +3 *or* +2, not +5). (2) **Missed windows** — an unset tracker for an early NPC
> (Mortuary/Hive) may simply be behind you and no longer reachable. Cross-check against where you are.

## 🎯 Practical path to the +3 axe (Law 35) from here (Law 27)

**Key finding from the DLG trace:** you do **not** need Law 35 when you first *meet* Vhailor — you need it at the moment of the **axe-ask**, and Vhailor's own recruitment banks lawful Law first. The axe option (`DVHAIL` ST35, *"Can you teach me how to use an axe?"*) is gated `Class(Protagonist,FIGHTER) InParty("Vhail")` — so it's only reachable **after** you've recruited him and switched to fighter, i.e. a re-engagement. The tier check (ST66: `<25 → +1` · `25–34 → +2` · `≥35 → +3`) therefore reads your Law *after* the recruitment/confession answers have already fired.

Realistic climb, all pre-axe-check:
- **Ravel (now): +5** — beauty question *"I would call you ugly"* (+3, **not** the +2 "beautiful" — same question, exclusive) + *"Your rules are fair"* (+1) + *"Only truths do I speak"* (+1). → **32**
- **Trias: +1** — *"I vow to spare your life…"* (kept). → **33**  *(the −3 "I lied" is the trap)*
- **Vhailor recruitment, truthful: +2 to +7** — join answers `Lawful_Vhailor_1/2` (+2) plus confession answers `Lawful_Vhailor_3/4/5` (+2/+2/+1) all bank before you re-ask about the axe. → **35–40**

So even the join answers alone (33 → 35) clear tier 3; the confessions are cushion. **You're fine for the +3 axe without the mod and without Coaxmetal's post-return +2/+3.**

## 🗺️ Ravel's Maze — max-payoff + max-Law walkthrough (`DRAVEL`)

**The real Ravel prize is `Ravel_Flatter`-gated, not Law-gated.** At ST223 her reward scales with a
hidden flattery counter: `Flatter <7` → WIS +1 / 90k XP · `7–10` → WIS +1 / 120k · **`≥11` → WIS +2 / 180k**.
And nearly every flatter line is gated `CheckStatGT(Protagonist,15,CHR)` = **CHA ≥ 16** — *this* is what
Charisma is for at Ravel (cast **Friends/SPWI114**, +2–8, if base CHA < 16; `CheckStat` reads the buff).

Flattery and Law only collide at one node (the beauty question), and you can win both:

**Step A — build Flatter ≥ 11 (need effective CHA 16).** Take every "beautiful Ravel" line:
| Flatter node | F | Note |
|---|--:|---|
| ST6, ST57, ST94, ST101, ST191 — ungated compliments | +5 | no gate |
| ST58 (+2), ST79, ST84/87, ST92 — CHA-16 gated | +5 | need CHA 16 |
| ST193 *"at least you tried"* — CHA-13 gated | +2 | need CHA 13 |

≈ **+12 without the beauty-lie**, clearing the ≥11 tier → **WIS +2 + 180k XP**.

**Step B — the Law answers (the beauty-lie is NOT needed):**
| # | Ravel's prompt | **Say this** | Law |
|---|---|---|--:|
| 1 | Sets her "law" (ST51) | **"Your rules are fair…"** | **+1** |
| 2 | "Only truths do I speak?" (ST59) | **Truth: "Only truths do I speak…"** | **+1** |
| 3 | Beauty question (ST145) | **Truth: "I would call you ugly."** | **+3** (+ transformation) |
| 4 | Offers to strike Grace/Annah (ST81/89) | **"I retract my words…"** | 0 (vs −3 trap) |
| 5 | "Did I do right freeing the Lady?" (ST191) | **Truth: "…you did the right thing…"** | 0 (Lie = −1) |

The flattering beauty-lie *"I would call you beautiful"* is **+1 Flatter but −2 Law** — skip it; you've
already cleared 11, so honesty ("ugly", +3 Law) is free. The CHA-16 "inner beauty" reply is the *worst*
beauty answer (+2 Law, she mocks it, no transformation).

**Net (CHA 16): WIS +2, +180k XP, and +5 Law (27 → 32).** Without CHA 16 the flatter lines lock and the
payoff caps at WIS +1 / 90k — so CHA 16 (or a Friends cast) at Ravel is the single highest-value lever here.

> Other Ravel reward branches exist by asking different things — ST179 INT +1/90k, ST181 INT +1 & WIS +1/120k,
> ST204 XP scaled by INT/Specialist. The flatter-gated WIS +2 above is the appearance-question line specifically.

## ⬜ Biggest remaining lawful sources (Law you can still gain)

| Dialogue | Available +Law | Notes |
|---|--:|---|
| Emoric  [DEMORIC] | +11 | |
| Iannis (Deionarra's father)  [DIANNIS] | +8 | |
| [DDEIONS]  "I shall wait for you in death's halls, my Love." She s | +7 | |
| Ravel Puzzlewell  [DRAVEL] | +7 | |
| Vhailor  [DVHAIL] | +7 | |
| Morte  [DMORTE] | +6 | |
| Coaxmetal  [DCOAX] | +5 | |
| Annah  [DANNAH] | +3 | |
| [DRATBONE]  This tall, spindly man keeps his eyes on the ground at  | +3 | |
| [DREEK]  This man is looking at you with a strange, bug-eyed sta | +3 | |
| [DASHMAN]  You see a man in long black robes and a pale face. He l | +2 | |
| [DAWAIT]  Before you is a young Dustman with stubble on his chin  | +2 | |
| [DBEDAI]  "Thanks for your help. Maybe I'll see you later... or s | +2 | |
| Dak'kon  [DDAKKON] | +2 | |
| Fall-from-Grace  [DGRACE] | +2 | |
| [DINCAR1]  "*Finally.* I thought I would die again waiting for him | +2 | |
| [DJOLMI]  This stern-looking older woman seems to be looking for  | +2 | |
| [DSAROSSA]  "You think to threaten me without cost to yourself? Foo | +2 | |
| [DTHILDON]  "Here now! Who's this? GUARDS! HE'S OVER HERE!" | +2 | |
| [DVAXIS]  The shambling corpse gazes at you with vacant eyes. The | +2 | |
| [D300MER5]  "Jump, Lim-Lim, jump!" The merchant seems to suddenly n | +1 | |
| [DANGYAR]  This man looks... haunted. His eyes are half-lidded, as | +1 | |
| [DBARKING]  This wild-eyed man is hunched over, snarling and giving | +1 | |
| [DCSTCSER]  You see a weary-looking, middle-aged man. When he notic | +1 | |
| [DDERAN]  You see a boisterous auctioneer. He is very animated an | +1 | |
| [DDRUNK]  This man is drunk out of his gourd. He can barely stand | +1 | |
| [DFHJULL]  "Thanks to you, all is lost to me! They shall keep comi | +1 | |
| [DHARGRI]  This skeleton wears what appear to be ancient priest's  | +1 | |
| [DHIVEF1]  You see a man in tattered clothing. As you approach him | +1 | |
| [DMARISSA]  Squinting at the figure behind the partition, you can b | +1 | |
| [DMARTA]  You see a blockish woman dressed in a heavy burlap robe | +1 | |
| [DMFTREE]  You see a tired-looking, sorrowful old man who is gazin | +1 | |
| [DNONAME]  This rotting, female corpse is shaking her head and moa | +1 | |
| [DRIDSKEL]  This skeleton is shaking its head and giggling to itsel | +1 | |
| Soego  [DSOEGO] | +1 | |
| [DTHUGOE]  You see a shady looking fellow standing before you, pic | +1 | |
| Trias  [DTRIAS] | +1 | |
| [DTRIST]  You see a young woman who is trying to get your attenti | +1 | |
| Uhir  [DUHIR] | +1 | |
| [DWANGYAR]  This woman looks to be in her middle years, and her hai | +1 | |
| [DYI'MINN]  This feral, yellow-skinned man looks you over for a mom | +1 | |

**Total lawful Law still available across all dialogues: +101**  (gross; many are mutually exclusive within a conversation).

## Full per-creature ledger

### Emoric  [DEMORIC]  —  ⬜ up to +11 available

- ⬜ `+3`  “Make Vow: "I promise to serve the Dustmen. I have left Life behind. I seek only the True Death."”  ·  (Lawful_Emoric_4)
- ⬜ `+3`  “Truth: "I think I'm immortal."”  ·  (Lawful_Emoric_6)
- ⬜ `+2`  “Truth: "They come from the catacombs beneath Sigil. He's taking the bodies you have buried and selling them back to you.”  ·  (Lawful_Emoric_1)
- ⬜ `+2`  “Truth: "Not entirely."”  ·  (Lawful_Emoric_2)
- ⬜ `+1`  “Truth: "I think I'm immortal."”  ·  (Lawful_Emoric_5)
- ⬜ `-1`  “Even though signed Mortai's contract: "I'll sign."”  *(+2 other phrasings)*  ·  (Chaotic_Emoric_3)
- ⬜ `-2`  “Lie: "A number of people in Ragpicker's Square have contracted a wasting sickness that causes their bodies to decay prem”  ·  (Chaotic_Emoric_1)
- ⬜ `-2`  “"Lie: Yes."”  ·  (Chaotic_Emoric_10)
- ✅ `-2`  “Truth: "They come from the catacombs beneath Sigil. He's taking the bodies you have buried and selling them back to you.”  ·  (Chaotic_Emoric_2)
- ⬜ `-2`  “Even though you've signed Mortai's contract, finish signing Emoric's contract anyway.”  *(+2 other phrasings)*  ·  (Chaotic_Emoric_4)
- ⬜ `-2`  “Lie: "You are mistaken. I belong to no other faction."”  ·  (Chaotic_Emoric_5)
- ⬜ `-2`  “Lie to infiltrate Dustmen for Anarchists: "If that is what must be, that is what must be. My mind must be true to itself”  ·  (Chaotic_Emoric_6)
- ⬜ `-2`  “Lie: "Yes."”  ·  (Chaotic_Emoric_7)
- ⬜ `-3`  “Lie: "I promise to serve the Dustmen. I have left Life behind. I seek only the True Death."”  ·  (Chaotic_Emoric_11)

### Iannis (Deionarra's father)  [DIANNIS]  —  ⬜ up to +8 available

- ⬜ `+3` *(g+3)*  “Truth: "I am responsible for your daughter's death. I murdered her on the Negative Material Plane in the hopes that her”  *(+1 other phrasings)*  ·  (Lawful_Iannis_6)
- ⬜ `+1` *(g+1)*  “"I see. Do I owe you anything for this information?"”  ·  (Lawful_Iannis_1)
- ⬜ `+1`  “Vow: "I promise you shall have a chance to look at them. For now, however, perhaps there is something else you can help”  *(+1 other phrasings)*  ·  (Lawful_Iannis_2)
- ⬜ `+1`  “Truth: "I think so. I can't really remember myself."”  ·  (Lawful_Iannis_3)
- ✅ `+1` *(g+1)*  “"I came to ask your forgiveness and make amends for your loss, if it is within my power."”  ·  (Lawful_Iannis_4)
- ⬜ `+1` *(g+1)*  “"I do not wish to break the Sensates' laws. Nevertheless, I could speak to someone and see if something else might be do”  ·  (Lawful_Iannis_5)
- ✅ `+1`  “Vow: "I will do that."”  ·  (Lawful_Iannis_7)
- ⬜ `+1`  “Vow: "I promise I shall allow your daughter to rest peacefully."”  ·  (Lawful_Iannis_8)
- ⬜ `-1`  “Lie: "I will see that you get a chance to read them. In the meantime, perhaps there is something else you can help me wi”  *(+1 other phrasings)*  ·  (Chaotic_Iannis_1)
- ⬜ `-1`  “Lie: "No; I was mistaken. I don't believe we've ever met."”  ·  (Chaotic_Iannis_2)
- ⬜ `-1`  “Lie: "It was a magical compulsion that forced me to behave as I did. I have no knowledge of what transpired."”  ·  (Chaotic_Iannis_3)
- ⬜ `-1` *(g+1)*  “Half-Truth: "She perished on the Negative Material Plane from an act of betrayal."”  ·  (Chaotic_Iannis_4)
- ⬜ `-1`  “Lie: "She took ill upon the journey... apparently, the amount of time in Sigil did not prepare her for the environment o”  *(+1 other phrasings)*  ·  (Chaotic_Iannis_5)
- ⬜ `-1` *(g-1)*  “Lie: "I will do that."”  ·  (Chaotic_Iannis_6)
- ⬜ `-1` *(g-1)*  “Snap his neck.”  ·  (Chaotic_Iannis_7)

### [DDEIONS]  "I shall wait for you in death's halls, my Love." She s  —  ⬜ up to +7 available

- ⬜ `+3` *(g+1)*  “Truth: "When I brought you to this Fortress, it was my intention that you die here. I needed someone to remain behind so”  ·  (Lawful_Deionarra_2)
- ✅ `+2` *(g+2)*  “Make Vow: "I swear I will find some means to save you or join you."”  ·  (Lawful_Deionarra_1)
- ⬜ `+2`  “Truth: "I am sorry, Deionarra. I do not. I never knew you. Perhaps if I had met you under different circumstances..."”  ·  (Lawful_Deionarra_5)
- ⬜ `+1` *(g+1)*  “Truth: "I am sorry, Deionarra."”  ·  (Lawful_Deionarra_3)
- ⬜ `+1` *(g+1)*  “Truth: "Though I did not know you at first, I have come to love you. Your suffering has become mine, and I have found th”  ·  (Lawful_Deionarra_4)
- ⬜ `-1` *(g-1)*  “Lie: "Yes... yes, the name *does* sound familiar."”  ·  (Chaotic_Deionarra_1)
- ⬜ `-1` *(g-1)*  “Lie: "Your name evokes passionate thoughts, though their content is unclear. Perhaps if you were to tell me more..."”  ·  (Chaotic_Deionarra_2)
- ⬜ `-1`  “Lie: "This is not the Deionarra I remember. The Deionarra I loved was kind, gentle... and never abandoned a soul in need”  *(+1 other phrasings)*  ·  (Chaotic_Deionarra_4)
- ⬜ `-1` *(g-2)*  “Lie: "I am sorry, Deionarra."”  ·  (Chaotic_Deionarra_6)
- ⬜ `-1` *(g-1)*  “Lie: "Of course I love you. Even death cannot kill the bond between us."”  ·  (Chaotic_Deionarra_7)
- ⬜ `-2` *(g-2)*  “Lie: "I swear I will find some means to save you or join you."”  ·  (Chaotic_Deionarra_3)
- ⬜ `-3` *(g-1)*  “Lie: "When you died here in the Fortress, it was because of the adversary that awaits us. He wished you to die so that w”  ·  (Chaotic_Deionarra_5)

### Ravel Puzzlewell  [DRAVEL]  —  ⬜ up to +7 available

- ⬜ `+3`  “Truth: "I would call you ugly."”  ·  (Lawful_Ravel_2)
- ⬜ `+2`  “Truth: "I find you beautiful, Ravel. Not perhaps to the eye, but your mind seems sharp and vibrant."”  ·  (Lawful_Ravel_1)
- ⬜ `+1`  “"Your rules are fair. Ask your questions, Ravel."”  ·  (Lawful_Ravel_3)
- ⬜ `+1`  “Truth: "Only truths do I speak, Ravel."”  ·  (Lawful_Ravel_4)
- ⬜ `-1` *(g-2)*  “Lie: "I... think you did the right thing trying to set her free, Ravel."”  ·  (Chaotic_Ravel_1)
- ⬜ `-1`  “Lie: "Only truths do I speak, Ravel."”  ·  (Chaotic_Ravel_3)
- ⬜ `-2`  “Lie: "I would call you beautiful, Ravel."”  ·  (Chaotic_Ravel_2)
- ⬜ `-3` *(g-3)*  “"Strike her down, Ravel - I care nothing for her. She is chattel, nothing more."”  ·  (Chaotic_Ravel_4)
- ⬜ `-3` *(g-3)*  “"Strike her down, Ravel - I care nothing for her. She is Hive gutter trash, nothing more."”  ·  (Chaotic_Ravel_5)

### Vhailor  [DVHAIL]  —  ⬜ up to +7 available

- ❔ `+2` *(g-2)*  “"Go and repent, officer Adonis."”  ·  ((direct/untracked))
- ❔ `+2` *(g-1)*  “"That's right, Fragile-Tail. Today is the day you pay for all these thefts."”  ·  ((direct/untracked))
- ❔ `+2` *(g-1)*  “Bluff: "That's what I want. No one will trade stolen goods in *my* area without *my* approval. From larvae smugglers to”  ·  ((direct/untracked))
- ❔ `+2` *(g+1)*  “"That's what I want. Please forgive me, but this is necessary."”  ·  ((direct/untracked))
- ❔ `+2`  “"That's what I want. Hard law, but law."”  ·  ((direct/untracked))
- ❔ `+2` *(g-1)*  “"That's what I want. I'll take this as your admission of guilt. Today is the day you pay for all these thefts."”  ·  ((direct/untracked))
- ❔ `+2` *(g+1)*  “"That's what I want. I'll take this as your admission of guilt. Please forgive me, but this is necessary."”  ·  ((direct/untracked))
- ❔ `+2`  “"That's what I want. I'll take this as your admission of guilt. Hard law, but law."”  ·  ((direct/untracked))
- ⬜ `+2`  “Truth: "Yes."”  *(+1 other phrasings)*  ·  (Lawful_Vhailor_3)
- ⬜ `+2`  “Truth: "Yes."”  *(+1 other phrasings)*  ·  (Lawful_Vhailor_4)
- ❔ `+1` *(g-1)*  “"That's what I want. Just my whim. Completely random."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"That's what I want. I'll take this as your admission of guilt. Just my whim. Completely random."”  ·  ((direct/untracked))
- ⬜ `+1` *(g-1)*  “Truth: "I see I have found one who believes as *I* do. Mercy subverts justice. Mercy *is* weakness."”  ·  (Lawful_Vhailor_1)
- ⬜ `+1` *(g+1)*  “Truth: "I seek justice, too, Vhailor. Will you join me?"”  *(+1 other phrasings)*  ·  (Lawful_Vhailor_2)
- ⬜ `+1`  “Truth: "Yes... many have been wronged my hand, Vhailor. But I was another person inhabiting this body."”  *(+13 other phrasings)*  ·  (Lawful_Vhailor_5)
- ❔ `-1`  “Spit toward the Mercykillers. It is a rare opportunity to do so without them being able to do anything about it.”  ·  ((direct/untracked))
- ⬜ `-1`  “Lie: "I see I have found one who believes as *I* do. Mercy subverts justice. Mercy *is* weakness."”  ·  (Chaotic_Vhailor_1)
- ⬜ `-1`  “Lie: "I seek justice, too, Vhailor. Will you join me?"”  *(+1 other phrasings)*  ·  (Chaotic_Vhailor_2)
- ⬜ `-1`  “Lie: "No."”  ·  (Chaotic_Vhailor_3)
- ⬜ `-1`  “Lie: "No."”  ·  (Chaotic_Vhailor_4)
- ⬜ `-1`  “Lie: "No, I have not. I have never harmed another in all my lifetimes."”  ·  (Chaotic_Vhailor_5)
- ⬜ `-3`  “Attack the deva, despite your vow.”  ·  (Chaotic_Trias_1)

### Morte  [DMORTE]  —  ⬜ up to +6 available

- ⬜ `+3`  “Truth: "That doesn't seem constructive to anyone. Some order, some law is always necessary for progress to be made."”  ·  (Lawful_Vaxis_3)
- ⬜ `+2`  “Truth: "That seems like a noble goal. What's wrong with trying to make the multiverse more orderly?"”  ·  (Lawful_Nordom_2)
- ✅ `+1`  “"Except being sentenced to the hells. That sounds a lot worse than telling the truth."”  ·  (Lawful_Morte_1)
- ⬜ `+1` *(g+1)*  “"Look, I'm NOT going to lie to this guy. It's better we be up front with him."”  ·  (Lawful_Vaxis_2)
- ✅ `+1`  “Truth: "If you can prove to me the skull's yours, then I'll give it to you. It's only fair."”  ·  (Lawful_Yellow_1)
- ❔ `-1`  “Lie: "Sure, old pal. It's been some time."”  ·  ((direct/untracked))
- ❔ `-1`  “"Well, it was nice chatting, but it's time for me to go."”  ·  ((direct/untracked))
- ⬜ `-1`  “"What's that, cube hero? 'Morte's a stupid skull'? Why, yes he is, isn't he, cube hero?"”  ·  (Chaotic_Cube_3)
- ⬜ `-1`  “"Yes, it did! It said it just now!"”  ·  (Chaotic_Cube_4)
- ⬜ `-1`  “"No, it's mine. He only wants to hang out with me anyway. Don't you, cube hero? Yes, you do!"”  ·  (Chaotic_Cube_5)
- ⬜ `-1` *(g-1)*  “Grab Morte, shove the teeth in his mouth.”  ·  (Chaotic_Ingress_Teeth_2)
- ⬜ `-1` *(g-1)*  “"Well, that's true... but you've got to choose your lies carefully."”  ·  (Chaotic_Morte_1)
- ✅ `-1`  “"That was kind of the *idea,* Morte."”  ·  (Chaotic_Nordom_5)
- ⬜ `-1`  “"Let me tell you, Morte, there's nothing more satisfying than walking around, swinging your arms, breathing the crisp ai”  ·  (Chaotic_Skeleton_Mort_1)
- ⬜ `-1`  “Lie: "That seems like a noble pursuit. Any Anarchist dedicated to such a lofty goal is *definitely* a friend of mine."”  ·  (Chaotic_Vaxis_4)
- ⬜ `-1`  “"Wanted you to go *away,* maybe. She was obviously too distracted by ME to pay attention to some stupid bobbing head wit”  ·  (Morte_Zombie_1)
- ⬜ `-2`  “Truth: "I agree; chaos has its place... too much law, and we'd all stagnate. Look, I had some other questions for Nordom”  ·  (Chaotic_Nordom_4)
- ❔ `-3` *(g-3)*  “"Hey, Ignus, don't you want to turn up the heat even more? It's a bit cold in here."”  ·  ((direct/untracked))
- ⬜ `-3`  “Truth: "That seems like a noble pursuit. Order could use a good tearing down every once in a while."”  ·  (Chaotic_Vaxis_3)

### Coaxmetal  [DCOAX]  —  ⬜ up to +5 available

- ⬜ `+3` *(g+3)*  “"There is nothing more to be said. I will not free you; the planes do not need the chaos and suffering you will bring. F”  *(+2 other phrasings)*  ·  (Lawful_Coaxmetal_1)
- ⬜ `+2`  “"I disagree."”  ·  (Lawful_Coaxmetal_3)
- ✅ `+1`  “Truth: "Except the death you seem to offer is violent and senseless - without meaning."”  ·  (Lawful_Coaxmetal_2)
- ⬜ `-1`  “Truth: "True enough; so what made you take up entropy's cause?"”  ·  (Chaotic_Coaxmetal_2)
- ⬜ `-2`  “"Hmmm. I see your point... how can I free you?"”  ·  (Chaotic_Coaxmetal_3)
- ⬜ `-5` *(g-5)*  “"All right, here you are."”  ·  (Chaotic_Coaxmetal_1)

### Annah  [DANNAH]  —  ⬜ up to +3 available

- ✅ `+1`  “Truth: "No. I've no cause to do such a thing. Pharod's done me no harm."”  ·  (Lawful_Annah_1)
- ⬜ `+1`  “"I won't let you hurt her unless you can prove she's done something wrong."”  ·  (Lawful_Drunk_Harlot_1)
- ✅ `+1`  “Truth: "I... don't. In fact, ever since I woke up in the Mortuary, I feel like something's... missing. Something inside.”  ·  (Lawful_Fell_1)
- ✅ `+1`  “Promise: "Annah, no harm will come to you while I'm here - I promise. I just want to speak to him for a moment."”  ·  (Lawful_Fell_2)
- ⬜ `+1`  “"It matters to me. If that money isn't ours..."”  ·  (Lawful_Fleece_3)
- ⬜ `+1`  “"Can you prove it?"”  ·  (Lawful_Fleece_4)
- ❔ `-1` *(g-1)*  “"Zack, it's obvious she wants to strip you of your true nature. Will you really let yourself be castrated like that in t”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Sonne, can't you see that he's trying to take advantage of you? To make a harlot out of you? He wants to defile your pu”  ·  ((direct/untracked))
- ⬜ `-1`  “Bluff: "Enough of your lies. Tell me where Pharod is, girl, or you'll soon number amongst the city's dead."”  ·  (Chaotic_Annah_1)
- ⬜ `-1`  “Lie: "No. I've no cause to do such a thing."”  ·  (Chaotic_Annah_2)
- ⬜ `-1`  “Lie: "I mean, have you ever *looked* at yourself? Aside from the way you carry yourself, you're confident, sensible, and”  ·  (Chaotic_Annah_3)
- ⬜ `-1`  “Lie: "Annah, Fall-from-Grace doesn't mean as much to me as you do. Will you please travel with us again?"”  ·  (Chaotic_Annah_5)
- ⬜ `-1`  “Lie: "Annah, look, I don't *care* about her. I *need* you to travel with me, but I need her help, too, for a time. After”  ·  (Chaotic_Annah_6)
- ⬜ `-1`  “Lie: "I feel complete, yes. Why?"”  ·  (Chaotic_Fell_1)
- ⬜ `-1`  “Lie to silence her crying: "Annah, no harm will come to you while I'm here - I promise. I just want to speak to him for”  ·  (Chaotic_Fell_2)
- ⬜ `-1`  “"'Trimmin' my stem'? Do I have to pay for that?"”  ·  (Chaotic_Harlot_1)
- ⬜ `-1` *(g-1)*  “"Heh. Nice work."”  ·  (Chaotic_Harlot_2)
- ❔ `-2`  “"Kick their asses, officer Adonis."”  ·  ((direct/untracked))
- ❔ `-3` *(g-3)*  “"Hey, Ignus, don't you want to turn up the heat even more? It's a bit cold in here."”  ·  ((direct/untracked))

### [DRATBONE]  This tall, spindly man keeps his eyes on the ground at   —  ⬜ up to +3 available

- ⬜ `+1`  “Vow: "I won't try to harm you; you have my word."”  ·  (Lawful_Rbone_1)
- ⬜ `+1`  “"I... don't know."”  *(+1 other phrasings)*  ·  (Lawful_Rbone_2)
- ⬜ `+1`  “Vow: "I swear I won't mention your name."”  ·  (Lawful_Rbone_3)
- ⬜ `-1`  “Lie: "I won't try to harm you; you have my word."”  ·  (Chaotic_Rbone_1)
- ⬜ `-1`  “Lie: "I'm, uh... Adahn."”  *(+1 other phrasings)*  ·  (Chaotic_Rbone_2)
- ⬜ `-1`  “Lie: "I'm wondering if I could help."”  ·  (Chaotic_Rbone_3)
- ⬜ `-1`  “Lie: "He's a friend of mine."”  *(+1 other phrasings)*  ·  (Chaotic_Rbone_4)
- ⬜ `-1`  “Lie: "I swear I won't mention your name."”  ·  (Chaotic_Rbone_5)
- ⬜ `-1`  “Bluff: "Tell me what you know, or you'll share his fate."”  ·  (Chaotic_Rbone_6)
- ⬜ `-1` *(g-1)*  “True: "Tell me what you know, or you'll share his fate."”  *(+1 other phrasings)*  ·  (Chaotic_Rbone_7)

### [DREEK]  This man is looking at you with a strange, bug-eyed sta  —  ⬜ up to +3 available

- ⬜ `+2`  “Truth: "It must have an answer, and every tale has an ending. I will refuse to accept it any other way."”  ·  (Lawful_Reekwind_2)
- ⬜ `+1`  “Truth: "I don't have that much with me. I had some questions, though..."”  ·  (Lawful_Reekwind_1)
- ⬜ `-1`  “Lie: "I don't have that much with me. I had some questions, though..."”  ·  (Chaotic_Reekwind_1)
- ⬜ `-2`  “Truth: "Perhaps. What happens will happen, and it may be that this tale has no answer."”  ·  (Chaotic_Reekwind_2)

### [DASHMAN]  You see a man in long black robes and a pale face. He l  —  ⬜ up to +2 available

- ⬜ `+2`  “Vow: "You have my word that I will let you go... if you answer my question."”  *(+3 other phrasings)*  ·  (Lawful_Ash_1)
- ⬜ `-1`  “Bluff: "Give me everything you have, or I'll kill you."”  *(+2 other phrasings)*  ·  (Chaotic_Ash_1)
- ⬜ `-1`  “"No. I had some other questions for you..."”  ·  (Chaotic_Ash_2)
- ⬜ `-1` *(g-1)*  “Lie: "Er... yeah, I've heard of it. I'd be willing to help you... for a price."”  *(+1 other phrasings)*  ·  (Chaotic_Ash_3)
- ⬜ `-1` *(g-1)*  “Lie: "No, I'm sorry, I don't know where that is."”  ·  (Chaotic_Ash_4)
- ⬜ `-1`  “Bluff: "I'll let you *live* if you tell me."”  ·  (Chaotic_Ash_5)
- ⬜ `-3` *(g-1)*  “"No. I had some other questions for you..."”  ·  (Chaotic_Ash_6)

### [DAWAIT]  Before you is a young Dustman with stubble on his chin   —  ⬜ up to +2 available

- ⬜ `+1`  “"I didn't fulfill my terms of the agreement, so keep your money."”  ·  (Lawful_Await_1)
- ⬜ `+1`  “Truth: "I was lying when I told you I wanted to die. Who does? Do you?"”  ·  (Lawful_Await_2)
- ⬜ `-1`  “Truth: "Well, considering that death and I aren't on speaking terms anymore, I considered it a no-lose proposition."”  ·  (Chaotic_Await_1)
- ⬜ `-1`  “Lie: "Yes. Do you?"”  ·  (Chaotic_Await_3)
- ⬜ `-1` *(g-1)*  “Take the money and leave quietly.”  ·  (Chaotic_Await_4)
- ⬜ `-1` *(g-1)*  “Snap his neck.”  ·  (Chaotic_Await_5)
- ⬜ `-1`  “"I'm keeping the money, as the price for this lesson."”  ·  (Chaotic_Await_6)
- ⬜ `-2`  “Truth: "I was lying when I told you I wanted to die. Who does? Do you?"”  ·  (Chaotic_Await_7)

### [DBEDAI]  "Thanks for your help. Maybe I'll see you later... or s  —  ⬜ up to +2 available

- ⬜ `+1` *(g+1)*  “"No, I will not."”  ·  (Lawful_Bedai_1)
- ⬜ `+1` *(g+1)*  “"No. I will not do it."”  ·  (Lawful_Bedai_2)
- ⬜ `-1` *(g-1)*  “"Yes, I'll do it."”  ·  (Chaos_Bedai_1)
- ⬜ `-1` *(g-1)*  “"I'll do it."”  ·  (Chaos_Bedai_2)

### Dak'kon  [DDAKKON]  —  ⬜ up to +2 available

- ✅ `+3`  “Vow: "I swear I will find one, Dak'kon. I will find one that sets you free."”  ·  (Lawful_Dakkon_2)
- ⬜ `+1`  “Truth: "What if the city is *not* flawed, and you just do not know the reasons for its contradictions? There is order in”  ·  (Lawful_Dakkon_1)
- ✅ `+1`  “Truth: "I... don't. In fact, ever since I woke up in the Mortuary, I feel like something's... missing. Something inside.”  ·  (Lawful_Fell_1)
- ⬜ `+1` *(g+1)*  “"If I gave offense, I apologize. Do you forgive me?"”  ·  (Lawful_Githzerai_1)
- ✅ `-1`  “Truth: "What if the city is *not* flawed? A thing does not need to be ordered and have a purpose to know itself. What if”  ·  (Chaotic_Dakkon_1)
- ⬜ `-1`  “Lie: "I feel complete, yes. Why?"”  ·  (Chaotic_Fell_1)
- ⬜ `-1` *(g-1)*  “"Oh, yes it is. I couldn't stand here and listen to you gith dogs bark at each other for another moment."”  ·  (Chaotic_Githzerai_4)
- ❔ `-3` *(g-3)*  “"Hey, Ignus, don't you want to turn up the heat even more? It's a bit cold in here."”  ·  ((direct/untracked))
- ❔ `-3` *(g-3)*  “(His stoic calmness irritates you. Attack him.)”  ·  ((direct/untracked))

### Fall-from-Grace  [DGRACE]  —  ⬜ up to +2 available

- ❔ `+2` *(g+2)*  “"Go and get them, officer Adonis."”  ·  ((direct/untracked))
- ✅ `+1`  “Truth: "I... don't. In fact, ever since I woke up in the Mortuary, I feel like something's... missing. Something inside.”  ·  (Lawful_Fell_1)
- ⬜ `+1`  “Truth: "I only kissed Ravel because Ravel didn't think I would, that's why - I suspected she was testing me."”  *(+2 other phrasings)*  ·  (Lawful_Grace_1)
- ⬜ `+1`  “Truth: "Well, I kissed Ravel because I wanted to kiss you."”  ·  (Lawful_Grace_2)
- ⬜ `-1`  “Lie: "I feel complete, yes. Why?"”  ·  (Chaotic_Fell_1)
- ⬜ `-1`  “Flattering Lie: "...and I admire that about you. Not only the strength, but the ability to see such horrors as a way of”  ·  (Chaotic_Grace_1)
- ⬜ `-1`  “Lie: "I only kissed Ravel because Ravel didn't think I would, that's why - I suspected she was testing me."”  *(+2 other phrasings)*  ·  (Chaotic_Grace_2)
- ⬜ `-1`  “Flattering Lie: "Well, I kissed Ravel because I wanted to kiss you."”  ·  (Chaotic_Grace_3)
- ❔ `-2` *(g-2)*  “"That tasted sweet."”  ·  ((direct/untracked))
- ❔ `-2` *(g+2)*  “"Go and get them, ex-officer Adonis."”  ·  ((direct/untracked))

### [DINCAR1]  "*Finally.* I thought I would die again waiting for him  —  ⬜ up to +2 available

- ⬜ `+1`  “"Actually, he ended up tricking *me* into finding it for him. Why was it so important?"”  ·  (Lawful_Practical_1)
- ⬜ `+1`  “"Yes, I do. I brought it with me."”  ·  (Lawful_Practical_2)
- ⬜ `-1`  “"Uh, yeah... very true. I'm sure one of Pharod's men, uh, found it for him. Why was it so important?"”  ·  (Chaotic_Practical_1)
- ⬜ `-1`  “Lie: "No, it died with Pharod. Out of curiosity, what did it do?"”  ·  (Chaotic_Practical_2)
- ⬜ `-1`  “Bluff: "Very well... I shall surrender my will."”  *(+4 other phrasings)*  ·  (Chaotic_Practical_3)

### [DJOLMI]  This stern-looking older woman seems to be looking for   —  ⬜ up to +2 available

- ⬜ `+2`  “"Yes, I am."”  ·  (Lawful_Jolmi_1)
- ⬜ `-1`  “"No, I'm not."”  ·  (Chaotic_Jolmi_1)

### [DSAROSSA]  "You think to threaten me without cost to yourself? Foo  —  ⬜ up to +2 available

- ⬜ `+1`  “"No."”  ·  (Lawful_Sarossa_1)
- ⬜ `+1`  “"I cannot set aside my belief in the law for the benefit of a single individual."”  ·  (Lawful_Sarossa_2)
- ⬜ `-1`  “Truth: "Yes."”  ·  (Chaotic_Sarossa_1)
- ⬜ `-1`  “"If I had known he was your brother, I would not have implicated him."”  ·  (Chaotic_Sarossa_2)

### [DTHILDON]  "Here now! Who's this? GUARDS! HE'S OVER HERE!"  —  ⬜ up to +2 available

- ⬜ `+1`  “True: "If that were true, I'd see to it that he was brought to justice."”  ·  (Lawful_Thildon_1)
- ⬜ `+1`  “"No. Remain here or suffer even worse consequences. You can rest assured that I can make certain of that. Good day, Thil”  ·  (Lawful_Thildon_2)
- ⬜ `-1`  “Lie: "If that were true, I'd see to it that he was brought to justice."”  ·  (Chaotic_Thildon_1)
- ⬜ `-1`  “"I'm just having a little joke with you, Thildon. Relax. Answer some other questions for me."”  *(+1 other phrasings)*  ·  (Chaotic_Thildon_2)
- ⬜ `-1` *(g-1)*  “"Sure. Get going."”  ·  (Chaotic_Thildon_3)
- ⬜ `-1`  “"Why don't you give it to me and you can walk away a healthy man?"”  ·  (Chaotic_Thildon_4)

### [DVAXIS]  The shambling corpse gazes at you with vacant eyes. The  —  ⬜ up to +2 available

- ⬜ `+1` *(g+1)*  “Truth: "I promise I won't say anything to the Dustmen about you."”  ·  (Lawful_Vaxis_1)
- ⬜ `+1` *(g+1)*  “"I know it's hard to believe, but I'm not going to lie to you: I woke up from the dead on one of the slabs upstairs."”  *(+1 other phrasings)*  ·  (Lawful_Vaxis_2)
- ⬜ `-1`  “Lie: "Why... I was looking for you."”  *(+5 other phrasings)*  ·  (Chaotic_Vaxis_1)
- ⬜ `-1`  “Lie: "Yes, I'm spying on the... Dusties."”  ·  (Chaotic_Vaxis_2)
- ⬜ `-1`  “Lie: "Yes, I have a message for you."”  ·  (Chaotic_Vaxis_5)
- ⬜ `-1`  “"This Dustman woman sounds really attractive. Are you sure you don't want me to introduce the two of you?"”  ·  (Chaotic_Vaxis_6)

### [D300MER5]  "Jump, Lim-Lim, jump!" The merchant seems to suddenly n  —  ⬜ up to +1 available

- ⬜ `+1`  “"I can and I will; it's only just. You had told me it would follow me everywhere."”  ·  (Lawful_LL_1)
- ⬜ `-1`  “Bluff: "It's either me or the hells to pay; make your choice."”  ·  (Chaotic_LL_1)

### [DANGYAR]  This man looks... haunted. His eyes are half-lidded, as  —  ⬜ up to +1 available

- ⬜ `+1`  “"You have a reason to be angry. She should not have made the promise if she had no intention of keeping it. I cannot res”  ·  (Lawful_Angyar_1)
- ⬜ `-1` *(g-1)*  “"That's your own business, but let's settle matters between us first: What're you going to give me for this contract?"”  *(+1 other phrasings)*  ·  (Chaotic_Angyar_2)
- ⬜ `-1` *(g-1)*  “"She broke her promise because she loved you. Laws and vows sometimes have to give way before more important matters."”  *(+1 other phrasings)*  ·  (Chaotic_Angyar_3)
- ⬜ `-2`  “"I spoke to your wife. I think I can help you with..."”  *(+1 other phrasings)*  ·  (Chaotic_Angyar_1)
- ⬜ `-2`  “"You have a reason to be angry. She should not have made the promise if she had no intention of keeping it. I cannot res”  ·  (Chaotic_Angyar_4)

### [DBARKING]  This wild-eyed man is hunched over, snarling and giving  —  ⬜ up to +1 available

- ⬜ `+1`  “"Well, I don't think it's fair that I've become a factol and no one even asked me. There should be some rule about this.”  ·  (Lawful_Wilder_1)
- ⬜ `-1`  “Growl at him.”  ·  (Chaotic_Wilder_1)
- ⬜ `-1`  “Lie to infiltrate Chaosmen for Anarchists: "Join ready the Xaositects am I. Loyalty none faction old I have."”  ·  (Chaotic_Wilder_2)
- ✅ `-1`  “"Moon - moons - snoom - noom?"”  *(+2 other phrasings)*  ·  (Chaotic_Wilder_3)
- ⬜ `-3`  “Smile, narrow your eyes, and say: "Ex-act-ly."”  ·  (Chaotic_Wilder_4)

### [DCSTCSER]  You see a weary-looking, middle-aged man. When he notic  —  ⬜ up to +1 available

- ⬜ `+1`  “"Good. You know your place and should maintain it."”  ·  (Lawful_Servant_1)
- ⬜ `-1`  “"Well, you're not my servant so I would have you look at me as an equal."”  ·  (Chaotic_Servant_1)

### [DDERAN]  You see a boisterous auctioneer. He is very animated an  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "It sounds like a good way to pay their debt to society."”  ·  (Lawful_Deran_1)
- ✅ `-1`  “Truth: "It still sounds like slavery to me."”  ·  (Chaotic_Deran_1)

### [DDRUNK]  This man is drunk out of his gourd. He can barely stand  —  ⬜ up to +1 available

- ⬜ `+1`  “"Annah... don't."”  ·  (Lawful_Drunk_1)
- ⬜ `-1`  “Leave him be.”  ·  (Chaotic_Drunk_1)

### [DFHJULL]  "Thanks to you, all is lost to me! They shall keep comi  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "I swear."”  ·  (Lawful_Fhjull_1)
- ⬜ `-1`  “Lie: "I swear."”  ·  (Chaotic_Fhjull_1)

### [DHARGRI]  This skeleton wears what appear to be ancient priest's   —  ⬜ up to +1 available

- ⬜ `+1`  “Offer a vow: "I only wish to leave, Hargrimm. Grant me this, and you shall have my silence."”  *(+1 other phrasings)*  ·  (Lawful_Hargrimm_1)
- ⬜ `-1`  “Bluff: "Let me see him, or I'll shatter the lot of you into dusty shards."”  ·  (Chaotic_Hargrimm_1)
- ⬜ `-1`  “Lie: "No reason."”  *(+1 other phrasings)*  ·  (Chaotic_Hargrimm_3)
- ⬜ `-1`  “Lie: "No."”  ·  (Chaotic_Hargrimm_4)
- ⬜ `-1`  “Lie: "To negotiate a truce."”  *(+1 other phrasings)*  ·  (Chaotic_Hargrimm_5)
- ⬜ `-1`  “Offer a false promise: "I only wish to leave, Hargrimm. Grant me this, and you shall have my silence."”  *(+2 other phrasings)*  ·  (Chaotic_Hargrimm_6)
- ⬜ `-2`  “"I could tell the Dustmen that Pharod is selling them corpses he stole here. I'm certain they'd stop Pharod."”  ·  (CHaotic_Hargrimm_2)

### [DHIVEF1]  You see a man in tattered clothing. As you approach him  —  ⬜ up to +1 available

- ⬜ `+1` *(g+1)*  “"Fair enough." Give him a copper piece.”  *(+2 other phrasings)*  ·  (Lawful_Hive_Fright_1)
- ⬜ `-1` *(g-2)*  “"I'll spare your life. How's that?"”  ·  (Chaotic_Hive_Fright_1)
- ⬜ `-1` *(g-1)*  “Lie: "I'm sorry, I don't have any money to spare."”  ·  (Chaotic_Hive_Fright_2)
- ⬜ `-1` *(g-1)*  “Attack him.”  ·  (Chaotic_Hive_Fright_3)

### [DMARISSA]  Squinting at the figure behind the partition, you can b  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "I'm tall, muscular, and horribly scarred."”  ·  (Lawful_Marissa_1)
- ⬜ `-1`  “Lie: "I'm tall, with an athletic build... very handsome..."”  ·  (Chaotic_Marissa_1)

### [DMARTA]  You see a blockish woman dressed in a heavy burlap robe  —  ⬜ up to +1 available

- ⬜ `+1` *(g+1)*  “"No, I'm not here to pick them up, but I'll buy some from you."”  *(+1 other phrasings)*  ·  (Lawful_Marta_1)
- ✅ `-1`  “Throw Voice: "*You* are the one who should be shamed. Knocking out my teeth like that... have you no respect for the dea”  *(+1 other phrasings)*  ·  (Chaotic_Marta_1)
- ⬜ `-1`  “Throw Voice: "Ooooooh... you will be *punished* for your evil teeth-taking, Marta. I shall *haunt* you until the end of”  ·  (Chaotic_Marta_2)
- ⬜ `-1`  “Throw Voice: "Ooooooh... stop... stop... I will haunt you no longer... forgive me..."”  ·  (Chaotic_Marta_3)
- ⬜ `-1`  “Lie: "Yeah, I'm here to pick them up."”  ·  (Chaotic_Marta_4)

### [DMFTREE]  You see a tired-looking, sorrowful old man who is gazin  —  ⬜ up to +1 available

- ⬜ `+1`  “"What? That makes no sense."”  ·  (Lawful_Mourns_1)
- ⬜ `-1`  “"Does it matter?"”  ·  (Chaotic_Mourns_1)
- ⬜ `-1`  “"No. All things die, in time; why concern yourself with the when and how of it?"”  ·  (Chaotic_Mourns_2)
- ⬜ `-1`  “Lie: "Yes, I'll help you."”  ·  (Chaotic_Mourns_3)
- ⬜ `-1` *(g-1)*  “Look at the trees, imagine them dead.”  ·  (Chaotic_Mourns_4)
- ⬜ `-1`  “Lie: "I regret my rudeness, earlier... I wanted to apologize."”  ·  (Chaotic_Mourns_5)

### [DNONAME]  This rotting, female corpse is shaking her head and moa  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "I found the tomb, but the name was... ruined. Illegible."”  ·  (Lawful_NoName_1)
- ⬜ `-1`  “Lie: "I do feel whole, name or no."”  ·  (Chaotic_NoName_1)
- ⬜ `-1`  “Lie: "Yes, your name is... uh..."”  ·  (Chaotic_NoName_2)

### [DRIDSKEL]  This skeleton is shaking its head and giggling to itsel  —  ⬜ up to +1 available

- ⬜ `+1`  “"Sure. I promise not to tell. Farewell."”  ·  (Lawful_Riddling_1)
- ⬜ `-1`  “Bluff: "Tell me, or I'll break you into tiny, giggling bits."”  ·  (Chaotic_Riddling_1)

### Soego  [DSOEGO]  —  ⬜ up to +1 available

- ⬜ `+1`  “"Yes. I know this is hard to believe, but it's the truth: I woke up on one of your slabs upstairs."”  ·  (Lawful_Soego_1)
- ⬜ `-1`  “"No. No, I didn't."”  ·  (Chaotic_Soego_1)
- ⬜ `-1`  “"I needed a change."”  ·  (Chaotic_Soego_2)
- ⬜ `-1` *(g-1)*  “Lie: "What are you talking about?! I had nothing to do with it!"”  ·  (Chaotic_Soego_3)
- ⬜ `-3` *(g-1)*  “"The Silent King is dead, and has been for some time. There *is* no Silent King."”  ·  (Chaotic_Hargrimm_7)

### [DTHUGOE]  You see a shady looking fellow standing before you, pic  —  ⬜ up to +1 available

- ⬜ `+1`  “"Give me what you stole from him, and I'll see to it that he goes away."”  ·  (Lawful_OE_1)
- ⬜ `-1`  “Lie: "I have powers over the dead. All I need to quiet the spirit is an item that belonged to it in life."”  ·  (Chaotic_OE_1)

### Trias  [DTRIAS]  —  ⬜ up to +1 available

- ⬜ `+1`  “"I vow to spare your life if you give me the knowledge I seek."”  *(+1 other phrasings)*  ·  (Lawful_Trias_1)
- ⬜ `-3` *(g-1)*  “"I lied."”  *(+1 other phrasings)*  ·  (Chaotic_Trias_1)

### [DTRIST]  You see a young woman who is trying to get your attenti  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "You've been found guilty in a court of law and only the law can help you."”  ·  (Lawful_Trist_1)
- ⬜ `-1`  “"What's in it for me if I help you?"”  ·  (Chaotic_Trist_1)

### Uhir  [DUHIR]  —  ⬜ up to +1 available

- ⬜ `+1` *(g+1)*  “"Here, take it. It's rightfully yours."”  ·  (Lawful_Uhir_1)
- ⬜ `-1`  “Lie: "No... I haven't. Farewell."”  ·  (Chaotic_Uhir_1)

### [DWANGYAR]  This woman looks to be in her middle years, and her hai  —  ⬜ up to +1 available

- ✅ `+1`  “Vow: "I swear I will tell no one."”  *(+2 other phrasings)*  ·  (Lawful_Wife_of_Angyar_1)
- ⬜ `+1`  “"You shouldn't have told me... promises are meant to be kept, or else there can be no trust between two people. It would”  ·  (Lawful_Wife_of_Angyar_2)
- ⬜ `-1` *(g-1)*  “"Call your husband, and you'll both die. Now answer my questions."”  ·  (Chaotic_Wife_of_Angyar_1)
- ⬜ `-1` *(g-1)*  “Lie: "I swear I will tell no one."”  *(+2 other phrasings)*  ·  (Chaotic_Wife_of_Angyar_2)
- ⬜ `-1`  “"A promise accompanied by a threat is no promise at all. I would not be troubled by breaking it."”  *(+1 other phrasings)*  ·  (Chaotic_Wife_of_Angyar_3)

### [DYI'MINN]  This feral, yellow-skinned man looks you over for a mom  —  ⬜ up to +1 available

- ⬜ `+1`  “Truth: "I've known some githzerai. Nice people."”  *(+1 other phrasings)*  ·  (Lawful_Yiminn_1)
- ⬜ `-1`  “Lie: "I've known some githzerai. Nice people."”  *(+1 other phrasings)*  ·  (Chaotic_Yiminn_1)

### [CUBE]  This small metal toy is a replica of a cube-like creatu

- ✅ `-1`  “Move the arms and make sword-fighting noises.”  *(+1 other phrasings)*  ·  (Chaotic_Cube_1)
- ⬜ `-1`  “Wave its arms and make cheering noises.”  ·  (Chaotic_Cube_2)

### [DABLE]  This older man looks somewhat bookish. His clothing and

- ⬜ `-1`  “"Well I, for one, plan on discovering the secrets of the multiverse by rubbing cottage cheese on my belly and eating vas”  ·  (Chaotic_Able_1)
- ⬜ `-1`  “Lie: "Not a... what? I don't know what you're talking about."”  ·  (Chaotic_Able_2)

### [DACASTE]  The rot-stink of this ancient-looking ghoul-woman is na

- ⬜ `-1`  “Lie: "I am... Adahn."”  ·  (Chaotic_Acaste_1)
- ⬜ `-3` *(g-1)*  “"Did you know the Silent King is nothing but a skeleton?"”  ·  (Chaotic_Hargrimm_7)

### Adahn/Adabus  [DADABUS]

- ✅ `+1`  “Truth: "It lies in the abandoned building not far from here. It looks as though it got trapped inside somehow."”  ·  (Lawful_Adabus_1)
- ⬜ `-1` *(g-1)*  “Lie: "It lies in the abandoned building not far from here. I don't know how it died."”  ·  (Chaotic_Adabus_1)

### [DADYZOEL]  This slightly effeminate, well-groomed young man is wat

- ⬜ `-1`  “Attack him while he's off guard.”  ·  (Chaotic_Adyzoel_1)
- ⬜ `-1`  “Bluff: "I'll give you one last chance to talk, before I break you and your pretty sword in half."”  ·  (Chaotic_Adyzoel_2)

### [DAETHEL]  You see a scaled fiend who looks very similar to the on

- ⬜ `-1`  “Lie: "I could not find the erinyes; she was not in the alley you told me about."”  ·  (Chaotic_Abishai_1)

### [DALAIS]  The prime looks at you disgustedly. "What do YOU want?"

- ✅ `+1`  “Truth: "No."”  ·  (Law_Alais_One)
- ⬜ `-1`  “Lie: "Yes."”  ·  (Chaos_Alais_One)

### [DAMARYS]  You see a young woman dressed in a tight leather bodice

- ⬜ `-1`  “Lie: "Yes, yes I am. Why?"”  ·  (Chaotic_Amarysse_1)

### [DART01]  The hundreds of chips of translucent green glass which 

- ✅ `-1`  “Punch the glass.”  ·  (Chaotic_Sglass_1)

### [DBARIA]  Much as this creature's body is a combination of man an

- ⬜ `-1`  “"If you're looking for a fight, then you've found it..."”  ·  (Chaotic_BariA_1)
- ⬜ `-1`  “"Because I *feel* like it..."”  *(+1 other phrasings)*  ·  (Chaotic_BariA_2)
- ⬜ `-1`  “"Oh, I can; watch: Yah, yah, yah! Bork, bork, bork!"”  ·  (Chaotic_BariA_3)

### [DBARR]  The guard looks at you with mingled fear and loathing, 

- ⬜ `-1`  “Give forty coins: "Here you go. You'll find it's all there."”  *(+2 other phrasings)*  ·  (Chaotic_Barr_1)

### [DBROCUS6]  This strange, cubic creature seems to be as much machin

- ⬜ `-1`  “Lie: "I'm... Adahn."”  ·  (Chaotic_Brocust6_1)

### [DCOLSRGR]  Tall and lanky, this pale, grim-looking man exudes auth

- ⬜ `-1`  “Lie: "Pharod's simply lucky in finding dead bodies. That's it."”  ·  (Chaotic_Sharegrave_1)

### [DCOLYLFN]  This weaselly-looking fellow is skulking about the garb

- ✅ `+1`  “Truth: "If you can prove to me the skull's yours, then I'll give it to you. It's only fair."”  ·  (Lawful_Yellow_1)
- ⬜ `-1`  “Bluff: "You were robbing me. I'm going to kill you."”  ·  (Chaotic_Yellow_1)
- ⬜ `-1`  “Bluff: "I think you're lying. Spill the chant, or I'll kill you."”  ·  (Chaotic_Yellow_2)

### [DCRADDO]  You see a huge man, watching the area with a tight-lipp

- ✅ `+1`  “Truth: "He said to 'pike off,' that you were a 'cur of the lowest sort,' and that 'he wasn't going to work for you any l”  ·  (Lawful_Craddock_1)
- ⬜ `-1`  “Lie: "He said... uh, that I should fill in for him."”  ·  (Chaotic_Craddock_1)

### [DCRIER]  Streams of tears have carved channels down this man's d

- ⬜ `-1` *(g-1)*  “"The copper is all I wished for - there will be no tombstone for Es-Annon. Farewell."”  ·  (Chaotic_Crier_1)

### [DCRSTGRD]  You see one of Curst's guards. He is bloodied and panti

- ⬜ `-3`  “"I just want to go to prison."”  *(+1 other phrasings)*  ·  (Chaotic_CurstGuard_1)

### [DCSTNEP]  You see a handsome young man wearing fancy clothes. He 

- ⬜ `-1`  “Tapping him on the shoulder: "I've had just about enough of your attitude. Answer some questions or I'll bust your prett”  ·  (Chaotic_Nephew_1)

### [DCUBE]  This small metal toy is a replica of a cube-like creatu

- ✅ `-1`  “Move the arms and make sword-fighting noises.”  *(+1 other phrasings)*  ·  (Chaotic_Cube_1)
- ⬜ `-1`  “Wave its arms and make cheering noises.”  ·  (Chaotic_Cube_2)

### [DCWACTF]  This attractive young woman is closely watching a pair 

- ⬜ `-1`  “"Fine, call them."”  ·  (Chaotic_Audience_1)

### [DCWACTM]  This gentleman is closely watching a pair of performing

- ⬜ `-1`  “"Fine, call them."”  ·  (Chaotic_Audience_1)

### [DCWCAFEF]  This finely attired woman is relaxing here, enjoying th

- ✅ `-1`  “"You mean I look dead, right? Well I am, you know. Dead!"”  ·  (Chaotic_CWCitizen_1)
- ✅ `-1`  “"I said, 'I'm dead!' Dead, dead, la-la! Dead, dead, la-la..."”  ·  (Chaotic_CWCitizen_2)

### [DCWCAFEM]  This finely dressed, well-groomed man is relaxing here,

- ✅ `-1`  “"You mean I look dead, right? Well I am, you know. Dead!"”  ·  (Chaotic_CWCitizen_1)
- ✅ `-1`  “"I said, 'I'm dead!' Dead, dead, la-la! Dead, dead, la-la..."”  ·  (Chaotic_CWCitizen_2)

### [DCWPOETF]  This attractive young woman is closely watching an elde

- ⬜ `-1`  “"Fine, call them."”  ·  (Chaotic_Audience_1)

### [DCWPOETM]  This gentleman is closely watching an elderly poet givi

- ⬜ `-1`  “"Fine, call them."”  ·  (Chaotic_Audience_1)

### [DDAMSEL]  You see a pretty young woman. Her hair is in disarray a

- ⬜ `-1`  “Bluff: "You're up to something. Tell me what's going on or I'll kill you."”  ·  (Chaotic_Damsel_1)

### [DDEATHAD]  "Sigilians, welcome! Please, take your seats, and liste

- ⬜ `-1`  “Bluff: "Kill yourself, or I'll do it."”  ·  (Chaotic_DA_1)

### [DDILIGNC]  This older, stern-looking woman is clearly on her way s

- ⬜ `-1`  “"This look is all the rage in the Hive, you know. Perhaps you should try it; it just might suit you."”  ·  (Chaotic_Diligence_1)
- ⬜ `-1`  “Lie: "That is untrue."”  *(+1 other phrasings)*  ·  (Chaotic_Diligence_2)

### [DDOLL]  The years have not been kind to this tiny rag doll; it 

- ⬜ `-1`  “"Awwww... you're just a cute little dolly, aren't you? Aren't you? Yes, you are!"”  ·  (Chaotic_Lady_Doll_3)
- ⬜ `-3`  “"Awwww... you're just a cute little Lady of Pain, aren't you? Aren't you? Yes, you are!"”  ·  (Chaotic_Lady_Doll_1)
- ⬜ `-3`  “Joke: "Oh, great Lady of Pain, goddess of Sigil, hear my plea for aid and help me in my hour of need."”  ·  (Chaotic_Lady_Doll_2)

### Ecco  [DECCO]

- ✅ `+1` *(g+1)*  “Vow: "I swear I'll look for a way to return your voice."”  ·  (Lawful_Ecco_1)
- ⬜ `-1`  “"Well, you can speak again... sort of... so where's my reward?"”  ·  (Chaotic_Ecco_1)

### Fell  [DFELL]

- ✅ `+1`  “Truth: "I... don't. In fact, ever since I woke up in the Mortuary, I feel like something's... missing. Something inside.”  ·  (Lawful_Fell_1)
- ⬜ `-1`  “Lie: "I feel complete, yes. Why?"”  ·  (Chaotic_Fell_1)

### Fleece  [DFLEECE]

- ✅ `+1` *(g+1)*  “Truth: "I'm afraid that I have never heard of such a place."”  ·  (Lawful_Fleece_1)
- ✅ `+1`  “"No, you keep the rest... I only want what you stole from me."”  ·  (Lawful_Fleece_2)
- ⬜ `-1` *(g-1)*  “Lie: "I do. I can direct you there... for a price."”  ·  (Chaotic_Fleece_1)
- ⬜ `-1` *(g-1)*  “Lie: "She is south and east of the Mortuary, near one of the mausoleums."”  ·  (Chaotic_Fleece_2)
- ⬜ `-1`  “"Oh, wait. I thought you were asking about one of the brothels around here. Never mind."”  ·  (Chaotic_Fleece_3)
- ⬜ `-1`  “Bluff: "Give me everything else you have, and I won't kill you."”  ·  (Chaotic_Fleece_4)
- ⬜ `-1` *(g-1)*  “Truth: "Give me everything else you have, and I won't kill you."”  ·  (Chaotic_Fleece_5)
- ⬜ `-1` *(g-1)*  “Nod your thanks, then attack him.”  *(+2 other phrasings)*  ·  (Chaotic_Fleece_6)

### [DGHOULSG]  The golem's massive frame resembles an enormous ghoul, 

- ⬜ `-1`  “Apply the Gorgon Salve to the golem.”  ·  (Lothar_Chaotic_Golem_1)

### [DGIANTSK]  Before you is a giant skeleton in ornate bronze armor. 

- ⬜ `-1`  “"Mind if I hold that blade for a second? You must be getting tired of holding it."”  *(+1 other phrasings)*  ·  (Chaotic_Giant_Skel_Mort_1)

### [DGITH]  This man slowly turns as you approach. His eyes are lik

- ⬜ `-1` *(g-1)*  “"I will not yield. I suggest you sheath that pig-sticker before I sheath it in your body."”  *(+1 other phrasings)*  ·  (Chaotic_Githzerai_1)
- ⬜ `-1` *(g-1)*  “Lie: "You ask a man who travels with a githzerai *zerth* if he has killed a githzerai? You must have the wrong person."”  *(+2 other phrasings)*  ·  (Chaotic_Githzerai_2)
- ⬜ `-1` *(g-1)*  “"Actually, you were right the first time. I was just waiting for you to drop your guard..."”  ·  (Chaotic_Githzerai_3)

### Griswold  [DGRIS]

- ✅ `+1`  “"Yes."”  ·  (Lawful_Gris_1)
- ✅ `+1`  “Truth: "They're dead."”  *(+1 other phrasings)*  ·  (Lawful_Gris_2)
- ⬜ `-1`  “Lie: "No."”  ·  (Chaotic_Gris_1)
- ⬜ `-1`  “Lie: "They made it."”  *(+2 other phrasings)*  ·  (Chaotic_Gris_2)

### [DGROSUK]  You see a reptilian creature with a snake like-body, fo

- ✅ `-1`  “Taunt: "Grossssssuk no talk? Grosssssuk ssssure ssssays a lot for sssssomeone who doesssssn't talk."”  ·  (Chaotic_Grosuk_1)

### [DHARLOTD]  The woman stumbles into you, then takes a step back, lo

- ⬜ `-1`  “Bluff: "Give me everything else you have, and I'll let you live."”  ·  (Chaotic_Drunk_Harlot_1)
- ⬜ `-1` *(g-1)*  “Truth: "Give me everything else you have, and I'll let you live."”  ·  (Chaotic_Drunk_Harlot_2)
- ⬜ `-2` *(g-2)*  “Snap her neck with your free hand.”  *(+1 other phrasings)*  ·  (Chaotic_Drunk_Harlot_3)

### [DHGCORV]  You see a tall man in plate armor. The armor is of unus

- ⬜ `-1`  “"Certainly. I'm sure that all her suitors say the same thing about her..."”  ·  (Chaotic_Corvus_1)

### [DHGUARD]  This large, red-armored figure is standing in place, ap

- ⬜ `-1`  “"And what if I feel like staying right here?"”  ·  (Chaotic_Patrolman_1)

### [DHPATROL]  This large, red-armored figure is standing in place, ca

- ⬜ `-1`  “"And what if I feel like staying right here?"”  ·  (Chaotic_Patrolman_1)

### Ignus  [DIGNUS]

- ❔ `-1` *(g-1)*  “"What about adding fuel to the fire? He did very well. A bit of pest control never hurt anyone."”  ·  ((direct/untracked))
- ❔ `-2`  “"Farewell, officer Adonis."”  ·  ((direct/untracked))
- ❔ `-3` *(g-3)*  “"Hey, Ignus, don't you want to turn up the heat even more? It's a bit cold in here."”  ·  ((direct/untracked))

### [DILQUIXN]  You see a short, rotund man with a perplexed expression

- ✅ `+1`  “Truth: "I disagree."”  ·  (Law_Ilquix_One)
- ⬜ `-1`  “Truth: "I very much agree."”  ·  (Chaos_Ilquix_One)
- ⬜ `-1`  “Lie: "I very much agree."”  *(+1 other phrasings)*  ·  (Chaos_Ilquix_Two)

### [DIRONNA]  This broad-shouldered woman is shuffling about amongst 

- ⬜ `-1`  “Lie: "I'm not sure. I had some questions..."”  *(+1 other phrasings)*  ·  (Chaotic_Nalls_1)

### [DJUMBLE]  This short, balding man's orange tunic and rather sizea

- ⬜ `-1`  “Bluff: "I want some answers, and I'll beat them out of you if I have to."”  *(+3 other phrasings)*  ·  (Chaotic_Jumble_1)

### Karina  [DKARINA]

- ⬜ `-1`  “"Well, I can understand why."”  ·  (Chaotic_Karina_1)

### [DKELDOR]  His face is impassive, but there seems to be fury behin

- ⬜ `-2` *(g-2)*  “"Yes. Thildon is guilty. Saros gave me an awl he found near the body. It was Thildon's awl."”  ·  (Chaotic_Keldor_1)
- ⬜ `-2` *(g-2)*  “"Yes. Saros is guilty. He tried to frame Thildon by stealing Thildon's awl. He claimed that he recovered it near the bod”  ·  (Chaotic_Keldor_2)

### [DKUUYIN]  The despondent man looks up to you, a glimmer of hope i

- ✅ `+4`  “"Yes."”  ·  (Lawful_KY_2)
- ✅ `+1` *(g+1)*  “"Yes. Your name is Ku'u Yin. Take your number."”  ·  (Lawful_KY_1)
- ⬜ `-1`  “Lie: "No. Not yet."”  ·  (Chaotic_KY_1)
- ⬜ `-1`  “Lie: "No, I don't."”  *(+1 other phrasings)*  ·  (Chaotic_KY_2)

### [DLENNY]  You see a feral-looking young man in shoddy clothes. Hi

- ⬜ `-1`  “Bluff: "Give me the papers you stole from Trist or I'll kill you."”  ·  (Chaotic_Lenny_1)

### [DMALMANR]  This thin, sharp-faced man rushes towards you, calling 

- ⬜ `-1`  “Lie: "No, not yet. He says you owe him, uh, fifty copper for the work."”  *(+4 other phrasings)*  ·  (Chaotic_Malmaner_1)
- ⬜ `-1`  “Lie: "Uh... yes. Here's the Dustman robe. I mean costume."”  ·  (Chaotic_Malmaner_2)
- ⬜ `-1`  “Lie: "No, not yet. He has another, but wants, uh, eighty coppers for it."”  ·  (Chaotic_Malmaner_3)

### [DMANTUOK]  You see a half-man/half-rat hybrid. Its red eyes gleam 

- ⬜ `-3` *(g-1)*  “"The Silent King is merely a dead skeleton. He has no power."”  ·  (Chaotic_Hargrimm_7)

### [DMANYAS1]  You are staggered by the sound of hundreds of voices sp

- ⬜ `-3` *(g-1)*  “"I thought you might like to know this: The Silent King is merely a dead skeleton. He has no power."”  *(+5 other phrasings)*  ·  (Chaotic_Hargrimm_7)

### Marrow  [DMARROW]

- ✅ `+1`  “Let him bite you.”  ·  (Lawful_Marrow_1)
- ⬜ `-1`  “Bluff: "Stop following me, or I'll kill you. Farewell."”  ·  (Chaotic_Marrow_1)
- ⬜ `-1`  “Lie: "Don't eat that. I'll bring you a whole body to eat rather than that."”  *(+1 other phrasings)*  ·  (Chaotic_Marrow_2)

### Mebbeth  [DMEBBETH]

- ✅ `+1` *(g+1)*  “"If you need help with anything around here, you can still ask... it's the least I can do in exchange for you teaching m”  ·  (Lawful_Mebbeth_1)
- ⬜ `-1`  “Lie: "The name's Adahn. Who are you?"”  ·  (Chaotic_Mebbeth_1)

### [DMONTAGU]  This handsome young man seems lost in thought, his brow

- ⬜ `-1`  “Lie: "I... saw her reading letters from him."”  ·  (Chaotic_Montague_1)

### [DMORTAI]  This tiny, wizened man is dwarfed by his huge Dustman r

- ⬜ `-1`  “Bluff: "I suggest you hand over my contract right now, or I'll butcher you where you stand."”  *(+3 other phrasings)*  ·  (Chaotic_Mortai_1)
- ⬜ `-1`  “Lie: "Well, I was thinking about signing a contract with Emoric."”  ·  (Chaotic_Mortai_2)

### [DMOURN1]  This Dustman is dressed in long dark robes, and his han

- ⬜ `-1`  “Lie: "No, uh, I'm speaking for a friend, uh... Adahn - he's the one who feels anguish over the person who died."”  *(+1 other phrasings)*  ·  (Chaotic_Mourn_1)

### [DMOURN2]  This Dustman is dressed in long dark robes, and her han

- ⬜ `-1`  “Lie: "No, uh, I'm speaking for a friend, uh... Adahn - he's the one who feels anguish over the person who died."”  *(+1 other phrasings)*  ·  (Chaotic_Mourn_1)

### Messenger  [DMSGR]

- ✅ `+1`  “"Yes, that's me."”  *(+1 other phrasings)*  ·  (Lawful_Msgr_1)
- ⬜ `-1`  “"No, that's not me."”  *(+1 other phrasings)*  ·  (Chaotic_Msgr_1)

### [DMW3]  You see a wiry older man who appears to be watching all

- ⬜ `-1`  “Bluff: "I can break that skinny little neck of yours before you get the words out of your mouth..."”  ·  (Chaotic_Marketman_1)

### Nodd  [DNODD]

- ⬜ `-1` *(g+1)*  “Lie: "Yes, she is. She's a... servant... now, at a tavern in the Hive. She is doing well, and is worried for you."”  *(+2 other phrasings)*  ·  (Chaotic_Nodd_1)
- ⬜ `-1` *(g-2)*  “Lie: "Yes, I did. She's a whore, and one so foul I doubt I'd be with her even free of charge."”  *(+2 other phrasings)*  ·  (Chaotic_Nodd_2)
- ⬜ `-1`  “"Yes. She gave me some money to give you but... it is gone. My apologies. She will visit you, as soon as she can find th”  ·  (Chaotic_Nodd_3)
- ⬜ `-3` *(g-1)*  “Lie: "Only that she said she would visit you, as soon as she found the time."”  ·  (Chaotic_Nodd_4)

### Nordom  [DNORDOM]

- ❔ `+1`  “End the sorrows of the modron Wurr-thurr. Let it return to Mechanus.”  ·  ((direct/untracked))
- ❔ `+1`  “"Have a nice day, officer Adonis."”  ·  ((direct/untracked))
- ✅ `+1`  “Truth: "I don't really *have* a name, Nordom."”  ·  (Lawful_Nordom_1)
- ⬜ `-1`  “Lie: "Adahn. My name is Adahn."”  ·  (Chaotic_Nordom_1)
- ⬜ `-1`  “"Do a little dance for me, Nordom."”  ·  (Chaotic_Nordom_3)
- ⬜ `-3` *(g-3)*  “"Why don't you go hostile and attack everyone you can?"”  ·  (Chaotic_Nordom_2)

### Pharod  [DPHAROD]

- ✅ `+1`  “Truth: "I have forgotten myself, and I was told to seek you out. That you would know something of me."”  ·  (Lawful_Pharod_1)
- ✅ `+1`  “Truth: "Well, no one *told* me, exactly. There were these tattoos on my back... they told me to seek you out, if I ever”  ·  (Lawful_Pharod_2)
- ✅ `+1`  “Vow: "What I hear is for my ears only."”  ·  (Lawful_Pharod_3)
- ✅ `+1`  “Vow: "You have my word that I'll stay my hand, Pharod. But I need to know what you know."”  ·  (Lawful_Pharod_4)
- ✅ `+1` *(g+1)*  “Truth: "I'm sorry about your people, Pharod. That wasn't me... but if I can make up the loss to you, I will."”  ·  (Lawful_Pharod_5)
- ⬜ `-1`  “Bluff: "Maybe you should tell me what you know, Pharod, or I'll pen your name in the dead-book with your blood."”  ·  (Chaotic_Pharod_1)
- ⬜ `-1`  “Lie: "What I hear is for my ears only."”  ·  (Chaotic_Pharod_2)
- ⬜ `-1` *(g-1)*  “Lie: "You have my word that I'll stay my hand, Pharod. But I need to know what you know."”  ·  (Chaotic_Pharod_3)
- ⬜ `-1`  “Bluff: "Pharod, my patience is at an end. There will be nothing left of you or this village if you don't hand over the '”  ·  (Chaotic_Pharod_4)
- ⬜ `-3` *(g-1)*  “Truth: "Pharod, my patience is at an end. If you don't hand over what was stolen from me, I will see to it the Dustmen k”  ·  (Chaotic_Pharod_5)

### [DPILLAR]  The sight of this thing - this horrible, towering, puls

- ⬜ `-1`  “Lie: "He is... hidden somewhere, in the Outlands. Beneath a large mountain."”  *(+1 other phrasings)*  ·  (Chaotic_Pillar_1)
- ⬜ `-3` *(g-3)*  “"In the Outlands, where Ul-Goris, father of all goristro was laid low in the Crater War. Fhjull lives within its skeleto”  ·  (Chaotic_Fhjull_1)

### [DPOET]  This kindly-looking old man spares you only the briefes

- ⬜ `-1`  “Heckle the poet.”  *(+1 other phrasings)*  ·  (Chaotic_Poet_1)

### [DPORPHIR]  Every inch of this man's skin is covered in a web of bl

- ⬜ `-1`  “Lie: "Oh, I still killed the thieves."”  ·  (Chaotic_Porphiron_1)
- ⬜ `-1`  “Lie: "Uh... I talked them out of it. They were happy to give me the beads back."”  *(+1 other phrasings)*  ·  (Chaotic_Porphiron_2)

### [DPOST]  This filthy-looking corpse is in sad shape; its shoulde

- ✅ `-1`  “"I thought *I* was in bad shape. Don't all those nails hurt?"”  *(+1 other phrasings)*  ·  (Chaotic_Post_1)

### [DPUZSKEL]  This skeleton is muttering to itself, occasionally paus

- ⬜ `-1`  “Lie: "Eh... no. Farewell."”  ·  (Chaotic_Puzzled_1)
- ⬜ `-3`  “"Very well. It's 'tongue.'"”  *(+1 other phrasings)*  ·  (Chaotic_Puzzled_2)

### [DQUELL]  This older man is chewing on something, muttering softl

- ⬜ `-1`  “Pick it up and eat it.”  *(+1 other phrasings)*  ·  (Chaotic_Quell_1)

### [DQUINT]  "Heh. My thanks. You want what?"

- ⬜ `-1`  “Lie: "No, not yet. I'm still looking."”  ·  (Chaos_Quint_One)
- ⬜ `-1`  “Lie: "Pharod said I could trade here."”  ·  (Chaos_Quint_Two)

### Lady of Pain/Radine  [DRADINE]

- ✅ `+1`  “"Rightfully yours? You have no right to that name and number. You stole it."”  ·  (Lawful_Radine_1)
- ⬜ `-1`  “"You don't need his kind of number to live. Make your own number, if you need one to keep you warm."”  ·  (Chaotic_Radine_1)

### [DRKCHSR]  This filthy, robed man is meandering about, looking aro

- ⬜ `-1`  “Bluff: "Talk, or I'll beat the answer out of you, wretch!"”  ·  (Chaotic_Rchasr_1)

### [DRKCHSR2]  This grubby-looking Collector is staring off into space

- ⬜ `-1`  “Bluff: "Enough. Give me the pendant or die."”  *(+1 other phrasings)*  ·  (Chaotic_RCII_1)

### [DS42]  The skeleton turns to face you. "42" has been chiseled 

- ✅ `-1`  “"Pardon me, have you seen any skeletons walking around here?"”  ·  (Skeleton_Chaotic)

### [DSABAST]  You see an older man in elegant robes. He has bright ey

- ✅ `+1`  “Truth: "I understand your dilemma and agree with you. Could I release him instead?"”  *(+1 other phrasings)*  ·  (Lawful_Sebastion_1)
- ⬜ `-1`  “Truth: "I really don't give a rat's arse about contracts. What you did is wrong and he needs to be freed."”  ·  (Chaotic_Sebastion_1)
- ⬜ `-1`  “"I just wanted to compliment you on a job well done."”  ·  (Chaotic_Sebastion_2)

### [DSARHAVA]  The smell of alcohol wafts heavily from this young woma

- ⬜ `-1`  “Bluff: "Adhana? Is that you?"”  *(+1 other phrasings)*  ·  (Chaotic_Sarhava_1)
- ⬜ `-1`  “Bite a chunk out of her bare leg.”  ·  (Chaotic_Sarhava_2)
- ⬜ `-1`  “Make biting noises at her as she walks away.”  ·  (Chaotic_Sarhava_3)

### [DSAROS]  "Hey, don't touch me! Leave me alone!"

- ✅ `-1`  “"That defeats the entire point of anarchy, Saros. You're supposed to be your own person."”  ·  (Chaotic_Saros_1)

### [DSCOFF]  You see a thin, stooped man hunched over a desk, scribb

- ⬜ `-1`  “"No. I just want to be a member. If you're into destruction, I'm in."”  ·  (Chaotic_Penn_1)

### [DSDTHUG]  This wild-eyed man is hunched over, barking and howling

- ⬜ `-1`  “Give a long, forlorn howl.”  ·  (Chaotic_SDThug_1)
- ⬜ `-1`  “Snarl back at him, baring your teeth.”  ·  (Chaotic_SDThug_2)

### Sere the Anarch  [DSERE]

- ✅ `+1`  “Truth: "Old."”  ·  (Lawful_Sere_1)
- ✅ `-1`  “"A bunch of fun-loving Dustmen, drinking happily and conversing merrily, celebrating life to its fullest."”  ·  (Chaotic_Sere_1)
- ⬜ `-1`  “"Well, thank the powers for that. Farewell."”  ·  (Chaotic_Sere_2)
- ⬜ `-1` *(g+1)*  “Lie out of kindness: "Rather youngish."”  ·  (Chaotic_Sere_3)
- ⬜ `-1` *(g-1)*  “"I will. Farewell."”  ·  (Chaotic_Sere_4)

### Sev'tai  [DSEVTAI]

- ✅ `+1`  “"Their actions were unjust. If you wish, I can see that the matter is rectified. If three deaths they caused, then three”  ·  (Lawful_Sevtai_1)
- ⬜ `-1`  “Lie: "I found three of the Starved Dogs Barking and penned three of them in the dead-book."”  ·  (Chaotic_Sevtai_1)

### [DSKELBAR]  The skeleton turns toward you, rattling as he does so. 

- ⬜ `-1`  “"Nice tunic. I mean it."”  ·  (Skeleton_Bar_1)

### [DSPLINT]  This towering man's golden skin sparkles slightly, almo

- ⬜ `-1`  “Lie to infiltrate the Sensates for the Anarchists: "I'm ready to join the Society of Sensation. I feel no more loyalty t”  ·  (Chaotic_Splinter_1)

### [DSWORDOW]  The Sword of Wh'ynn - also known as the Cheater's Blade

- ⬜ `-1`  “Cheat: stand with your legs slightly apart, flex all your muscles and thrust the blade triumphantly towards the heavens.”  ·  (Chaotic_SwordoW_1)

### [DTHORP]  This creature is huge, standing over nine feet tall. Ri

- ⬜ `-1`  “Lie: "I'm Adahn."”  ·  (Chaotic_Thorp_1)

### [DTHUGP1]  This heavy-set thug is swathed in rags, dyed red and bl

- ⬜ `-1`  “Lie: "That *is* interesting. The rest of his order is looking for you; the brothers of *Erit-Agge* seem to know you quit”  ·  (Chaotic_PorThug_1)
- ⬜ `-1`  “Bluff: "No, not worth it. I'll let the rest of his order look for you - I think the brothers of *Erit-Agge* will be able”  ·  (Chaotic_Thug_1)
- ⬜ `-1`  “Lie: "I hope this is enough to call off the rest of his Order. Perhaps a donation to their cause is in order as well. It”  ·  (Chaotic_Thug_2)

### [DTHUGT1]  "Shiva's garters! What be *this* foul-smellin' thing?"

- ⬜ `-1`  “Moan like an undead creature: "Uhhhnnn..."”  ·  (Chaotic_Taunter_1)

### [DTHUGV1]  As you approach, the man takes a step back; his eyes ha

- ⬜ `-1` *(g-1)*  “Attack him anyway.”  *(+1 other phrasings)*  ·  (Chaotic_Vulture_1)

### [DTIRES]  You see an aging man dressed in tattered rags. As you d

- ⬜ `-1`  “"How about dark for dark. I can tell you the whereabouts of a certain thief hiding here in the Tenement."”  ·  (Chaotic_Tiresias_1)

### [DTOMB]  The statue looks like it is about to make some angry pr

- ✅ `-1`  “Apply the Gorgon Salve to the statue.”  ·  (Chaotic_Statue_1)
- ✅ `-1`  “Break the piece off.”  ·  (Chaotic_Statue_2)

### [DXANDER]  The old man gazes up at you without fear, and says, "So

- ⬜ `-1`  “"Forget it. I have better things to do than wait on a barmy's ravings." Leave.”  *(+1 other phrasings)*  ·  (Chaotic_Xander_1)
- ⬜ `-1`  “Lie: "I apologize. My temper got the better of me. It won't happen again."”  ·  (Chaotic_Xander_2)

### [DYVES]  This fetching young woman has a far-away look in her so

- ⬜ `-3` *(g-1)*  “Tell her the story of the Silent King.”  ·  (Chaotic_Hargrimm_7)

### [DZ1041]  This re-animated, male corpse has the numbers "1041" ca

- ⬜ `-1`  “"It certainly seems easy for you to just not care about your duties because you're dead. I don't know if I could just le”  ·  (Chaotic_Bei_1)
- ⬜ `-1` *(g-1)*  “"Never mind. I don't have time for this."”  ·  (Chaotic_Bei_2)

### [DZ1146]  The number "1146" is carved into the forehead of this w

- ⬜ `-1` *(g-1)*  “Strike him.”  ·  (Chaotic_Crispy_1)

### [DZ257]  The eyes of this corpse are set close together and the 

- ⬜ `-1`  “"I didn't quite catch that. Could you repeat it for me?"”  ·  (Chaotic_Zom257_1)

### [DZM1041]  This re-animated, male corpse has the numbers "1041" ca

- ⬜ `-1`  “"It certainly seems easy for you to just not care about your duties because you're dead. I don't know if I could just le”  ·  (Chaotic_Bei_1)
- ⬜ `-1` *(g-1)*  “"Never mind. I don't have time for this."”  ·  (Chaotic_Bei_2)

### [DZM1146]  The number "1146" is carved into the forehead of this w

- ⬜ `-1` *(g-1)*  “Strike him.”  ·  (Chaotic_Crispy_1)

### [DZM257]  The eyes of this corpse are set close together and the 

- ⬜ `-1`  “"I didn't quite catch that. Could you repeat it for me?"”  ·  (Chaotic_Zom257_1)

### [DZM79]  The corpse's meaty head was clearly severed at some poi

- ✅ `-1`  “"So... seen anything interesting going on?"”  ·  (Zombie_Chaotic)

### ZM985  [DZM985]

- ✅ `+1` *(g+1)*  “Try and help the corpse balance.”  ·  (Lawful_ZM985_1)
- ⬜ `-1` *(g-1)*  “Give the corpse a push.”  ·  (Chaotic_ZM985_1)

### [DZOMFBAR]  This female corpse is dressed in a heavy burlap shirt c

- ⬜ `-1`  “"So... doing anything later?"”  ·  (Chaotic_Zombie_Bar_1)

### [DZOMMBAR]  The zombie gazes at you with vacant eyes. His lips have

- ⬜ `-1`  “"So... seen anything interesting around here?"”  ·  (Chaotic_Zombie_Bar_1)

### [G-BBD006]  There is a whisper from the depths of your inventory. "

- ❔ `-1`  “Lie: "The Doomguard."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "The Lady of Pain."”  ·  ((direct/untracked))
- ❔ `-1`  “"Dark Alley Shivs."”  ·  ((direct/untracked))
- ❔ `-1`  “"Razor Angels."”  ·  ((direct/untracked))

### [G-BBD008]  Brill sees you approaching and immediately reaches out 

- ❔ `+2`  “"The chaos matter went through me and it was... horrible."”  ·  ((direct/untracked))
- ❔ `-1`  “"Whoa..."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "I don't have that much money."”  ·  ((direct/untracked))
- ❔ `-1`  “Pour the water onto the floor.”  ·  ((direct/untracked))

### [G-BBD010]  "No-no. Our victims!"

- ❔ `-1`  “Lie: "Of course. Praise Istishia! Please remind me, brother, how to best honor our lord."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"One of our prisoners, the bladeling. He looks tasty. Cook him for me."”  ·  ((direct/untracked))
- ❔ `-1`  “"As your new lord, I order you to offer monetary sacrifices to me. You may start right now."”  ·  ((direct/untracked))

### [G-BBD016]  One of the prisoners destined to be sacrificed to the k

- ❔ `-1`  “"Share the secrets of Entropy with me."”  ·  ((direct/untracked))

### [G-BBD017]  "Ah... the terrifying double-wattled cassowary finally 

- ❔ `+1`  “"Long hair getting in the way of a fight, yet she's ready to fight even in her sleep? Reckless behavior? Unrealistic goa”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Shut up, thesp."”  ·  ((direct/untracked))
- ❔ `-1`  “"I guess she really is devoted to chaos and entropy. A truly free person. This is what impresses me."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Attack him directly.”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Damn."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Exeunt. Time to evacuate."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Exeunt."”  ·  ((direct/untracked))
- ❔ `-3` *(g-3)*  “"You won't distract me. I'd use your swords. Die."”  ·  ((direct/untracked))

### [G-BBD019]  Before you stands a half-human, half-rat hybrid with gr

- ❔ `+2` *(g-1)*  “"That's right, Fragile-Tail. Today is the day you pay for all these thefts."”  ·  ((direct/untracked))
- ❔ `+2` *(g-1)*  “Bluff: "That's what I want. No one will trade stolen goods in *my* area without *my* approval. From larvae smugglers to”  ·  ((direct/untracked))
- ❔ `+2` *(g+1)*  “"That's what I want. Please forgive me, but this is necessary."”  ·  ((direct/untracked))
- ❔ `+2`  “"That's what I want. Hard law, but law."”  ·  ((direct/untracked))
- ❔ `+2` *(g-1)*  “"That's what I want. I'll take this as your admission of guilt. Today is the day you pay for all these thefts."”  ·  ((direct/untracked))
- ❔ `+2` *(g+1)*  “"That's what I want. I'll take this as your admission of guilt. Please forgive me, but this is necessary."”  ·  ((direct/untracked))
- ❔ `+2`  “"That's what I want. I'll take this as your admission of guilt. Hard law, but law."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"That's what I want. Just my whim. Completely random."”  ·  ((direct/untracked))
- ❔ `+1`  “"I'll help you, Vhailor! We'll catch this lousy criminal!"”  ·  ((direct/untracked))
- ❔ `+1` *(g+1)*  “Truth: "Yes. I'm sorry. It was necessary."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “Truth: "Yes. It was fun."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"That's what I want. I'll take this as your admission of guilt. Just my whim. Completely random."”  ·  ((direct/untracked))
- ❔ `+1`  “Truth: "They most certainly froze to death almost instantly on the other side."”  ·  ((direct/untracked))
- ❔ `-1`  “Joke: "Erm... No... Tell me."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "No, I don't. Is it valuable?"”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"What about adding fuel to the fire? He did very well. A bit of pest control never hurt anyone."”  ·  ((direct/untracked))
- ❔ `-1` *(g+1)*  “Lie: "Yes. I'm sorry. It was necessary."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Lie: "Yes. It was fun."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "No. I did no such thing."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "Perhaps not. Maybe I can find them."”  ·  ((direct/untracked))

### [G-BBD021]  Your eyes catch sight of a tall skeleton standing motio

- ❔ `-1` *(g-1)*  “Attack it. "Death to the undead!"”  ·  ((direct/untracked))
- ❔ `-1`  “"Well, it was nice chatting, but it's time for me to go."”  ·  ((direct/untracked))

### [G-BBD025]  Howls, shrieks, and curses are not uncommon in any esta

- ❔ `+1`  “"The abishai is right. Rules are meant to be followed."”  ·  ((direct/untracked))
- ❔ `-1`  “"The glabrezu is right. Rules are made to be broken."”  ·  ((direct/untracked))

### [G-BBD026]  This fiend is tall and frighteningly thin, with skin lo

- ❔ `-1`  “"Coincidence. I rarely think about what I do."”  ·  ((direct/untracked))

### [G-BBD027]  Mare looks at you full of anger. "What are you still lo

- ❔ `+1`  “Truth: "I can sense the uncertainty in your voice. He is not in danger, I just need to talk to him."”  ·  ((direct/untracked))
- ❔ `-2`  “Lie: "I can sense the uncertainty in your voice. He is not in danger, I just need to talk to him."”  ·  ((direct/untracked))

### [G-BBD028]  A half-orc standing nearby shouts at you. "A juist said

- ❔ `-1`  “"Thanks! I have some questions."”  ·  ((direct/untracked))

### [G-BBD029]  In a tiny, begrimed room, you see a lone githzerai. His

- ❔ `+2`  “"Looks to me like you're making excuses. You stole from a law abiding citizen. Your actions were not justified."”  ·  ((direct/untracked))
- ❔ `+1` *(g+1)*  “"Looks to me like you are a victim of circumstance. However, theft was not the answer. Your actions were not justified."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"I'm not here to judge. Just give me the vial."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"I'm not here to negotiate, and you need to hand over what you took. Now."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Attack. "I'm not here to judge, only to exact punishment."”  ·  ((direct/untracked))
- ❔ `-1` *(g+1)*  “"I'm not here to judge, but I did not like Brill since I laid my eyes on him. Keep your karach."”  ·  ((direct/untracked))
- ❔ `-1` *(g+2)*  “Look at Dak'kon. "Can you help him regain control of his karach?"”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Attack. "No. *I* am the villain here."”  ·  ((direct/untracked))
- ❔ `-3`  “"I'm going to tell him you were found dead and the vial was gone. I'll pin it on Joseph. It's the perfect lie."”  ·  ((direct/untracked))

### [G-BBD031]  "Hey, pay attention!"

- ❔ `-1`  “Lie: "I have trouble seeing at such light levels."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "My reaction time isn't what it used to be."”  ·  ((direct/untracked))

### [G-BBD042]  A tall, lean woman with dark skin stands to the side of

- ❔ `-1`  “Lie: "My name is Adahn."”  ·  ((direct/untracked))

### [G-BBD049]  The newly sobered up Harmonium officer, Adonis sees you

- ❔ `+3`  “"I tasted human flesh, I'm an abomination. I came to surrender my meaningless life to the law."”  ·  ((direct/untracked))
- ❔ `+2` *(g+2)*  “"Go and get them, officer Adonis."”  ·  ((direct/untracked))
- ❔ `+2` *(g-2)*  “"Go and repent, officer Adonis."”  ·  ((direct/untracked))
- ❔ `+1`  “"I accidentally killed a woman on the cliff. I came to turn myself in."”  ·  ((direct/untracked))
- ❔ `+1`  “"I do care. You are in public service, you are representing this city. You should be ashamed."”  ·  ((direct/untracked))
- ❔ `+1`  “"Have a nice day, officer Adonis."”  ·  ((direct/untracked))
- ❔ `-2` *(g+2)*  “"Go and get them, ex-officer Adonis."”  ·  ((direct/untracked))
- ❔ `-2`  “"Farewell, officer Adonis."”  ·  ((direct/untracked))
- ❔ `-2`  “"Kick their asses, officer Adonis."”  ·  ((direct/untracked))
- ❔ `-5`  “"Fine. Let's dance."”  ·  ((direct/untracked))
- ❔ `-5`  “Attack the guard. "You will never take me alive, Hardhead."”  ·  ((direct/untracked))

### [G-BBD050]  A dark, motionless silhouette blends into the murky wat

- ❔ `+1`  “"He didn't do anything illegal. Law is the ultimate guide as to what is right. At least, until a more perfect one is inv”  ·  ((direct/untracked))
- ❔ `+1`  “"Sounds like someone who isn't very reasonable. Trying to change phenomena so fundamental to the planes can only lead to”  ·  ((direct/untracked))
- ❔ `+1`  “"Thank you for that. Unfortunately, you are a danger to this city, so you *do* have to die."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "My name is Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “"You're right. She sounds like a pretty interesting person. I'd also like to leave a mark on the planes by changing what”  ·  ((direct/untracked))
- ❔ `-2` *(g-2)*  “"That tasted sweet."”  ·  ((direct/untracked))

### [G-BBD052]  In the center of the upper deck of the Oarsman stands a

- ❔ `-1`  “Lie: "My name is Adahn."”  ·  ((direct/untracked))

### [G-BBD053]  You see one of the theater's patrons. It is a rather yo

- ❔ `-1`  “"I killed him."”  ·  ((direct/untracked))

### [G-BBD054]  "Are you absolutely barmy, you wretched, piking, hypocr

- ❔ `-1`  “"Oh, him? I killed him."”  ·  ((direct/untracked))
- ❔ `-1`  “"Oh, that's what it's all about. I killed him."”  ·  ((direct/untracked))

### [G-BBD055]  "Honey, come on..." From this position you cannot hear 

- ❔ `-1` *(g-1)*  “"Zack, it's obvious she wants to strip you of your true nature. Will you really let yourself be castrated like that in t”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Sonne, can't you see that he's trying to take advantage of you? To make a harlot out of you? He wants to defile your pu”  ·  ((direct/untracked))

### [G-BBD056]  As you reach the end of the pier running through the Su

- ❔ `+1`  “"What you call 'love' is nothing more than a simple manifestation of the reproductive instinct. An illusion imposed on i”  ·  ((direct/untracked))
- ❔ `-1`  “"Devoting oneself to sensuality is also a form of liberation from norms. Rebellion leads to freedom, and freedom... chao”  ·  ((direct/untracked))
- ❔ `-1`  “"Devoting oneself to sensuality is also a form of liberation from norms. Rebellion leads to freedom, and freedom, chaos,”  ·  ((direct/untracked))
- ❔ `-1`  “"As long as you allow yourself to be driven by passion, you will never rid yourself of the urges of life, and therefore”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Zack, it's obvious she wants to strip you of your true nature. Will you really let yourself be castrated like that in t”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Sonne, can't you see that he's trying to take advantage of you? To make a harlot out of you? He wants to defile your pu”  ·  ((direct/untracked))

### [G-BBD058]  In the vicinity of the medical tent, a man is circling 

- ❔ `-1`  “Lie: "Oh, not the Rowan Academy! I heard it's just a nest of corruption and pseudoscience."”  ·  ((direct/untracked))

### [G-BBD059]  A scarred man stands at the bar. He is unhealthily thin

- ❔ `-1` *(g-1)*  “"Just for fun."”  ·  ((direct/untracked))

### [G-BBD066]  In one of the theater seats, you see a woman dressed in

- ❔ `+1`  “"...this seemed to be the solution that would restore order to Sigil."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"Bloodlust. I wanted a good fight."”  ·  ((direct/untracked))
- ❔ `-1`  “"...this seemed more fun. Chaos reigns."”  ·  ((direct/untracked))

### [G-BBD068]  You see a strange creature in front of you. It stands a

- ❔ `+1`  “"Aren't you too old for this kind of play?"”  ·  ((direct/untracked))
- ❔ `+1`  “"Life is not a game, remember that. Perhaps you could share a secret with me? Maybe about... a rat temple?"”  ·  ((direct/untracked))
- ❔ `-1` *(g+1)*  “Pretend to be terrified. "Oh no! Please don't hurt me, o mighty abishai!"”  ·  ((direct/untracked))

### [G-BBD070]  You see a man in his forties dressed surprisingly well 

- ❔ `+1`  “"You're wrong. Pores are just indentations in a surface, not holes through which you can walk. Even if you could shrink”  ·  ((direct/untracked))
- ❔ `+1`  “"Please wait, I'll prepare precise calculations to prove my point. Nordom, would you be so kind?"”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"We require a fee of 10 copper pieces for the mathematical services rendered."”  ·  ((direct/untracked))

### [G-BBD074]  Before you there is an old woman whose life surely was 

- ❔ `+1`  “Sigh with disapproval. "This is completely frivolous and immature, not befitting an adult. Goodbye."”  ·  ((direct/untracked))
- ❔ `-1`  “"He he, piss. Funny! I have to go now, though."”  ·  ((direct/untracked))
- ❔ `-1`  “"He he, piss. Funny! I hav e to go now, though."”  ·  ((direct/untracked))

### [G-BBD077]  You see before you an emaciated humanoid figure, crouch

- ❔ `-1` *(g-1)*  “"That was a joke. Without them, your face looks much prettier."”  ·  ((direct/untracked))

### [G-BBD079]  You see a middle-aged woman with sharp features and bro

- ❔ `+1`  “"But it defies any logic. How can you expect different results if repeating the same action so far has not produced them”  ·  ((direct/untracked))
- ❔ `-1`  “"Muuuud! I like mud too! It's so sticky and wet. Although it sometimes gets under the nails and it's not so fun then."”  ·  ((direct/untracked))

### [G-BBD085]  In an alley between buildings you see a frail but well-

- ❔ `-1`  “Lie: "Yes, I have. Two streets over. I think she was going that way."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Laugh at him. "Look, people! A woman took his clothes!"”  ·  ((direct/untracked))

### [G-BBD086]  Behind the counter of The Rat and Splat stands a werera

- ❔ `+1`  “Turn to Vhailor. "Is he clean? Is that the truth?"”  ·  ((direct/untracked))
- ❔ `+1`  “Talk to Vhailor. "Did the wererat tell me the truth? Is he innocent?"”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Attack the wererat. "You got me. This is why I came. You don't look clean, and it's time to pay."”  ·  ((direct/untracked))
- ❔ `-1`  “"Let's sow chaos. Who knows what the consequences of him dying might be?"”  ·  ((direct/untracked))
- ❔ `-1`  “"I have some questions."”  ·  ((direct/untracked))
- ❔ `-1`  “"Never mind. Goodbye."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Attack the wererat. "So, he's around here somewhere? Excellent. Then die."”  ·  ((direct/untracked))

### [G-BBD087]  You feel a disapproving stare of the angel on the back 

- ❔ `-1`  “Lie: "I broke into his house unnoticed and stole the sword from the center of his trophy gallery."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "I fought Kabatum in a harrowing battle until, with great effort, I wrested the holy sword from his dead body."”  ·  ((direct/untracked))
- ❔ `-1`  “"My memory is poor, but... call me Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “"Really? I'm sure you will find something more to honor the recovery of this sacred relic, as well as my effort and time”  ·  ((direct/untracked))

### [G-BBD088]  "You monster! Get away from me."

- ❔ `-1`  “Lie: "I am Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "I... am Adahn."”  ·  ((direct/untracked))

### [G-BBD089]  The amnesiac just cowers in fear.

- ❔ `-1`  “Lie: "Sure, old pal. It's been some time."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "After a closer look... of course I know you!"”  ·  ((direct/untracked))
- ❔ `-1`  “"We fought together in the Blood War. Your name is Maurice and you are a mighty warrior."”  ·  ((direct/untracked))
- ❔ `-1`  “"You are the fearless vampire slayer from Ysgard, Silva Reyes Freysonn."”  ·  ((direct/untracked))
- ❔ `-1`  “"Your name is Dipper, you are a detective here in The Ditch."”  ·  ((direct/untracked))
- ❔ `-3`  “"Your name is Stephan. You are a sulfur trader from Torch."”  ·  ((direct/untracked))

### [G-BBD091]  In a bright, circular room filled with strange devices,

- ❔ `+1`  “"This weapon was forged by the gods. I understand what the deva meant when he said that it's sacred and it deserves grea”  ·  ((direct/untracked))
- ❔ `+1` *(g+1)*  “"I want this sword's destiny to be fulfilled. I want it to return to the Heavens and find its way into the hands of anot”  ·  ((direct/untracked))
- ❔ `-1`  “"I wish for the forces of the multiverse to be equalized. I'll hurl this sword into the Abyss so that the two opposing f”  ·  ((direct/untracked))

### [G-BBD092]  A young man, about sixteen years of age, stands in the 

- ❔ `-1`  “"Why are you here, D-Damian?"”  ·  ((direct/untracked))

### [G-BBD094]  It is hard for you to describe the person in front of y

- ❔ `-3` *(g-3)*  “"That does it. I think I'll simply kill you."”  ·  ((direct/untracked))

### [G-BBD095]  You see a young woman in dirty, torn clothes typical of

- ❔ `-1`  “Lie: "Of course! I haven't seen a more beautiful one!"”  ·  ((direct/untracked))

### [G-BBD096]  You see a woman who could be considered beautiful among

- ❔ `+1` *(g+1)*  “"I thought you might want to give them back to their owner."”  ·  ((direct/untracked))
- ❔ `+1` *(g-1)*  “"You'll regret what you have done. I'll serve you justice here and now."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “"You'll regret ignoring me!"”  ·  ((direct/untracked))

### [G-BBD097]  You see a man with dried yellow skin and a fierce, albe

- ❔ `-1`  “"Actually, I was wondering if you like how I look."”  ·  ((direct/untracked))
- ❔ `-3` *(g-3)*  “(His stoic calmness irritates you. Attack him.)”  ·  ((direct/untracked))

### [G-BBD119]  Before you stands a monumental, imposing fiend. One of 

- ❔ `-1`  “Direct it to the kuo-toa. "They are hiding in the old house in the flooded settlement, the only house still standing."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "They are holed up on the other side of the Ditch, by the Taker's Lock."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "They left Sigil, they may already be in Curst."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "They have taken refuge in the Temple of the Abyss."”  ·  ((direct/untracked))
- ❔ `-1`  “"Oh wait. Wererats? It's a misunderstanding, then. I was thinking of other thieves."”  ·  ((direct/untracked))

### [G-BBD120]  A strange scene is unfolding in the distance in front o

- ❔ `+1`  “"People like you are the cause of the moral rot that is eating away at this city. It's you who will go to the land of an”  ·  ((direct/untracked))
- ❔ `+1`  “"Vhailor, don't you think a great injustice has been done here?"”  ·  ((direct/untracked))

### [G-BBD141]  "How about you gallop back to Ysgard, to your idyllic r

- ❔ `-1`  “"Don't you mean Haer'Dalis's death? This is very much my business. I killed him."”  ·  ((direct/untracked))

### [G-BBD142]  "You want to try it, berk? You're alone, and we got blo

- ❔ `-1`  “"Oh yeah. I killed him."”  ·  ((direct/untracked))

### [G-BBD156]  "Little one, I'm not quite sure what your best qualitie

- ❔ `+5`  “"Yes. The skull belongs to you. Justice demands that I return it to you."”  ·  ((direct/untracked))
- ❔ `-1`  “"It is for me to decide when and with whom I speak."”  ·  ((direct/untracked))

### [G-BBD157]  Standing at the back of the room is an unusually short 

- ❔ `-3` *(g-3)*  “"Hey, Ignus, don't you want to turn up the heat even more? It's a bit cold in here."”  ·  ((direct/untracked))

### [G-BBD159]  Your feet carry you forward through the marble structur

- ❔ `+1`  “"Logic."”  ·  ((direct/untracked))
- ❔ `+1`  “"Justice."”  ·  ((direct/untracked))

### [G-BBD161]  This large, segmented fiend is wandering around the upp

- ❔ `-1`  “Lie: "Likewise. I am Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "I am Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “Watch.”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "No, I was... someplace else. Definitely."”  ·  ((direct/untracked))

### [G-BBD167]  You come upon a creature like no other you have seen in

- ❔ `-5` *(g-5)*  “Drink from the cup.”  ·  ((direct/untracked))

### [G-BBD177]  From a distance, you can hear a loud clang of metal str

- ❔ `+1`  “End the sorrows of the modron Wurr-thurr. Let it return to Mechanus.”  ·  ((direct/untracked))

### [G-BBD181]  You see a well-dressed man, obviously out of place. His

- ❔ `-1`  “Lie: "That's better! I am Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “Joke: "I briefly dated a night hag."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "I am Adahn."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie. "Oh, of course. I just never met you, is all."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "Oh, of course I wanted them to help. I'm not a butcher."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "Frost giants are way too big to carry around, and probably have a lot of bones they don't really need, right?"”  ·  ((direct/untracked))

### [G-BBD189]  As you travel alongside Ulfbrand and his forces toward 

- ❔ `+1` *(g-1)*  “Good. Only the strong will survive. This is the way of the planes. There is no reason to feel regret about it.”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Very good. The carnage will be wonderful.”  ·  ((direct/untracked))
- ❔ `-1` *(g-2)*  “Very good. Utter a silent prayer to your new patron, the Lady of the Dead. The loss of life that is about to unravel is”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Good. The city will bathe in blood this day. The Fated want to take everyone's belongings. The Doomguard embrace decay a”  ·  ((direct/untracked))
- ❔ `-1`  “You are unsure. You have to believe that your choices are right. If this path is the only way to stop the slow undoing o”  ·  ((direct/untracked))
- ❔ `-1`  “Spit toward the Mercykillers. It is a rare opportunity to do so without them being able to do anything about it.”  ·  ((direct/untracked))

### [G-BBD190]  As you travel alongside Ulfbrand and his forces toward 

- ❔ `+1` *(g-1)*  “Good. Only the strong will survive. This is the way of the planes. There is no reason to feel regret about it.”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Very good. The carnage will be wonderful.”  ·  ((direct/untracked))
- ❔ `-1` *(g-2)*  “Very good. Utter a silent prayer to your new patron, the Lady of the Dead. The loss of life that is about to unravel is”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Good. The city will bathe in blood this day. The Fated want to take everyone's belongings. The Doomguard embrace decay a”  ·  ((direct/untracked))
- ❔ `-1`  “You are unsure. You have to believe that your choices are right. If this path is the only way to stop the slow undoing o”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "*Obviously* this is a lie. Ulfbrand, nobody would be stupid enough to do what I am *supposedly doing*."”  ·  ((direct/untracked))

### [G-BBD209]  Next to a small makeshift pier on a heap of garbage, un

- ❔ `-1`  “"Greetings and goodbye, garbage sailor."”  ·  ((direct/untracked))

### [G-BBD218]  You do not always feel like talking to fiends, but this

- ❔ `+1`  “"He gave me a task. So what?"”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "Oh, you mean the red abishai I just killed by the Gatehouse? He's dust now."”  ·  ((direct/untracked))
- ❔ `-1`  “Lie: "Relax. We had a brief chat. There's nothing between us."”  ·  ((direct/untracked))

### [G-BBD219]  A plump man is keeping watch in front of the shack. He 

- ❔ `-1`  “Lie: "My name is Adahn."”  ·  ((direct/untracked))

### [G-BBD220]  A wererat leans against a wooden post. He looks like th

- ❔ `-1`  “"Adahn."”  ·  ((direct/untracked))

### [G-BBD221]  You see a black-haired man dressed in a black suit with

- ❔ `+1`  “"Sounds perfect. This is how the planes should be organized."”  ·  ((direct/untracked))
- ❔ `-1` *(g-1)*  “Lie: "Oh, it is totally safe to use now. Just a bit of a headache. I won't spoil what I have seen, you should just see f”  ·  ((direct/untracked))
- ❔ `-1`  “"Sounds awful. I don't like hierarchy and order."”  ·  ((direct/untracked))

### [G-BBD226]  A middle-aged man somberly leans against the statue. Al

- ❔ `-1`  “Lie: "What? No, I don't know what you're talking about."”  ·  ((direct/untracked))

### [G-BBD234]  This merchant's sales pitch is not as refined as the ot

- ❔ `+2`  “"Hey... my friend here 'borrowed' some rope from you and I came to make it up to you..."”  ·  ((direct/untracked))

### [GFCLERK]  "So you're the one they're making all the fuss about, e

- ⬜ `-1`  “Lie: "Yes."”  ·  (Clerk1)
- ⬜ `-1`  “Lie: "What? That's not what I need it for!"”  ·  (Clerk2)

### [GFGUARD]  "You again? Guards! Guards!"

- ⬜ `-1`  “Lie: "Yes."”  ·  (Chaotic_Guards_1)

### [SWORDOW]  The Sword of Wh'ynn - also known as the Cheater's Blade

- ⬜ `-1`  “Cheat: stand with your legs slightly apart, flex all your muscles and thrust the blade triumphantly towards the heavens.”  ·  (Chaotic_SwordoW_1)
