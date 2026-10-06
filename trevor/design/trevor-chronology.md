# Trevor: in-world chronology (2026-09-26)

Ian's requirement: chronology means in-world order, not release order, and Trevor should work it out, framing the timeline on explicitly stated years, so that it can answer "what is the timeline history of Terra?" and "was this event before or after that one?". Code: `scripts/chrono.py` (stages `cues`, `extract`, `events`, `exceptions`, `place`, `eval`); data in `artifacts/chrono/`.

## 1. What the text offers

Explicit years are rare: a Terra year (1000 to 1110) appears in 80 of 1,861 stories, a month-and-day date in 42, "N years/months ago/later" in 455. Across stories and the operator archives (`handbook_info_table`, 888k tokens Trevor does not index yet) there are 197 lines holding a number that could be a year. The wiki's dense timeline is mostly derived from relative statements and cross-referencing, not read off stated years.

## 2. The answer key (eval only, never shown to a model)

The Arknights Terra Wiki timeline (https://arknights.wiki.gg/wiki/Arknights_Timeline, fetched through its API), parsed into 290 rows mapping dated entries to our groups and stories. T1: 16 groups with an explicit "the event during which X takes place" entry; T2: 22 more cited under exactly one year (main chapters excluded: their citations are mostly flashbacks, e.g. Episode 10 cited for 1072). Groups cited under several years (8, e.g. `act18d0` under 1075, 1080, 1084) are excluded as ambiguous. It is a fan reference: a yardstick, not truth.

## 3. Release order is the baseline to beat

On year-distinct pairs, release order is right for 83 of 87 T1 pairs (0.954) and 480 of 578 T1+T2 pairs (0.830). So release order is not chronology, but it is a strong prior for the 1097 to 1102 storyline. Its errors are mostly events released early but set late (the Yan events `act15side`, `act16d5`, `act23side`, `act31side`, `act40side` are dated 1101 to 1102 by T2). The reference barely covers the cases where release order is structurally wrong: flashback stories, prequels, historical side stories and operator records.

## 4. Explicit-year Terra events: built, and it holds up

Stage `events`: each of the 197 candidate lines (139 story, 58 archive) goes to Gemma 4 12B under a grammar: is the number a calendar year, what happened in that year per the line, month or season, and whether the line dates the scene's own present (a caption, log or letter). A year not present in the line itself is rejected (0 were). Result: 153 calendar years; counts, issue numbers and prices correctly rejected ("998... 999... 1000", "Issue 1051", "471 LMD"). 91 are scene captions, several of them flashback scenes.

Against the wiki: 75 of the 81 events before 1097 (0.926) fall in a year the wiki also has entries for; the other 6 read as genuine minor facts the wiki lacks (two births, a painting from 580, the Feranmut detected near Yumen in 1087). Coverage is the gap: 38 distinct years before 1097 against the wiki's 115.

## 5. Story and event placement: what was tried

**Temporal fact extraction** (stage `extract`, 1,106 stories: the reference groups and all main chapters): per story, the speaker lines holding a time cue plus one line either side (1.28M tokens corpus-wide, median 531 per story, 183 stories have none), and Gemma lists facts (date, relative, flashback, age) with verbatim quotes. 1,986 verified facts from 702 pilot stories (1,397 relative, 440 flashback, 121 date, 28 age), 27 rejected as not verbatim, 0 unparsed. Throughput 2.6 to 5.0 s per story.

**Explicit present-year majority** (no model): places 6 of 38 reference groups, 5 at the exact year; pairwise 9 of 14.

**Model placement v1, refuted.** Gemma given 7 script-dated reference points plus each group's summary and facts: placed 22 of 38, 12 exact, pairwise 0.661 (release order 0.830). Failure modes, all inspected: historical years read as the present (a song "Lied des klaren Himmels (1078)", an inscription "Giovanna and Cellinia, 1080", a flashback caption "Autumn 1019"); errors propagating through anchors (one event's flashback year 1062 accepted as its present, two others then placed relative to it); 8 of 15 main chapters unplaceable, leaving 36 of 52 groups with no basis.

**Backbone** (no model): a monotone year-by-release fit through groups' latest dated scene caption, rejecting anchors more than 3 years under the trend. It correctly rejects the flashback-only captions (`act28side` 1019, `act30side` 1072, `act40side` 1062, `act33side`); operator records were first used as anchors and removed, since a record's release date says nothing about when it is set. 7 event anchors remain (1097 to 1100). Ordering equals release order by construction (0.830, 0.954 on T1); exact year 9 of 38 before the exception pass.

**Exception pass, refuted.** Gemma told the storyline year at an event's release and asked whether the event's own present is that period, clearly earlier, or clearly later: it answered "earlier" for 65 of 76 events (10 storyline, 1 later), including plainly present-day ones (`1stact`, `act12d0`), and again took flashback or quoted years as the present (`act26side` 1011, `act28side` 1019, `act40side` 1074). Pairwise falls to 0.676 (T1 0.713). A first run was also polluted by the record anchors (backbone 1098 at 2020 to 2021 releases) and discarded.

## 6. Where this stands

