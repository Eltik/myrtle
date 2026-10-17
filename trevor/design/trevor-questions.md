# What people will ask Trevor

Requirement (2026-09-28): before serving, work out what questions people will ask an AI about the lore. The three seed examples: "When did Ceobe get high on drugs?", "How survivable is the region of Sami?", "What endings in IS3 are canon/not canon?"

This doc began as a theory of the question space and is now measured on 883 real questions (section 6). The theory was checked three ways: where the game's lore text actually lives (measured), a live probe of 10 representative questions through `ask` (2026-09-28, gemma-4-12b, P3a corpus, 17.5 s each), and the routes already built. The categories are theorized, not measured: no real user question log exists yet. The first thing to measure is how often each one is asked (section 5).

## 1. Where lore text lives

Trevor indexes the story scripts (13,837 chunks) and the operator archives (the handbook, 519k words of prose, 1,925 chunks). Everything below is prose in the EN game data that Trevor has not indexed, counted as unique strings of 8 words or more:

| table | words | what it holds | lore value |
|---|---|---|---|
| `charword_table` | 314k | voice lines (11,081) | personality, relationships, reactions; asked for directly ("what does X say") |
| `uniequip_table` | 259k | module stories (1,731) | new backstory per operator, often told nowhere else |
| `roguelike_topic_table` | 181k | Integrated Strategies: encounters, endings, relics, notes (8,001 texts; 1,703 ending texts; 222 mention Sami) | IS2 to IS6 lore in the game's numbering (IS1, Ceobe's Fungimist, is CN only and untranslated in the data); IS4 is set in Sami |
| `stage_table` | 120k | stage descriptions | light |
| `activity_table` | 82k | event mechanics and blurbs; includes "Ceobe's Fungimist" | light to medium |
| `enemy_handbook_table` | 68k | enemy entries (2,718) | who a boss is, faction lore |
| `medal_table` | 53k | medal flavor text | light |
| `retro_table` | 46k | retro event descriptions | light |
| `skin_table` | 43k | skin descriptions (1,426) | small lore bits, often asked about |
| `item_table` | 31k | item flavor text | world details (currencies, foods, tech) |
| `sandbox_perm_table` | 25k | Reclamation Algorithm | its own lore |
| `climb_tower_table` | 12k | Stationary Security Service | its own lore |

About 1.2M words of lore-bearing text are outside the index, the largest pieces being voice lines, module stories and IS.

## 2. Categories

Each row: example questions, where the answer lives, what Trevor does today (the probe result where there is one), and what it needs.

| # | category | examples | answer lives in | today | needs |
|---|---|---|---|---|---|
| A | who or what is X | Who is Kal'tsit? What is Rhine Lab? | archives, dossiers, story | good (dossiers, archives; SilverAsh and Cliffheart answered "siblings" from the archive) | fine |
| B | what happens in a story | What happens in Episode 8? What did I miss in Lone Trail? | summaries, story | good (summary bank, retrieval) | fine |
| C | fate and status | Is Talulah dead? Does W die? Is Mon3tr alive? | story, archives | death events route; Mon3tr answered well from her archive | status of the living ("is X still at Rhodes Island"), as of the reader's progress |
| D | relationships | How are SilverAsh and Cliffheart related? Who raised Ch'en? | archives, story, voice lines | good when an archive states it | a relationship table would answer "all of X's family" |
| E | identity and aliases | What is Nearl's real name? Who is the Priestess? | archives, story | real-names table (121), identity links | reveals are spoilers: gate by the revealing story |
| F | time and order | When is IS3 set? Was X before Y? | chronology, story | chronology routes; IS absent | IS placement once IS is indexed |
| G | power and ranking | Who is stronger, Ch'en or Talulah? Who is the strongest character? | story, archives, fandom | probe: Ch'en vs Talulah gave "not stated definitively" plus the evidence (good); "strongest" declined | a stated policy: evidence both ways, no invented ranking |
| H | worldbuilding | Why is Originium dangerous? How survivable is Sami? How do Arts work? | story, archives, items, IS, enemies | probe: Originium good; Sami good from 2 stories (act15mini, Młynar's record), without IS3 | IS, enemies, items for region and faction questions |
| I | false premise | When did Ceobe get high on drugs? Why did Amiya die in Episode 8? | story (the true version) | probe: Ceobe declined, although the answer is in the corpus (section 3) | premise check and query rewriting |
| J | source-specific | What does Kal'tsit's module say? What does Exusiai say when poked? What does the X skin description say? | modules, voice lines, skins | probe: module question answered from story passages as if they were the module (silent substitution); voice line declined | index these sources with a source type; say when a named source is not indexed |
| K | canon and meta | Which IS3 endings are canon? Is IS canon? Is the anime canon? | out of game, or partly in later story | probe: declined, but cited an unrelated passage | a stated policy: the game does not rule on canon; report what later stories reference |
| L | roster and aggregates | Operators from Kazimierz? Tallest Sarkaz? Real names of everyone? | game data tables | routes built (attributes, real names, deaths, listing) with a 26-question route check | fine; extend fields as asked |
| M | speculation and theory | Is the Doctor a Precursor? Who is Amiya's father? | story hints | untested | label what is stated vs inferred (the synthesis instruction exists) |
| N | where to read | Which story explains the Sarkaz? Where does Lappland first appear? | story index, speaker index | retrieval citations only | a first-appearance table from the speaker index (5,911 speakers) is cheap |
| O | spoiler-bounded | I finished Episode 6, who is Patriot without spoilers? | primers, gates | primers exist; `ask` takes no reader progress | the serving gate; `ask` could take a progress flag for testing |
| P | out of scope | Is Mudrock good? Best team for IS3? | not lore | untested | redirect to myrtle's operator tools |

## 3. The three seed examples

- **"When did Ceobe get high on drugs?"** The premise is off, and the true version is in the corpus: in *The Great Chief Returns* (act12d0) Ceobe eats wild mushrooms and hallucinates. Gavial: "There's a hell of a lot of mushrooms and fruits growing wild along the road here, and more than most of them you can't eat. If you do, you end up like that." Then Ceobe: "I found it! The Mushroom Sea! Bath time!" `ask` declined ("I could not find it"): "drugs" and "high" do not retrieve a scene worded around mushrooms and poison. A separate activity, "Ceobe's Fungimist", exists in `activity_table`, and its text is not indexed. Needs: a premise-check step that rewrites the question into what the story would say ("Ceobe mushrooms hallucination") before retrieval, and that says "not drugs: mushrooms" in the answer.
- **"How survivable is the region of Sami?"** Answered well from two stories: "savage, inaccessible, and frigid", supplies are hard to restock, the Sami kill outsiders, "demons and a vengeful spirit known as the Witch". IS4 (set in Sami) would add most of the regional lore; 222 IS texts mention Sami and none are indexed.
- **"What endings in IS3 are canon?"** Declined, correctly, but the answer cited an unrelated passage about "a few versions of the ending" of a folk tale. The game text does not rule on canon; a good answer says so, lists the endings from the IS text, and names any later story that references one. Needs IS indexed and a canon policy line in the prompt.

## 4. Behaviors the categories call for

1. **Premise check** (I): before retrieval, have the generator check whether the question presumes something and rewrite it into the story's own terms; answer by correcting the premise, with a citation. The probe shows retrieval alone misses it.
2. **Source honesty** (J, K): when a question names a source Trevor has not indexed (module, voice line, skin, IS, enemy file), say so instead of answering from other passages. The module probe is the worst failure seen here: fluent, cited, and not from the source asked about.
3. **Canon and ranking policy** (G, K): state what the text says on each side, never invent a ranking or a canon verdict, and say that the game does not rule.
4. **Stated vs inferred** (M): the synthesis instruction already asks for it; untested on theory questions.
5. **Scope redirect** (P): gameplay questions go to myrtle's tools, not the lore model.

## 5. Next, in order

1. **Measure the question mix.** A gold set by category (10 to 15 questions each for A to P, about 180), part written by hand or taken from real player questions (Discord, Reddit), part generated, each with the source it needs. Scoring `ask` per category turns this theory into numbers and shows the weakest classes. Real player questions are worth more than generated ones.
2. **Index the missing sources as typed units** (a source type on every chunk, so "module story" questions can filter to modules and the answer can name its source): IS (181k words, asked about directly), module stories (259k), enemy handbook (68k), voice lines (314k, short texts, best as one unit per operator), then skins and items. Each needs a spoiler gate (IS by run content, modules by the operator's promotion, voice lines by trust unlocks).
3. **Premise check and source honesty** in `ask`, measured on categories I and J of the new gold set.

## 6. Measured: 883 real questions from r/arknights (2026-09-28)

Source: every Lore-flaired r/arknights post from 2021-04 to 2026-09 (3,682), fetched through the Arctic Shift archive of Reddit's data dumps (reddit.com itself was blocked from the sandbox); the 883 whose title is a question. A model read and classified each one (no keyword rules) into the section 2 categories plus four the theory missed: R races and biology, Q visual and design (needs the art), S cross-media and real world (Endfield, anime, real-life inspirations), X noise. Each also got the source that could answer it. Files: `eval/reddit-lore-questions.all.jsonl` (all 883), an earlier keyword-labelled sample of 222 (`eval/reddit-lore-questions.jsonl`, removed 2026-10-17: no code read it, and 219 of its 222 questions are in the full file). Spot check of 25 labels: about 22 right (misses: "What is Nine's and Ch'en's relation?" labelled cross-media, relationship is right; "Mostima lore?" labelled power). Caveat: post titles only, no comments; a Lore flair skews toward curious long-form questions, and a site search box will get shorter ones.

After 120 noise posts (thirst, memes, polls, deleted) and 6 gameplay questions, 757 real lore questions:

| group | questions | share | answerable from the story and archives |
|---|---|---|---|
| H worldbuilding (116) + R races and biology (76) | 192 | 25.4% | 161 |
| A who or what (71), B plot (46), D relationships and motives (39), F time (22), C fate (19), E identity (14) | 211 | 27.9% | 181 |
| M speculation and opinion (69) + G power (50) | 119 | 15.7% | 23 (G only) |
| N reading order and where to read | 65 | 8.6% | labelled external, but Trevor's chronology and release data answer most |
| Q visual (38), S cross-media (38), K canon and translation (36) | 112 | 14.8% | about 0 |
| J source-specific text (24) plus other-game-text elsewhere (16) | 40 | 5.3% | 0 until IS, modules, voice lines are indexed |
| L roster (23), O spoiler-bounded (7), I false premise (4, plus 10 as secondary) | 34 | 4.5% | tables and story |

What this changes:
- **Worldbuilding is the largest class** (a quarter): "Can Sankta fly?", "Does Terra have tanks?", "How fast are mobile cities?", "Do we know how much of Terra's population is infected?". The answers are scattered across many passages, which suits topic summaries (a dossier per race, nation and concept, built like the character dossiers) better than 10 retrieved passages.
- **Reading order is 8.6%**, with a wave of Endfield newcomers from 2025 on ("where do I start", "when do I read Babel", "is the old cast dead in Endfield"). Trevor already holds the release order, the in-world timeline and the primers, so a reading-guide route is cheap.
- **Opinion and power questions are 16%.** No fact answers them; the policy line (what the story shows on each side, no invented ranking) matters for 1 in 6 questions.
- **Art, cross-media and canon questions are 15%** and are mostly outside any game text: the answer should say what the text does and does not cover instead of declining flatly or, worse, answering from unrelated passages.
- **Other game text is 5%**, less than section 5 assumed: indexing IS, modules and voice lines moves down the list. **False premises are rare** as titles (under 2%): the Ceobe example is a real failure mode but not a common question.

Revised order of work: (1) score `ask` on a sample of these real questions per category; (2) a reading-guide route; (3) topic summaries for the most asked races, nations and concepts; (4) the opinion, canon and scope policy lines; (5) IS, modules and voice lines.

**Reading-guide route: data gap (2026-09-28).** The EN game data dates main Episodes 10 to 14 (`zone_table.MainlineAdditionInfo.ZoneOpenTime`, 2022-10-19 to 2024-10-31) and every event (release times in `spoiler.jsonl`), but gives -1 for Episodes 0 to 9, so events cannot be interleaved with the first ten episodes from the game data alone. `spoiler.jsonl.requiredStages` lists each event's own stages, not main-story prerequisites. What exists: in-world order (chronology v1), release order of events, and each event's primer (the earlier groups it builds on). A full release-order guide needs the Episode 0 to 9 EN dates from outside the game data (the wiki), which is a decision about sources (the wiki is not a serving source).

## 7. Measured: 345 questions from the Discord lore channel (2026-09-28)

Source: an export of the CN-news and lore channel (`eval/discord-lore.json`, a local file not in git; 26,705 of 26,706 messages recovered from a truncated file, 2024-03-06 to 2026-01-14, 113 authors). A model read all 2,163 messages containing "?" with 3 messages before and 2 after, kept 345 genuine lore questions (about 15%; the rest were banners, kits, pulls, leak reactions, jokes), and rewrote each minimally to stand alone ("is [Ulpianus] actually dead?"). No usernames in the output: `eval/discord-lore-questions.jsonl`. The raw export (usernames, user ids, avatar links) was deleted on 2026-09-29 for privacy; only the extracted questions remain.

| category | n | % | Reddit % |
|---|---|---|---|
| A who or what | 42 | 12.2 | 8.0 |
| R races and biology | 39 | 11.3 | 8.6 |
| B plot | 30 | 8.7 | 5.2 |
| S cross-media (almost all Endfield continuity, late 2025) | 29 | 8.4 | 4.3 |
| D relationships and motives | 28 | 8.1 | 4.4 |
| H worldbuilding | 26 | 7.5 | 13.1 |
| C fate and status | 25 | 7.2 | 2.2 |
| M speculation, N reading order | 18 each | 5.2 each | 7.8, 7.4 |
| E identity, F time | 16 each | 4.6 each | 1.6, 2.5 |
| T upcoming content and CN news (new) | 13 | 3.8 | (none) |
| K canon, J source text, Q visual, G power, L roster, I false premise | 12, 10, 10, 6, 4, 3 | | |

Answerable: story 203 (59%), external 52, CN-only 39, art 18, other game text 17, opinion 16; 59 (17%) are about content that was on CN but not EN when asked (chapter 15, anniversary streams, IS6). The EN corpus is newer than the chat (EN game data through 2026-09 includes Episodes 15 and 16), so part of the CN-only share is answerable now.

What the two sources agree on and where they differ. Discord asks while reading ("X died?", "who is this?", "this is before Babel, right?"): who, plot, fate and relationship questions run higher than on Reddit, and power scaling nearly disappears (1.7% against 5.7%). Reddit asks big-picture questions: worldbuilding and speculation run higher. Races and biology are high in both (9 to 11%), clustered on specific cases (Sankta halos, Beast Lords, sheep versus goat). Cross-media (Endfield) is 4 to 8% and rising. Other game text stays small in both (3 to 5%). Across the 1,102 real questions (757 Reddit, 345 Discord), 592 (54%) are answerable from the story and archives Trevor holds, and the rest need a policy (opinion, canon, art, cross-media) or sources Trevor does not have.

## 8. Scored: `ask` on 120 real questions (2026-09-28)

`scripts/real-probe.py`: a stratified sample of the Reddit set (at least 5 per category, no noise or gameplay; `eval/real-probe.jsonl`), answered by `ask` on P3a with Gemma (22.6 s each), judged by Qwen for behavior, not correctness (real questions have no reference answers): **responds** (addresses the question, even partly), **faithful** (answer sentences supported by the passages given), and for opinion, art, cross-media and canon questions **handles** (no invented verdict, says what the text covers).

| category | n | responds | faithful |
|---|---|---|---|
| C fate, D relationships | 6, 6 | 0.83, 0.83 | 1.00, 0.87 |
| G power, H worldbuilding | 7, 10 | 0.71, 0.70 | 0.75, 0.75 |
| E identity, R races, A who/what | 6, 11, 8 | 0.67, 0.64, 0.62 | 0.73, 0.83, 0.96 |
| M opinion, L roster, O spoiler-bounded | 8, 5, 5 | 0.62, 0.60, 0.60 | 0.69, 0.84, 1.00 |
| F time, I false premise | 7, 4 | 0.57, 0.50 | 0.82, 0.67 |
| Q visual, S cross-media | 7, 5 | 0.43, 0.40 | 0.58, 0.88 |
| B plot | 6 | 0.33 | 0.82 |
| N reading order, K canon, J source text | 7, 5, 7 | 0.29, 0.20, 0.14 | 0.58, 0.33, 0.86 |
| **all** | 120 | **0.550** | **0.806** (208/258) |

Handles: 42/43 (mostly by saying the text does not cover it). On the 68 questions labelled answerable from the story, `ask` responds to 45 (0.66); most misses are declines where the answer exists ("Can someone explain the ending of Invitation to Wine?", "What was Manfred's fate after the war?", "Can Aegir breathe under water?"). Faithfulness is lower than on gold set v2 (0.806 against 0.971): real questions ask for synthesis across passages. Plot recaps at 0.33 point at the corpus: the probe ran on P3a, which holds no event summaries or dossiers (they are in P3b); a paired P3b run is next.

**Routing on real phrasing.** 7 of the 120 were answered by a table route and 6 of those were misfires: keyword substrings ("explanation" contains "nation": Mostima's affiliation answered "why does Mostima treat Exusiai the way she does"), and questions that only mention an operator ("Do we know how Amiya got infected?" answered with her infection status). Single-subject routes now need whole-word keywords and no content words beyond the subject, keyword and question words; the event death list the same; death events named "He", "Dad", "Dead", "One" and the like are dropped (550 to 532 named). Swept over all 1,102 real questions with `ask --route-only`: 9 table answers before the event-list guard (3 wrong), 5 after, all right ("Did Allerdale die?", "How tall is Texas?", "specter' oripathy ?", "Jie died?", "Is there a canon list of deaths for RI operators?"). The route check holds 39 questions, 39 pass.

**P3b paired (same 120 questions, `ask --corpus artifacts/p3b`, event summaries and dossiers in the index).** Responds 0.550 to 0.575 (4 gained, 1 lost; +0.025, bootstrap 95% CI [-0.008, +0.067]); on the 68 story-answerable questions 45 to 48; plot recaps 0.33 to 0.50; races 0.64 to 0.73; power 0.71 to 0.86; identity 0.67 to 0.83; visual 0.43 to 0.29; faithfulness 0.806 to 0.826; handles 42/43 to 43/43. Positive and not significant, the same direction as gold set v2 (answers +0.023, retrieval -0.015). The same 7 table answers appear in both runs because both used the binary from before the routing fix.

**Where this leaves the priorities.** (1) Real questions answerable from the story are answered 0.66 to 0.71 of the time; the misses are retrieval misses on synthesis questions (endings, fates, race traits), which is where summaries helped. (2) Reading order (0.29), canon (0.20) and source text (0.14) are the weakest and map to the reading-guide route, a canon policy line, and indexing IS, modules and voice lines. (3) Worldbuilding and races (a quarter of real questions) sit at 0.70 to 0.73, better than feared.

## 9. Built for the weak categories (2026-09-28 night)

Requirement: improve the reading order, canon, source-specific questions and plot recaps, with P3b as `ask`'s default and a reading guide from the wiki's episode dates. The research survey (`design/archive/trevor-improvement-research.md`, 12 experiments ranked by evidence) put the reading guide and the policy lines first and fine-tuning last.

- **`ask` defaults to P3b** unless `--corpus` is given (search and eval keep P3a).
- **Reading guide** (`scripts/reading_guide.py`): EN release dates of Episodes 5 to 16 from the wiki's Main Theme act pages (Episodes 0 to 4 at the EN launch, 2020-01-16; the wiki equals the game data on 5 of 5 dated episodes, 10 to 14), events from the game data, 85 entries. Each event lists the earlier stories it builds on (its primer's prior context) and the later ones that build on it; the most built-on events are Heart of Surging Flame (22 later stories), Code of Brawl (19), Operational Intelligence (16), Stories of Afternoon (16), Grani and the Knights' Treasure (14). Routes: the whole guide ("Recommended reading order?"), one event ("When should I read Babel?": between Episodes 13 and 14, what it builds on, what builds on it, and its primer), reading time (Lone Trail 83,375 words, about 5.6 hours), and first appearance by speaker (4,310 speakers; "When do I get to the part where Priestess shows up?": Lone Trail). Of the 83 real reading-order questions, 27 route to the guide; most of the rest ask for videos, manga or summary sites. In-world timing wording ("placed before", "was before", "timeline") stays with retrieval.
- **Recap route**: "What happens in X", "explain the ending of X" answer from the event summary (and the last story's summary for an ending) when the question names nothing beyond the event.
- **Policy lines**, added only when the question calls for them: canon and translation (the text does not rule on canon), opinion and ranking (evidence on each side, no verdict), and source questions (say when no passage comes from the source asked about). Passages are now labelled by source kind (story, operator file, Trevor's dossier or event summary, and the P4 kinds).
- **P4 typed sources** (`scripts/sources.py`, `build-units`): 4,271 units, 960k words, packed into 6,347 chunks appended to P3b: voice lines (413 operators, 1,307 chunks), module stories (467 modules, 928), Integrated Strategies 2 to 6 (the game's numbers) (encounters, endings, collectibles, operations, floors; 746), enemy entries (1,533), outfits (474), items (1,359). A question naming a source kind gets the 3 best chunks of that kind first.
- **Decline retry** (`--no-retry` off): when the first answer declines, Gemma rewrites the question into the story's own words, correcting slang or a wrong premise, and `ask` answers again from the new retrieval; the first answer stands if the retry also declines.
- **Route check**: 57 questions, 57 pass, including every misfire found on real phrasing.

## 10. Reading order as players recommend it, bank coverage, the retry (2026-09-29)

**What players recommend** (`design/trevor-reading-order-research.md`: 57 story-order threads and a 1-in-5 sample of 145 weekly question threads from the Arctic Shift archive of r/arknights, about 250 comments read, the wiki timeline and 7 community guides). Release order is recommended by about 40 commenters; main story first with a few must-read side stories by about 12; one storyline at a time by about 10 plus the most-linked 2025 to 2026 guide and the game's own Movements view (default since September 2025 on EN); in-world chronological order by about 3, with at least 6 advising against it because Babel, Darknights Memoir and A Walk in the Dust come first in the timeline and spoil the Doctor's amnesia. The side stories named essential: Darknights Memoir (31 commenters), Babel (14), What the Firelight Casts (12), A Walk in the Dust (11), Lone Trail with Dorothy's Vision (10), Vigilo (8). The top placement rules: Darknights Memoir after Episode 6 (31), Babel right after Episode 13 (14), A Walk in the Dust after Episode 8 (11), Under Tides before Stultifera Navis (11), Dorothy's Vision before Lone Trail (10).

Trevor's own ranking of "most built-on" events (shared recurring cast: Heart of Surging Flame 22, Code of Brawl 19) did not match what players call essential, so it is now a secondary line. The curated layer (`data/reading_community.json`: 10 essential events and 18 placement rules with commenter counts, all matched to Trevor's event ids) leads the general answer and adds the players' placement to one-event answers ("When should I read Babel?" now says right after Episode 13, 14 commenters, and 4 say any time after Vigilo). A question about reading in chronological order gets the in-world order with the reason most players advise against it, and the pre-Prologue prequels placed as the community and wiki place them (A Walk in the Dust 1070s to 1080s, Babel 1090 to 1094, Darknights Memoir about 1094 to 1096), since Trevor's timeline puts flashback events at their release position (Babel at about 1100). Route check 61/61.

**The answer bank barely matches real phrasing.** The 1,102 real questions against the bank's 3,395 questions and aliases (gte cosine): 6 at 0.95 or above (0.5%), 26 at 0.90 (2.4%), 95 at 0.85 (8.6%), and matches at 0.85 to 0.90 are often wrong ("What's Zuo Le's mother's name?" matched "Who is Zuo Le?" at 0.896). The seeded bank ("Who is X", "What happens in Y") is not how players ask; for serving, retrieval-and-answer quality matters far more than bank size, and a bank would have to be built from real question clusters with a strict threshold.

**The decline retry, first version: refuted.** A query rewrite by Gemma, then a normal retrieval: on the 120 real questions 36 answers declined, the retry ran on each (27.8 s per answer against 21.8 s without) and answered none of them. Gemma cannot restate a question in the story's terms without knowing the story ("When did Ceobe get high on drugs?" became "Ceobe drug use event"), as the research survey warned for query expansion (Gemma answers 13% of gold questions closed-book). The second version keeps the rewrite as extra terms but widens retrieval for the retry (16 passages, 2 neighbours, a 14,000-token budget); measured on gold set v2 below.

## 11. Final evaluation of the night's changes (2026-09-29)

Configuration: P4, every route and policy line on, the retry on (rewrite only for the real probe, rewrite plus wide retrieval for the gold set, because `ask` was rebuilt between the two batches). Baselines: P3b with the code before the night's changes (real probe `real-probe-p3b`, gold `gs2_v4`).

| measure | baseline (P3b, old code) | final (P4, new code) |
|---|---|---|
| real probe responds (120) | 0.575 | 0.583 |
| real probe faithful | 0.826 | 0.837 (242/289) |
| real probe handles (policy) | 43/43 | 42/43 |
| gold v2 correct (131) | 84 (0.641) | 83 (0.634); paired 3 gained, 4 lost, -0.008 [-0.046, +0.031] |
| gold v2 declined answerable | 18 | 11 |
| gold v2 unanswerable declined | 15/15 | 15/15 |
| gold v2 faithful | 0.971 | 0.946 (209/221) |

Real probe by category (responds, baseline to final; n in brackets): plot recaps 3 to 5 (6), source-specific 1 to 3 (7), reading order 2 to 3 (7), visual 2 to 3 (7), time 4 to 5 (7); power 6 to 4 (7), identity 5 to 4 (6), roster 3 to 2 (5), spoiler-bounded 3 to 2 (5), false premise 2 to 1 (4); the rest unchanged. At n of 4 to 11 per category these moves are within noise (2 of 7 has a 95% interval of about 0.08 to 0.64); only the direction of the targeted categories is informative. The real probe routed 4 of 120 questions to tables; the reading-guide gains show in the routing sweep (27 of 83 real reading-order questions answered from the guide, section 9) more than in this sample of 7.

Decisions:
- **The retry is off by default** (`--retry` turns it on): 0 of 36 declines answered on the real probe, 0 on gold set v2 in its wide form, about 6 s more per question.
- **`ask` uses P3b, and P4 only for a question naming a source kind** (module, voice lines, outfit, IS, enemy, item): P4 cost 0.008 correct and 0.025 faithfulness on gold set v2, where its extra chunks crowd out story passages, and helped the source questions it was built for. Batch runs stay on P3b unless `--corpus` is given.

**The shipped default on gold set v2, and a larger passage budget (2026-09-29).** P3b with all the night's code, 8 passages (the default): 84/131 correct (0.641), identical to the P3b run before the night's changes, 18 answerable declined, unanswerable 15/15, faithfulness 0.963, 19.3 s per answer; the routes and policy lines cost nothing on the gold set. 12 passages with a 13,000-token budget (the research survey's first retrieval experiment): 83/131 (0.634), gold chunk among the passages 109 against 106, declines 13 against 18, faithfulness 0.964, 29.7 s per answer. More passages reached the model and fewer answers declined, but correct answers did not rise, at 54% more time: refuted, `ask` keeps k 8.

## 12. Experiment log (moved)

The dated experiment log that used to be section 12 (tools instead of keyword routes, 2026-09-29 onward) and section 13
(which model answers each live rung, 2026-10-16) is now [experiment-log.md](experiment-log.md), organised by topic.