- Built and trustworthy: the explicit-year Terra events (153, 0.926 year-corroborated), the verified temporal facts, and the backbone ordering, which is release order with years attached.
- Not solved: identifying which events and stories break release order. Gemma 4 12B does not make that judgment reliably from summaries and facts, under two different framings.
- Not yet measured: story-level flashback detection, which is where release order is structurally wrong, and the derived-years layer (relative facts in dated stories).

## 7. Chronology v1, shipped as data plus a CLI (2026-09-26)

Ian's call: release order as the starting point, the AI finds the exceptions, and explicitly stated years frame the timeline. `python3 scripts/chrono.py build` writes `artifacts/chrono/timeline_v1.json`: 437 groups at their backbone storyline year (7 anchored by their own dated captions, the rest by release position), each with its dated lines, plus 153 explicitly dated history lines. `history` writes `terra_history.md` (the Terra history from stated years, by century, captions marked). `order "A" "B"` answers before or after, citing both sides' dated lines and saying whether the answer rests on dated evidence or on storyline position. Ordering accuracy on the reference: 0.830 (T1+T2), 0.954 (T1), identical to release order by construction. Known label error: "1062 (scene dated): Blaze was born" is history, not a caption, so the caption flag is not fully reliable.

## 8. Exceptions with reasoning: better, still not moved by default

`scripts/chrono_exceptions_reasoning.py`: the same exception question on Qwen3.5 9B with thinking on (`--reasoning on --reasoning-budget 2048`), asked only of the 6 groups whose own dated evidence disagrees with the backbone by more than 3 years, plus the 16 T1 groups as controls. 96 s per group, 0 unparsed.

- Controls (all present-day): 13 of 16 called storyline, against 2 of 16 for Gemma without reasoning. The three misses (`1stact`, `act12side`, `act47side`) all justify themselves with outside lore ("a prequel", "generally established in lore") and cite no dated line.
- The v1 failure cases are fixed: the song (1078), the inscription (1080) and the construction date (1011) are read as mentions of the past.
- Guard: an exception counts only when the year it gives is stated by a dated line of that group. It drops all three lore-based calls (controls 16 of 16) and keeps three: `act33side` (Babel) at 1094, plausibly right (the wiki cites it only under 1086 to 1094), and `act28side` at 1019 and `act30side` at 1072, both flashback-only captions and likely wrong (the wiki has `act28side` in 1099).
- Moving the kept exceptions costs 11 reference pairs: T1+T2 0.830 to 0.811, T1 unchanged at 0.954, all from `act28side`.

Ruled out by measurement: moving exceptions by default. The unsolved case is an event whose only dated scene is a flashback; a rule that one caption cannot establish the frame story would be fitted to three examples, so it is not built. Instead the kept exceptions are annotations (`candidateException` in the timeline), and `order` answers lead with the dated scenes, give the storyline-position order when it differs, and say the answer is uncertain.

## 9. Story-level flashback tags

`chrono.py flashbacks`: a story "depicts" the past when it has a scene caption dated more than 3 years before its event's storyline year, and "recounts" it when it has only flashback or past-dated facts. On the 1,106 extracted stories, against the wiki's story-level citations: of 60 stories cited for a year well before their storyline, 41 are tagged (0.683), 12 as depicts; of 68 cited near their storyline year, 0 are wrongly tagged depicts. The misses are mostly deep-history citations (690, 730, 890, 980) where the text holds no time cue and the wiki's year comes from elsewhere.

Full run (2026-09-26): temporal facts for all 1,861 stories (the last 755, mostly operator records, in 4,807 s at 6.4 to 9.2 s each): 4,913 verified facts (3,295 relative, 1,216 flashback, 325 date, 77 age), 57 rejected as not verbatim, 0 unparsed. Tags over all stories: 20 depicts, 588 recounts, 1,253 none. Against the wiki: of 76 stories cited for a year well before their storyline, 48 tagged (0.632), 12 as depicts; of 91 cited near their storyline year, 0 wrongly tagged depicts. Recall fell from 0.683 on the first 1,106 stories as the added stories brought more deep-history citations with no cue in the text. Output: `artifacts/chrono/flashbacks.jsonl`.

## 10. Commands and what is next

```sh
python3 scripts/chrono.py cues|extract|events|flashbacks|build|history|eval    # extract/events need Gemma on :8081
python3 scripts/chrono.py order "A Light Spark in Darkness" "Lone Trail"
python3 scripts/chrono_exceptions_reasoning.py                                # needs Qwen with --reasoning on
```

Next, in order of value: the derived-years layer (relative facts in stories whose present is dated, turned into years by arithmetic; this is where most of the wiki's dates come from); folding story-level flashback tags into `order` answers; and the operator archives (888k tokens) as a lore source in P3, since 58 of the 197 dated lines are there.

## 11. Derived years (2026-09-26 night)

`chrono.py derive`: of 4,511 relative and flashback facts, 673 carry a computable "N years/decades/centuries ago/later/before/after/since"; the scene's present is the nearest dated caption before the quote in the same story (an "ago" inside a flashback counts from the flashback), else the event's dated anchor, else the storyline estimate. Vague amounts ("a few years") are skipped, as are quotes that state their own year (3; the first run derived "Four Years Ago... Summer, 1090" as 1086, a bug) and 31 with no present year. Result: 548 derived years (22 from a dated scene, 84 from an event anchor, 442 from the storyline estimate).

Measuring them. Coarse ("the wiki has an entry in that year") is uninformative: a placebo with every year shifted by 3 still matches 0.705 to 0.767, against 0.773 (scene), 0.907 (anchor), 0.756 (storyline estimate). Semantic: for each dated event, wiki entries within 10 years that share a proper name are candidates, and Qwen judges whether one describes the same event; then years are compared. The instrument on explicit years: 17 matched, 14 same year (0.824), 15 within a year. On derived years it matched 14, but on inspection 6 of those are the judge pairing different events ("Reunion Gododdin stomped out" with the Chernobog crisis, "wrong about you six years ago" with the Snowcap Incident) and 1 is the derivation bug above. The 6 genuine matches all agree on the year: the Sami prophecy 1095, the Sargon-Minos truce 1095, the Kazdel siege 898 ("two centuries ago"), the occupation of Londinium 1094, the Texas purge 1092, Londinium's walls 1028 ("almost seventy years ago"). Six is a small sample; it says the arithmetic is sound when the base year is, not that every derived year is right.

`terra_history.md` now carries 701 dated lines (153 stated, 548 derived), each derived line saying what it was counted from and at what confidence. Many derived lines are personal ("five years since I last went home"): correct in kind, minor in weight.

`order` answers now also note an event's scenes dated well before its storyline position and how many of its stories recount the past without dating it.

## 12. Derived years after Ian's review (2026-09-27)

Ian left the 8 derived-year items blank and asked for verification against the wiki. Checking each against the wiki timeline: 2 right (the Sami prophecy, 1095; "that night twenty-three years ago" in a Leithanien story, 1077, matching "Die Septemberaufstand, September 1077"), 1 likely right (1079 in a Columbian story; the wiki has a Columbian 1079 event), 1 wrong (#6), 4 not on the wiki.

Two defects found and fixed:
- **Compound numbers.** "twenty-three years ago" was read as three years ago (item #2 said 1097). The parser now reads tens-and-units first. 38 of 546 derived years changed, all compound numbers (for example "twenty-six years ago" in a 1098 scene, 1092 to 1072); 3 rows dropped, 505 unchanged.
- **Round amounts are not offsets.** "A century ago, the Siracusans ... turned their backs on the Güldenesgesatz" derived to exactly 1000; the wiki dates Siracusan independence to 967 to 969. Centuries, decades, round tens of 30 or more, and hedged amounts ("almost", "about", "over") are now marked approximate and shown as "ca." (185 of 545). The history now shows "ca. 1000" directly under the explicit 969 line, so a reader sees both.

## 13. More anchors (2026-09-28): refuted

Ian asked whether more anchors would make the years more accurate. Measured against the wiki reference, present-era years only (1090 to 1105), backbone as built: setting-tier entries (the wiki's own "the event takes place in" list) 16 events, mean absolute error 0.469 years, every one within 1 year; cited-tier (years the wiki cites from a story, which include flashbacks) 43 events, 1.558. The large cited-tier errors are mostly the wiki citing a flashback year (Zwillingstürme cited 1092, its own caption says 1100).

- Linear interpolation between the 7 anchors instead of the step: 1.48 against 1.47 on 57 events, no gain.
- Wiki years as extra anchors, a ceiling test: half of the wiki's event years added as anchors, error measured on the other half: 1.63 to 1.82, worse. More points on a release-date curve add noise, because an event's year is not a function of its release date (release order agrees with in-world order on 0.774 of pairs).
- The script has no more present-year statements to mine: 11 events state their present year and all are used (7) or dropped as flashback captions (4). Of 8 non-caption year lines at 1095 or later in events, 2 could lift the 1100 ceiling (Retracing Our Steps, a letter dated 1101; Here A People Sows, "promoted ... in the year 1101").
- Option A, archives' present dates as anchors past 1100 (Zuo Le 1102, Mon3tr, Lemuen, Branch 1101): not buildable. No gamedata table links an operator to its debut (`docs/release/CN_EN_RELEASE_MAPPING.md`); pool metadata names 168 operators, but only in the pools the table still holds, so a first pool is often a rerun (Ascalon's reads 2026-03, a Returning Headhunting).
- The two 1101 lines as anchors, simulated: 24 events move from 1100 to 1100.67, and the error is unchanged on both tiers (setting 0.469, cited 1.558): Retracing Our Steps and Unrealized Realities move closer to 1101 while Act or Die and Ato (1099) move away. Events released 2024 to 2026 span 1099 to 1102 in-world, so no monotone release curve places them better. Not built.

What would move the years is per-event evidence, not backbone anchors: an event's own present-year statement (all used), cross-references to dated events (a handful), or the wiki's setting years used directly for the events it covers (Ian's call; it makes those years wiki-sourced and needs a held-out split to stay measurable).
