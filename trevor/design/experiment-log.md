# Trevor experiment log

What was tried, what it measured and what the project decided, from the first tool-based routing (2026-09-29) to the
code cleanup (2026-10-18). Entries are grouped by topic and ordered by date within each topic. Each entry gives what
was **Tried**, the measured **Result** and the **Decision** (shipped as the default, kept opt-in, refuted and removed,
or parked), with the kill switch whose off value reproduces the earlier behaviour where there is one. Entries that
came from code comments end with a **Code.** line naming where the behaviour lives now.

Earlier history (2026-09-22 to 2026-09-28) is in [archive/project-origins.md](archive/project-origins.md),
[trevor-retrieval-baseline.md](trevor-retrieval-baseline.md) and sections 6 to 11 of
[trevor-questions.md](trevor-questions.md).

**Reading the entries.**
- *The probe set* is `eval/ian-questions.jsonl` (ids `ian1`, `ian2`, ...). It grew over time (5, 9, 14, 25, 32, 33,
  then 35 questions), so each entry states the size it was measured on. Its answers were verified against the wikis in
  [archive/trevor-probe-questions-verified.md](archive/trevor-probe-questions-verified.md).
- *The real probe* is `eval/real-probe.jsonl`: 120 real player questions from r/arknights (ids `r000` to `r119`),
  scored per category ([trevor-questions.md](trevor-questions.md) section 8). *Gold v2* is `eval/goldset_v2.jsonl`,
  the 146-question automated gold set (ids `g0001` ...). *Held-out* is `eval/lore-heldout.jsonl` (40, ids `h000` ...)
  and *trivia* is `eval/trivia.jsonl` (109, ids `t000` ...).
- *Night N* names a batch of runs whose answer records are in `artifacts/answers/nightN/`.
- Paths are as of each entry's date. The code was split afterwards: `src/bin/ask.rs` is `src/bin/ask/`, `src/tools.rs`
  is `src/tools/`, `src/bin/ask/serve.rs` is `src/bin/ask/serve/`, `src/bin/ask/server.rs` is
  `src/bin/ask/llama_server.rs`, `src/goldgen/llm*.rs` is `src/llm/`, `src/search/horizon.rs` is
  `src/search/horizon/`, `src/corpus/embed.rs` is `src/corpus/embed/`.
- Flags and scripts that an entry says were removed no longer exist; the entry is their record.

## Topics

- [Retrieval](#retrieval) (54 entries): Search (BM25, hybrid, rerank), the corpora and passage sets (P3b, P4, P4x, art units, topic and dossier passages, lore v1 and v2), passage budgets, scenes and line pools.
- [Routing](#routing) (31 entries): The routers (model, keywords, the retired kNN and hybrid), the table tools and their arguments, route checks, question detection.
- [Answer rules](#answer-rules) (45 entries): Answer prompts and the question form, decline, partial, retry and verdict rules, relation and opinion rules, citations and answer checks.
- [Entities and topics](#entities-and-topics) (46 entries): Identities, names and aliases, topics and topic summaries, dossiers, canon evidence for the Integrated Strategies endings, deaths, places, organisations, operator data.
- [Chronology](#chronology) (15 entries): Dates, the timeline, eras, gatePos and the spoiler horizon, reading order.
- [Design inference (retired)](#design-inference-retired) (18 entries): `design_infer.py`, art captions read for design, visual models, `--design-deduce`. The chain was retired on 2026-10-17; these entries are its record.
- [Embeddings](#embeddings) (3 entries): Embedder models, ONNX exports, INT8 and fp32, fine-tuning, re-embedding. The fp32 re-embed and its guard are documented in full in [trevor-reembed.md](trevor-reembed.md).
- [Eval](#eval) (25 entries): Gold sets, judges, the real probe, the probe set, sign tests and the eval harness.
- [Serving and integration](#serving-and-integration) (45 entries): Which model answers each rung, llama-server and the model lock, the worker, profiles, the power guard, caching, the backend and frontend integration, the VPS, timing, memory and heat.

## Retrieval

Search (BM25, hybrid, rerank), the corpora and passage sets (P3b, P4, P4x, art units, topic and dossier passages, lore v1 and v2), passage budgets, scenes and line pools.

### 2026-09-24: Chunking rules derived from the command frequencies of the EN story scripts

**Tried.** Two obvious chunkers were checked against the corpus, then a scene-aware packer was built from the command frequencies that the backend's own parser reports over the 1,862 scripted EN stories (measured 2026-09-24). Token counts use the real gte-modernbert tokenizer with a calibrated overhead (2 per sequence, 1 per join, both measured).
**Result.** A fixed-size token window splits speaker turns: a line of dialogue is stranded from the `[name="..."]` that owns it, and the chunk becomes unattributable. Breaking on every background command is wrong in a smaller way than first believed: 822 of 15,039 `[Background]` commands name no plate and are fades, 5.5%, not the roughly half an earlier regex sample suggested. The larger errors were elsewhere (see `hard_boundary`).

Command frequencies:

| command | count | command | count |
|---|---|---|---|
| name | 334594 | stopmusic | 9215 |
| charslot | 165931 | playmusic | 7801 |
| character | 130277 | subtitle | 7367 |
| delay | 65758 | sticker | 5295 |
| blocker | 65619 | image | 5082 |
| dialog | 57856 | predicate | 4754 |
| playsound | 36127 | decision | 3883 |
| text | 32292 | header | 1862 |
| background | 15039 | multiline | 1450 |
| camerashake | 13291 | | |

`blocker` and `delay` outnumber `background` 4.4 to one and are intra-scene pacing. `header` occurs exactly once per story, as its first command. Scenes are roughly chunk-sized, not smaller than chunks as an earlier estimate claimed: 13,700 scene boundaries over 1,862 stories against 13,837 chunks.

On the real corpus: 13,837 chunks, p10 328, p50 548, p90 595, max 1,355 tokens; 13.8% under the 400 floor and 0.43% (60) over the 600 ceiling, every one of those 60 a single turn too long to split. The join cost is identical across all 24,757 real joins. The stored `token_count` equals a whole-chunk encode for all 13,837. Exact reconstruction of every story's prose from its chunks holds for all 1,862: nothing is lost and nothing is duplicated.
**Decision.** `blocker` and `delay` are never boundaries; an invariant test asserts that adding or removing them cannot move a chunk edge. `header` as a boundary only opens scene 1. The packer does both jobs: it merges short scenes up to the 400-token floor and cuts long ones at turn boundaries under the 600-token ceiling.
**Code.** `src/corpus/chunk.rs`, module doc and `hard_boundary`

### 2026-09-24: Ingest wire types checked against the backend

**Tried.** The ingest wire types read only the fields Trevor needs from the backend's story API; serde ignores the rest. They were verified against `backend/src/app/services/story.rs` and `backend/src/core/story/mod.rs` on 2026-09-24.
**Decision.** The wire types stay a subset of the backend's types.
**Code.** `src/corpus/ingest/mod.rs`, wire types banner

### 2026-09-24: Reranker ONNX output shape

**Result.** Checked 2026-09-24: the reranker's output is `last_hidden_state [batch, seq, 256]`.
**Code.** trevor/src/search/rerank.rs, module doc

### 2026-09-27: Operator archives in the default corpus

**Decision.** The default corpus includes the operator archives (`build-archives`, adopted 2026-09-27); `--corpus artifacts` is the stories alone.
**Code.** trevor/src/search/runtime.rs, `RuntimeArgs::corpus`

### 2026-09-30: Lore passages for typed-source questions (round 6, issue B)

**Tried.** `--source-lore`: for typed-source questions, the best unit's own text retrieves story and operator-file passages placed after the source units. First version measured on the 25 real-probe and 6 gold questions whose route or passages changed.
**Result.** 0 gained, 2 lost on the real probe (Kal'tsit boiling water, Mon3tr's file), faithfulness 34/49 to 30/50; gold lost 3 correct answers (g0063, g0100, g0117, all operator-file or voice sources). The Angelina probe question then opened "Trevor does not have that text" and cited Swire's Bloodline of Combat outfit.
**Decision.** Fixed (units of another operator dropped, an explicit rule that passages [1] to [k] are the source and must not be declined) but not yet measured at that point, so it stayed opt-in. See the later entry on source lore becoming the default.

### 2026-09-30: Source lore for outfits, items, modules and enemies made the default

**Tried.** The fixed `--source-lore` (round 6c) re-measured on the real probe and gold set.
**Result.** 0 losses, r046 0 to 1. `--route-only` with the flag is byte-identical to the pre-round output on all 266 real-probe and gold questions (verified again).
**Decision.** Source lore for outfits, items, modules and enemies is now the default; `--no-source-lore` restores the old retrieval.

### 2026-09-30: Story-words query rewrite for vocabulary misses: refuted

**Tried.** `--story-query`: one more Gemma call rewrites the question in the words the story's lines would use, and its hits are interleaved with the question's. Two more prompts were tried on the four new probe questions: a keyword list with crude and slang words, and a line of imagined dialogue.
**Result.** Gemma kept the question's own word ("Who said they wanna pee in public?"), and the Chiave chunk stayed out of the passages on all four flag sets. The keyword list ("operator, pee, public, urinate, bathroom, toilet, piss, ...") ranks the Chiave chunk outside the top 30, as the raw question does; the list moves Ulpianus's line from 16th to 5th and Insider's donut file from 1st to 2nd. The imagined dialogue says nothing retrievable ("I might just have to"). Even the right word ranks the chunk only 7th ("piss in public") or 24th ("operator wants to piss in public"), because 163 P3b chunks match "piss" (mostly "pissed") and the scene is a pissing contest at a campsite, not "in public".
**Decision.** Refuted as the vocabulary fix; `--story-query` is kept opt-in and was not measured further. A vocabulary fix for this case is not a query rewrite at this model size.
**Follow-up.** The vocabulary miss stays open.

### 2026-09-30: Ettin reranker over P3b: refuted as a reranker, built as a one-passage addition (`--rerank-add`)

**Tried.** The Ettin 17m reranker, re-measured on gold v2 over P3b, the corpus `ask` uses. Earlier on P0 it had failed the gate against hybrid (recall@10 -0.090 [-0.173, -0.009], 9 gained, 19 lost, hit@1 0.336 to 0.382, 538 ms). Also the evidence ranks for the 9 probe-set questions, and a simulated union of the fused list with the reranker's best passages.
**Result.** Hybrid recall@10 0.732, hit@1 0.344. Rerank 40: recall@10 0.605, hit@1 0.282, -0.127 [-0.207, -0.045], hit@10 22 lost and 8 gained, FAIL, p50 529 ms. Rerank 100: recall@10 0.620, hit@1 0.260. Evidence ranks for the 9 probe-set questions (P3b, hybrid then rerank 40): Matsukiri's file 8 to 1, Andoain's dossier 34 to 1, Harold's family lines 9 to 3, Ulpianus's "tight-lipped" line 16 to 4, Insider 1 to 1, Midnight 4 to 5, Perfumer's file 46 to 46. Chiave and the praised cooks are outside the fused top 100, so no reranking of candidates reaches them; the phrasing miss (pee/piss) is not a reranker problem. In a union (simulated from the two runs' rankings): fused top 7 plus the reranker's best passage not among them, against fused top 8, is recall +0.063 [+0.023, +0.109], 10 gained and 2 lost; fused top 8 plus 2 against fused top 10 is +0.029 [-0.015, +0.074]. On the 9 probe-set questions, `--rerank-add 1` changes the passages of 4 and the answers of 2 (Chiave still declined; Ulpianus still "a man of few words").
**Decision.** Reranking outright is refuted again, worse on P3b than on P0. The union is built as `ask --rerank-add N` (0 = off, the default; off loads no reranker and is the old path).
**Caveat.** The answer A/B on the 120 real-probe and 146 gold questions (both arms, Qwen judging the answers that change) first stopped at 39 real-probe answers when llama-server aborted (Abort trap 6); the rerun restarts the server after a crash.

### 2026-09-30: `--rerank-add 1` answer A/B: refuted, stays at 0

**Tried.** `--rerank-add 1` (the reranker's best new passage replaces the 8th fused one), judged on the changed answers against `--rerank-add 0` from the same server session.
**Result.** It changed the passages of 31 of 120 real-probe and 61 of 146 gold questions. Real probe: responds 18 to 16 (lost r045, r080, gained none), faithful 65/75 to 72/78. Gold: correct 42 to 40 (3 gained, 5 lost: g0018, g0046, g0060, g0113, g0117), faithful 97/100 to 92/98.
**Decision.** Refuted: it stays opt-in at 0.

### 2026-09-30: Operator files and voice lines keep plain retrieval under the lore versions

**Tried.** The first two lore versions were compared answer by answer with plain retrieval.
**Result.** All 5 answers the first two lore versions lost were operator-file or voice-line questions: Kal'tsit's voice, Mon3tr's and two gold operator files, and a gold voice question.
**Decision.** Operator files and voice lines are excluded from `LORE_KINDS` and keep plain retrieval.
**Code.** `src/bin/ask/kinds.rs`, `LORE_KINDS`

### 2026-09-30: Yes/no questions interleave the hits of their evidence query

**Tried.** A yes/no question interleaves the hits of its evidence query (from the question-form call) with its own hits.
**Result.** "Has Harold married twice? Does he have children?" ranked the lines naming his wife and daughter 29th to 44th; the evidence query "Harold wife daughter children married" ranks them 1st to 8th.
**Decision.** Shipped as the default with the question-form call (`--no-question-form` turns the form call off).
**Code.** `src/bin/ask/retrieval.rs`, `passages` (was `retrieve_for`), yes/no evidence interleave

### 2026-09-30: Opinion questions gather candidates from operator files and dossiers

**Tried.** A judgment question gathers candidates first: the evidence query (for example "skilled cook, culinary talent") over a wide list, kept to operator files and dossiers, at most two chunks per character, without neighbours (six candidates cost six chunks). Characters the question names come first, each from its own files. `named_characters` finds them through identity links, matched as whole words, case folded, 3 letters or more.
**Result.** "Who is the best cook?" retrieved the Dungeon Meshi collaboration and no operator file in its top 100; the evidence query puts Matterhorn's, Hung's, Lee's and Gummy's files in the top 52. Without the named-first rule, "Midnight or Matsukiri?" filled four of six places with other operators' files and dropped Midnight's. "Margaret vs Degenbrecher" found no file for Margaret before the real names were added, because Nearl's files carry only codenames.
**Decision.** Shipped as the default for opinion-form questions; real names were added to the name matching.
**Code.** `src/bin/ask/retrieval.rs`, `passages` (was `retrieve_for`), opinion candidates; `src/bin/ask/people.rs`, `named_characters`

### 2026-09-30: Lore for questions about one kind of source text

**Tried.** A question about one kind of source text (outfit, item, module, enemy, voice line, Integrated Strategies) gets lore: the best unit's own text is a second query, and its best story and operator-file passages follow the units, so the answer can explain who and what the unit names. With lore on, units of another operator no longer follow the best unit. The model router also names the source of a retrieval route (story, operator_file, module, voice, skin, is, enemy, item, any), which picks P4 and the preferred kind instead of `source_kind`'s keyword list. `SOURCE_RULE` tells the answer model the passages are that kind of text.
**Result.** "What does Angelina's Bloodline of Combat outfit say?" was answered with the outfit's two sentences and no lore, and put Swire's Bloodline of Combat outfit third. The first probe answered "what does Kal'tsit's module say" from story passages as if they were the module.
**Decision.** Default since 2026-09-30. `--no-source-lore` drops the lore (as before 2026-09-30); `--no-router-source` restores the router before the source.
**Code.** `src/bin/ask/retrieval.rs`, `passages` (was `retrieve_for`), typed-source lore and owner filter; `src/bin/ask/cli.rs`, `Args::no_source_lore`, `Args::no_router_source`; `src/bin/ask/prompts.rs`, `SOURCE_RULE`

### 2026-10-01: A summary tree for whole-story questions (`scripts/overview.py`)

**Why.** Item 2 of the work list. "Summarize the Arknights story in one word" was declined ("The provided passages do not contain a single word that summarizes the entire Arknights story"): retrieval finds scenes, and nothing in the corpus states the whole story.
**Tried.** `scripts/overview.py gen` (Gemma, 16 calls, 6 min 12 s) writes `artifacts/overview/overview.jsonl`: one summary per storyline of the game's Movements view (14, the main story "For Tomorrow" included, from the 81 P2 group summaries, method C, in the game's order; inputs 349 to 6,176 words, outputs 293 to 337 words), then an overview of the whole story (430 words, from the 14 storyline summaries) and of the world of Terra (358 words, from the storyline summaries plus the first 80 words of each of the 46 topic summaries, 8,355 words in). Units are keyed on input and prompt sha as in `incr.py` (PLAN=1 prints the plan; update.sh does not run it yet). Qwen judged each sentence against that unit's own input.
**Result.** 208 of 210 supported (0.990), 14 of 16 units fully supported (ssLine_4 11/12, ssLine_10 12/13).
**Caveat.** That is faithfulness to the P2 summaries, not to the story text; an error the event summaries already carry passes through.
**Follow-up.** Add `overview.py` to update.sh step 4b.

### 2026-10-01: `update.sh` dropped the topic units from P4: fixed

**Why.** P4 holds the 46 topic units only because it was built from `artifacts/sources/all_units.jsonl` (byte for byte `sources/units.jsonl` plus `topics/units.jsonl`), but steps 4 and 4b ran `build-units` with its default `sources/units.jsonl`, which drops them (a first rebuild did: 0 topic chunks in P4).
**Tried.** Both steps now write `all_units.jsonl` and build from it.
**Result.** P4 rebuilt that way has 22,917 chunks, 46 topic chunks, 46 vectors embedded and 22,871 reused.

### 2026-10-01: Lore v2: topics and dossiers chosen from data

**Tried.** Lore v1 is the 46 topics and the P3b and P4 built with the 200 dossiers. Lore v2 adds the topics and dossiers chosen from data (`Tools::new_dossiers`, not in P3b), a new dossier, and a game-data passage at answer time. Only topics from data are ever marked thin; thin topics are never served.
**Decision.** v2 is the default since 2026-10-01; `--lore v1` is the kill switch and reproduces the lore before 2026-10-01.
**Code.** `src/bin/ask/cli.rs`, `Args::lore`, `LoreSet`; `src/tools/mod.rs`, `Tools::new_dossiers`, `Tools::load` (thin topics)

### 2026-10-01: Topics from data (`--lore v2`) and the v1 topic set

**Tried.** On 2026-10-01 the topic tool's router line was rewritten from the topics in data, with a wider topic line for `ask --lore v2`.
**Decision.** `ask --lore v1` keeps the 46 topics as before 2026-10-01 (the rows without a `source`, without the merged `names`) and the topic line as written before.
**Code.** trevor/src/tools/topics.rs, `Tools::topics_v1`; trevor/src/router/model.rs, `TOPIC_LINE` and `RouterPrompt::wide_topics`

### 2026-10-02: `--lore v2` first build: topics and dossiers inside the retrieval corpus, kept opt-in

**Tried.** `ask --lore v2` reads artifacts/p3b-lore (P3a plus all 536 dossiers and 88 event summaries, 16,386 chunks, 336 embedded and 16,050 reused) and artifacts/p4-lore (plus the typed units and 344 topic units, 23,551 chunks, 298 embedded), resolves the topic tool over all served topics plus the merged names, and sends the router a wider topic line (places, organizations, factions, events; one line differs, unit-tested). `--lore v1` (the default) drops every topic row with a `source` and the merged names and reads artifacts/p3b and p4, which `dossiers.py sets`, `topics.py units` (units.v1.jsonl) and update.sh rebuild byte-identical to before (chunks.jsonl of both compared with cmp). Measured in one Gemma run (about 30 s per answer), judged by Qwen on the changed answers. New held-out sample `eval/lore-heldout.jsonl`: 40 H and R questions from Reddit and Discord not in the probe, 20 naming a new topic or merged name and 20 others, seed 11.

**Result.**
- Route check: keywords 61/61 under both; model 75/76 under both (the Sami miss as before, also under v2).
- Real probe: of 120 questions 51 change (50 in passages, 1 in wording only); no real-probe question routes to a new topic, the changes are the new dossiers in P3b. Responds 39 to 38 of 51 (gained r030, r036, r050, r094; lost r028, r078, r105, r108, r116), handles 20 to 21 of 23, faithful 198/216 (0.917) to 181/206 (0.879).
- Held-out: 13 change (9 and 4), and 4 of them get a new topic summary (Nearls, Yan-Lungmen for Lungmen, Siracusa for "Siracusan" through the merged names); responds 11 to 10 of 13 (lost h002 "Do the Nearls have wings?", which got the new Nearls topic; gained none), faithful 42/43 to 46/48.
- Gold v2: the v1 arm ran all 146; the v2 arm was stopped at 41 for a deadline, and 17 of those 41 change (all in passages); correct 9 to 9 of 17 (0 gained, 0 lost), gold chunk among passages 10 and 10, cites a gold chunk 8 to 9, faithful 14/15 to 15/16. The other 105 gold questions are unmeasured under v2.

**Decision.** Losses on the real probe and the held-out set, so nothing is made the default: `--lore v1` stays, and the topics, dossiers, lore dirs and wider router line are reached only with `--lore v2`. Steps 4 and 4b of update.sh build both lores. Answers and judgments of both arms are in artifacts/answers/lore2/.

**Follow-up.** Read the 5 lost real-probe answers to see whether a new dossier displaced the passage that answered (r028, r078, r105, r108, r116), finish the gold v2 arm (105 of 146 v2 answers not run), and test a dossier boost limited to the named character before a default.

### 2026-10-02: `--lore v2` rebuilt as additive: v1 retrieval plus one named dossier after the passages

**Why.** The 6 losses of the first v2 run, read: r078 and r105 got the same passages in another order (the 336 dossiers inside P3b shift the fused ranks; v1 cited Catapult's second interlude at [4] to [8], v2 had it at [10] and [11] and said it found none); r028, r108 and h002 are both arms declining in substance (judge noise); r116 lost Philae's and Qanik's files to the Kirara and Myrrh dossiers (crowding). No loss came from a wrong identity link (Estelle to Perfumer, Shirayuki to Midnight remain unfixed; the dossier rules never use those links). The dominant cause is the dossiers inside the retrieval corpus.

**Tried.** v2 no longer uses artifacts/p3b-lore or p4-lore: it retrieves from the v1 P3b and P4 exactly, and adds at answer time a new topic through the topic tool (and the wider router line) and at most one new dossier, the character the question names by the dossier's own name, as written (case and whole words; folded only for multi-word names), never a one-word name under 4 letters and never the start of a longer name ("Lin Qingyan", "Deputy Minister Yu", "on the mountain" all matched wrongly before those two rules), placed after the retrieved passages (placed first it lost g0107).

**Result.**
- `--lore v1` answers are byte-identical to the previous binary's on the 10 real-probe questions re-answered (answers and passages, 10 of 10).
- Route checks: keywords 61/61 under v2; model 75/76 under both (Sami).
- Only these questions now differ v1 to v2: real probe 10 of 120, held-out 3 of 40 (topics only: Nearls, Yan-Lungmen, Siracusa), gold v2 29 of 146 (26 with a dossier, 3 by the router line), the probe set `eval/ian-questions.jsonl` 6 of 14.
- Judged on the differing answers: real probe responds 5 to 6 of 10 (gained r080, r046; lost r021, an IS question that got Tin Man's dossier), faithful 35/40 to 30/35; held-out 3 to 2 of 3 (lost h002, both arms declining, judge noise as before).
- Gold: the first placement (dossier first) gave correct 19 to 20 of 26 answerable (gained g0063, g0129; lost g0107), faithful 46/47 to 40/43; with the dossier placed last the gold judge was stopped at 15 of 29 v1 answers when the work was paused.

**Decision.** Paused, not the default: losses remain (r021, h002) and gold is unfinished. artifacts/p3b-lore and p4-lore are no longer read (update.sh no longer builds them; they can be deleted). Answers, judgments and the scripts that made them are in artifacts/answers/lore3/; judging resumes by qid with `CORPUS=artifacts/p4 python3 scripts/answer-eval.py eval/goldset_v2.jsonl artifacts/answers/lore3/goldset_v2-v1.chg3.jsonl`, then v2.

**Follow-up.** Finish the gold judge, read r021 (Tin Man's dossier on an IS question; a dossier may need to stay off P4 routes), and decide whether a judge-noise loss like h002 counts.

### 2026-10-02: `--lore v2`: no dossier on typed-source routes; one loss left, judge noise

**Why.** r021 ("Is this the same Tin Man from the random encounter in IS2?", routed to IS) got Tin Man's dossier; both arms say the passages do not mention the IS2 encounter, v1 then describes Tin Man's Kazdel and San Sophie history, v2 his Columbian revenant story. A question routed to a typed source (IS, voice, module, outfit, item, enemy) asks about that text, not the character.

**Tried.** A new dossier is now never added to a typed-source route (`--dossier-on-typed` restores the earlier behavior; it skips only this filter, not re-run). This takes the dossier off r021, r046 (outfit; it had gained) and g0117 (voice), so their v2 prompts equal v1's and their v1 answers stand for both arms. The gold judge was finished for both arms (resumed by qid, Qwen, 7 min).

**Result.** Current code, judged on the answers that differ:
- Real probe: 8 of 120 differ, responds 4 to 5 (gained r080, lost none), faithful 24/28 to 19/24.
- Held-out: 3 of 40 differ, responds 3 to 2 (lost h002), faithful 7/7 to 9/9.
- Gold v2: 25 answerable and 3 unanswerable of 146 differ, correct 18 to 20 (gained g0063, g0129, lost none; g0107 is no longer lost with the dossier placed last), unanswerable declined 3 and 3, faithful 43/44 to 41/43.
- The one loss, h002 "Do the Nearls have wings?", starts with the same sentence in both arms ("The passages do not mention whether the Nearls have wings."); v1 goes on to Nearl as a Radiant Knight, v2 to the Nearl family as Kazimierz knights from the new Nearls topic; the judge scored v1 as responding and v2 not.
- The 6 differing questions of the 14-question probe set were answered in both arms and not judged (artifacts/answers/lore3/ian-*).

**Decision.** h002 is judge noise by reading, but it is a loss as measured, so `--lore v2` stays opt-in until judge-noise losses are ruled on.

### 2026-10-02: `--lore v2` final measurement: no losses, default held back by an unverified kill switch

**Tried.** Current code (additive v2 with game-data passage and race `summaryV2`), changed questions only, both arms judged by Qwen (v1 answers byte-identical to the earlier v1 run).

**Result.**
- Real probe: 9 change, responds 5 to 6 (gained r080, lost none), faithful 26/30 to 22/26.
- Held-out: 3 change, responds 3 to 3, faithful 7/7 to 11/11, h002 answered "most likely no, they are Kuranta".
- Gold v2: 27 answerable change, correct 19 to 21 (gained g0063, g0129, lost none), unanswerable declined 3 and 3, faithful 47/48 to 44/46.
- Probe set (`eval/ian-questions.jsonl`, 14): 7 differ (not judged): ian11 "what operator is based off an animal with phantom as part of its name" goes from Mint (wrong; expected Phantom/Tragodia) to a decline, because the wider router line moves its source from any to story; ian13 as above; the other 5 say the same in other words.
- Route checks: keywords 61/61, model 75/76 under both lores (Sami).
- Kill switch: three probe-set questions answered twice under `--lore v1` with the same binary and the same passages had prompts 1 to 2 tokens apart (ian10 9,139 then 9,137; ian11 7,260 then 7,258; ian3 9,091 then 9,090) and different wording, so some part of the v1 prompt looked not deterministic between runs (pre-existing; the 10 real-probe questions checked earlier were byte-identical).

**Decision.** Not made the default, although the three measured sets lose nothing: the kill switch could not be verified byte for byte. Answers, judgments and scripts: artifacts/answers/lore4/.

**Follow-up.** Find the prompt nondeterminism (likely an unordered map in a passage label or name expansion), verify `--lore v1` on the 14 probe-set questions and the route-check cases, then flip the default.

### 2026-10-02: `--lore v2` made the default; `--lore v1` is the kill switch

**Tried.** Verifying `--lore v1` against the pre-v2 default. There is no pre-v2 binary (trevor/ is untracked, no old build was kept), so the evidence is the v1 run against the last pre-v2 records. The flip itself is verified exactly with `--no-cache-prompt` in one session.

**Result.**
- ian-form2.jsonl (2026-10-01, ian1 to ian9) has identical passages and forms on 9 of 9 and identical answers on 8 of 9 (cache noise on the 9th), and every later v1 record of ian10 to ian14 has identical passages (lore3 6 of 6 and 1 of 1, lore4 7 of 7).
- The new binary's default answers the 14 probe-set questions identically to the old binary's `--lore v2` (H against G: 14 of 14 records, 46 of 46 prompts and outputs), and the new binary's `--lore v1` identically to the old default (F against C).
- Route check after the flip (model router, `--no-cache-prompt`): 75/76 under the new default and 75/76 under `--lore v1`, Sami failing in both as before.
- Under v2, 7 of the 14 probe-set answers differ from v1 (ian2, ian3, ian7, ian8, ian10, ian11, ian13); ian11 is a decline under v2 where v1 said Mint (wrong), and ian13 still gives no number.

**Decision.** With 0 losses measured on 2026-10-02 (real probe responds 5 to 6 of 9 changed, held-out 3 to 3 of 3, gold correct 19 to 21 of 27, none lost), `--lore v2` is now the default and `--lore v1` the kill switch.

### 2026-10-02: The new dossier goes after the retrieved passages

**Tried.** The new dossier was placed first, before the retrieved passages.
**Result.** Placed first, it lost `g0107`.
**Decision.** The new dossier goes after the retrieved passages (`Pre::last`).
**Code.** `src/bin/ask/retrieval.rs`, `Pre::last`

### 2026-10-03: Operator E2 art captions and art units in their own corpus, `artifacts/p4-art`

**Tried.** Captions with Gemma 4 12B and its projector for the remaining 133 E2 arts (127 new, 6 redone after repetition loops), then the tiled text pass on 334 more images. `scripts/art_captions.py units` writes one unit per captioned image (`artifacts/art/units.jsonl`, groupId "art"): the heading "Operator art: Pallas, E2 (Elite 2) art" (or the outfit's name and brand), the illustrators, the outfit's flavor text from `skin_table` (the E2 boilerplate every promoted operator shares, "An outfit that has been improved after the Operator's Promotion...", is left out), the caption headed "Description (written by a vision model from the image, not game text)", and, only from the tiled pass, "Text visible in the art (vision-model transcription, may be misread)". Whole-image text is never used. A transcribed length equal to the operator's handbook height (`operator_attributes.jsonl` `heightCm`) adds one line saying so.
**Result.** Captions: median 7.86 s, 3 loop again (Nearl the Radiant Knight, Quartz, Weedy), so 376 of 379 E2 arts have a caption. Tiled text: 344 of 379 in all (median 12.37 s, 174 with some text). Outfits got no model time (0 of 472). Thermal: no warning level recorded; memory free 58% at the end of the run. 376 units, 172 with tiled text. Without the height line, the question's "height" and the art's "162cm" share no word and Pallas's unit ranked below 8 other art units for the probe question `ian14`; with it, first. The height line fires on exactly one image of the 344 tiled, Pallas's.
**Decision.** The units go into their own corpus, `artifacts/p4-art` (P4's 22,917 chunks byte for byte as a prefix, checked with `cmp`, plus the art units; vectors reused from P4, only the art units embedded), never into P4 itself, so every answer not routed to art reads exactly what it read before.
**Follow-up.** Outfit captions resume with `caption --kind skin --seed 7`, then `text --kind skin --seed 7`.

### 2026-10-03: Origin chain for why/how-it-began questions (ORIGIN_RULE plus story-order retrieval): the default

**Tried.** A question with a why, when or how word and a beginning word ("start", "began", "origin", "founded", "join", "become", "decide", "first", ...) gets ORIGIN_RULE (trace the causes back to the earliest one the passages show, then forward to what is asked, one step per sentence with citations) and a retrieval change. It fires on 3 of the 25 probe questions, 3 real-probe and 6 gold questions (0 trivia, 0 held-out).
**Why.** "Why did swire join the LGD?" (`ian15`) ranks five main-story chunks first and her own story's L.G.D. years (`story_swire_set_1_story_1#0009`) 6th, her story's opening 7th and 8th and the kidnapping (`#0003`) 20th, and the 9,000-token budget keeps only the first 6 hits' clusters (8,991 prompt tokens).
**Result.** Three retrieval versions were measured.
- Version 1 (the earliest top-60 scene of each of the top 3 stories, after the first 4 hits) reached nothing for `ian15` (her story is the 6th distinct one) and lost gold g0107 (correct 2 to 1 of 5; real responds 2 to 2 of 3).
- Version 2 (the story with the most chunks in the top 20, its earlier chunks after the first 4 hits) measured "earlier" from the lowest position among the hits, which was `#0000`, so it added nothing either; with the rule alone it had 0 losses (real 2 to 2 of 3, gold 2 to 2 of 5).
- Version 3: the story with the most chunks in the top 20 (at least 3) goes first, its top-20 chunks that come before its best-ranked hit, without neighbours, in story order (at most 5), then that hit with its neighbours, then the other hits.

Under version 3, `ian15` becomes "Swire decided to become a police officer after a cop saved her during a kidnapping case when she was a child [3, 4]. She later realized that she wanted to join the L.G.D. to show her appreciation to that officer [4]. Additionally, she chose to take on work in the family business and eventually the L.G.D. to find the courage to choose her own path and stand up to her grandfather's influence [5, 7]", citing `#0003`, `#0005`, `#0007` and `#0009`, which is the reference's whole chain. `ian20` (Abyssal Hunters) keeps "born nearly two centuries ago ... never afforded any systematic scrutiny or research, consisting only of endless debates ... only twenty-some years ago that Ulpianus made the project a reality" (`act34side_level_act34side_05_beg#0004`, already reached by the default); the "two centuries of debate before implementation" line (`act34side_level_act34side_02_beg#0004`) and the taboo are in act34side and are reached by the question's retrieval, while act17side holds no Hunter-project scrutiny line (searched for taboo, forbid, debate, scrutiny, century: hits only in act34side). `ian17` stays a decline. Judged on the changed answers: real probe responds 2 to 2 of 3 (0 lost), gold correct 2 to 2 of 5 (0 lost; g0107 is answered again).
**Caveat.** Faithfulness went down (real 14/15 to 5/9 supported sentences, gold 10/11 to 11/14): the rule asks for a chain of steps and some steps are the model's own connections, which the faithfulness judge does not find stated.
**Decision.** 0 losses, so version 3 is the default; `--no-origin-chain` turns off both the rule and the retrieval change.

### 2026-10-03: Game-data coverage audit and the new "gametext" units in `artifacts/p4x`

**Tried.** An audit sampled every excel field and story file under `../assets/output/en` against P4 and the story corpus (up to 30 strings of 60 characters or more per field path, their first 50 normalized characters searched in the 60 MB of corpus text). `scripts/gametext.py` (new, no model, seconds) then turns the lore-bearing missing text into units of one new labelled kind, groupId "gametext", headed "Game text (<kind>): <title>".
**Result.** Already indexed (found fraction about 1.0): operator files (3,449 strings), voice lines (8,960), module stories (the 380 generic ORIGINAL-module lines are skipped on purpose), enemy, item, outfit and IS texts, `gamedata/story/obt/main` (403 of 407 files), `obt/memory` (367 of 367), `obt/rogue` (177 of 177), and 19 of 21 sampled event folders. Missing and lore-bearing, largest first:
- the game's own story intros (`[uc]info`, 1,860 of 1,881 files, 364,193 characters);
- event entry, hidden and ending scripts (act12d6, act17d7, act13side hidden_st01 to 03, act1lock; 18 files, 174,065);
- act29side chats and marks and minigame scripts (57 files, 265,094, partly lore);
- sandbox battle scenes (35 files, 84,626);
- the main-story record notes (`obt/record/main_10` to `13`, 35 files, 50,431);
- event archives (`story_review_meta_table` ActArchiveResData: logs, diaries, landmarks, pictures, news; 193,112);
- sandbox event text (102,790);
- `activity_table` event text (about 302,000 across many fields);
- the mail archive (`display_meta_table`, 57 letters, 25,684);
- operator record intros (23,901) and the world-view loading tips (3,686).

In all 426 missing lore-bearing field paths, about 1.23 million characters clearly lore and 1.42 million partly (stage, retro, furniture and medal descriptions, which this ingestion leaves out as noisy). `gametext.py` produced 1,415 units: 581 intro units (one per event, each line "<story name> (<tag>, <code>): <intro>", names from `story_review_table`), 188 story-script units (scripts no review entry or corpus story id covers; IS and guide scripts left to `sources.py`, training levels left out), 47 record notes, 109 event-archive units, 57 letters, 351 operator record intros, 2 world-tip units, 80 event-text units; 203,566 words, 1,699 chunks. They go into their own corpus, `artifacts/p4x` (P4's 22,917 chunks byte for byte as a prefix, checked with `cmp`, then the new ones; vectors reused from P4, 1,699 embedded in 182.6 s), read at first only by `ask --game-text` for P4 routes.
**Caveat.** A first build put the script's file name in its heading and BM25 matched "dialog_sandbox_1_main1_3_ed" to "Who is Ed?"; the heading now leaves file names out.

### 2026-10-03: Game text (`artifacts/p4x`) becomes the default corpus for P4 routes

**Tried.** Prefilter (no model): of the questions the router sends to P4 (probe set 6, trivia 35, held-out 0, real 24, gold 3), those with a gametext chunk in p4x's hybrid top 10 (probe set 3, trivia 3, real 6, gold 0) were answered both ways. The other 56 P4-routed questions (gametext not in the top 10) were checked without a model.
**Result.** Real probe responds 2 to 4 of 6 changed (gained r094 "A lotta fooken questions m8s yes?", which now says the question is a colloquial expression rather than a lore question, and r102 "How big is this Chernobog ferris wheel (M8-1)?"; lost none; faithful 17/19 to 17/22). Trivia correct 2 to 2 of 2 (t107 adds the cause of Doc's hand wound). `ian8` keeps "extremely fond of sweets ... frosted donuts" and drops the childhood detail; `ian14` and `ian25` keep their substance. For the other 56, the p4x ranking with the gametext chunks removed equals P4's top 50 for 54, and for `ian13` and r070 it differs only from rank 12 on (a voice chunk of Angelina's and a story chunk move), below the 8 hits an "any" route reads and outside the operator-file units `ian13`'s route puts first, so their passages and answers are P4's.
**Decision.** 0 losses, so it is the default: `ask` reads `artifacts/p4x` instead of P4 for every P4 route (any, operator file and the typed kinds) since 2026-10-03; `--no-game-text` reads P4 as before.

### 2026-10-03: Outfit captions, a first third; `artifacts/p4-art` rebuilt

**Tried.** `art_captions.py caption --kind skin --seed 7` on Gemma 4 12B with its projector, for 28 minutes; then `art_captions.py units` and a rebuild of `artifacts/p4-art`.
**Result.** 164 of the 472 outfits captioned (median 8.2 s an image over all 543 captions; 168 captions in all now carry some visible text, from the whole-image pass, which units never serve), so 543 of 851 images have a caption (379 E2, 164 outfits). The tiled text pass (`text --kind skin --seed 7`) got no time, so no outfit has served lettering. `units` wrote 540 art units (172 with tiled text, all E2). The rebuilt `artifacts/p4-art` keeps P4's 22,917 chunks as its byte prefix, 164 new vectors, 23,457 chunks. Since `p4-art` is read by `--art` (opt-in) and by the typed fallback, the rebuild was checked without a model on the fallback's 34 candidates: the unit kind and the 3 typed units it picks are the same for all 34, and no outfit unit reaches the top 12 of any of them.
**Decision.** The measured fallback answers stand.
**Follow-up.** Resume with the same two commands (308 outfits to caption, then the tiled text of all 472).

### 2026-10-03: Outfit captions not resumed (leftover f)

**Result.** The model lock was busy with the A/B runs; 308 outfits still have no caption and none has tiled text.

### 2026-10-03: Deeper topic answers and Dossoles: named topic, topic events, place nation (items 3, 4 and 5)

**Why.** Diagnosis of `ian30`: the router sent "Who are the three main factions in dossoles?" to plain story retrieval though Dossoles is a topic; the Dossoles summary is about the city of act32side, and the factions are in Bolívar's summary ("the nominal government propped up by Leithanien, the Autonomous Government, and the True Bolívarians") and in Dossoles Holiday (`act12side_level_act12side_st03#0002`).

**Tried.** Three opt-in parts, measured together:
- `--named-topic` serves the summary of the one nation, place, race or organization topic a question names when the router chose none and the question names no operator (fires on 56 of 447);
- `--topic-events` adds the P2 summaries of the 3 groups with the most chunks naming the topic (`scripts/topic_groups.py`: Sarkaz main_14, main_13, main_10; Sami act15mini, act36side, act18d0; Dossoles act32side, act12side);
- `--place-nation` adds, after a place topic, the nation topic its summary names most (Dossoles: Bolívar; a trade, not a derivation: no table gives a place's nation, and it maps Chernobog to Yan).

**Result.** Changed answers: real responds 10 to 11 of 14 (gained r054, r104; lost r106), held-out 10 to 7 of 10 (lost h004, h017, h033: all three are declines in both arms with near-identical wording; judge noise by reading, losses as measured), gold correct 16 to 17 of 24 (gained g0116, unanswerable declined 9 to 9), faithfulness up (real 34/41 to 47/51, held-out 28/30 to 31/32). `ian30` becomes "The modern political landscape of Bolívar is defined by a "tug-of-war" between three primary factions [5]: the nominal government propped up by Leithanien, the Autonomous Government ..., and the True Bolívarians, ... seeking to establish a real country", which matches the expected three (the Singas dynasty is the Leithanien-backed one) with one of the three aims. `ian28` (Sarkaz) and `ian29` (Sami) gain little: the Sarkaz answer stays at Kazdel, horns, mercenaries and "devils", and the Sami answer moves from the northern front to "guards the endless icefields" and warriors who "sever the tracks of demons".

**Decision.** The package loses 3 held-out answers, so it stays opt-in.

**Follow-up.** Not built: summaries regenerated from many more passages (map-reduce); the event summaries were the cheaper half and did not deepen `ian28`.

### 2026-10-03: Outfit text pass, 47 more outfits, and the p4-art rebuild (night 8, item 4)

**Tried.** `art_captions.py text --kind skin --seed 7 --minutes 11` on the vision server; then `art_captions.py units`, and `artifacts/p4-art` rebuilt.

**Result.** 47 more outfits transcribed at 11.7 to 11.9 s each (145 of 472 outfits now have tiled text, 327 left; 489 tiled transcriptions in all, 266 with text). `art_captions.py units` wrote 846 art units (264 with tiled text) of 851 images; `artifacts/p4-art` rebuilt: 23,763 chunks, 23,736 vectors reused and 27 embedded, index rebuilt in 3.6 s.

**Decision.** Nothing reads it by default except the typed fallback; no answer was re-measured on it. The night 8 answers, judgments and job scripts are in `artifacts/answers/night8`.

### 2026-10-03: Term lookup from P4x with a glossary: `ian31` explains the Khaganquest from its record

**Tried.** A short what-is question (`asks_term`, the term without its article) is looked up in P4x as one to three consecutive words written together or apart, so "khagan quest" finds "Khaganquest" (the commonest surface form is the spelling). The 2 chunks of distinct stories naming it most, records and files first (archive, game text, IS, modules, items, enemies, outfits; Trevor's own summaries, dossiers and topics never), come before the passages, labelled with their kind and "it names <spelling>". Then comes a glossary of the served topics those chunks name (case-sensitive whole words, by first mention, the term itself excluded, at most 3, the first two sentences of each summary). When the question does not contain the spelling, retrieval searches with it appended. Measured on 447 questions; `--detect-only` prints `term=`.

**Result.** It fires on 4 of 447 questions: t000 and `ian29` unchanged (their terms, "indigo obsessed with" and "sami guarding", are not in the corpus), `ian28` and `ian31` changed, 0 losses.
- `ian31` ("What is the khagan quest?") goes from "a traditional rite of adulthood for the Kuranta" to "an ancient tradition and rite of adulthood for the Nightzmora, ... a warrior must leave their birthplace to travel to a distant site of trial, which they choose themselves, while carrying only a weapon gifted by their parents", citing Narantuya's file and the act13side hidden-story record (`gametext_story_activities_act13side_level_act13side_hidden_st03_4#0000`), with a glossary of Rhodes Island, Terra and Precursors (the first three served topics the two chunks name). It repeats the corpus's "mysterious and demonic army" for the Nightzmora, which the verifier's wiki reading calls an Elder subgroup of the Kuranta known for illusory Arts.
- `ian28` adds the Sarkaz talent for Arts and Originium, cited from Ifrit's file and an IS endbook (glossary: Terra, Kazdel, Rhodes Island).

**Decision.** Shipped as the default. `--no-term-lookup` is the kill switch (verified in the kill-switch check of 2026-10-04).

**Follow-up.** Generic terms crowd out the rare ones in the glossary, so ranking the glossary by how rare a topic is would be the next refinement.

### 2026-10-03: Unindexed game text added to P4 as gametext units

**Tried.** The coverage audit (item 6) found game text P4 did not index. `scripts/gametext.py` adds it as "gametext" units: story intros, unfetched story scripts, record notes, event archives, mail, operator record intros, world tips and event text.
**Decision.** Read for P4 routes by default since 2026-10-03; `--no-game-text` reads P4 as before.
**Code.** `src/bin/ask/cli.rs`, `GAME_TEXT_DIR`

### 2026-10-03: Origin chain: the dominant story's earlier chunks go first for why/how-it-began questions

**Tried.** For why/how-it-began questions, the story with the most chunks in the top 20 (at least 3) goes first: its chunks there that come before its best-ranked hit, without neighbours, in story order (at most 5), then that hit with its neighbours, then the other hits. `ORIGIN_RULE` asks the answer to start from the earliest cause the passages show.
**Result.** "Why did swire join the LGD?" was answered from her grandfather's shadow alone (`story_swire_set_1_story_1#0009/#0010`), though the same story opens with the kidnapping after which she decided to become a police officer (#0003). Retrieval ranks her story's L.G.D. years (#0009) 6th and its opening chunks 7th and 8th, and the 9,000-token budget keeps only the first 6 hits' clusters, so the kidnapping (#0003, 20th) and the badge (#0005) never reached the answer. Two earlier versions added nothing to this question: the earliest top-60 scene of the top 3 stories (her story is the 6th distinct one), and the dominant story's earlier chunks placed after the first 4 hits while "earlier" was measured from the lowest position among the hits (#0000, so nothing came first).
**Decision.** Default since 2026-10-03; `--no-origin-chain` is the kill switch.
**Code.** `src/bin/ask/retrieval.rs`, `passages` (was `retrieve_for`), origin chain; `src/bin/ask/prompts.rs`, `ORIGIN_RULE`

### 2026-10-03: Term lookup and the rarity-ordered term glossary

**Tried.** `--term-lookup` (night 9) gives the corpus's spelling of a term and the chunks naming it most. The term glossary lists the topics named with the term (at most `k`, each as the first two sentences of its summary).
**Result.** On night 9, "What is the khagan quest?" got Rhodes Island, Terra and Precursors, the generic topics.
**Decision.** With `rarity` (default since 2026-10-05) the rarest topics come first: fewest chunks naming one of the topic's forms, first mention breaking ties.
**Code.** trevor/src/bin/ask/terms.rs, `term_lookup`, `term_glossary`

### 2026-10-04: Outfit text pass: 292 more outfits transcribed, `artifacts/p4-art` rebuilt

**Tried.** `art_captions.py text --kind skin --seed 7 --minutes 66` on the vision server, then `art_captions.py units` and a rebuild of `artifacts/p4-art`.

**Result.** 292 of the 327 outfits left were transcribed (12.5 to 20.4 s each), so 437 of 472 outfits now have tiled text and 35 are left. `art_captions.py units` wrote 846 art units of 851 images, 451 with tiled text (264 before). `artifacts/p4-art`: 23,765 chunks, 23,577 vectors reused and 188 embedded, index 3.6 s.

**Follow-up.** The next run of the same command resumes the remaining 35 (about 8 min).

### 2026-10-04: `--art` re-measured on the rebuilt p4-art: two losses, stays opt-in

**Tried.** `--art` was re-measured on the 20 questions whose route it moved on 2026-10-03 (`artifacts/answers/night5/s4`), base and arm with the new p4-art in one Gemma session, judged by Qwen.

**Result.** 2 identical; trivia correct 4 to 6 of 7 changed (gained t052, Almond's knee crystal under a band-aid, and t056, Rosa's flowers and desserts); held-out responds 3 to 2 of 4 (lost h019 again); real 4 to 3 of 5 (lost r114 again; r053, lost last time, holds); gold 1 to 1. `ian14` answers "Pallas ... 162cm" from the model-written description, labelled as such.

**Decision.** Two losses to moved routes, so `--art` stays opt-in. The default is untouched by the rebuild on the 32-question probe set: the final default's answers equal the step-0 answers on the 30 questions term lookup does not change.

### 2026-10-05: Deep topic entries gated to whole-topic questions (`Tools::broad_topic_question`): default

**Why.** Diagnosis of night 9's two deep-entry losses from the answers: held-out h033 ("Do Liberi, as a culture, give their feathers as gifts?") and real r079 ("Do black Sankta halos adn energy wings mean something?") decline in both arms. The base's summary passage gave the one related fact the responds judge credited (Liberi tribes of the rainforest; Sankta halos and wings of light), and the 146- and 505-word entries put less related text in its place (a Sami operator's gifts; Mostima's black horns). Both ask about one detail, not the topic.

**Tried.** `Tools::broad_topic_question` (new) is the gate: the question names a served topic and, besides the topic's forms and function words, has at most one content word of three letters or more ("Who are the sarkaz?" 0, "What is sami guarding?" 1; h033 6, r079 5).

**Result.** Of the 14 topic-routed questions with an entry (the routes of `artifacts/answers/wiki-ref/routes.txt`), only `ian28` is broad. With the gate the other 13 are byte-identical to the default (14 of 14 answer records, h033 and r079 among them), and `--deep-topics-all` (measurement only) reproduces night 9's deep answers on 11 of 11 questions without a term lookup. `ian28` changes from Kazdel, horns, mercenaries, "devils" and the Arts talent to a history of suffering and injustice, "devils", "war criminals" and "inferior", racial memories spanning millennia, Oripathy, witchcraft linked to modern Arts, a wandering people after Kazdel's destruction, and the King of Sarkaz and the Royal Court. Qwen pairwise prefers the entry in both orders (graded correct both ways). Against the verifier's note it adds 3 of the items the note lists as missing (witchcraft drawing on racial memory, discrimination, the people's long history) while still omitting the original inhabitants (the Teekaz) and the many races; the judge-passed Sankta error of night 9 is not in this answer. 1 changed answer, a gain, 0 losses.

**Decision.** The gated deep entry is the default; `--no-deep-topics` is the kill switch.

### 2026-10-05: Term-lookup glossary ranks topics rarest first: default, 2 changed answers, 2 ties

**Tried.** `term_glossary` now orders the served topics the term's chunks name by how many P4x chunks name one of the topic's forms (case-sensitive whole words, Trevor's generated kinds excluded), first mention breaking ties.

**Result.** It changes only the 2 term questions whose term is in the corpus: `ian31` ("What is the khagan quest?") goes from Rhodes Island, Terra and Precursors to Kuranta, Grand Knight Territory and Gaul, and `ian28` from Terra, Kazdel and Rhodes Island to Terra, Kazdel and Originium Arts.
- `ian31` keeps the rite and the trial site, adds "on foot" and the spoils ("heads of prey, gold, or spices"), and still repeats the corpus's "demonic and mysterious army" for the Nightzmora (partial both ways against the verifier, which calls them an Elder subgroup of the Kuranta).
- `ian28` drops "themes of war, struggle, and exile" and adds that some Sarkaz preserve their culture, language and knowledge (partial both ways).
- Qwen pairwise: 2 ties (grades correct/correct and partial/partial). 0 losses.

**Decision.** Shipped as the default; `--no-glossary-rarity` reproduces the night 9 final answer records on both (answer, passages and citations, 2 of 2 against `artifacts/answers/night9/ans-ian-final.jsonl`).

**Caveat.** The scan is one pass over P4x's chunks per candidate topic, only on a term question with a glossary; its time was not measured apart from the answer (the answer records' ms covers generation only: 25.66 s against 26.61 s for `ian31`).

**Follow-up.** The Nightzmora are not a served topic (none of the 354 rows, aliases included), so no ranking can put them in the glossary; that needs a topic, not an order.

### 2026-10-05: Topic package losses h004 and h017 diagnosed: a displacement and judge noise

**Why.** Held-out h004 ("What's the difference between the Silverlance Pegasi and the rest of Campaign/gendarmerie knights?") under night 8's package: the Gendarmerie summary and two event summaries took passages [1] to [3], and the story chunk the base cited for the Pegasi being faster than a normal squad of campaign knights left the prompt, so the arm answered from the Gendarmerie summary alone. That is a displacement on a two-subject comparison where the named topic is one subject. h017 ("Do Rhodes Island have some form of a Wall of fallen operators?") has the same substance in both arms (the unclaimed-objects warehouse and the damaged shield): judge noise.

**Decision.** The general fix is the broad-topic gate on `--named-topic`: h004, h017 and h033 are not whole-topic questions, so the gated package does not touch them (by the detector; they were not re-answered).

### 2026-10-05: Scoped scenes for a question naming an event and a character

**Why.** The probe question "what is the name of the npc that mon3tr suggests assassinating in masses travels" (added as `ian33` to `eval/ian-questions.jsonl`, verified against `act42side_level_act42side_08_beg#0000`) was declined although the line is in the corpus, because nothing in the scene says "assassinate" ("Mon3tr: (I can take her out. Nobody would notice.)").

**Tried.** `Tools::event_in_question` finds the event a question names ("Episode 7"/"chapter 7", or the longest event name held as whole words, case and punctuation folded, without a leading "the", 6 characters or more: "in masses travels" is The Masses' Travels), but not when the event is named as a time boundary ("after Episode 10"). `Tools::event_characters` finds the characters of the appearance index (and their identity-link names) the question names who speak or are named in that event, minus race, nation, place and organization speaker labels ("Sarkaz"); `ask` keeps those the corpus writes capitalized (`proper_name`). The candidates are the event's chunks whose speakers or text hold a form; they are ranked by the question (dense and BM25 fused by reciprocal rank, k 60) and the best go first, up to 6,000 tokens (16 scenes at most), with `SCOPED_RULE` (match by meaning, an idiom or aside can answer the question, do not decline because the wording differs, quote the line). The retrieved passages then get 3,000 tokens, as with IS-ending passages, and do not repeat a scoped chunk. For `ian33` the scope is 20 chunks (8,878 tokens) and the line ranked 9th, so a first budget of 4 scenes left it out.

Reach: it fires on 2 of the 448 set questions (`r007` "To Be Continued"/Jesselton, `ian33`) and 27 of the 1,228 Reddit and Discord questions (before the time-boundary gate, also `r014` and its Reddit twin). Answers: one Gemma session, `--no-cache-prompt`, Qwen pairwise in both orders.

**Result.**
- `ian33`: base "I could not find it in the story text ... they do not mention Mon3tr suggesting the assassination of any specific NPC". With the first rule the line was passage [9] and Gemma still declined ("she does not suggest assassinating her; rather, she makes a paranoid suggestion"). With the final rule: "Mon3tr suggests that she could take out Idida, who is a girl associated with a dessert cart [9] ... "Nobody would notice" [9]". New wins, graded wrong to correct.
- `r007`: tie (both correct).
- `r014` ("What happened to Mandragora after Episode 10?") lost pairwise when scoped to Episode 10 (the shorter retrieval dropped the scene of her last moments), which is what the time-boundary gate is for: under the final binary `r014` is byte-identical to the base (answer and passages).
- 1 win, 1 tie, 0 losses on the questions it fires on.

**Decision.** Shipped as the default. `--no-scoped-scenes` is the kill switch (verified in the kill-switch entry below). `--scoped-only --batch` (hidden) prints the scoped chunk ids without a model; `TREVOR_SCOPED_DEBUG=1` prints the ranking.

### 2026-10-05: Scoped and character-scoped scene excerpts

**Tried.** Scoped scenes are placed first (2026-10-05), with a rule head saying that the question may name the act in other words than the scene uses. Character-scoped excerpts (`character_scenes`, 2026-10-06) get their own label and answer rule.
**Code.** `src/bin/ask/prompts.rs`, `SCOPED_RULE_HEAD`, `CHAR_LABEL`

### 2026-10-05: The deep-entry gate: only questions about a topic as a whole

**Tried.** Deep topic entries were given only to a question that names a served topic and, besides that topic's forms and function words, has at most one content word of 3 letters or more.
**Result.** "Who are the sarkaz?" (0 content words) and "What is sami guarding?" (1, "guarding") pass. "Do Liberi, as a culture, give their feathers as gifts?" (held-out `h033`) and "Do black Sankta halos adn energy wings mean something?" (real `r079`), the two answers the ungated deep entries lost on night 9, do not.
**Decision.** Shipped as the gate; a test pins it.
**Code.** trevor/src/tools/topics.rs, `Tools::broad_topic_question`; trevor/src/tools/tests.rs, deep-entry test

### 2026-10-05: Scoped scenes: the named event and the named character

**Tried.** A probe question, "what is the name of the npc that mon3tr suggests assassinating in masses travels", was declined although the line is in the corpus ("Mon3tr: (I can take her out. Nobody would notice.)", `act42side_08_beg#0000`): the whole-corpus search never ranks it, because nothing in the scene says "assassinate". Scoped scenes add the scenes of the event a question names where the character it names speaks or is named, at most `SCOPED_K` scenes; the retrieved passages then get `SCOPED_REST`.
**Result.** With 2,400 tokens (4 scenes) the Mon3tr scene ranked 9th of the 20 candidates (8,878 tokens) and was left out. Scoping to Episode 10 lost real `r014` pairwise, because that question asks about what lies outside the event.
**Decision.** Default since the 2026-10-05 night run; `--no-scoped-scenes` turns it off. The event detector reads "Episode 7" or "chapter 7" and event names without "the" or the apostrophe (tests pin the detectors). A question that asks about what lies outside the named event is a known loss case for scoping (`r014`).
**Code.** trevor/src/bin/ask/scenes.rs, `scoped_scenes`, `SCOPED_K` / `SCOPED_TOKENS`; trevor/src/tools/appearances.rs, `Tools::event_in_question`; trevor/src/tools/tests.rs, appearance test

### 2026-10-06: Character-scoped scenes for a question naming one character and no event

**Why.** `ian12` ("During which event Mon3tr suggested assassinating a child") was declined because the night-11 scoped scenes need a named event. The whole-corpus search cannot reach the line: `act42side_level_act42side_08_beg#0000` ranks 6,619th dense, 93rd BM25 and 195th fused of 16,050 P3b chunks (`search --mode`; P4x 7,536, 92 and 192 of 24,616).

**Tried.** `character_scenes` fires when the question names no event and is no appearances question, `people_named` finds exactly one character (not a nation, place, race or organization topic), and `asks_scene` makes that character the subject of an act: one of their names followed, with at most one word between, by a said verb ("said", "suggested", "promise", "mention", ... 39 forms) or, when the question asks which or what event, story, chapter, episode, scene or stage, by a past-tense word. A first reading (any said word or place phrase anywhere in the question) also fired on `r038` ("Jokes aside, can Originium slugs be pets"), `r075` ("Frostleaf's hat say"), `t058` ("what does the captain say about elysium") and `g0079` (a "Kazimierzian businessman"), and was narrowed before any answer was run. The final detector fires on 8 of the 448 set questions (`g0004`, `g0012`, `g0029`, `g0049`, `g0091`, `g0103`, `g0110`, `ian12`) and 3 of the 1,228 Reddit and Discord questions (`rd368`, `rd431`, `rd544`).

Ranking, measured among Mon3tr's 172 speaking scenes (421 lines of hers): the scoped scenes' dense + BM25 fusion over chunks puts the scene 71st, chunk dense 158th, line dense (each of her lines with the line before it, embedded at question time in 7.5 s) 134th, so the bi-encoder is not paraphrase-tolerant here. The reranker (the Ettin cross-encoder, loaded only when the detector fires) reading the question with each line and the line before it puts it 1st, in 0.97 s (0.04 s for Necrass's 8 lines to 2.48 s for Talulah's 884).

Each picked line keeps 2 lines either side, at most 2 lines a scene and 24 scenes, merged per scene with "..." between pieces, labelled "story: <event>, <stage>; an excerpt of a scene in which <name> speaks", with `CHAR_RULE` (the scoped rule's match-by-meaning wording, plus: name the event from the label when the question asks for one).

**Result.**
- Version 1 (the excerpts within 6,000 tokens and the retrieved passages cut to 3,000, as for scoped scenes): gold correct 3 to 2 of 7 (lost `g0004`: the retrieved "I won't burn Rhodes Island, or my homework" fell out of the shorter retrieval), `ian12` declined to right, Qwen pairwise 1 win (`g0091`) and 1 loss (`g0004`).
- Version 2: the excerpts take at most 3,000 tokens on top of the retrieved passages' whole budget (2 to 14 scenes on the 8; `ian12` 9 scenes, prompt 7,801 to 10,226 tokens). Gold (`answer-eval.py`, Qwen, `CORPUS=artifacts/p4x`): correct 3 to 3 of 7 with the same verdict on every question (`g0004`, `g0012`, `g0091` right both ways; `g0029`, `g0049`, `g0103`, `g0110` wrong or declined both ways), cites a gold chunk 5 to 4 (`g0049`, judged wrong in both arms, now cites the excerpt of the same scene), faithfulness 9/10 to 10/12; Qwen pairwise in both orders 1 win (`g0091`), 7 ties, 0 losses.
- `ian12` under the default: "In the story "The Masses' Travels, Pilgrimage," Mon3tr suggests that she could kill the Gap-Toothed Girl without anyone noticing [1]. This paranoid suggestion is made while the girl is offering Mon3tr a cup of ice cream [1]", right against the probe set's expected answer (the girl is Idida; the scene's speaker label is Gap-Toothed Girl).

**Caveat.** The pairwise judge (`artifacts/answers/night12/pairwise.py`) answered A in both orders on 7 of 8, so its ties are weak evidence; the gold verdicts are the bar.

**Decision.** Version 2 is the default (0 losses). `--no-character-scenes` is the kill switch. `--detect-only` prints `charscope=<name>`, `--scoped-only --batch` prints the excerpts' chunk ids and tokens, `TREVOR_SCOPED_DEBUG=<chunk id>` prints that chunk's rank under the fusion and the reranker. Records: `artifacts/answers/night12/` (N2 = version 2, N = version 1, B = the old default).

### 2026-10-06: ian6 is out of reach of any measured retrieval; nothing built

**Tried.** Ranked the target line for `ian6`, "What operator declared that they want to pee in public?" (Chiave, `act15d5_level_act15d5_st02#0000`, "why don't we see who can piss farther").

**Result.** For the raw question it ranks 5,754th dense, 6,790th BM25 and 7,528th fused of 16,050 P3b chunks (P4x: 7,400th, 8,204th and 9,273rd of 24,616). Dense-only top-k widening for a question with no named entity would need k of at least 5,754 (36 % of P3b). The character scope does not apply (the question names a category, "operator", not a character), and the reranker that finds Mon3tr's line works inside a 421-line scope: P3b holds 225,077 lines, which at the measured 430 lines a second is about 520 s a question.

**Decision.** Nothing built. Top-k widening is ruled out by the rank; a whole-corpus line pass is not affordable.

**Follow-up.** A vocabulary bridge ("pee" to "piss") or an index of short line units, neither measured.

### 2026-10-06: A line-level dense index does not reach single-line facts that chunk retrieval misses

**Tried.** A study (`ask --line-study --batch`, hidden, no model server) made every non-empty line of the 16,050 P3b chunks outside the generated kinds, together with the line before it, a retrieval unit: 221,517 units. The gte INT8 embedder reads them at 51.9 units a second (7,384 in 142.2 s), so a whole line index would cost 4,268 s (about 71 minutes) to build and 170 MB as int8 vectors (680 MB as f32), against 49 MB for the chunk vectors. The rank of the evidence line among all units was estimated on a stride sample (every 30th unit for line dense, every 300th for the reranker: sample units scoring above the evidence, times the stride), next to the evidence chunk's exact dense, BM25 and fused (RRF k 60) ranks. The set is 87 single-line questions: `ian6`, `ian12` and `ian33` with their lines, plus the 84 gold v2 questions with one evidence quote and one anchor. Trivia was left out: its rows give a chunk and an expected answer, not an evidence line.
**Result.** Line dense puts the evidence line in its top 200 for 68 of 87 (median rank about 31). The chunk fusion has the chunk in its top 200 for 83 and in its top 16 for 67 (median 4). Line dense is worse than the chunk ranking it would supplement.
- `ian6` ("What operator declared that they want to pee in public?"): line dense about 19,411th; the reranker over a 739-line sample put 7 lines above it, about 2,101st of 221,517. Even a whole-corpus cross-encoder pass (about 520 s) would not surface "why don't we see who can piss farther". Chunk ranks: 5,754 dense, 6,790 BM25, 7,520 fused.
- `ian12` and `ian33` (Mon3tr's "I can take her out"): line dense about 14,401st and 86,971st; the reranker puts them in the top 301.
**Decision.** Ruled out by measurement, nothing indexed: a line dense index as a candidate source (68 of 87 against the chunk fusion's 83 of 87, for 71 minutes and 170 MB), and with it any line-level fix for `ian6` (bi-encoder 19,411th, cross-encoder about 2,101st).
**Follow-up.** `ian6` stays open: neither model reads "pee in public" as "piss farther". A stated vocabulary bridge or a stronger reader would decide it. Records: `artifacts/answers/night13/study.tsv`, `set.jsonl`.

### 2026-10-06: Line pool: reranking the lines of the hybrid top 100 chunks (`--line-pool`)

**Tried.** The same study with no index: the lines of the fused top 100 chunks (median 1,432 lines) reranked against the question, in a median 3.76 s (at most 4.47 s). Of the 17 gold questions whose chunk the fusion ranks below 16th, 8 are answered wrong without the gold chunk among their passages (gold146 records) while line dense has the line in its top 200. `--line-pool` reranks the pool lines inside `answer_from`. When the best line lies outside the retrieved passages, it adds the best 4 lines outside them (at most 2 a chunk, 2 lines either side, merged per chunk) after the retrieved passages, within 1,500 tokens on top, labelled "an excerpt around a line that matches the question". `--line-pool-all` drops the outside-the-passages gate. Measured with one Gemma 12B session (-np 1, `--no-cache-prompt`, one client) on 30 gold questions: the 17, plus 13 controls drawn with seed 6 from those the fusion ranks in its top 16. Gold judge `answer-eval.py` (Qwen, `CORPUS=artifacts/p4x`) on the changed answers, Qwen pairwise in both orders, and the 33-question probe set (`eval/ian-questions.jsonl`) against the night-12 records.
**Result.** Pool recall on the 87: evidence line in the top 5 for 52 of 87 and the top 20 for 66; 6 are outside the pool (`ian6`, `ian12`, `g0005`, `g0008`, `g0011`, `g0036`). The pool ranks the 8 missed lines 3, 1, 3, 1, 1, 1, 4 and 48 (`g0003`, `g0016`, `g0032`, `g0038`, `g0041`, `g0047`, `g0074`, `g0095`).

On the 30 gold questions the pool fires on 17 (15 of the 17, 2 of the 13); 74 of 91 rendered prompts and 16 of 30 answers are identical. The gold chunk enters the prompt for 7 (`g0016`, `g0032`, `g0038`, `g0041`, `g0047`, `g0071`, `g0074`) and is cited by all 7. Gold judge on the 14 changed answers:

| measure | base | line pool |
|---|---|---|
| correct | 4 | 10 |
| declined | 4 | 1 |
| gold chunk among passages | 3 | 10 |
| faithfulness | 23/28 | 27/28 |

Gained `g0016`, `g0032`, `g0038`, `g0041`, `g0047`, `g0071`, `g0074`; lost `g0034`. Qwen pairwise both orders: 6 wins, 0 losses, 8 ties.

On the 33-question probe set it fires on 8 (`ian1`, `ian2`, `ian5`, `ian15`, `ian17`, `ian27`, `ian30`, `ian31`); 25 answers are byte-identical to the night-12 records; pairwise 0 wins, 2 losses, 6 ties. `ian17` now ends "Estimate: 1086" and drops the 1090 bound; `ian30` goes from a decline to "the Coalition government, La Unidad, and an unnamed third faction". `ian5` loses the "burdenbeast named Lily" garble and `ian31` no longer calls the Nightzmora "a demonic and mysterious army" (both ties). Cost: answers took 34.2 s instead of 28.9 s on the 30.
**Decision.** Not 0 losses, so opt-in (`--line-pool`).
**Follow-up.** A gate that keeps the 7 gold gains without the 3 losses was not measured. The reranker score of the best pool line on the 10 changed-verdict questions would decide whether a threshold separates them; with 10 points it would be a fit to check on held-out questions. Records: `artifacts/answers/night13/` (`B.gold`, `LP.gold`, `LP.ian`, `*judged*`, `pw.*`).

### 2026-10-06: No reranker-score threshold gives the line pool 0 losses (`--line-pool-min`)

**Tried.** `--line-pool-min F` fires the pool only when the best line outside the retrieved passages scores at least F (reranker logit). `TREVOR_LINE_POOL_DEBUG=1` prints each question's gate and best score; `TREVOR_LINE_POOL_DETECT=1` (measurement) skips the answer call. The scores of the 30 gold questions (29 reach the pool) were read in one Gemma session before any of the probe-set scores. The threshold was written down before the probe-set scores were read: 8.6, just under the lowest gain (`g0016`).
**Result.** Scores of the questions the ungated pool changed: `g0034` (lost) 10.175, `g0032` 10.054, `g0041` 10.032, `g0038` 9.631, `g0047` 9.283, `g0071` 9.030, `g0074` 8.985, `g0015` 8.935, `g0101` 8.835, `g0016` 8.606 (the 7 gains are `g0032`, `g0041`, `g0038`, `g0047`, `g0071`, `g0074` and `g0016`), then `g0008` 8.337, `g0028` 8.151, `g0060` 8.056, `g0036` 7.643. The loss `g0034` has the highest score of all 29, so every threshold that keeps a gain keeps `g0034`, and the only 0-loss threshold (above 10.175) fires on nothing.

At 8.6 the gate fires on 12 of the 29 gold. A fired question gets exactly the ungated excerpts, so its gold verdicts are the ungated arm's: correct +7, -1 (`g0034`); pairwise 5 wins (`g0016`, `g0032`, `g0038`, `g0041`, `g0047`; `g0060`'s win no longer fires), 0 losses. On the 33-question probe set it fires on 3 (`ian15` 9.090, `ian30` 8.830, `ian31` 9.552): `ian17` (6.003), `ian1` (7.743), `ian2`, `ian5` and `ian27` drop out, but `ian30`, a pairwise loss, still fires with the same prompt. On the real probe it fires on 15 of the 96 questions detected (`r012`, `r016`, `r017`, `r034`, `r038`, `r041`, `r043`, `r054`, `r055`, `r058`, `r063`, `r078`, `r079`, `r087`, `r089`) before the memory watchdog stopped the session (swap 15,990 to 18,060 MB, +2.07 GB, pressure at warning). The last 24 were not detected and the 15 were not answered, so the real-probe set judge and pairwise were not run.
**Decision.** Ruled out by measurement: a minimum-score gate as the route to a 0-loss default (`g0034` outranks every gain). `--line-pool-min` is built, opt-in, not default. Without it the opt-in pool is the ungated one; the default path does not reach the pool at all.
**Caveat.** The gold and probe-set verdicts at 8.6 are derived from the ungated arm's records on the reading that a fired question's prompt is the same. No spot check reran one, so this is unverified. Records: `artifacts/answers/night13/gate.*`.

### 2026-10-06: Character-scoped scenes, reranked by line (`ian12`)

**Tried.** `ian12`, "During which event Mon3tr suggested assassinating a child", names no event. A line study (`--line-study`, measurement only, item 1 of the request list) scored every line of the corpus outside the generated kinds. Character scenes: a question that names no event, names one character (`people_named`) and makes them the subject of a said or done act (`asks_scene`) gets excerpts of the scenes where that character speaks, placed first, on top of the retrieved passages' whole budget.
**Result.**
- Whole-corpus search ranks the chunk `act42side_08_beg#0000` 6,619th dense, 93rd BM25 and 195th fused of 16,050 in P3b.
- Among Mon3tr's 172 speaking scenes (421 lines) it ranks 71st by the scoped scenes' dense + BM25 fusion, 158th by chunk dense and 134th by line dense (the bi-encoder is not paraphrase-tolerant here).
- It ranks 1st when the reranker (a cross-encoder) reads the question with each of her lines and the line before it, in 0.97 s.
- A first version took the scoped scenes' 6,000 tokens and cut the retrieved passages to 3,000: it lost gold `g0004` (the retrieved "I won't burn Rhodes Island, or my homework" fell out).
**Decision.** Default since 2026-10-06; `--no-character-scenes` turns it off. Each picked line keeps `CHAR_WINDOW` lines either side, at most `CHAR_LINES_PER_SCENE` lines a scene and `CHAR_K` scenes, within `CHAR_TOKENS`. The scenes go before the retrieved passages, which keep their whole budget.
**Code.** trevor/src/bin/ask/scenes.rs, `character_scenes` and `line_study`; trevor/src/bin/ask/answer.rs, `answer_one_inner` (character scenes)

### 2026-10-16: Group evidence: each group an opinion question compares gets its own chunks

**Tried.** An opinion question naming two or more groups gets chunks for each group first, before the question's and the evidence query's hits. Measured on `ian34`.
**Result.** With only the question's and the evidence query's hits, "Karlan Trade Group and Blacksteel" filled all 15 passages with Karlan Trade and Silverash, and the answer said Blacksteel is not mentioned (150 P3b chunks name it).
**Decision.** Default since 2026-10-16; `--no-group-evidence` is the kill switch.
**Code.** `src/bin/ask/retrieval.rs`, `passages` (was `retrieve_for`), group evidence; `src/bin/ask/cli.rs`, `Args::no_group_evidence`

### 2026-10-16: BM25-only retrieval for serving without a query embedder

**Tried.** A `--retrieval` switch: `hybrid` (dense + BM25) or `bm25`, which never loads the dense index or the query embedder and runs every dense or hybrid retrieval as BM25. The VPS has no INT8-safe query embedder.
**Result.** Dev set recall@10 0.862 for `bm25` against 0.915 for hybrid.
**Decision.** `hybrid` stays the default; `bm25` is for hosts without a usable query embedder. `TREVOR_RETRIEVAL=bm25` sets it for a worker.
**Code.** `src/bin/ask/cli.rs`, `Args::retrieval`

### 2026-10-18: Scoped scenes for "chapter 15 ... Priestess" and "Shu in Here A People Sows": ranking, a line rerank (refuted) and story summaries (opt-in)

**Why.** Two preview questions (stories page, spoilers on, no horizon, profile mid on E2B) were declined. Truth from the game text: (a) "In chapter 15, when Priestess separates the Doctor, Amiya, and Kal'tsit from the elite operators, where does she take them?": after the rift the elite operators find themselves alone (`main_15_level_main_15-15_beg#0003`, "Where's the Doctor? ... And where are Dr. Kal'tsit and Amiya?"), and Priestess says "Let's go for a walk outside" and stands with the two of them on the deck of the Rhodes Island landship, where she clears the Catastrophe clouds from the sky (`main_15_level_main_15-15_beg#0000` and `#0001`). (b) "How did Shu come back in Here a People Sows? She died, and I believe was revived.": act31side. Shu vanishes during the demonic contamination (`act31side_level_act31side_05_end#0001`, `#0002`: "Why would she do this?"), a shred of her consciousness stays tied to the land (`08_beg#0003`), she calms the newborn consciousness in the fields (`08_end#0001`), and she returns in the last fight: "I didn't think you'd just go and disappear altogether" / "Think of it as a much-needed rest ... the scene I come back to" (`09_end#0001`). The text never says she was revived; it calls her absence a rest.

**Result.** Both questions reach the scoped scenes (`--detect-only`: `scoped=main_15:Priestess?|Kal'tsit|Amiya?|Amiya`, `scoped=act31side:Shu`), so event detection is not the cause, and under `mid` the scoped scenes keep their 6,000 tokens on top of the 3,000-token rest (prompt 9,000 to 10,000 tokens).
- (a) is a ranking miss: 130 candidate scenes (62,296 tokens); the dense and BM25 fusion ranks `15-15_beg#0003` 8th and the deck scene `#0000` 57th, outside the 11 scenes that fit.
- (b) is not a retrieval miss on the current binary: `09_end#0001` is the 5th scoped scene. E2B (mid, `--no-cache-prompt`) answers "Shu came back because of the scene he returned to ... [5]", citing it; the preview's decline came from an older worker binary (started 07:59, the binary rebuilt 08:32).
- A line rerank of the scoped candidates (each line with the line before it through the Ettin cross-encoder, chunk score = best line, 1,389 lines in 4.23 s and 816 in 2.07 s) moved (a)'s deck scene from 57th to 21st (still outside) and (b)'s return scene from 5th to 19th (out of the prompt). Refuted and removed.
- Trevor's story summaries of the scoped scenes' stories, after the scenes, within 1,600 tokens (`--scoped-summaries`): the 15-15 summary says the three "encountered ... the Priestess on the deck of Rhodes Island", and enters (a)'s prompt as one of 7 summaries. E2B still declines (a) and cites another summary. For (b) the answer turns to the 05_end summary ("Shu died in the fields ... her trapped soul found peace"), a claim the script only implies. On the 2 eval questions it touches (no-model `--scoped-only` over the probe set, real probe, held-out and gold v2): `ian33` wrong in both arms ("assassinating Kal'tsit" against "Exusiai should be contacted"), `r007` a tie. No verdict changed, and the 12B was not run (37% free memory, the 12B needs about 9.2 GB).

**Decision.** No default change for scoped scenes. `--scoped-summaries` is built and opt-in. Open: a ranking that reaches a scene described only by its moment ("when X happens, where ...") within a 130-scene scope; and whether the `mid` profile should cap the scoped scenes to its own 3,600-token budget (the VPS slot was sized for 3,600).
**Code.** `src/bin/ask/scenes.rs`, `scoped_summaries`, `SCOPED_SUMMARY_TOKENS`; `src/bin/ask/cli.rs`, `Args::scoped_summaries`

### 2026-10-18: Idida: the hint verbs and the scoped line rank (no model yet)

**Why.** "What NPC did Mon3tr hint at wanting to take out with no one noticing" (E2B, mid) answered Kal'tsit. The truth, from the game text: in The Masses' Travels, `act42side_level_act42side_08_beg#0000`, a gap-toothed girl pushes ice cream on Mon3tr, Mon3tr thinks "(I can take her out. Nobody would notice.)", and the girl introduces herself: "Dadadeedeeda, I'm called Idida." (5 P3b chunks name Idida).
**Tried.** The mid prompt, rendered with no model (a stub llama-server answering `/props`, `/apply-template` and `/completion` with recorded or canned text, `--no-cache-prompt --dump-prompt`): the question names no event, and "hint" is not a said verb, so it got no character scenes; its 7 retrieved passages (Dissociative Recombination, Abnormal Spectrum, The Masses' Travels "Voyage") hold no Idida line. `ian33` ("the npc that mon3tr suggests assassinating in masses travels") gets the scoped scenes, but their fused ranking left `08_beg#0000` out of its 7 passages. Two retrieval changes:
- the hint verbs (`detect.rs` HINT_VERBS: hint, imply, insinuate, allude and their forms) count as said verbs in the scene-subject test, so the Idida question gets Mon3tr's character scenes, ranked line by line by the reranker (`--said-verbs-v1` turns them off);
- the scoped line rank (`--no-scoped-line-rank`): when the question makes a scoped character the subject of a said act, the scoped scenes holding that character's best lines by the reranker (each line with the line before it) go first, then the rest in the fused order. The 2026-10-18 line rerank of every scoped question (refuted, above) ranked all lines; this one is gated on the said act and reads only the character's own lines.
**Result (no model).** `08_beg#0000` goes from absent to passage [1] of the Idida question and from absent to [2] of `ian33`; `ian12` keeps it at [1]; the Priestess and Shu questions (no said act) keep their passages exactly. Over the 1,754 questions of the probe set, real probe, held-out, gold v2, trivia, `routes.jsonl` and the Reddit and Discord sets, the hint verbs change 0 detections, and the scoped line rank changes the scenes of 2 of the 34 questions with scoped scenes (`ian33` and Discord 7, "Is the Black Knight mentioned in the Near Light event the operator Blacknight", where "mentioned" is a said verb). With both switches the 341 prompts of the four sets are byte-identical to HEAD's.
**Decision.** On by default pending the model window (E2B mid: the Idida question, `ian33`, `ian12`, Discord 7, both arms); after window 1 the scoped line rank was replaced by scoped excerpts (entry below, under Answer rules). Not fixed: E2B misreading evidence that is in the prompt (the Ep15 Priestess and Shu questions) is a model limit.
**Code.** `src/bin/ask/detect.rs`, `HINT_VERBS`, `asks_scene_with`; `src/bin/ask/scenes.rs`, `ranked_lines`, `scoped_scenes`

## Routing

The routers (model, keywords, the retired kNN and hybrid), the table tools and their arguments, route checks, question detection.

### 2026-09-27: Keyword routes to the tables for whole-corpus questions

**Tried.** Requirement: whole-corpus questions are answered from Trevor's tables instead of a few passages.
**Result.** "every story in chronological order" had listed six items from 16 passages.
**Decision.** Shipped as keyword routes.
**Code.** trevor/src/bin/ask/routing.rs, `keyword_route`

### 2026-09-28: Table-route misfires and open listings

**Result.** 6 of 7 table answers to 120 real Reddit questions were misfires: questions such as "why Mostima treats Exusiai the way she does?" name several entities and must go to retrieval. A probe question asking for the real names of every operator got 7 names from 17 passages, presented as the answer.
**Decision.** Such questions are left to retrieval. "List all X of every operator" with no table behind it is detected as an open listing, since retrieval sees only a sample.
**Code.** trevor/src/bin/ask/routing.rs, leftover rule; trevor/src/bin/ask/detect.rs, `is_open_listing`

### 2026-09-29: Keyword routes become tools picked by a router

**Tried.** Requirement: table routes that answer without the model looked hardcoded, and the assistant should scale rather than grow keyword rules. The keyword routes became tools with structured arguments (`src/tools.rs`: 14 functions plus `canon` and `topic`), picked by one grammar-bound Gemma call (`--router model`, the default), by embedding kNN over labelled questions (`--router knn`), or by the old rules (`--router keywords`). Arguments resolve by id, episode number, folded name, identity links and a one or two letter misspelling allowance; anything unresolved falls back to retrieval. Measured on the route check (`eval/routes.jsonl`, 72 cases; the 11 new cases for canon, topic and storylines apply to the model and kNN routers only).
**Result.** Keywords 61/61. Model 57/61 on the first prompt, 71/72 after general rules and after removing the 7 route-check questions from the prompt's examples (the miss: "what's the deal with Sami, why is it so dangerous" goes to plain retrieval instead of the Sami topic). kNN 30/72, with no wrong table answer, because only 107 of 1,140 labelled real questions want a table. The routing call costs 706 ms per question (median 683, p90 828).
**Decision.** The model router is the default. `--router keywords` is the kill switch: its `--route-only` output is byte-identical to the old binary on all 1,266 real and route-check questions.

### 2026-09-29: New tools: canon, Storylines reading guide, topic, exact recap

**Tried.** `canon {run}` reports, per IS ending, the later stories that continue it (artifacts/canon/is_endings.json, from the contrastive pass in scripts/canon_refs.py), with a strength label and split evidence stated, and says the game never labels an ending canon. The reading guide now leads with the game's own Storylines links (18 named read-before links from stage_table); community rules are marked as agreeing with the game (4) or labelled player advice. `topic` prepends a generated topic summary as passage [1], labelled as generated, ahead of the retrieved passages. The recap tool now matches an episode's own stories exactly, under the model and kNN routers only.
**Result.** Recap for Episode 1: 12 stories, was 211 by substring.
**Decision.** Shipped. `--no-storylines` restores the old reading-guide text; `--no-topics` drops the topic summary.

### 2026-09-29: Real probe: model router against keyword router

**Tried.** Same binary, both routers, on the real probe (120 real Reddit questions, Gemma answers, Qwen judges; runs `real-probe-rmodel` and `real-probe-rkeywords`).
**Result.** Responds 0.608 against 0.575, 4 gained and 0 lost (sign test p 0.125, not significant), all four from the new tools: two topic questions ("Were the Sankta a distinct 'imp' sub-race of Sarkaz?", "The halos?") and two newcomer questions now routed to the reading guide. Handles 40/43 against 41/43; the one change is "Chronology Events of Arknights?", now answered with the chronology listing, which the judge's handles prompt reads as a verdict. Faithful 233/268 (0.869) against 241/282 (0.855). Tables answered 8 against 4. 107 of 120 answers are identical.
**Decision.** The model router stays the default; keywords stays as the kill switch.

### 2026-09-29: Router choice: model, keywords or off

**Tried.** The router that picks a table tool before retrieval is selectable: `model` (one grammar-bound call to the answer model), `keywords` (the keyword rules of 2026-09-29), or `off` (no table tools; the policy lines and timeline notes stay, unlike `--no-route`).
**Result.** `keywords` (the kill switch) gives identical output on all 1,266 real and route-check questions.
**Decision.** `--router keywords` is the kill switch.
**Code.** `src/bin/ask/cli.rs`, `Args::router`

### 2026-09-29: Table-tool arguments repaired after model-router misfires

**Tried.** The model router's table-tool calls were checked against what it wrote for real questions.
**Result.** On 2026-09-29 it wrote `{"run": "all"}` for "any of the IS endings"; passed a branch with its class attached ("... defenders"); and wrote `{"place": "Soloblade"}` for "List all Soloblade operators". On 2026-09-30 it wrote "Masses Travel" for The Masses' Travels on a probe question.
**Decision.** The canon tool accepts the `all` run. Branch values resolve with the class word on. A value in the wrong field moves to the field it resolves in (only a place that is no place moves). Event names match without a leading "the" and without each word's trailing "s".
**Code.** trevor/src/tools/mod.rs, `Tools::run` (canon arm); trevor/src/tools/operators.rs, `Tools::resolve_value` and `Tools::operators`; trevor/src/tools/resolve.rs, resolve event (keyed match)

### 2026-09-30: kNN router trained with paraphrased questions: refuted

**Tried.** Gemma rewrote the 132 training-split table questions 20 times each (57 min); the model router confirmed 1,387 of 2,352 rewrites (`eval/intents.synth.jsonl`, `--knn-synth`). Measured on the same 231 held-out real questions (35 want a table), with several embedders including base gte and a fine-tuned one.
**Result.** kNN alone at its leave-one-out training pick answers at most 2/35 from a table with any embedder, with or without rewrites; tool accuracy stays 0.835 to 0.870. At looser thresholds the rewrites buy recall with precision (base gte: 5 of 6 right, recall 5/35, becomes 13 of 29 right, recall 13/35), mostly by sending real retrieval questions to topic ("Can you contract Oripathy by getting shot?").
**Decision.** Refuted: rewrites do not make kNN usable, and the fine-tuned embedder is no better than base gte. `--knn-synth` is off by default; off reproduces the round-2 evaluation exactly.

### 2026-09-30: Hybrid router (kNN when unanimous, else the model)

**Tried.** `--router hybrid`: kNN decides when all 5 neighbours agree, otherwise the model router decides. Same 231 held-out real questions, plus the route check.
**Result.** Skips the routing call on 98/231 real questions (0.424) at 0.978 agreement with the model router and 30 of 30 table answers right, recall 30/35; with rewrites it skips only 0.10. Route check: hybrid 61/72 against model 71/72, because kNN confidently sends table questions whose neighbours are all retrieval ("Who died in Babel?", both IS canon questions) to retrieval.
**Decision.** The model router stays the default. The hybrid saves about 0.3 s per question on answers that take 28 s, and costs 10 route-check cases; it is kept as an option for a serving setup where the routing call matters.

### 2026-09-30: The model router names the source of a retrieval route

**Tried.** Real-probe category J (source-specific) responded 1 of 7 on the model router, and the typed-source corpus (P4) was chosen by a keyword list. The retrieval route now carries a `source` (story, operator_file, module, voice, skin, is, enemy, item, any) chosen by the router: a typed source goes to P4 with that kind's best 3 chunks first, "any" to P4 without preference, story to P3b. Measured on the real probe and gold set v2 questions whose route changed, plus the J set (Gemma answers, Qwen judges), and on the route check.
**Why.** Diagnosis of the six failing J questions, without a model: the Angelina outfit text is in P4 (its Bloodline of Combat skin entry ranks first in P4 retrieval) but not in P3b; Mon3tr's operator file is in both and was retrieved, so that miss was the answer's wording ("does not rule on translation"); Kal'tsit's boiling water line is a main-story line (Episode 7, `main_7_level_main_07-05_beg`), not a voice line, and neither corpus ranks it in the top 8 for the real phrasing (third for "Kal'tsit boiling water"); Dorothy's operator record ("Where is the Friend's House?") is in both, and the old answer denied it existed; "Cake Embryo" and the scalpel quote are in no text Trevor has (0 matches in P4's 22,917 chunks).
**Result, first prompt.** Without the source the model router is the previous one exactly (`--route-only` identical on all 120 real-probe and 146 gold questions), and the keyword router keeps its list. On the 31 real-probe questions whose route changed plus the J set: responds 13 to 17 (0.419 to 0.548), faithfulness 40/53 to 57/70; J 1 to 4 of 7 (Kal'tsit, Mon3tr, Dorothy; one of these, Kal'tsit, is a judge error: the answer declines); gains also on the Tin Man IS2 encounter and the W vs Ines CG knife; one loss, "Were the Sankta a distinct 'imp' sub-race of Sarkaz?", which lost its topic passage. That prompt also moved 8 questions from the topic tool to plain retrieval and 2 operator-attribute questions to retrieval. Gold set v2: 4 questions change corpus (3 to operator files or any, 1 from topic to retrieval), all 4 judged identically before and after (3 correct, 1 unanswerable declined). Route check 70/72: the Sami miss as before, and "is the phantom roguelike ending canon or not" now fails on its expected text because the rebuilt canon data (`is_endings.json`) no longer ties IS2, not on its route.
**Result, second prompt.** A rule that the source applies only after the tool is chosen, so a race, nation or concept question stays topic and one operator field stays operator_attribute, with differently worded topic and attribute examples. Against the flag-off router it moves 30 of 120 real-probe questions (9 to a named source, 14 to "any", 4 from retrieval to topic, 1 from topic to retrieval, 1 to a recap, 1 to a longer event argument) and 4 of 146 gold questions; route check 71/72 (Sami still misses the topic). On those 30 plus the 2 unmoved J questions: responds 7 to 14 of 32 (0.219 to 0.438), faithfulness 37/52 to 48/63 (0.712 to 0.762), 7 gained, 0 lost (Tin Man IS2, Kal'tsit (a judge error, the answer declines), Mon3tr, Dorothy, the W vs Ines knife, Draco's fire Arts through the topic, and the Joy of Our Reunion takeaways through a recap); J 1 to 4 of 7. Two answers gained an unsupported sentence (Mon3tr 2/2 to 1/2, "When exactly does the R6 collab happen?" 1/1 to 1/2). Gold: the 3 moved questions of the first run judge the same, and g0117 (now voice lines) goes from incorrect to correct.
**Decision.** Made the default: 7 gained and 0 lost on responds, gold 1 gained and 0 lost, faithfulness on the moved set up overall, the two single-sentence drops within sentence-level judge noise. `--no-router-source` restores the previous model router. The hybrid router keeps the keyword source list, since its kNN decisions carry no source.

### 2026-09-30: reading_compare tool in the model router

**Tried.** The model router gained a `reading_compare` tool for questions like "can I read X after Y" and "does X spoil Y".
**Decision.** Default since 2026-09-30; `--no-reading-compare` removes the tool, as before.
**Code.** `src/bin/ask/cli.rs`, `Args::no_reading_compare`

### 2026-09-30: Router prompt: the source field and the `reading_compare` tool

**Tried.** On 2026-09-30 a retrieval route was made to name its source (the kind of text that holds the answer), and the router got a `reading_compare` tool.
**Decision.** Shipped. `source: false` sends exactly the prompt and grammar of before; without `compare` the router has no `reading_compare` tool, as before 2026-09-30.
**Code.** trevor/src/router/model.rs, `RouterPrompt::source`, `RouterPrompt::compare`, `without_compare`, `source_variant`, `route`

### 2026-10-01: An `overview` router tool (`ask --overview`): kept opt-in

**Tried.** `ask --overview` adds an `overview` tool to the model router (one tool line and 2 examples not from any eval set, one grammar alternative). The answer gets the story overview as [1], the world as [2] and the storylines from [3], with a rule that an interpretive request (one word, a theme, a verdict) is answered as a labelled interpretation with reasons instead of declined. Measured on the one-word question and 12 whole-scope real-probe questions (r025, r031, r056, r064 to r067, r071, r073, r097, r113, r116).
**Result.** The one-word question: "This is an interpretation: the Arknights story can be summarized as "Survival." This is because the narrative consistently focuses on the struggle for survival against the biological hazards of Oripathy and Catastrophes [2, 16], the desperate fight for the rights and existence of the Infected against established nations [1, 2], and the personal efforts of various characters to find safety, justice, or a new home ... [5, 14, 16]" (5 passages cited, 0 invalid, 7,275 prompt tokens, 16.9 s). Of the 12, the router sent 2 to the tool: r073 ("Hey, I'm kinda new, What is the lore of this?") and r097 ("Is there anywhere I could get a brief rundown of the lore?"), both answered by the reading guide table before; both respond in both arms (r097 handles in both), now with 8/8 and 7/7 sentences supported where the table had no sentences to judge. The other 10 route as before (5 retrieval, 3 reading_guide, timeline_all, reading_event); 9 of the 13 answers are byte-identical, and r025 differs only in wording with identical passages and form (server nondeterminism; responds in both, 5/5 against 4/6 supported). Responds 11 of 12 in both arms.
**Decision.** Opt-in, because the router lines moved one route-check case: with the tool the model router passes 74/76 instead of 75/76 ("does reading Babel before chapter 13 spoil anything?" now retrieves instead of reading_compare; Sami fails as before), and one of the 9 probe-set questions changes its retrieval source (story to any). Without `--overview` the router prompt and grammar are the old ones line for line (the tool's 5 prompt lines and 1 grammar line are dropped, unit-tested), which is the configuration verified byte-identical on the 9 probe-set questions in the model-flags entry.
**Follow-up.** An example or rule that keeps reading questions about two events on reading_compare with the tool present, then a route check and a wider whole-scope set before making it the default.

### 2026-10-01: Overview router rule: two-event reading questions stay on reading_compare

**Tried.** One rule line in the `--overview` addition ("a question that names two events or episodes and asks about reading order or spoilers between them stays reading_compare ...; overview is only for the whole story with no event named, and such a question is not retrieval either"). It names overview, so `without_overview` drops it. The prompt without the tool equals the one before line for line (checked by replaying the line filter on the old and new files, also without reading_compare), and the unit tests count 6 and 7 dropped lines.
**Result.** With the tool the model route check is 75/76 again (Babel before chapter 13 is reading_compare; Sami fails as before), and the one-word question still answers "Survival" with the same 5 citations (26 s). Under the tool ian3 ("Who would commit marriage fraud or adultery more, Midnight or Matsukiri?") still moves its retrieval source from any to story, 8 of 9 kept.
**Decision.** Not made the default. A binary built with the tool on by default and `--no-overview` reproduced the old binary's route-only output on 76 of 76 route-check cases and 9 of 9 probe-set questions, then was put back to opt-in (`--overview`).

### 2026-10-03: `--overview` router tool stays opt-in: ian11 loses with it

**Tried.** Measured with `--no-cache-prompt` on the new default (`--lore v2`), so a route that moves is the prompt's doing, not the cache's. Route check, then the 14 probe-set questions route-only with the tool against the default's router outputs in run H, then answers for the questions whose routes moved.

**Result.**
- Route check: 75/76 with the tool, 75/76 without it, 75/76 under `--lore v1` (Sami fails in all three, as before).
- Probe set: 12 of 14 routes identical, and two retrieval sources move from story to any, ian3 and ian11. The 2026-10-01 note that ian3 moved from any to story was measured with the cache on, and runs A and B show that question's source flipping between any and story from an identical prompt, so that note did not measure the tool.
- ian3 ("Who would commit marriage fraud or adultery more, Midnight or Matsukiri?") answered both ways: the same 18 passages in another order, and the same answer by reading: both say no passage has either man commit marriage fraud or adultery, give Midnight's host past and Matsukiri's guarded politeness with citations, and decline a verdict; neither gives the hedged lean the reference asks for (Midnight's profession is the closer fit), so ian3 is no worse and no better.
- ian11 ("what operator is based off an animal with phantom as part of its name") is WORSE with the tool: the source any brings P4's Mint scene and the answer is "Mint, who is a new Caster Operator" (wrong; the reference is Phantom or Tragodia, or that the text does not state the animal), where the default without the tool declines.
- The tool's router lines also move unrelated routes on the other sets (route-only, cache on): real probe 14 of 120 (r064, r073 and r097 go to the overview tool, the rest between topic, reading_guide and retrieval sources), gold 2 of 146, held-out 2 of 40, trivia 0 of 109; those answers were not measured, since ian11 already decides it.

**Decision.** One loss on the probe set, so `--overview` is not the default.

### 2026-10-03: `--voice-rule`: an everyday-life router clause finds the voice lines but moves unrelated routes, stays opt-in

**Why.** The 6 displaced trivia misses are questions about one operator's everyday life ("what does broca do for a hobby?") that the router sent to "story", whose corpus P3b has no voice lines. Measured without a model first: of the 24 voice-sourced trivia questions, the source chunk ranks in P4's top 60 for 22 and P3b's for 0, and among P4's voice units alone it is first for 18 and in the top 3 for 21, so the typed-source path (best 3 voice units first) reaches it when the route says voice; archive-sourced questions rank the same in P4 as in P3b (37 checked), so a route to P4 costs them little.

**Tried.** `ask --voice-rule` (opt-in) widens the router's voice clause: "voice" also for a question about one operator's everyday life at Rhodes Island that their own lines would mention (likes, downtime, opinions of the base or of other operators, feelings about their past, how they act and talk day to day) when it names no event or story, plus one example not from an eval set ("what does Hoshiguma get up to on her days off?"); off, the prompt is the old one (unit-tested). Routes were prefiltered with the cache on (fast); the answers are `--no-cache-prompt`, both arms in one session each.

**Result.**
- Routes: trivia 39 of 109 change (29 story to voice, 5 story to operator_file, 3 operator_file to voice, 1 story to any, 1 any to operator_file; of the 24 voice-sourced questions 10 move to voice, and 20 archive, record, profile and scene questions move too), gold 9 of 146 (5 story to voice, all but 3 unanswerable), real probe 1 of 120, held-out 1 of 40, the 14-question probe set 4 of 14.
- Trivia, judged on the answers that differ: correct 24 to 28 of 37 (gained t052, t054, t056, t057, t059, t073, t088; lost t019, whose operator_file route moved to voice and lost Muelsyse's "simple and clean" archive line, t071, judge noise by reading since the new answer is the reference's "the day they came to this world", and t079, Silence's file displaced by voice lines), source chunk in passages 19 to 27 of 37, voice-sourced 2 to 7 correct of 10.
- Gold: 8 differ, answerable correct 1 to 1 of 2, unanswerable declined 6 to 6, faithful 3/4 to 6/6.
- Real probe: r047 differs, responds 0 to 0.
- Held-out: LOST h024 "[Nasti is] a banshee, right?": the new clause moved its route from the Banshee topic to the operator_attribute table, which answers "Nasti's race, from their operator file in the game data: Sarkaz" where the base answered yes with four citations.
- Probe set: ian11 moves from story to any and answers "Mint" (wrong) again; ian3, ian8 and ian9 change wording with the same substance.

**Decision.** Net on the rule's sets: trivia +4, held-out -1, so it stays opt-in. The rule works where it is meant to (voice questions 2 to 7 of 10) and its cost is the router prompt itself: any added line shifts a few unrelated routes (h024, ian11; the overview lines and the art lines each move 20 routes across the five sets).

**Follow-up.** The next shape to try is a voice fallback that leaves the router prompt alone: for a question routed to story that names one operator, add that operator's best voice unit after the passages, as the new dossier is added (not built; it would change every story question naming an operator, so it needs the full trivia, gold and real-probe runs).

### 2026-10-03: `ask --art`: an "art" source in the router, with ART_RULE

**Tried.** `ask --art` (opt-in) adds "art" to the router's source list (one clause after "item", one example, one grammar alternative; without `--art` the prompt and grammar are the old ones, unit-tested by `voice_rule_and_art_are_inert_when_off`), loads `p4-art` for an art route (falling back to P4 when it is not built), puts the best 3 art units first as for every typed source, labels them by their first line, and adds ART_RULE: the description was written by a vision model from the picture, is not game text and may be wrong; say so when using it; quote lettering only from the "Text visible in the art" part. Measured on the changed questions, `--no-cache-prompt`, both arms in one Gemma session, judged by Qwen in one session.
**Result.** Acceptance on `ian14` with `--art --no-cache-prompt`: routed `{"source": "art"}` (skin without it), answer "Pallas has their height written in their E2 art [1]. According to a model-written description of the art, the reading may be wrong, but it shows "Pallas 162cm" [1]." (cites `art_char_485_pallas_2` only). Without `--art` the same question is declined ("the provided passages do not contain any information regarding "E2 art""). The art lines move 20 routes across the five sets (route-only, cache on), and only `ian14` goes to "art"; every other change is a side effect of the longer prompt.
- Trivia correct 4 to 6 of 7 (gained t052 and t056, whose routes moved from story to operator_file and any; source chunk in passages 4 to 6).
- Gold: 1 of 2 answers differs, correct 1 to 1.
- Real probe responds 4 to 2 of 5 (lost r053 "Is there an archive for story arts?", which the topic route for Arts had answered and now retrieves from any, and r114 "What are the important takeaways in the joy of our reunion event?", which lost its recap table route to story retrieval).
- Held-out responds 3 to 2 of 4 (lost h019 "Do land Ægirs have a different metabolism compared to the sea ones?", topic Ægir moved to topic Seaborn).

**Decision.** Three losses, all to routes the new lines moved, so `--art` stays opt-in.
**Why.** The art itself works; what costs is, again, adding lines to the router prompt.
**Follow-up.** A route that does not touch the router (a keyword or form test for "art", "E2", "illustration", "outfit picture" that sets the art source after routing) would leave every other route alone; not built.

### 2026-10-03: `cross_ref` table for "every boss who joins our side"; the router line costs 3 losses, opt-in

**Tried.** `scripts/crossref.py` (new, no model) matches the 229 BOSS entries of `enemy_handbook_table` to the 407 obtainable operators by the boss's character name (quotes stripped, the title after the first comma and a leading "The" dropped) against the codename, the operator's real name (`real_names.jsonl`) or a name the identity links put with the operator. The `cross_ref {table}` tool (`src/tools.rs`) prints the list with what made each match and says that a boss who joins under another name is found only through a real name or link. The router gets one tool line, one example and one grammar alternative only with `ask --cross-ref` (unit-tested: off, the prompt and grammar are the old ones).
**Result.** 14 rows for 12 operators, `artifacts/crossref/bosses_playable.json`: Amiya ('Amiya' and 'Amiya', Furnace's Finale), Ascalon, Crownslayer, Degenbrecher, Kal'tsit, Mudrock, Rosmontis (Simulated Sentry), Tragodia ('Tragodia', and Lucian 'Blood Diamond' by her real name), Phantom (Lucian, by the identity link "You may also call me... Phantom."), Varkáris (Xanthos, the Avenger, by his real name), W and Wang. Missed by construction: Theresa (no table links Civilight Eterna to her) and Harold Craigavon (no table gives the operator Harold a surname); a first-word match would add Harold but also Ash Spear (Ash), Dusk Beauty (Dusk) and Bestial Ju (Ju), so it was not built.
`ian16` routes to the tool and answers with the 12 operators where the default declined ("I could not find it in the story text"). But the added lines moved other routes in every set (route-only: trivia 22, held-out 4, real 4, gold 3, probe set 2), and of the answers that changed:
- Trivia correct 13 to 16 of 22 (gained t047, t052, t053, t085; lost t067 "what does ethan like to do for fun when he's at the base?").
- Held-out responds 3 to 2 of 4 (lost h024 "[Nasti is] a banshee, right?", now routed to the operator_attribute table, "Nasti's race ... Sarkaz", the same loss the voice rule caused on 2026-10-03).
- Real probe 3 to 2 of 3 (lost r053 "Is there an archive for story arts?").
- Gold correct 3 to 3 (unanswerable u004 declined 1 to 1).
- `ian11` moves from story to any and answers "Mint" (wrong; the default declines), the same as under `--overview` and `--voice-rule`.

**Decision.** Three losses on the judged sets and one on the probe set, so `--cross-ref` stays opt-in. As before, what costs is the router prompt itself.

### 2026-10-03: Aroma's grandmother is answered; Mutsumi's helpers are a routing miss

**Result.** `ian21` ("what was Aroma's grandmother's job?") is right under the default: "a famous cleaner who was known for her ability to make even the most terrible scenes look spotless [2] ... the matriarch of a family that ran a laundromat as well as a workshop that produced fragrances and detergents" (`archive_char_446_aroma#0004`; her voice line, `voice_char_446_aroma#0001`, says the family "served as 'cleaners' charged with clearing out the aftermath" of Siracusa's murders). `ian22` ("who helped Mutsumi care for the plants?") is wrong: "Other operators helped Mutsumi care for the plants ... [2]", citing her operator file, which says only that other operators are impressed by her.
**Why.** The answer is in her module story New Friends (`module_uniequip_002_mortis#0000` and `#0001`: the robots Lancet-2, THRM-EX, Friston-3 and Castle-3 help her in the Convalescent Garden), which is a P4 unit; the router sent the question to story and operator files (P3b passages: her file, act44side, act45side, Muelsyse's story), so no module text was among the passages. It is the voice-line miss of 2026-10-01 in another kind: the answer lives in a typed source the question does not name.
**Follow-up.** The typed fallback (item 10) does not reach it, since the answer does not decline; a question naming one operator whose answer cites only that operator's file is a candidate for the same fallback. Not built.

### 2026-10-03: Typed fallback for one-operator declines, without touching the router: the default

**Tried.** When the default answer declines, the question names exactly one operator (codenames of the attribute table, whole words, the longest when one name holds another) and it was not routed to a typed source, `ask` tries again from P4 plus the art units: the best-ranked unit of that operator among voice lines, module stories and art decides the kind, the best 3 units of that kind go first (with the lore passages of a module, as for a routed module question), and the new answer is kept only when it does not decline and cites one of that operator's own units; otherwise the decline stands. The router prompt is untouched, so no other route moves. Candidates were the one-operator questions whose answer declined in the last full runs (lore2 and trivia-form2).
**Result.** First version (no own-unit check): trivia correct 0 to 3 of 3 changed (gained t054 "how does coldshot prefer to handle conflicts?", t063 "what does skyfire do to practice their fire control?", t068 "what did bibeak used to do for their family during the new year?", each from that operator's voice lines; source chunk in passages 0 to 3), gold correct 0 to 1 of 3 (gained g0084, Delphine's head injury, answered from the story passages that came with her module), real probe 0 changed of 6 candidates, held-out none. But `ian11` ("what operator is based off an animal with phantom as part of its name") named Phantom, got Melantha's voice lines first and answered "Melantha" (wrong) where the default declines. The own-unit check (second version) keeps t054, t063, t068, g0005 and g0121 (all cite the operator's own unit) and returns `ian11` and g0084 to the default decline (`ian11` cited Melantha's voice lines and Phantom's file; g0084 cited only story passages): trivia +3, gold 0 (the g0084 gain is given back), real 0, held-out 0, the 25 probe questions unchanged.
**Decision.** 0 losses, so the second version is the default; `--no-typed-fallback` turns it off.
**Caveat.** A question that declines now but did not decline in the last full runs is not in this count.

### 2026-10-03: Cross-reference table triggered without a router line (leftover d)

**Tried.** `Tools::cross_ref_question` sends a question with "boss" or "bosses" and a joining word ("playable", "join", "operator", "recruit", "obtain", "ally", "our side") to the `cross_ref` tool before the router (`--no-cross-ref-trigger`). The router prompt is unchanged, so no other route moves.

**Result.** It fires on 1 of 447 questions, `ian16`, which now answers with the 12 operators where the old default declined. The 34 routes the `--cross-ref` router line moved stay where they were.

**Decision.** The default; `--no-cross-ref-trigger` turns it off.

### 2026-10-03: IS-ending questions resolve to the ending's scene and endbooks (item 2)

**Tried.** `Tools::is_ending_question` finds the run ("IS4", "IS 4", "Integrated Strategies 4" with the game's numbering, IS n = rogue_{n-1}, or a run's full name) and the ending (number, "second", or its name) from `is_endings.json`, numbered by the id suffix, which equals Priority + 1 in `roguelike_topic_table` (`ro3_ending_c`, The Fleetest of Light, has no number and no script and is left out). A question that names both and does not ask about canon goes, before the router, to a route whose passages are the run's ending order ("Integrated Strategies 4 (Expeditioner's Jǫklumarkar) has 4 numbered endings, in the game's order: Ending 1 'To Talk with the Mountains', Ending 2 'Winterfall', ...") and every chunk of `is_rogue_3_ending_2_script` and `is_endbook_rogue_3_2_1` to `_2_4` from P4, then 3,000 tokens of plain retrieval (`--no-is-ending`).

**Result.** It fires on 1 of 447 (`ian27`), which goes from "Trevor does not have that text" (the router sent it to the recap tool, which failed) to "Santalla oversees a ceremony of parting for the warriors of the north ... Endbook Story 1: ... Eikthyrnir ... Endbook Story 2: Eikthyrnir discovers that the shaman of Colddeath ..." with each endbook story.

**Decision.** The default; `--no-is-ending` turns it off.

**Caveat.** The endbook item names (Warriors Never Welcome Death, ...) are not in Trevor's tables, so the answer numbers them.

### 2026-10-03: An appearance index: "What chapters does Elysium appear in?" as a table answer (item 7)

**Tried.** `scripts/appearances.py` (no model, 80 s) lists, for 5,870 speakers over 449 story groups, the groups where each speaks (the chunk's speaker labels, with identity links of kind real name, codename, former name, alias or stage name, never titles) and apart the groups where they are only named (whole word, case-sensitive, outside speaker labels), plus the other operators' files, voice lines, modules and outfits that name them; groups are named and ordered by the reading guide. `Tools::appearance_question` sends a question with an appearance word ("appear", "show up", "featured") and a story word ("chapter", "event", "story", "episode"), without "first" (the `first_appearance` tool's question), to the `appearances` tool before the router (`--no-appearances`).

**Result.** It fires on 1 of 447 (`ian32`). Against the reference list: he speaks in Episodes 7, 8 and 16, Rewinding Breeze (8 scenes), Preluding Lights (1), Stultifera Navis (34), Ideal City (33), Exodus from the Pale Sea (47) and his record Long Journey (10), all listed. The index adds Heart of Surging Flame (2 scenes: `act3d0_level_act11d7_st01` "Seaside Serendipity", where he phones Thorns, checked). Thorns' paradox simulation has no script in the game data (HandbookStageData holds only a description that does not name him), so it is missing. Mentioned: Path of Life (1 scene, Kal'tsit's line) and Blaze's record "Will You?", not Weedy's second record, which never writes his name. Before: "Elysium appears in the following chapters: Abnormal Spectrum, ..., Long Journey, Rewinding Breeze, Stultifera Navis, Exodus from the Pale Sea" (5 of the 9 that have text).

**Decision.** The default; `--no-appearances` turns it off.

### 2026-10-03: The appearances tool and the cross-reference test without the router

**Tried.** Item 7 of the request list asked where a character appears ("which events is Mudrock in"). The `appearances` tool gives the story groups where they speak and, listed apart, the other groups they appear in. The detector needs an appearance word and a story word, not "first" (that is the `first_appearance` tool's). For leftover d, `bosses_playable` is picked for a question with "boss" or "bosses" and a word for joining or playing ("playable", "join", "operator", "recruit", "obtain", "ally", "side").
**Result.** A router line for the cross-reference table moved 34 unrelated routes.
**Decision.** The cross-reference test reads the question only, without the router.
**Code.** trevor/src/tools/appearances.rs, `Tools::appearances`, `Tools::appearance_question`, `Tools::cross_ref_question`

### 2026-10-03: The IS-ending route

**Tried.** Item 2 of the request list: a question naming an Integrated Strategies run (an "IS" number) and one numbered ending goes to an IS-ending route.
**Code.** trevor/src/bin/ask/routing.rs, `is_ending_route`; trevor/src/tools/canon.rs, `Tools::is_ending`

### 2026-10-04: The `design_basis` tool and the own-unit rule for the typed fallback (`ian11`)

**Tried.** `ian11`, "what operator is based off an animal with phantom as part of its name", got a design-inspiration tool, `design_basis`, with a detector over the normalized question and its content words; it returns `None` when the question is not one. The typed fallback named Phantom, got Melantha's voice lines first and answered "Melantha".
**Decision.** The typed fallback is kept only when it cites a unit of the operator itself.
**Code.** trevor/src/tools/design.rs, `design_question`, `Tools::design_basis`; trevor/src/bin/ask/answer.rs, `answer_one_inner` (typed fallback own unit)

### 2026-10-05: `--named-topic` gets the broad-topic gate: `ian29` reaches the Sami entry, but `ian17` loses, so it stays opt-in

**Tried.** Routing `ian29` by data, not a router line: `--named-topic` (night 7, opt-in) now has the same broad-topic gate, so of the 56 set questions naming one nation, place, race or organization topic it fires on 6 (r012, r086, h006, `ian17`, `ian28`, `ian29`); `--named-topic-all` is night 7's reach. With the deep default, a named topic that has an entry serves it. Measured with `--named-topic` on the final binary.

**Result.** r012, r086, h006, `ian17` and `ian29` change (`ian28` is topic-routed already).
- Set judges: 0 losses (real responds 2 to 2, faithfulness 5/5 to 8/9; held-out 1 to 1, 1/2 to 2/2).
- Pairwise: r086 new ("No, Gaul is not back", against a hedge); `ian29` new (the Sami entry: "guarding the northern front", "the endless icefield", the Sami as "fangs" that "sever the tracks of demons" which contaminate the land; the base's error "to keep out ... other Sami" is gone); r012 and h006 ties; `ian17` BASE: the Rhodes Island entry turns "between 1086 and 1090, based on T1 and T2" into "formerly known as Babel ... founded or led by Theresa ... Estimate: 1086" (and the summary-only arm into "1094 or earlier").

**Decision.** One pairwise loss, so `--named-topic` stays opt-in and `ian29` keeps the story route under the default.

**Follow-up.** `ian29` against the verifier is partial both ways; the entry still does not name the Collapsals, the Myrkwood or the Aethergate.

### 2026-10-16: Operator list check

**Tried.** An operator list check for questions that ask for operators (`ian35`; `Tools::asks_operators`).
**Decision.** On by default; `--no-operator-list-check` is the kill switch.
**Code.** `src/bin/ask/cli.rs`, `Args::no_operator_list_check`

### 2026-10-18: Summary requests reach the recap tool under the keyword router

**Tried.** "Give me a summary of The Masses Travels" (mid profile, keyword router) went to retrieval although the event detector (`Tools::event_in_question`) finds act42side in it. Two causes in `recap_route`: `named_group` needs the event's name as written ("the masses' travels" is not in "the masses travels"), and `leftover` folds the question's apostrophes to spaces but not the name's, so "What happens in The Masses' Travels?" counted "masses" as a leftover word. With `ToolOptions::summary_route` the rule falls back to the event detector, takes a bare "summary" ("Guide Ahead summary"), and folds the name's words as the question's are.
**Result.** 7 summary questions routed with no model: 4 moved from retrieval to the recap (the two Masses' Travels wordings above, "Can you summarise The Masses Travels for me?", "Guide Ahead summary"), 3 were recaps already. Over the 21 no-model outputs (`--detect-only` and `--route-only --router keywords` on 10 question sets, and `route-check.py`), the new binary is byte-identical to HEAD's on 21 of 21: no eval question changes route. `--no-summary-route` gives HEAD's routes on the 7 summary questions (7 of 7).
**Decision.** On by default; `--no-summary-route` is the kill switch.
**Code.** `src/bin/ask/routing/rules.rs`, `recap_route`; `src/tools/mod.rs`, `ToolOptions::summary_route`

### 2026-10-18: "Tell me about <event>" and "What is <event>?" reach the recap under the keyword router

**Why.** From the preview (mid, E2B, spoilers on): "Give me a summary of the Masses Travels" and "Tell me about the Masses Travels" were declined. The preview's worker was started at 07:59 and `target/release/ask` was rebuilt at 08:32, so it ran without the summary route of the same morning; through the current binary the summary request gets the recap and "Tell me about" went to retrieval.
**Tried.** With `ToolOptions::event_about` the recap rule also takes "tell me about", "what is", "what's" and "what was" when nothing but the event is asked (the leftover test, "tell" and "about" counted as used), and not when the event's name is also an operator, a character of the appearance index or a served topic or alias ("Tell me about Babel" is the organization).
**Result.** On a probe of every event name (85 names, "Tell me about X" and "What is X?", 170 questions, no model): HEAD routed 166 to retrieval, the new rule routes 162 to the recap, 6 stay with retrieval (Ato, whose name is under the detector's length; "Tell me about Babel" and "Tell me about Pinus Sylvestris", topics; both Stultifera Navis wordings, not traced), and "What is Babel?" and "What is Pinus Sylvestris?" keep their operator listing. Over `--route-only` and `--detect-only` on the probe set, real probe, held-out, gold v2, trivia, `routes.jsonl` and the 883 Reddit and 345 Discord questions, the new binary is byte-identical to HEAD: no eval question changes route. E2B mid: "Tell me about the Masses Travels" goes from a retrieval answer to the recap. `--no-event-about` gives HEAD's route.
**Decision.** On by default; `--no-event-about` is the kill switch. Restarting the preview (`bun run dev:trevor`) is needed for it and the summary route to reach the worker.
**Code.** `src/bin/ask/routing/rules.rs`, `recap_route`, `event_name_is_entity` (1 test); `src/tools/mod.rs`, `ToolOptions::event_about`

### 2026-10-18: First and last appearances from the appearance index (Talulah)

**Why.** "When was Talulah last mentioned (chronologically)?" (E2B, mid, spoilers on) answered "in passage [4] in the story Roaring Flare, Behold, the Twilight Comes" (another run: R8-5). Retrieval returns a few similar chunks and cannot know which is the last. The truth from the game text: she speaks in Episodes 13, 14 and 16, See You Soon (act18mini) and First of A Thousand Autumns (act49side, 1 scene), and in the refreshed corpus of 2026-10-18 in Episode 17 (22 P0 chunks name her, from `main_17_level_main_17-11_end#0003`), which the appearance index of 2026-10-04 does not hold yet.
**Tried.** `appear.rs` (`--no-appearance-order`), before every router: a last word ("last", "latest", "most recent", "final", "newest") or a first word ("first", "earliest", only with a mention word or "chronologically", since "first appear" is the `first_appearance` tool's) within 4 words of an appearance word, one character of the appearance index as written (or the whole question lowercase), and no event named. The table answer lists the latest (or first) story group where the character speaks or is named, in EN release order (the reading guide's position), with the 3 before it, and with "chronologically", "in world" or "timeline" leads with Trevor's in-world estimate (the chronology's year, ties in release order, and it says so). Under a spoiler horizon the answer is recomputed within it (`spoiler::table_within`): groups wholly after it drop out, and a group the horizon cuts counts only when a visible chunk names the character.
**Result (no model).** The Talulah question now answers "the latest story group where Talulah appears is First of A Thousand Autumns, placed at about 1100" (in-world) and the same in release order, from the 2026-10-04 index; after `scripts/appearances.py` reruns on the refreshed corpus it will name Episode 17. The first cut fired on gold `g0118` ("Biegler mention[ed] ... the area's recent restlessness"), so "recent" was dropped and the nearness rule added: then 0 of the 1,754 questions of the eight sets change route, and 9 of 10 hand-written variants route ("When was the last time we saw Talulah?", "where was frostnova last seen", "When was Amiya first mentioned chronologically?").
**Decision.** On by default; `--no-appearance-order` is the kill switch (verified with every switch off: the 341 answer records and prompts of the four sets are byte-identical to HEAD's). Needs the appearance index rebuilt after the corpus update.
**Code.** `src/bin/ask/appear.rs` (2 tests), `src/bin/ask/routing/mod.rs`, `src/bin/ask/spoiler.rs`, `table_within`

## Answer rules

Answer prompts and the question form, decline, partial, retry and verdict rules, relation and opinion rules, citations and answer checks.

### 2026-09-28: A file found by another name says so

**Result.** With "Margaret vs Degenbrecher" and Nearl the Radiant Knight's file among the passages, the answer still said it found nothing about "a character named Margaret".
**Decision.** A passage from a file found through another name of its character is labelled with that name.
**Code.** trevor/src/bin/ask/answer.rs, `answer_from` (passage label)

### 2026-09-30: Question-form call: fact, yes/no or opinion

**Tried.** Five probe questions with reference answers from the data (`eval/ian-questions.jsonl`). What the data says: no text names a best cook (Matterhorn's file is the strongest praise, "his mastery of the art of cooking is awe-inspiring", with Hung, Lee, Shu, Yu and Gummy also praised); no chunk has Matsukiri (Tooru Moriuchi) or Midnight commit marriage fraud or adultery (Midnight is a former host, "the Devil of the Eastern Night"); none of the 132 P4 chunks naming Andoain mentions a marriage; Harold has a wife and one daughter (lines in The Rides to Lake Silberneherze, his outfit description and his voice lines) and nothing says he married twice. The fix built: one more grammar-bound Gemma call (`prompts/form.system.txt`, `prompts/form.gbnf`) that sorts a retrieval question into fact, yes_no or opinion and, for the last two, writes an evidence query. A yes/no question interleaves the evidence query's hits with its own and gets a rule to answer each part and, for a part the passages do not show, say they do not mention it and give what they do say. A judgment question gathers up to 6 candidates from operator files and dossiers (at most 2 per character, no neighbours, characters the question names first from their own files) and gets a rule to give each candidate's evidence and a hedged conclusion. Measured on the five questions, and on the real probe (the 57 non-fact questions plus 6 fact controls, Gemma answers, Qwen judges).
**Why.** On the default path all five routed to retrieval and all five were declined: the best-cook question retrieved the Dungeon Meshi collaboration and Warmy, with none of the six praised cooks' files in the top 100 on P3b or P4; the comparison was declined although Matsukiri's files were in the passages, because the OPINION_RULE keywords do not match "who is the best" or "who would ... more"; the Harold question was declined although the passages held "my daughter was the same at her age" and "with the help of my wife", and his family lines rank 29th to 44th for the raw question. Name resolution was not a cause: Harold is an operator (char_4114_harold) and Andoain has a dossier.
**Result.** The five answers with the form call: best cook, six candidates with quotes including Matterhorn's "awe-inspiring" mastery, "the story does not rank them"; Matsukiri, "the passages do not mention" it, then his file (oden-stall owner, information dealer); the comparison, both men's files cited, no basis to judge; Andoain, not mentioned; Harold, "he has a wife and a daughter", with one misread (the daughter compared to the burdenbeast Lily, where the line compares her to Leto). By the references: 4 correct and 1 partial, against 0 correct, 2 partial (Matsukiri and Andoain declines that happen to fit) and 3 wrong before. The form call sorts the 120 real-probe questions into 63 fact, 38 opinion and 19 yes_no, and all 146 gold v2 questions into fact, where the answer is unchanged by construction (a fact form adds no query and no rule). Off (`--question-form` absent) gives the same answer, passages and citations as the binary before it on all five questions. Real probe: the 6 fact controls are byte-identical (answer, passages, citations). On the 63, responds 42 to 50 (0.667 to 0.794), 13 gained (r021, r028, r033, r042, r044, r051, r062, r074, r082, r090, r094, r098, r106) and 5 lost (r030, r034, r046, r061, r080); handles 26/27 to 24/27 (r030, r056 lost); faithfulness 140/155 (0.903) to 247/276 (0.895), longer answers with more sentences. The clearest real loss is r034 "Margaret vs Degenbrecher: Who is Stronger?": off compared the two from the passages, on answered "I could not find any information regarding a character named Margaret" although her dossier ("Character profile: Margaret (also known as Nightzmora, Radiant Knight)") exists, so the loss is in the answer step, not in name resolution. r061 (Mountain running for president) went from a decline to an answer the judge reads as not responsive; r030, r046 and r080 changed from one hedged answer to another. Route check with the model router 75/76 (Sami, as before), keywords 61/61.
**Decision.** Kept opt-in (`--question-form`): with 5 losses it does not meet the no-loss bar for a default.
**Caveat.** The on arm took 56.6 s per question against about 26 s, with the machine under other load, so latency was not measured. The Harold misread is Gemma resolving "she" in "she sure is a lively little kid" (said of Leto riding Lily) to the burdenbeast. It is a pronoun error in the answer step, not a name-specific one, and no cheap general fix was found, so it is left as is.

### 2026-09-30: Question-form call: the five real-probe losses re-run after the Margaret fix

**Tried.** The five real-probe losses of the question-form call (`--question-form`) were re-answered and judged with the named-character fix above.
**Result.** r034 and r061 respond again (r061 7/7 supported). r030 and r046 give byte-identical answers and still lose. r080 still loses (0/1). On the 63 real questions that makes responds 42 to 52 (13 gained, 3 lost) and handles 26 to 23 of 27 (lost r030, r056, r034). The 5 questions then in `eval/ian-questions.jsonl` are unchanged in substance (Harold's Lily misread included).
**Decision.** With three losses the call stays opt-in.
**Caveat.** The off path matches the off arm byte for byte on 9 of the 11 re-run real-probe questions. r001 and r080 have identical passages and citations, and their wording diverges late in the answer across server sessions (r080 also diverged between the two on runs with identical passages and form). That is llama-server nondeterminism between sessions, not the code.

### 2026-09-30: Question-form call, opinion-form and yes/no rules

**Tried.** One more grammar-bound Gemma call sorts a retrieval question by form. A judgment question gets `OPINION_FORM_RULE`; a yes-or-no question gets `YES_NO_RULE`. Before this, `OPINION_RULE` (opinion, prediction and ranking questions, 16% of real questions, which no fact answers) was added only on a keyword test.
**Result.** "Who is the best cook?" and "Who would commit marriage fraud or adultery more, Midnight or Matsukiri?" were declined outright, since no passage states a verdict and the keyword test for `OPINION_RULE` did not fire on either. "Has Harold married twice? Does he have children?" was declined whole although the passages held his wife and his daughter.
**Decision.** The form call is the default since 2026-09-30; `--no-question-form` turns it off (as before 2026-09-30).
**Code.** `src/bin/ask/cli.rs`, `Args::no_question_form`; `src/bin/ask/prompts.rs`, `OPINION_RULE`, `OPINION_FORM_RULE`, `YES_NO_RULE`

### 2026-10-01: Question form made the default

**Decision.** The question form is the default from 2026-10-01 (+13/-3 on 63 real questions). `--no-question-form` reproduces the previous default exactly on the 9 probe-set questions and the 6 fact controls (answers and passages identical, 15 of 15).

### 2026-10-01: One-character questions leave the opinion path: form second look and a gated OPINION_RULE

**Tried.** Two fixes, both default, each with a kill switch.
- An opinion form gets a second look (`prompts/form.system.v2.txt`: opinion only for judgments across several people or things; a question about one character's personality, feelings, habits or likes is fact; whether one character has a trait is yes_no). It runs only when the first call says opinion, and an opinion it confirms keeps the first call's evidence query, so every other question retrieves exactly as before (`--form-v1` turns it off). Standalone, the v2 prompt also moved 6 real-probe fact questions to yes_no and rewrote the evidence of 36 confirmed opinions, which is why it is a second look and not a replacement.
- The keyword OPINION_RULE no longer fires when the form call ran and chose fact or yes_no (`--keyword-opinion` restores it). Affected: trivia t002, t031, t064, the probe set's ian8, gold u015.

**Result.** Flips: trivia 28 of 29 opinions (25 to fact, 3 to yes_no), real probe 1 of 38 (r063 "Is Amiya yandere?" to yes_no), probe set 2 of 4 (ian7 Flamebringer to fact, ian9 Ulpianus to yes_no), gold v2 0 (all 146 fact).

Probe set (verbatim, short): ian9 before "Ulpianus is characterized as a relatively untalkative individual ... a 'quiet warrior' who is often overlooked [Nightblade's file]", after "Ulpianus is generally depicted as untalkative or reserved ... Amaia notes that he is 'less and less tight-lipped' but still keeps quiet frequently" (Amaia's line, the expected evidence, now in the passages; no other operator's file). ian8 before "The story does not settle on a single favorite food, but ...", after "Insider is extremely fond of sweets [2] ... frosted donuts". ian7 before "The provided passages do not contain any information regarding whether Flamebringer is romantically attracted to anyone. While the text mentions a 'Shy Operator' ...", after "I could not find the answer ... The passages do not mention any romantic interests or attractions for Flamebringer" (right, still without the Perfumer friendship). The other 6 are byte-identical. Route check: model 75/76 (Sami, as before), keywords 61/61.

Trivia (all 109; 77 unchanged questions reuse the previous run's answers, verified byte-identical on 78 of 109): 74/109 to 74/109, 3 gained (t034 Passenger, t040 GreyThroat, t064 Blaze) and 3 lost (t086 Ebenholz, t088 Nasti, t089 Lemuen), against the off arm answered in the same session; faithful 156/170 to 82/86 on the 32 (0.918 to 0.953), 226/236 overall. t009 and t096 flip to yes_no and still miss: YES_NO_RULE makes them open "The passages do not mention whether ..." although the passages settle it. Real probe: r063 is the only changed answer, responds and handles in both arms (faithful 9/9 to 3/3). Gold v2: 0 of 146 changed (u015 byte-identical).
**Why.** The three trivia losses share one cause: the opinion path's named-character step put the character's own dossier (`profile_ebenholz`, `profile_nasti`, `profile_lemuen`, each the source chunk) into the passages, and the fact path does not retrieve it.
**Decision.** With 0 losses on real probe and gold, both changes stay the default; the trivia trade is +3/-3. Kill switches: with both flags the new binary answers the 9 probe-set questions byte-identically to the old binary in the same server session (answer, passages, citations and form, 9 of 9), and the old binary reproduced the previous run's `ian-night2` answers 9 of 9. On the 32 trivia questions the change touches, the off arm matched the previous answers on 29 and diverged late in the wording on 3 with identical passages (t075, t076, t080, server nondeterminism between sessions); its judgments equal the previous ones on all 32.
**Caveat.** Thermal: no warning level recorded; memory free 66% after the jobs.
**Follow-up.** In order: keep the named character's own files (the named-candidate step without other operators' candidates) for a one-character question, which should recover t086, t088 and t089 (a hypothesis, not measured); YES_NO_RULE for a trait the passages settle (t009, t096); the corpus choice for voice-line questions (6 displaced misses).

### 2026-10-01: Answer-rule flags from a model call instead of English keyword tests (`--model-flags`): kept opt-in

**Why.** Item 1 of the work list. Four keyword tests still added answer rules on the default path: `is_time_question` (TIME_RULE and the timeline notes), `is_synthesis_question` (SYNTH_RULE), `is_canon_question` (CANON_RULE) and `is_source_question` (SOURCE_RULE when no lore units lead). `is_opinion_question` already fires only with the form call off or `--keyword-opinion`, so there is nothing to replace there.
**Tried.** `ask --model-flags` replaces the four with one more grammar-bound Gemma call after the form call (`prompts/flags.system.txt`, `prompts/flags.gbnf`: `asks_time`, `asks_canon`, `asks_synthesis`, `asks_source_text`, a general instruction and 8 examples, none from an eval set). It is a separate call rather than more fields in the form call, so the form, its evidence query and the passages stay exactly as they were (passages identical in 68 of 68 changed questions) and only the answer rules move. The keyword tests decide whenever the form call is off, and `is_time_question` still gates the death tables in routing, which is not an answer rule. Judged by Qwen on the changed questions only, both arms answered in one Gemma session.
**Result.** The first prompt flagged synthesis on 37 real-probe questions the keyword test does not ("what do we know about X"), 47 of 120 disagreements in all; a narrower synthesis definition plus three all-false examples brought it to 37 of 120. Agreement with the keyword tests (second prompt): real probe 83 of 120, gold v2 128 of 146, trivia 96 of 109, probe set 6 of 9. Per flag (both, model only, keywords only):

| set | time | canon | synthesis | source |
|---|---|---|---|---|
| real probe | 1/5/0 | 5/3/0 | 1/26/0 | 4/3/1 |
| gold | 2/5/3 | 1/1/1 | 0/3/0 | 1/3/2 |
| trivia | 0/1/1 | | 0/2/1 | 0/7/1 |
| probe set | | | 0/2/0 | 0/1/0 |

The model's synthesis remains much wider than the keyword list, and its source flag fires on "what does X say" and "what kind of magazines does X like" questions.

Answers: real probe responds 27 to 27 of 37 (gained r041, r055, r084; lost r027 "how old is Pallas?" to time, r045 "has anyone translated mon3trs operator file?" to canon and source, r108 "Are Petrams related to Aegirs" to synthesis), handles 11 to 11 of 15, faithful 98/115 to 126/145 (0.852 to 0.869). Gold correct 10 to 11 of the answerable changed (gained g0129, lost none), but unanswerable u006 "Who designed the Rhodes Island company logo?" no longer declines (canon flagged, the answer talks about canon rulings); faithful 24/24 to 28/28. Trivia correct 7 to 10 of 13 (gained t008, t051, t071, lost none), faithful 23/25 to 27/28. Probe set: 3 answers change, and ian6 now opens "Trevor does not have that text" (source flagged on "What operator declared that they want to pee in public?"), worse than the plain decline.
**Decision.** With 3 losses on real probe and 1 on gold it is not the default: `--model-flags` is opt-in and without it every rule is decided as before. Kill switch: the new binary with the keyword tests and no overview tool answers the 9 probe-set questions identically to the old binary in the same server session (every field but `ms`, 9 of 9); the flags were then renamed to opt-in, which changes only the default. Route check: model 75/76 (Sami, as before), keywords 61/61.
**Caveat.** Cost: the flags call is one more Gemma request per retrieval question. With `-cram 512` the server keeps one prompt cache entry, so the form and flags prompts evict each other (`--form-only` ran at about 4.9 s per question with both calls; the flags call alone was not timed).
**Follow-up.** The synthesis flag needs a stricter definition or a rule that tolerates it (SYNTH_RULE on a one-subject question is what lost r108), and the source flag must not fire for operator files and voice lines that P3b already holds (ian6, t097 lost the same way, though t097's trivia judgment did not move).

### 2026-10-01: Model flags: tighter synthesis, source and canon definitions

**Tried.** `prompts/flags.system.txt` now limits synthesis to named comparisons, rankings and many-event overviews (a yes/no question whether two things are related is false), limits source to a named kind of text (an operator's file, and a question about what a character said or likes without naming a text, are false), and says asking whether a text was translated is not canon; 2 all-false examples, not from an eval set. Measured as agreement with the keyword tests on the 120 real-probe questions (`--form-only --model-flags`, 10 min 8 s).
**Result.** 85 of 120 (83 before). Per flag, both/model only/keywords only (before in parentheses): time 1/6/0 (1/5/0), canon 4/2/1 (5/3/0), synthesis 1/22/0 (1/26/0), source 2/2/3 (4/3/1). r108 no longer gets synthesis and r045 no longer gets canon or source; r027 still gets time.
**Decision.** No answers were measured; it stays opt-in, and without `--model-flags` the prompt is never sent.

### 2026-10-01: Second look at the opinion form; the form call's choice decides OPINION_RULE

**Tried.** A second look at an opinion form (`FORM_SYSTEM_V2`), so a question about one character's trait or feelings is not left as an opinion. `OPINION_RULE` is added only when the form call chose opinion, not on the keyword test ("favorite", "strongest", ...) when the form call ran and chose fact or yes_no.
**Result.** "What is Insider's favorite food?" (form fact) opened "The story does not settle on a single favorite food", and trivia `t064` "what's blaze's favorite yanese saying?" hedged its right answer away the same way.
**Decision.** Default since 2026-10-01. `--form-v1` keeps the first call's opinion (no second look); `--keyword-opinion` restores the keyword test.
**Code.** `src/bin/ask/cli.rs`, `Args::form_v1`, `Args::keyword_opinion`

### 2026-10-02: Game-data inference passage for body, race-trait, ability and age questions

**Why.** Decision on h002: it is a real loss with both answers wrong. The Nearls are Kuranta (horse-like: ears, tails; Nearl, Nearl the Radiant Knight, Blemishine and Młynar are race Kuranta in operator_attributes.jsonl), and wings and halos are the Sankta's; the reference is now h002's `note` in `eval/lore-heldout.jsonl`.

**Tried.** Built for `--lore v2` (`--no-game-data` turns it off): a question naming characters (a name as written from 4 letters, folded from 6, or a family plural, "Nearls" for every operator whose first word is Nearl) and asking about a body feature, race trait, ability or age (a word list, a trade: no model call) gets a game-data passage after the retrieved ones: each character's race, birthplace, nation, faction and height from the operator files, the first 120 words of each race's topic summary, and for each body word asked, the races whose summaries mention it with that sentence; and an answer rule allowing a labelled inference or estimate instead of a decline.

**Result.** h002 now answers "The game-data passage identifies the Nearls as Kuranta; ... the game-data passage specifically mentions wings as features of the Sankta and Chimera races. Therefore, it is most likely that the Nearls do not have wings" (responds, 4/4 supported). ian13 ("how old is angelina") now cites "much like other high schoolers" and "a high school girl" but still gives no explicit estimate.

**Decision.** Shipped inside `--lore v2`; `--no-game-data` is the kill switch.

**Follow-up.** Give ian13 an explicit estimate.

### 2026-10-02: Game-data passage and inference rule for body, race, ability and age questions

**Tried.** With `--lore v2`, a question about a named character's body, race features, abilities or age gets a game-data passage and the inference rule `INFER_RULE`.
**Result.** "Do the Nearls have wings?" was declined though the operator files list every Nearl as Kuranta, and "how old is angelina" was declined though the text implies it.
**Decision.** Default with lore v2 since 2026-10-02; `--no-game-data` is the kill switch.
**Code.** `src/bin/ask/cli.rs`, `Args::no_game_data`; `src/bin/ask/prompts.rs`, `INFER_RULE`

### 2026-10-03: `--model-flags` re-measured with the tightened prompt: still 3 losses, stays opt-in

**Tried.** Changed questions only, both arms answered with `--no-cache-prompt` in one Gemma session and judged by Qwen in one session. Prefilter: the flags call against the keyword tests on every question (`--form-only --model-flags`), kept where they disagree and the question goes to retrieval.

**Result.**
- Agreement (per flag both / model only / keywords only): real probe 85 of 120 agree (time 1/6/0, canon 4/2/1, synthesis 1/22/0, source 2/2/3, the same as the 10-01 check), gold 124 of 146 (time 2/5/3, canon 0/1/2, synthesis 0/7/0, source 1/2/2), held-out 27 of 40 (synthesis 1/12/1). 28, 22 and 13 retrieval questions disagree, and 26, 15 and 13 of their answers differ (2 and 7 are byte-identical: the flag changed no rule they use).
- Real probe: responds 21 to 20 of 26. Gained r041 (time; both decline in substance, judged as responding only with the flag). Lost r045 "has anyone translated mon3trs operator file?" (the keyword canon rule answered "The story text does not rule on whether the operator file has been translated"; without it the decline is plainer and judged as not responding, judge noise by reading, a loss as measured) and r078 "Does anyone know the knife in the W vs. Ines CG?" (source flagged, so SOURCE_RULE opens "Trevor does not have that text" and drops the butterfly-knife passage the base answer cited, a real loss). Handles 7 to 8 of 10; faithful 99/118 (0.839) to 125/144 (0.868).
- Held-out: responds 11 to 11 of 13, faithful 53/57 to 73/78.
- Gold: answerable correct 8 to 9 of 13 (gained g0128, synthesis), unanswerable declined 2 to 1 of 2 (lost u006 "Who designed the Rhodes Island company logo?": canon flagged, the answer opens with a canon ruling instead of declining, the same loss as 10-01), faithful 24/28 to 33/35.
- Trivia and the 14-question probe set: no retrieval question disagrees.

**Decision.** Net +2 -3, so `--model-flags` stays opt-in; without it the flags prompt is never sent (the base arm here is the default binary).

**Follow-up.** What still fires wrong: the source flag on a question about an image or scene that is not a text kind (r078), and the canon flag on a question asking who made something (u006); both rules are prompt wording, not measured further.

### 2026-10-03: Explicit labelled age estimate for age questions: the default

**Tried.** `ian13` ("how old is angelina") gets the game-data passage under v2 and still said only "a high school girl". `ask` now adds, with the game-data passage and a question that asks an age ("how old", "age", "aged", "years old"), a rule to end with an explicit labelled estimate, a number or range with its evidence. Only r027 ("Wait, how old is Pallas?") and `ian13` ask an age in the five sets. Both were answered both ways with `--no-cache-prompt` in one session and judged by Qwen.
**Result.** r027 responds in both arms (faithful 2/2 to 2/3; the unsupported sentence is the estimate itself, "about 18 to 25, because she is described as a "young woman" [13] who has already completed her education at a priest school"). `ian13` now ends "Estimate: about 15 to 18, because she is described as a high schooler [1, 11]", which is the reference's answer (a labelled high-school-age estimate). 0 losses.
**Decision.** The rule is the default; `--no-age-estimate` is the kill switch. Verified in a separate session: the new default reproduces the opt-in arm and `--no-age-estimate` the old default on both questions, every answer field but `ms` (2 of 2 each way). Answers, judgments, prompt dumps, route and form prefilters and the scripts that made them (`job.sh`, `pipeline.sh`, `prep.py`, `prep4.py`, `chg.py`, `score.py`) are in `artifacts/answers/night5/` (n0: runs A to D; s1: items 0 to 3; s4: art and age; s6: the age kill switch).

### 2026-10-03: RELATION_RULE: keeping people and relations apart; opt-in

**Tried.** `ask --relation-rule` (opt-in) adds RELATION_RULE to a question that asks how people are related or whom someone admires ("relationship", "related", "relatives", "fan of", "idolise", "admire", "look up to"): keep what two people literally are to each other apart from what one calls the other or whom one reminds the other of, never turn a nickname, an endearment or a resemblance into a family relation, and keep two people with one title (two "masters") apart. It fires on 2 of the 25 probe questions, 1 trivia, 2 real-probe and 2 gold questions, 0 held-out.
**Result.** First wording: `ian19` got WORSE, "Trevor Friston is the father of Kristen [1]", because the passage is the dream scene whose narration says "his daughter"; `ian23` kept "Master Logos" (correct in both arms); real responds 2 to 2, gold correct 2 to 2, trivia 1 to 1, faithful 5/5 to 4/4 (real). Second wording adds "a dream, vision, illusion or memory shows what a character believes or wishes, not what is so": `ian19` becomes "In a dream or vision, Trevor Friston identifies a girl as his daughter ... Friston later realizes in the same dream that she is not his "sunshine" ... Friston is also described as a "father" to a girl who looks a lot like Kristen [12]", which is the reference's substance (not related; she reminds him of his daughter), but real probe responds 2 to 1 (lost r108 "Are Petrams related to Aegirs?", where both arms say the passages do not mention it and the judge took only the base as responding: judge noise by reading, a loss as measured).
**Decision.** Stays opt-in. No "relationships" line was added to the dossiers (optional in the plan; not built).

### 2026-10-03: DATE_RULE: a labelled date estimate when no year is stated; no measured effect, opt-in

**Tried.** `ask --date-estimate` (opt-in) adds DATE_RULE, the age rule's counterpart, to a time question that asks when ("when", "what year", "which year", "how long ago"): end with an explicit, labelled year or range from the timeline notes and the passages' relative times.
**Result.** It changes 1 answer in the five sets. `ian17` ("When was Rhodes Island founded?") is declined in both arms ("not long after the civil war in Kazdel ... they do not provide a specific date"), because the timeline notes for its passages hold nothing about the founding (they list Beyond the Snowy Mountain, Painful Reversal, Darknights Memoir and Medjehtiqedti Bound, the events of the retrieved passages, at about 1097 to 1100). Gold g0088 and u004 fire and are byte-identical.
**Why.** The chronology does hold the anchor (`artifacts/chrono/terra_history.md`: 1086, Babel excavates the Rhodes Island landship; 1090 to 1094, scenes "at Rhodes Island of Babel in Kazdel"), but only in groups the question's retrieval does not reach.
**Decision.** 0 changes on the judged sets, so there is nothing to make default; it stays opt-in.
**Follow-up.** A date question needs the chronology searched for its subject, not only notes for the passages it already has. Not built.

### 2026-10-03: LORE_ONLY_RULE for off-topic questions: ignored by Gemma, loses r042; opt-in

**Tried.** `ask --lore-only` (opt-in) adds LORE_ONLY_RULE only when the question names nothing Trevor's tables know (no operator, speaker, event, topic or merged topic name, place or identity name as whole words, possessives included; or the word "arknights"), so a lore question that names its subject keeps its prompt byte for byte. It fires on 3 of the 25 probe questions (`ian1` "Who is the best cook?", `ian18` "Who is Ed?", `ian25` the brownies), 3 trivia, 3 held-out, 21 real-probe and 3 gold questions (route-only; a first version missed possessives, "siege's", and was fixed before the answers).
**Result.** The brownie question: the default already declines without a recipe ("I could not find instructions for baking a tray of brownies in the story text. The passages provide instructions for making an apple pie [11] and a large cookie [5], but do not mention brownies."). First wording ("say in one sentence that Trevor only answers questions about Arknights lore"): "I could not find the instructions for baking a tray of brownies in the story text." Second wording ("your whole answer is the sentence "Trevor only answers questions about Arknights lore.""): the same decline, with the rule in the prompt (checked in the dump). Gemma keeps the base prompt's decline sentence over the appended rule, so neither wording produces the lore-only line. Measured: first wording, real probe responds 9 to 8 of 19 (gained r030; lost r042 "Am I missing something here ?" and r071 "A Quick Primer on the Lore for this New Event?", both now bare declines), gold correct 1 to 1, trivia 1 to 1, held-out 1 to 1; second wording, real probe 9 to 9 of 19 (gained r070, lost r042), held-out 2 to 2.
**Decision.** One loss either way, so it stays opt-in. Trevor does stay on Arknights and gives no recipe in all three arms; what it does not do is say that it answers lore only.

### 2026-10-03: Typed fallback for answers that cite only the operator's own file (leftover b)

**Tried.** The typed fallback of item 10 now also runs when the answer does not decline but cites only the operator's own file chunks (`--no-own-file-fallback`); the retry is kept on the same condition as before (it does not decline and cites one of that operator's own voice, module or art units). It needs the operator found by a word the question uses: "Mutsumi" is the first word of exactly one operator name, Mutsumi Wakaba, and no operator's whole name, so `one_operator_wide` takes it (only for this fallback; the decline fallback keeps the exact match). Candidates: 23 (the one-operator answers citing only the own file in the last full runs, lore2 and trivia-form2, plus `ian22`).

**Result.** 7 changed. Trivia correct 5 to 6 of 6 changed (gained t062 "does mon3tr know how to lead a squad?", now "Yes, Mon3tr has mastered the art of tactical command"; lost none; faithful 11/11 to 16/16; source chunk in passages 3 to 6). Gold and held-out 0 changed of 1 each. `ian22` becomes "Mutsumi Wakaba is helped by other operators who work together with her to keep the garden beautiful without saying a word [10]. Additionally, she has interacted with specific individuals like Lancet-2, THRM-EX, Master Friston-3, and Castle-3 to cultivate produce and manage the garden's systems [1]" from `module_uniequip_002_mortis`, which is the expected answer.

**Decision.** 0 losses, so it is the default. `--no-own-file-fallback` turns it off.

### 2026-10-03: Relation rule re-measured on 17 relationship questions (leftover e)

**Tried.** Ten new relationship questions from the Reddit and Discord pools (`asks_relation`, not in any set) plus the 7 in the sets.

**Result.** Changed: the new ten responds 7 to 7 of 9 (faithful 32/35 to 34/37), gold correct 2 to 2, trivia 1 to 1, real 2 to 1 (lost r108 "Are Petrams related to Aegirs?": both arms say the passages do not mention Petrams; judge noise by reading, a loss as measured). `ian19` keeps the dream reading ("In a dream or vision, Trevor Friston identifies a girl as his daughter ... later realizes in the same dream that she is not his "sunshine"").

**Decision.** Same verdict as the earlier measurement that day: opt-in (`--relation-rule`).

### 2026-10-03: A rule to explain terms (item 6)

**Tried.** A "what is/are/was X" or "who are the X" question of at most 4 words after the phrase, without a possessive, gets `TERM_RULE` (explain in plain words and define each in-world term at first use, do not copy sentences unexplained; `--no-term-rule`).

**Result.** It fires on 4 of 447: trivia t000 correct 1 to 1, and `ian28`, `ian29`, `ian31` change wording only. `ian31` still copies Narantuya's file line by line and never says what Nightzmora is, because no passage defines it: the defining lines are in the act13side hidden-story game text (`gametext_story_activities_act13side_level_act13side_hidden_st03_3#0000`), which is in P4x, and the question goes to P3b story retrieval.

**Decision.** No gain, 0 losses, so it is the default; `--no-term-rule` turns it off.

**Why.** A rule cannot define a term its passages do not hold.

### 2026-10-03: Age estimate rule

**Tried.** `AGE_RULE`, with the game-data passage, for a question asking someone's age.
**Result.** Under `INFER_RULE` alone, "how old is angelina" cited "a high school girl" and still gave no number. With `AGE_RULE`: `r027` and `ian13`, the only age questions of the five sets, show 0 losses, and `ian13` now gives "about 15 to 18".
**Decision.** Default since 2026-10-03; `--no-age-estimate` is the kill switch.
**Code.** `src/bin/ask/prompts.rs`, `AGE_RULE`

### 2026-10-03: Date rule for when-questions whose subject has chronology lines

**Tried.** `DATE_RULE`, the age rule's counterpart, for when-questions whose subject has chronology lines.
**Result.** "When was Rhodes Island founded?" was declined ("no specific date is provided") although the text places it relative to Babel.
**Decision.** On by default; `--no-chrono-subject` is the kill switch.
**Code.** `src/bin/ask/prompts.rs`, `DATE_RULE`

### 2026-10-03: Term rule for what-is questions

**Tried.** `TERM_RULE`, the answer rule for a question asking what something is (coverage audit item 6).
**Result.** "What is the khagan quest?" copied a passage and left Kuranta, Nightzmora and Khaganquest unexplained.
**Code.** `src/bin/ask/prompts.rs`, `TERM_RULE`

### 2026-10-03: Relation rule

**Tried.** `RELATION_RULE` for relationship and admiration questions.
**Result.** "What is the relationship between Kristen and Friston?" answered that Friston "refers to Kristen as his daughter" (he calls her "my sunshine"; she reminds him of his own daughter), and Lava's music teacher and the Logos she admires were both her "master". The first wording made `ian19` worse ("Trevor Friston is the father of Kristen", from the dream scene's narration "his daughter").
**Decision.** The rule now also says that a relation shown only in a dream, vision or memory is not a fact. Default since 2026-10-05; `--no-relation-rule` is the kill switch.
**Code.** `src/bin/ask/prompts.rs`, `RELATION_RULE`

### 2026-10-03: Art rule for operator art units

**Tried.** `ART_RULE`, the answer rule for operator art units (the typed fallback's art kind).
**Why.** The art descriptions were written by Gemma from the image, not taken from game text.
**Code.** `src/bin/ask/prompts.rs`, `ART_RULE`

### 2026-10-05: Answer category check (`Tools::answer_constraint`, `Tools::failing_answer_name`): default; turns `ian24`'s wrong name into a decline under `--identity-expand`

**Tried.** `Tools::answer_constraint` reads the category a who/which question asks for from its words alone: not playable ("not playable", "non-playable", "unplayable", or "NPC" when the question does not also offer "operator"), playable, or from a nation of operator_attributes that is not an organization topic ("from Victoria", "Victoria operator"). `Tools::failing_answer_name` takes the first character name in the answer's first sentence (operator names, real_names, identity-link names, longest first, not a name the question uses) and fails it by the data:
- for "not playable": a playable operator (an operator name, an operator's real name, or a name one identity link from either);
- for "playable": no operator file;
- for a nation: an operator file whose nation and birthplace are both other than the asked nation.

When it fails, `ask` answers once more with a last passage labelled "Trevor's check of an earlier answer against the game data" ("An earlier answer to this question named Sakiko Togawa. By the game data, Sakiko Togawa is a playable operator, and the question asks for someone who is not a playable operator. Do not give Sakiko Togawa as the answer ..."), and a second failure gets the data note appended.

**Result.** The detector fires on 1 of the 447 set questions (`ian24`) and 2 of the 1,228 Reddit and Discord questions ("Who is the strongest playable (EN) character lore wise?" and a long Sui-sibling speculation, both "playable"). Two earlier readings fired on real r055 ("Which operator/Rhodes Island NPC ...", which accepts both) and gold u002 ("Which Rhodes Island operator ...", an employer, not a nation); the two data rules above removed them before any answer was run.
- Under the default it changes nothing: `ian24` declines (no name to check) and the strongest-playable question names Lin Yühsia first, a playable operator, so 2 of 2 answer records are identical with and without it.
- Under `--identity-expand`, `ian24` went from "The Rhodes Island Operator who visited Iris's castle of dreams is Sakiko Togawa [11, 12, 14]" (the act45side collaboration operator, char_4182_oblvns) to "The passages do not identify a Rhodes Island Operator who is not a playable unit and had visited Iris's castle of dreams ..."; Qwen pairwise prefers the decline in both orders.
- The first version appended the data note to that decline because its second sentence names Rosmontis as context; reading only the first sentence fixed it (the measured final binary's answer has no note).
- Cost: one more full answer, only when the check fails (`ian24`: 18.55 s for the second answer).

**Decision.** Shipped as the default; `--no-answer-check` is the kill switch (verified 2 of 2). `--identity-expand` stays opt-in (no gains).

**Follow-up.** `ian24` is still not answered: with the interlude where Iris looks for Mabel Grimm and the Bluishsilver link both in the prompt, Gemma names no one, so the check removes the wrong name but cannot supply the right one.

### 2026-10-05: Relation rule gated to people (`Tools::names_group_topic`): r108 untouched, still 2 pairwise losses, stays opt-in

**Why.** r108 ("Are Petrams related to Aegirs?") asks whether two races are related, and RELATION_RULE is about people (literal relations apart from names and resemblances); both arms declined and the judge took only the base as responding.

**Tried.** `Tools::names_group_topic` (new) gates the rule: not for a question naming a served nation, place, race or organization topic (a form of four letters or more, plural allowed). Measured on the 17 relationship questions of night 7.

**Result.** The gate drops 3 (r108, gold g0114 naming Dur-nar, reld227 naming Rhine Lab), which are byte-identical to the default under `--relation-rule` (3 of 3); `--relation-rule-all` is the old reach. On the 14 it keeps, 13 change.
- Set judges: 0 losses (real responds 1 to 1, gold correct 1 to 1, trivia correct 1 to 1, the extra relationship questions responds 6 to 6 of 8, faithfulness 24/27 to 25/28).
- Pairwise: the rule wins g0099, `ian19`, `ian23`, relx299 and reld200, and loses relx26 (both say the text does not give the elder sibling of Drudge and Picale) and relx810 (both decline Mon3tr and the Aggeloi).
- `ian19` under it goes from wrong ("Friston refers to Kristen as his daughter and my sunshine ... part of Kristen's family") to partial ("In a dream or vision, Trevor Friston identifies a girl as his daughter ... later realizes in the same dream that she is not his sunshine ... a girl who looks a lot like Kristen"; it still has the girl calling him "my sunshine").

**Decision.** The two losses are judge noise by reading but losses as measured, so `--relation-rule` stays opt-in.

### 2026-10-05: The question form's opinion branch drops the "text does not settle" sentence (r030 regression)

**Result.** r030 ("In lore how strong are my favourite characters?", G) lost both responds and handles. Base: "The provided story text does not settle on a specific ranking or definitive measurement of strength for the characters, as it focuses on different narrative contexts rather than a power scaling system." New: "Based on the provided passages, the strength of the characters mentioned can be described as follows: Degenbrecher: She is described as a "natural-born warrior" and the "Black Knight," ...". A real regression by reading, if small: the question names no characters, and the new answer lists the strength of characters the opinion candidates brought in as if they were the asker's. The same mechanism loses handles on r034 (Margaret vs Degenbrecher), r056 (all dark lore) and r091 (Unleashed Amiya vs Prime Patriot): the base opened by saying the text does not settle the comparison, the new answers present the evidence per character without that sentence (r065 gained handles).

**Why.** All 4 route to `retrieve` with source story under the default (`--route-only`), their form is opinion, and `--no-question-form` (the 2026-09-30 default, +13/-3 on the 63 then measured) reproduces the baseline answer byte for byte on 4 of 4, so the question form's opinion branch is the change responsible; no other switch was needed.

### 2026-10-05: The wide opinion gate (`--opinion-gate`): fixes r030 but loses 3 questions pairwise

**Tried.** Requirement: an opinion question that names no one should get `OPINION_RULE` ("Say that the story does not settle it, then give the evidence") without candidates. The gate: the question form's opinion branch gathers candidates from operator files and dossiers only when the question names a character (an operator, a story speaker or an identity-link name it holds as whole words that the corpus writes capitalized, or that the question itself capitalizes after its first word: "Emperor", "Doctor"; or a file `named_characters` finds) or a nation, place, race or organization, or asks who or which; otherwise `OPINION_RULE` without candidates. By the detector it would hold back 73 of the 448 set questions, but only those the form calls opinion change: 16 real-probe questions and `h007`, `h008`, `h025` (forms read with `--form-only`; none of gold, trivia or the probe set's 33).

**Result.** Round 2, same session as the base: real-probe judge on the 16 plus `r007` and `r014`, responds 12 to 15 of 18, handles 8 to 10 of 10 (`r030` and `r056` regain it), faithfulness 81/93 = 0.871 to 72/81 = 0.889, 0 losses on responds or handles. `r030` goes back to "The provided story text does not settle on a specific ranking or definitive measurement of strength ...". Qwen pairwise: wins `r025` and `h025`, losses `r039`, `r067` and `r068` (the answer now opens "The story does not settle whether there are large playlists ..." on questions that are not opinions), 14 ties.

Two other answer rules for the gated questions, measured in round 3 against a base answered in the same session:
- The keyword opinion rule only (the handling before the form, the other gated questions as fact): responds 10 to 9 of 16, handles 8 to 10, pairwise 2 wins and 5 losses (it declines `r033`, `r042`, `r067`, `r068`, `r070` outright).
- `OPINION_FORM_RULE` without candidates (keyword questions `OPINION_RULE`): responds 10 to 13, handles 8 to 9, pairwise 3 wins (`r025`, `r032`, `r103`) and 3 losses (`r039`, `r042`, `h007`) on 18 of 19 (`h025` was not answered before the run ended).

**Decision.** None has 0 losses, so the default keeps the form's opinion branch; `--opinion-gate` is the requested variant, opt-in. Without the flag the code path is the old one (`f.gated` is never set). Superseded the next day by the narrow gate (below).

### 2026-10-05: RELATION_RULE narrowed to relation questions naming two people (or admiration questions)

**Why.** The two pairwise losses of the earlier relation-rule run were a question naming one person (`relx26`, Drudge, about a translation) and one naming a person and a people (`relx810`, Mon3tr and the Aggeloi).

**Tried.** `people_named` counts the characters a question names (operator names, story speakers, identity-link names, one per person: a name inside a longer one or linked to an earlier one counts once), each capitalized in the corpus or by the asker after the first word. The rule applies when it counts 2 or more, or when the question asks whom someone admires (`asks_admire`); the earlier group-topic gate still applies. Measured on the 14 relationship questions (round 2, both arms in one session).

**Result.** The rule now applies to `r016`, `g0099`, `t105`, `ian19`, `ian23`, `relx176`, `relx299`, `relx676` and `reld13`, not to `relx26`, `relx810`, `relx856`, `reld200` and `reld323`. Qwen pairwise: wins `g0099`, `ian19`, `ian23` and `relx299`; ties `t105`, `relx176`, `relx676`, `relx810` and `reld13`; 0 losses. `r016`, `relx26`, `relx856`, `reld200` and `reld323` are byte-identical. `relx810`'s two answers differ only by concurrent decoding (its prompt has no rule, checked with `--dump-prompt`, and a serial rerun reproduces the rule arm's answer). Over the 448 set questions the rule applies to 5 (`r016`, `g0099`, `t105`, `ian19`, `ian23`), all among the 14. `ian19` under the default: "In a dream or vision, Trevor Friston identifies a woman as his daughter [1] ... Friston later realizes in the same dream that she is not his "sunshine" [1] ... the girl is noted to look a lot like Kristen [12]", partial (it still has her call him "my sunshine"); it was wrong before.

**Decision.** Default (0 losses, 4 wins). `--no-relation-rule` is the kill switch; `--relation-rule-any` restores the earlier reach.

### 2026-10-05: Evidence composition for who-questions with a description (`--compose-evidence`)

**Tried.** When the answer to a question whose answer category the check reads (`Tools::answer_constraint`) declines or "does not specify" (`leaves_open`), the stated identity links whose source is among its passages become candidates: a note ("Bluishsilver and Mabel are the same person: Mabel is the former name of Bluishsilver ("Mabel, the owner of the radio, was ... the deceased Operator Bluishsilver of the Rosmontis Squad.")") followed by the P4x chunks naming either name, ranked by the question, within 3,000 tokens per link. The ask reads only these under `COMPOSE_RULE` (join the scenes, name the person who fits as a labelled inference); with the retrieved passages after them, it named the collaboration's Sakiko Togawa again. The answer is kept only when it names someone who passes the answer check.

**Result.** For `ian24` the composed passages are the note, Iris's file 2 and 4, the interlude scenes where Iris looks for "Mabel Grimm ... a Feline ... brown hair and green eyes ... very nimble with her hands" and the Field Operator's Victorian Mabel, Bluishsilver's Chapter 8 scene, and Iris's voice line ("I'm looking for a girl named Mabel ... that was twenty years ago"); with the 3-scene first version the voice line was missing. Gemma answers "The story does not state that any specific Rhodes Island Operator visited Iris' castle of dreams ...", so the decline stands and nothing changes; under `--identity-expand` the check still turns Sakiko Togawa into a decline. It fires on `ian24` alone of the 448 (the constraint fires on 1 of 448).

**Decision.** Opt-in (`--compose-evidence`); it puts Mabel's evidence in the prompt but changes no answer.

**Follow-up.** The join (Mabel met Iris as a girl, Mabel became Bluishsilver) is left to a stronger reader or a stated link.

### 2026-10-05: The relation rule's gate: questions about peoples

**Tried.** The relation rule's gate tells a question about a group topic from one about people.
**Result.** "Are Petrams related to Aegirs?" asks about peoples, not people.
**Decision.** `names_group_topic` gates the relation rule: a question that names a group topic asks about peoples.
**Code.** trevor/src/tools/topics.rs, `Tools::names_group_topic`

### 2026-10-05: Answer check: the category a who/which question asks for

**Tried.** `ask --answer-check` reads the category a who/which question asks its answer to belong to, and checks the answer's first sentence.
**Result.** A decline still named someone in its first sentence ("... that Iris met with the elite operator Rosmontis").
**Code.** trevor/src/tools/answer_check.rs, `Tools::answer_constraint`, `Tools::check_answer` (first sentence)

### 2026-10-06: The narrow opinion gate: the default, 0 losses on the 2 questions it touches

**Why.** The wide gate of 2026-10-05 held back the candidates of every opinion-form question naming no one, which lost `r039` ("How big do you think Fort barron is?"), `r067` (playlists) and `r068` ("How to come back to AK story?"), questions that are not opinions.

**Tried.** The narrow gate applies only when the question form says opinion, the keyword opinion test also fires ("favourite", "do you think", "strongest", ...), and no word of the question, capitalized, is a proper name by the corpus (`names_proper_word`: "Barron" 13 capitalized, 0 lowercase). Detector reach: 2 of the 448 set questions (`r030`, `r116`; the wide gate 73) and 19 of the 1,228 (the wide gate 433). `r039` (Barron), `r067` and `r068` (no opinion keyword), `r034` and `r091` (they name people) and `r056` (no keyword) are not touched. Measured in one Gemma session, `--no-cache-prompt`, one client.

**Result.** Real-probe judge (Qwen, `CORPUS=artifacts/p4x`): responds 1 to 2 of 2 and handles 1 to 2 of 2 (`r030` regains both: "The provided story text does not settle on a specific ranking or definitive measurement of strength for the characters ..."); `r116` responds and handles both ways. Faithfulness 16/18 to 12/14. Qwen pairwise 2 ties, 0 losses. What it does not buy: the wide gate's two pairwise wins (`r025`, `h025`) carry no opinion keyword and keep the old answers.

**Decision.** Default. `--no-opinion-gate` is the kill switch; `--opinion-gate` is now the wide reach of 2026-10-05 (measurement only); `--detect-only` prints `opinion-gate` (narrow) or `opinion-gate-wide`.

### 2026-10-06: Why partial answers are partial: 13 classified

**Tried.** The 13 partial answers of the 33-question probe set were classified against the night-12 answers (`artifacts/answers/night12/F.ian.jsonl`) and their passages.
**Result.** 7 fail in the answer step with the evidence in the prompt.
- Missing passage, 4: `ian1` (Hung, Shu, Yu and Gummy's files absent), `ian17` (the post-1094 reorganisation), `ian28` (the civil war and Episode 14), `ian29` (the Collapsals and the Aethergate, 0 hits in the prompt).
- Poor synthesis, 4: `ian5` joins Leto and the beast Lily of one scene; `ian19` pins Ho'olheyak's "a pair of father and daughter" vision on Friston; `ian27` omits the sealed core, Santalla's seat and the Snowpriest's horn, all in its passages, and stops mid-sentence; `ian31` gives the Nightzmora's reputation ("still seen as") as their definition.
- Hedge, 2: `ian26` "The passages do not mention whether ...", which is `YES_NO_RULE`'s own wording; `ian25` the base decline, really the off-topic gate.
- Wrong focus, 1: `ian22` opens with the unnamed operators of [10], while the four robots are [1] and [2].
- Missing data, 2: `ian10`, `ian16`.
**Decision.** Two fixes were tried against the answer-step failures (next two entries). Not built: an attribution sentence in the shared SYSTEM prompt, which would touch every answer and could not be measured in the time available.

### 2026-10-06: Answer extension: a cut answer is asked again with twice the limit (default)

**Tried.** An answer that stops at `--n-predict` (400) is asked again with twice the limit. Greedy decoding without the prompt cache repeats it, so the old answer is an exact prefix of the new one.
**Result.** `ian27`: 1,731 to 1,746 characters, now ending "rewritten by the calamity [12]." It touches 1 of the 108 distinct questions answered that day (`ian27`; 0 of the 30 gold and 0 of the 50 yes/no); pairwise a tie. It does not add the three omitted facts. Kill-switch check: `--no-answer-extend` on the new binary against the night-12 records on `ian1`, `ian22`, `ian26` and `ian27` gives 4 of 4 answers byte-identical (`ian27` the cut one). Under the new default only `ian27` changes on the 33-question probe set (completed sentence, still partial), so the count stays 14 right, 13 partial, 0 wrong, 6 declined by the night-12 reckoning. With the line pool and the verdict rule off, the new binary reproduces the night-12 answers on the 25 probe questions the pool does not fire on and on the 5 yes/no ones (5 of 5). Tests 82 + 3 + 1 (86 with embed), clippy clean.
**Decision.** Shipped as the default. `--no-answer-extend` is the kill switch and reproduces the old answers.

### 2026-10-06: Verdict-first yes/no rule refuted (`--verdict-first`)

**Tried.** `YES_NO_VERDICT_RULE` (`--verdict-first`): each part opens with its verdict, "Not that the passages show" when the passages describe the person at length without it, plus "keep each fact with the person and scene its passage gives it to". Measured on the 50 questions whose form is yes_no (14 held-out, 5 from the 33-question probe set, 20 real probe, 11 trivia) with Qwen pairwise.
**Result.** 50 of 50 answers change. Pairwise 5 wins (`ian9`, `r004`, `r082`, `r108`, `t051`), 7 losses (`h022`, `h028`, `r063`, `r101`, `r106`, `t061`, `t099`), 38 ties. `ian26` still opens "The passages do not say whether Deepcolor is part of the Church", `ian5` now says Harold "refers to her as Lily", and `ian4` shrinks to one decline sentence.
**Decision.** Refuted; the flag stays opt-in.

### 2026-10-06: Opinion gate for opinion questions that name no one

**Tried.** When an opinion question asks who or which but names no proper word, and the keyword test also calls it an opinion, the opinion gate gives it `OPINION_RULE` ("the story does not settle it", then the evidence) without candidates.
**Result.** In the 2026-10-05 regression run, "In lore how strong are my favourite characters?" (real `r030`) listed Degenbrecher's strength as if she were the asker's favourite.
**Decision.** Default since 2026-10-06; `--no-opinion-gate` is the kill switch.
**Code.** `src/bin/ask/people.rs`, `opinion_needs_candidates`

### 2026-10-16: Operator list check (`ian35`): a non-operator ranked as an operator

**Why.** "Give a ranking of operators who break the most Geneva convention rules" ranked Aurela first. Aurela is the fallen Laterano gun-knight and mercenary in Gracebearer's file 4, not an operator.
**Tried.** A new default, the operator list check (`--no-operator-list-check`):
- It applies to a question asking for operators (`Tools::asks_operators`: "operators", or "operator" with who/which/rank/list; not with "NPC" or "character" offered, and not a NonPlayable question).
- When the answer lists, as a bold item, someone who is not a playable operator (`Tools::non_operator_items`; topics excepted; an item whose own line already says "not a playable operator" excepted), the question is asked once more with a note naming the person and whose operator file the cited passage is.
- When the second answer still lists one, the data note is appended.
**Result.** `ian35` now lists Caper, Crownslayer and Rosmontis with what their files actually say, keeps Aurela only as "not a playable operator", and declines the ranking.
**Decision.** Shipped as the default; `--no-operator-list-check` is the kill switch.

### 2026-10-16: Answer extension for answers cut at the token limit

**Tried.** An answer that stopped at `--n-predict` is asked again with twice the limit.
**Result.** `ian27` stopped mid-sentence at 400 tokens.
**Decision.** On by default; `--no-answer-extend` is the kill switch.
**Code.** `src/bin/ask/cli.rs`, `Args::no_answer_extend`

### 2026-10-16: Operator list check (`ian35`)

**Tried.** `ian35`, "Give a ranking of operators who break the most Geneva convention rules", ranked Aurela first, a Sankta mercenary named in Gracebearer's file 4. `Tools::asks_operators` detects a question asking for operators: it says "operators", or "operator" in a who/which question, and not that the answer is not playable (that is `answer_constraint`'s NonPlayable), and not when it offers NPCs or characters as the alternative ("Which operator/Rhodes Island NPC comes closest to being a Psychologist?", real `r055`, accepts both).
**Decision.** An answer whose list items name someone who is not a playable operator is asked once more with a note naming each such person and the operator file the cited passage came from. A test pins that Aurela is caught. `--no-operator-list-check` turns it off.
**Code.** trevor/src/tools/answer_check.rs, `Tools::asks_operators`, `Tools::answer_constraint` (NPC); trevor/src/bin/ask/answer.rs, `operator_list_check`; trevor/src/tools/tests.rs, `operator_list_items`

### 2026-10-18: Identity-claim check: the Quiet Woman is not Silence

**Tried.** On the stories page (mid profile, E2B, horizon gatePos 722, era Shatterpoint), "Tell me more about the Quiet Woman" ended with "Later, she is referred to as Silence in the context of a secret meeting [3]". The facts, from the corpus: the Quiet Woman speaks in Guide Ahead's GA-ST-1 "Right Place, Right Time" (`act16side_level_act16side_st01#0000`, lines 13 to 109; speakers Wasting Man, Quiet Woman, Cecelia, Sickly Mother): a woman in a wheelchair in Laterano who tells the Wasting Man not to come back. Silence is not a name anywhere in Guide Ahead: its one "Silence" is "the Profound Silence", an Iberian event (`act16side_level_act16side_08_beg#0000`). Passage [3] is Mansfield Break MB-1 "Secret Meeting" (`act15d0_level_act15d0_01_beg#0000`, lines 16 to 116): Silence and Mayer meet Muelsyse in a Columbian bar. As an eval reference only, arknights.wiki.gg's GA-ST-1 page lists the Quiet Woman as Lemuen and the Wasting Man as Andoain, and its MB-1 story page sets MB-1 in Columbia with no mention of Laterano.
Cause, reproduced byte for byte with `ask --serve --profile mid --share-server --no-cache-prompt --dump-prompt` at horizonPos 722: retrieval, then the model. The dense retriever puts "quiet" next to "Silence": its top hit for the question is that MB-1 chunk (cosine 0.764), and BM25's top 4 hold no Silence chunk. Passages [3] to [6] of the 6 in the prompt were Mansfield Break chunks full of Silence's lines. No identity link names the Quiet Woman (0 in `identities.json`, 0 in `identities.v2.json`), the passage labels are story titles, and no term lookup or note was in the prompt: E2B wrote the bridge sentence itself.
The check (`answer/identity.rs`): after the answer and its other checks, a sentence with an identity phrase ("referred to as", "known as", "is actually", "is also called", "revealed to be", 35 phrases) whose subject is a name, or "she"/"he" standing for the answer's subject, and whose target is a capitalized name, stays only when an identity link (`Tools::identities`, `Tools::identity_links`) joins the two names or one passage given to the model holds both as whole words; otherwise the sentence is dropped and the citations are recounted (when every sentence would go, the answer stays with a "Trevor's check" note). No model call, so the prompts do not change.
**Result.** The question at horizon 722: the Silence sentence is dropped, citations 2 -> 1; the prompt is byte-identical to HEAD's in both arms (2 of 2 calls) and `--no-identity-check` returns HEAD's answer exactly. Where it could fire, with no model (`ask --identity-scan DUMP` reruns the check over a prompt dump): the first cut fired on 2 of 393 answer calls in 5 older dumps, both wrong ("Regarding Midnight, the story describes him as a former host who was known as the Devil...", and "they are called Originium slugs" with the answer's first word as subject); the subject must now sit right before the phrase and only "she" and "he" stand for the answer's subject, which takes both to 0 (tests pin them). Then E2B mid runs (`--share-server --no-cache-prompt`, dumps) over Ian's 35, the real probe's 120, the 40 held-out and gold v2's 146: 336 answer calls, 5 answers with an identity phrase, 0 changed (2 had a common-noun subject, 1 a relative clause, 1 an article target "a Banshee", and "Sussurro is also identified as the Landship Medic Operator" has both names in its passage [4]). 0 changed answers, so 0 losses on 341 questions; no judge was needed.
**Decision.** On by default; `--no-identity-check` is the kill switch. Left: the dense retriever still brings Silence's chunks to a question about the Quiet Woman; the check removes the claim, not the passages.
**Code.** `src/bin/ask/answer/identity.rs` (`check`, 5 tests), `src/bin/ask/answer/mod.rs`, `answer_one`; `src/bin/ask/batch.rs`, `identity_scan`

### 2026-10-18: Referent checks and deaths evidence: Mandragora is dead, but not by that line

**Why.** "Is Mandragora alive or dead?" (E2B, mid, spoilers on) answered "Mandragora is dead [5]. Harmonie states that 'she's dead' in reference to The Leader.", citing Stormwatch "Verse in the Ashes" (`main_9_level_st_09-02#0002`, from line 124). The facts, from the game text: in that passage Mandragora herself says "Hah, she's dead. ... the traitor's been executed" of Saoirse Kelly, whom The Leader has just executed; Harmonie never says it. The verdict is right for another reason: Mandragora dies in Shatterpoint's interlude "Crossroads" (`main_10_level_st_10-02#0002` to `#0004`: Manfred's "You're already dead, Taran", then Misery: "she has met the true Death"), and in What the Firelight Casts "Unlucky Reunion" The Brigadier says "Mandragora is dead." (`act22side_level_act22side_02_end#0004`, the one row `deaths.jsonl` has for her).
**Tried.** Three changes, each with a kill switch:
- the fate check (`answer/referent.rs`, `--no-referent-check`): a sentence stating that a named character (or "she"/"he" for the answer's subject) is dead or alive, died, was killed or survived, with citations, stays only when a cited passage ties the fate to them: a line naming them (or an identity-linked name) beside a line with a death or survival word, or their own line with a first-person fate phrase ("I'm dying"); a line they speak about someone else does not count, nor does a passage label unless it states the fate itself;
- the quote check (`--no-quote-check`): "X says/states/notes ... 'quote'" is dropped when the passages hold the quote only in lines another named speaker says;
- the deaths evidence (`fate.rs`, `--no-death-evidence`): an alive-or-dead question about one character of the deaths table gets the lines its rows quote (the first row that shows the death and the latest report, at most 2, two lines either side) as the first passages, with a rule that a line about someone else's death is not about them; the passages are story chunks, so a spoiler horizon filters them.
Dropped sentences follow the identity check: when every sentence would go, the answer stays with a "Trevor's check" note.
**Result (no model).** Through the stub server with Ian's answer text: without the deaths evidence the passages are those of the preview and [5] is `st_09-02#0002`, and the answer now ends "Trevor's check: the passage cited for what happened to Mandragora does not say it of Mandragora; in the passages that line is said by Mandragora, not Harmonie."; with it, passage [1] is The Brigadier's line and "Mandragora is dead [1]." stays while the Harmonie sentence goes. Over 1,170 recorded answer calls (98 prompt dumps since night 5, 12B and E2B) the checks change 1 answer: a held-out answer that gave Carnelian's "no gentle little lambs" to a Noble Attendant (a true misattribution; the sentence's other, correct quotes go with it). Two first cuts were refuted on that scan: an appositive ("**Mabel Grimm (Bluishsilver):** A deceased Operator", 5 answers) and a name after a preposition ("The Young Chef in Baizao claims"), and the first-person test first took "I just couldn't hold on! ... Dead." as Mandragora's own death. The deaths evidence fires on 7 of 1,754 questions (after "what" questions and lowercase role names were excluded: gold `g0126` and "Was Crownslayer's teacher killed" had fired), 0 of them in the four sets the stub replays.
**Decision.** All three on by default pending the model window (E2B mid: the Mandragora question in both arms); the off values are verified byte-identical on the four sets.
**Code.** `src/bin/ask/answer/referent.rs` (4 tests), `src/bin/ask/fate.rs` (2 tests), `src/bin/ask/answer/mod.rs`, `answer_one`; `src/bin/ask/answer/generate.rs`, the deaths rule; `src/bin/ask/prompts.rs`, `DEATH_RULE_TAIL`; `--identity-scan` runs all three answer checks over a dump

### 2026-10-18: Passage labels out of the answer text

**Tried.** `answer/cite_words.rs` (`--no-passage-label-fix`): "passage"/"passages" right before a citation (or a bare number, which becomes the citation) is written "the text"/"the texts" ("The text" at a sentence start, no article after "the", "provided" and the like), so a reader sees only bracketed citations.
**Result (no model).** 5 of the 341 recorded E2B mid answers change, all labels ("Elysium appears in the texts [1], [2] ...", "In the text [3], Grace asks", "based on T1, T2, and the text [2]"), and 13 of 1,170 recorded answer calls.
**Decision.** On by default; no prompt changes.
**Code.** `src/bin/ask/answer/cite_words.rs` (1 test)

### 2026-10-18: A labelled theory for declined whether-questions (Fyodor), measured where it can fire

**Why.** Ian wants "Is Fyodor infected?" answered with the hints rather than declined: Fyodor coughs (`main_17_level_main_17-08_beg#0006`, "Absurd... Ridiculous! ...*cough* *cough*!"), asks "What of the Infected situation?" and closes the Infected mines (`main_8_level_st_08-06#0020`), and Witte tells him of "Talulah the Infected" (`#0021`). The text never says he is infected.
**Tried.** `theory.rs` (`--no-theory-rule`): when the first answer declines, the question is a whether-question whose subject is the one character it names, right after its first word, and not about the game (sprites, skins, design and the like) or about life and death (the deaths evidence covers those), the hint lines are the passage lines that touch a stem of the question's other words (Oripathy words for "infected": cough, crystal, Originium) and are the character's own, name them, or reply to them. With at least one, the question is asked once more with the lines as a HINTS block and THEORY_RULE ("The story does not say outright, but ...", the hints with citations, a closing "Theory:" sentence as Trevor's reading); the second answer is kept only with its "Theory:" line. The first prompt never changes.
**Result (no model).** On the 341 recorded E2B mid answers, the first cut (any whether-question, any stem) fired on 6, of which 5 were wrong targets (game questions such as "Does Fyodor deserve a sprite", "Does anyone know what crossbow Faust uses", group names), and `ian26`'s "deep" matched inside "Deepcolor's" (its verified answer is the decline). After the subject rule, the game words and the name exclusion it fires on 0 of 341. With a declining stub answer, "Is Fyodor infected?" gets 3 hint lines from Roaring Flare on the stale P3b (Episode 17 is not in it yet).
**Decision.** On by default pending the model window: no recorded answer changes, so only the Fyodor question (after the P3b rebuild) and the 6 first-cut questions measure it.
**Code.** `src/bin/ask/theory.rs` (1 test), `src/bin/ask/answer/mod.rs`, `theory_retry`; `src/bin/ask/prompts.rs`, `THEORY_RULE`

### 2026-10-18: Model window 1 for the night's fixes (E2B mid) and what changed after it

**Tried.** The orchestrator ran `scratchpad/n/window.jsonl` (7 questions) on E2B, `--profile mid --no-cache-prompt`, arm A the defaults of `97a472ae`, arm B every new switch off, after the corpus refresh (P3b: 273 Episode 17 chunks, Fyodor in 51 chunks; `appearances.py` and `ask --gate-order` rerun).
**Result.**
- Mandragora: A "Mandragora is dead according to the lines in the text [1]." citing The Brigadier's line; B repeated the misattribution. Fixed (the wording is vague).
- Idida: A "...take the Gap-Toothed Girl out without anyone noticing [1]" (The Masses' Travels); B "Kal'tsit". The scene is found; the name, given two lines later ("Idida: Dadadeedeeda, I'm called Idida."), is not used.
- `ian33`: wrong in both arms. A had the Idida scene at [2] and answered from [1], `act42side_st04#0000`, whose line "the instigators were Exusiai, Executor, and a silver-haired Sankta" the reranker scores above "(I can take her out. Nobody would notice.)".
- Discord 7 ("Is the Black Knight mentioned in the Near Light event the operator Blacknight"): A "Sona", B "Margaret Nearl", both wrong. Truth from the game text: Zofia's "The Black Knight's a very unusual case of Leithanian ... born to be martial" (`act13side_level_act13side_03_beg#0007`) is the champion Degenbrecher, whose operator file says "you definitely can't leave out the Black Knight ... Degenbrecher's third victory" (`archive_char_4116_blkkgt#0001`); Blacknight (`char_476_blkngt`) is a Rim Billiton bounty hunter. A's scoped line rank fired because "mentioned" followed a scoped name, and it displaced B's `03_beg#0007`. Its scoped characters come from identity groups that join "Knight" to Sona and "the Black Knight" to Trilby Asher (`identities.json`), which are wrong links outside this change.
- Talulah: A answered from the appearance index but named First of A Thousand Autumns. Cause: the rebuilt index gives `main_17` order 10000, because the reading guide does not list Episode 17 (`main_release.json` has no Episode 17 date), and `gate_order.json` puts all 24 Episode 17 stories in `never`, so they are hidden at every horizon.
- Fyodor: declined in both arms; the theory never fired. Cause: E2B wrote "The provided passages do not state ...", and `detect::declines` knows "does not state" but not "do not state".
**Changes (no model).**
- Theory: its own decline test `theory::declined` (plural and hedged forms; `detect::declines` is unchanged); the second ask pins the first ask's passages with hint lines first, adds up to 3 one-line excerpts of the character's own lines that touch the question (Episode 17's "...*cough* *cough*!"), retrieves without neighbouring chunks and gives up 800 tokens for the HINTS block, so its prompt (13,855 characters) stays below the first (15,385). On the four sets it now fires on 3 of 341 recorded answers ("Isn't Muelsyse the last of the elves", "Is Phantom currently a sleeper agent for the Crimson Troupe?", "Is the Doctor non-infected?").
- Scoped scenes: the scoped line rank is replaced by scoped excerpts (`scenes::scoped_excerpts`, same switch `--no-scoped-line-rank`): a wh-question about what one character of the named event said there gets excerpts of that character's best reranked lines, labelled and ruled as character scenes, as the Idida question gets; a whether-question keeps the scoped scenes. It fires on 1 of the 34 questions with scoped scenes in the eight sets (`ian33`); Discord 7 gets B's passages again.
- `CHAR_NAME_RULE` (`--char-rule-v1` turns it off): with character scenes, a who or name question gets the name the scene gives the person, also when it comes a few lines after the answering line. It changes the prompts of the 9 character-scene questions of the four sets.
- Deaths rule: after A's "dead according to the lines in the text [1]", `DEATH_RULE_TAIL` asks for what the line says, who says it and in which story (only the 7 deaths-evidence questions' prompts change).
- Appearance order: groups ordered by the gatePos of their first story; a main episode the order does not place goes last and is labelled "newer than Trevor's release order"; "Episode 17" when the index has no name; the answer is 2 to 3 sentences plus a one-line source note. Talulah now: "In EN release order, the latest story where Talulah appears is Episode 17 (newer than Trevor's release order, so placed after it): Talulah speaks in 20 scenes."
**Verified.** On the rebuilt corpus, every switch off reproduces HEAD before the night's work (`d16d65ba`) on all 341 records and prompts of the four sets; the defaults change 2 answers by the label fix (`ian32`, `r112`), 9 prompts by the name rule, `ian33`'s passages, and add the 3 theory asks.
**Decision.** Defaults pending window 2. Not fixed here: Episode 17 placement (reading guide dates), the stale story names (`artifacts/goldgen/names.json`, 2026-09-25, has no Episode 17 or People story, so their labels read "story: <id>"), and the Knight and Black Knight identity links.
**Code.** `src/bin/ask/theory.rs`, `src/bin/ask/scenes.rs` (`scoped_excerpts`, `pick_excerpts`), `src/bin/ask/appear.rs`, `src/bin/ask/prompts.rs` (`CHAR_NAME_RULE`)

### 2026-10-19: Model windows 2 and 3 (E2B mid, 17 questions)

**Tried.** `scratchpad/n/window2.jsonl` (the 7 window-1 questions, 7 gold questions the name rule touches, and the 3
questions where the theory fires) on E2B, `--profile mid --no-cache-prompt`, one client. Arm A the defaults, arm B
every switch of the night off. Window 2 ran `50da4213`; window 3 ran `cab32936` (review fixes `c1be4fba`, the name rule
only for who and name questions) on identities rebuilt by `5cc5a323`.
**Result (window 2).** A better than B on 5: Mandragora (A quotes The Brigadier's "Mandragora is dead", B mixes her up
with the Leader), Idida (A "the Gap-Toothed Girl", B "Kal'tsit"), `ian12` (A names The Masses' Travels), Talulah (A
Episode 17), Fyodor (A a labelled theory, weak: no cough hint in it). A theory instead of a decline on `h023`, `r015`,
`r101` (labelled, thin). A worse on 2: `g0091` (the name rule was the only prompt difference; the answer moved Count
Vincent's line to Heidi) and `g0029` (A adds "present with her" to a question about who is absent; the same rule,
read as noise). Wrong in both: Discord 7 (both "Nearl"; the prompt held 5 Blacknight file chunks and an Ebenholz scene
drawn by the identity link Operator -> Ebenholz) and `ian33` (both name Exusiai and others; truth Idida).
**Result (window 3).** B equals window 2's B on 16 of 17 (Discord 7 differs: the identities changed under both arms).
A: `g0091` is right again (Count Vincent); Discord 7 declines ("does not mention the Black Knight in relation to the
Near Light event") instead of naming Nearl; `ian12` lost the event name it had only through the name rule; the rest
match window 2's A or B.
**Decision.** Keep the defaults. Open: Discord 7 needs a Near Light passage about Degenbrecher; the appearance index
leaves titles out by design ("King of Sarkaz" would credit Amiya with Theresa's lines), and Near Light names her only
"the Black Knight". `ian33` still answers from the reranker's top line.

### 2026-10-19: Model windows 4 and 5 (E2B mid, the probe sets)

**Tried.** Window 4: `eval/ian-questions.jsonl` and `eval/real-probe.jsonl` (155 questions) on `cab32936`, arm A the
defaults, arm B every switch of the night off (147 of 155 in B: the run hit a 2-hour job limit). Window 5: the 35
probe questions and 4 real-probe theory questions, defaults only, on `7b435d9b`.
**Result.** Window 4: 7 of 147 answers differ. 2 are the passage-label fix only (`ian32`, `r066`); 4 are theories where
B declined (`r015`, `r020`, `r101` reasonable; `r052` "Is Specter Alter Canon?" noise, so canon words joined the
theory's META list, `7b435d9b`); `ian33` is wrong in both. `ian32` ("What chapters does Elysium appear in?") was
answered from 7 passages in both arms: under the keyword router the question-only detectors never ran (`728f0827`).
Window 5 against window 4's A: exactly the 3 rerouted questions change. `ian16` gives the 12-operator bosses table
(the probe's expected answer), `ian32` the appearance index (Episodes 7, 8 and 16, Rewinding Breeze, Preluding
Lights, Stultifera Navis, Exodus, Ideal City, Long Journey, mentioned in Path of Life and Will You?: the expected
list), `ian27` names the four IS4 endings with Ending 2 "Winterfall" but not its content (was "could not find").
**Decision.** Keep the defaults.

### 2026-10-19: Model window 6 (Gemma 12B, full profile, 17 questions)

**Tried.** `scratchpad/n/window2.jsonl` on the 12B with `--profile full --no-cache-prompt`, binary `7b435d9b`, arm A
the defaults, arm B every switch of the night off (the same B flags as windows 2 to 4).
**Result.** 10 of 17 identical (Mandragora is a deaths-table answer in both). A better on 3: Fyodor (a labelled theory:
his concern for the Infected and the mines he closed; B declines), Idida ("the Gap-Toothed Girl", The Masses' Travels;
B "could not find it"), Talulah (Episode 17; B named a 1099 story). A worse on 1: `ian33` (A "the Gap-Toothed Girl",
B "Idida, a girl associated with a dessert cart", the expected name). `r015` and `r101`: theory against an informative
decline, a tie. Discord 7 declines in both.
**Decision.** Keep the defaults. `ian33` on the 12B: CHAR_NAME_RULE is in A's prompt and the answer still gives the
description, so the rule does not do its job on this question; open. The excerpt [2] holds "Gap-Toothed Girl: It's
got my answer for me, confessor." and then "Idida: Dadadeedeeda, I'm called Idida.": the evidence is in the prompt and
the model keeps the speaker label. Fix direction (unmeasured): link a description speaker label to the name label that
follows it with a self-introduction in the same scene ("I'm called X", "my name is X") as an identity, so the passage
can be labelled "Gap-Toothed Girl (Idida)". Census over P3b (no model): a speaker label with a description word (Girl, Man,
Woman, Youth, Voice, Old, Young, ...) followed within 3 lines by a label that introduces itself by that name gives 33
label pairs; true renames among them (Gap-Toothed Girl -> Idida, Old Woman -> Myrrh, Elegant Woman -> Blaze, Gentle
Woman -> Mercia, Boy in Patient Gown -> Popukar) and false ones where someone else introduces themselves after the
described speaker (Soldier -> Alita, Calm Youth -> Red). A rule needs a further condition, for instance that the
described label never speaks again in the scene after the switch. Tried on whole stories: it keeps 22 and drops 16, and
it separates nothing: it drops true renames whose description label returns later in the story (Old Woman -> Myrrh,
Elegant Woman -> Blaze, Mysterious Fortune Teller -> Kal'tsit) and keeps false ones (Calm Youth -> Red; both
Raggedly-Dressed Boy and Raggedly-Dressed Girl -> Arturia). Refuted as the condition; scene boundaries (not stories)
and the self-introduction being the first line of the new label are the next candidates.

### 2026-10-19: Renamed speaker labels in passages (Idida)

**Tried.** `scripts/label_renames.py` (11 story-scoped renames, see the window 6 entry for the census and the two
refuted conditions) and `labels::with_label_names`: in those stories' passages "Gap-Toothed Girl:" becomes
"Gap-Toothed Girl (Idida):". Also STORY_NAME_RULE for which-event questions (1 of 450 eval prompts: `ian12`).
**Result.** Stub, 334 prompts of the four sets: 5 change; `--no-label-renames` reproduces 94b04249's prompts byte for
byte. E2B mid: `ian12` complete ("Gap-Toothed Girl (Idida) offered ice cream to Mon3tr, and Mon3tr then suggested she
could take the little girl out", The Masses' Travels), `ian33` still wrong (answers from the explosion line), `r119`
reworded, 2 identical. 12B full: `ian33` "Mon3tr suggests that she could take out Idida, who is also referred to as
the Gap-Toothed Girl" and the Idida question names her too; with the switch off both say "the Gap-Toothed Girl" only.
**Decision.** Default on (`--no-label-renames`, `LABEL_RENAMES=0` for the builder; `--no-story-name-rule`). `ian33` on
E2B remains a reading miss: the scene is in the prompt and the model answers from passage [1].
**Code.** `scripts/label_renames.py`, `src/bin/ask/labels.rs` (`with_label_names`), `src/bin/ask/answer/generate.rs`
**Regression (final binary 7a82a428, E2B mid, the 155 probe questions, defaults only, against window 4's A on
cab32936).** 148 identical; the 7 that change are all ones the later commits target: `ian16`, `ian27`, `ian32`
(keyword-router detectors), `r052` (declines; no theory on a canon question), `ian12` (event and Idida named),
`ian33` (still the explosion line, reworded), `r119` (reworded). No unexpected change.

### 2026-10-19: Probe set on the 12B, full profile, after the night

**Tried.** `eval/ian-questions.jsonl` (35) on the 12B, `--profile full --no-cache-prompt`, final binary of the night
(`7a82a428`), graded by hand against each item's `expected` (one grader; no judge model).
**Result.** 25 right, 5 partial (`ian10` NPC list, `ian18` Ed, `ian19` Kristen not stated as unrelated, `ian22` adds
"other operators", `ian35` no ranking), 5 declined (`ian6` pee/piss wording gap, `ian11`, `ian12`, `ian24`
Bluishsilver, `ian30` Dossoles factions), 0 wrong. The last recorded tally was 14 right, 13 partial, 0 wrong, 6
declined of 33 (2026-10-06); the set grew by 2 and the grader differs, so the comparison is indicative. Newly right
include `ian16` (bosses table), `ian27` (Winterfall with Santalla, from P4 under the model router), `ian32`
(appearance index), `ian33` (Idida, label renames). `ian12` declines on the full profile although E2B mid answers it.
**Decision.** None (measurement). `ian12` on full: the first ask declined with the Idida scene at [1] and the typed
fallback's correct retry was discarded for citing no Mon3tr unit; fixed by `43e30986` (the retry is kept when it cites
only the lead passages both asks had), so the tally is 26 right, 5 partial, 4 declined, 0 wrong. Open: the 4 declines.

### 2026-10-19: Group summaries that stop after the first stories (Episode 15)

**Why.** "Give a summary of chapter 15" returned three paragraphs about the opening interlude and Theresis
(`st_15-01`, `15-01_beg`, `15-01_end`, `st_15-02`, which recaps the end of Episode 14), so it read as a summary of
Episode 13 or 14. The P3b text and the 25 story summaries of `main_15` are right (Hierda and Raydistorter, the
Priestess, Oracle and AMa-10, Kal'tsit vanishing in 15-16, the PRTS core in the Abyss); the 12B's group summary
(method C, 293 words, under its 700-token limit) spent its length on the first stories.
**Measured (no model).** Name coverage per group: the capitalized names that recur in at least a quarter (and 3) of a
group's story summaries, and the share of them missing from the group summary (method C). Of 86 groups with 6 or
more stories the worst are act41side 0.83, main_17 0.71 (written on 2026-10-18), act21side 0.60, main_15 0.57,
act12side 0.44; the measure is noisy ("Finally", "Nearby" count as names).
**Decision.** Open, needs the model: regenerate the low-coverage groups with a prompt that gives each part of the
story its share (for instance a third of the stories per paragraph, or 1 to 2 sentences per story, then a merge),
gated on the coverage measure, and add the measure to update.sh as a check.
**Built (no model yet).** `scripts/summary_coverage.py` is the measure as a script: a word counts as a name only
when the group's story summaries use it inside a sentence at least once and it is not on a noise list (Finally,
Nearby, Meanwhile and other openers), a possessive counts as its name; it writes `artifacts/p2/coverage.json` and
exits 2 when a group of 6+ stories misses more than `--threshold` (0.4) of its names. On the current tree 87 groups
have 6+ stories and 5 are above 0.4: act41side 0.75 (9 of 12), main_17 0.75 (15 of 20), act21side 0.52 (11 of 21),
main_15 0.46 (6 of 13: Closure, Hierda, Island, PRTS, Priestess, Rhodes), act42side 0.41 (9 of 22); then act51side
0.38, act12side 0.36, act18mini 0.33, act14side 0.30, act15side 0.30. The noise filter moves act12side from 0.44 to
0.36 and act32side from 0.44 to 0.28 (act32side's misses are now Arrowhead, Commanding, Lucent, Officer, Operation).
update.sh runs it after the group summaries as a report (`COVERAGE_CHECK=0` skips it, `COVERAGE_STRICT=1` stops on
it). Method D in `p2-summaries.py group --method D` sections a group of more than 8 stories (`GROUP_PARTS_MIN`) into
3 to 5 consecutive parts (ceil(n/7), sizes within one: main_15's 25 stories are 6, 7, 6, 6), one 100 to 130 word call
per part (`prompts/summary.group.part.txt`), then a merge into 300 to 400 words that keeps every part
(`prompts/summary.group.merge.txt`); rows carry `method: 'D'` and `parts`. `summary.group.txt` is unchanged, so A, B
and C plan 0; the 7 groups plan 7. Part prompts of the 7 groups are 1,441 to 1,900 Gemma tokens (main_15: 1,531,
1,847, 1,633, 1,609; merge 285 plus the parts). The served summary (P3b `summary_<group>` units in build-profiles,
the bank's `event:` rows that `ask`'s recap reads) is D where present (`SUMMARY_METHOD_D=0`: C; P3b chunks and the
bank seed byte-identical with D rows present, verified on a scratch copy). Primers, overview and chronology still
read C and topic_deep leaves D rows out, so a D row changes no other model stage's input.

### 2026-10-19: "How does Mon3tr react when she wakes up after Kal'tsit dies" (open)

**Truth from the text (corrected: the waking is in Episode 15, not The Masses' Travels).** In `main_15-16_beg#0000`
Kal'tsit "has disappeared ... The only thing left on the deck is a jacket" and Mon3tr shrinks "into a black,
crystalline cocoon"; Amiya carries the cocoon out of the Abyss (`15-18_end#0001`). The waking is `15-18_end#0007` to
`#0009`: the cocoon opens, "You hear an unfamiliar voice. ???: ...Kal'tsit?", she feels rain for the first time, puts
on Kal'tsit's clothes, calls Kal'tsit's name "in uncertainty", finds she has all of Kal'tsit's memories but cannot
sense her, then "Mon3tr: My name is Mon3tr. You gave me this name." She asks where they are (New Rhodes Island),
says "I faced Priestess together with Kal'tsit, my maker... Then I lost her", asks to count as an operator, and reads
Kal'tsit's restricted log ("You are free"). The Masses' Travels (`act42side_st04#0003`) only looks back on it: "when I
awoke, you just turned around and walked away after giving me a 'welcome back'".
**Measured (no model).** Hybrid retrieval on P3b for the question ranks `act42side_st04#0003` first and
`main_15-16_beg#0000` fourth; `15-18_end#0007` is not in the top 10 (its text has "cocoon" and "unfamiliar voice",
not "wake"). "How does Kal'tsit die, and how does she get revived?" retrieves neither scene: the text says
"disappeared", not "died".
**Tried (opt-in, not a default).** Two levers, each behind an opt-in flag: `--react-verbs` adds react, respond, feel,
wake, behave and cope to the scene-subject verbs, and `--scene-subject` picks, among several named people, the one a
verb follows (the question names Mon3tr and Kal'tsit, and character scenes needed exactly one). On E2B mid (two calls
that went to the running preview server by mistake) the answer moved from the decline to "Mon3tr reacts with a
pained howl when Kal'tsit cries because she is dying [2]" from Return to Mist (`main_11-15_beg#0005`): the
character scenes fire, but the reranker puts the chapter 11 line above the waking scene: of Mon3tr's 208 speaking
scenes the waking chunk is 13th by fused retrieval and 19th by reranked lines (TREVOR_SCOPED_DEBUG), outside the 6
excerpt slots. Not shipped as a default on that evidence; the defaults render prompts byte-identical to `43e30986` (stub). Next: a window that measures both
flags on the probe sets, and a lift for the waking scene itself (its voice is "???" until "My name is Mon3tr", so a
chunk-scoped label rename is the candidate; a story-scoped one is unsafe, "???" names several speakers per story).

### 2026-10-19: CPU-only throughput of E2B and the 12B (step 1 of the self-contained deployment)

**Tried.** On the development machine with `-ngl 0` and 8 threads: the 35 probe questions on E2B `mid` through a
plain llama-server (no Metal layers), then `llama-bench -p 4096 -n 128 -r 2` on the 12B Q4_0 and E2B Q4_K_M.
**Result.** E2B `mid`: 1,249 s for 35 questions; model time median 38 s, mean 39 s, range 26 to 54 s; 93 answers an
hour at full duty. 12B: 36.4 tokens/s prompt, 17.4 tokens/s generation. E2B: 92.7 and 48.0. The bench reports
`BLAS,MTL` as loaded backends: Accelerate BLAS serves the CPU path, and unified memory makes generation faster than a
VPS slice by an estimated 2 to 4 times, so these bound the VPS from above.
**Decision.** Recorded in `trevor-self-contained-deploy.md` section 5; the VPS numbers are taken on the VPS before
step 5 is built. Contract v1.1's 1,000 to 1,900 answers a day on one core is flagged for correction.

### 2026-10-19: The stage runner's first run with real models

**Tried.** `scripts/stages.py --once --duty 1` on the development machine over the live tree, with a scratch state
dir, after the day's update had left 4 units to do plus whatever the topic stages found.
**Result.** Five stages ran, each with its own server start and stop: real names 24.8 s (1 unit), dossiers 58.1 s
(3), topic classify 5.4 s, deep topics 171.2 s (2 units, the 12B), the deep judge 99.7 s (2, Qwen). Every plan
after that was 0, the pass ended, state `stopped`, exit 0, no llama-server left, 15 minutes wall. The deterministic
rebuild after a stage with progress ran (build-units for P4 and P4x). These are GPU timings on an M5 Pro; the VPS
runs the same stages on 3 CPU cores (section 5 of `trevor-memory-budget.md`), so they bound the box from below.
**Decision.** The runner is proven on this machine; the window and the VPS remain to be run for real.

## Entities and topics

Identities, names and aliases, topics and topic summaries, dossiers, canon evidence for the Integrated Strategies endings, deaths, places, organisations, operator data.

### 2026-09-29: Canon evidence for IS endings: contrastive pass

**Tried.** A contrastive pass (scripts/canon_refs.py) over 223 candidate passages from later stories, asking which ending, if any, each passage names.
**Result.** 115 none, 85 general, 23 name one ending. By distinct later story: IS3 Stella Caerula 2 (a 2023 module story and Dissociative Recombination, 2025); IS2 split between Grand Opening and Silent Chapter within Act or Die; IS5 The End? 1 (Unrealized Realities); IS6 split three ways within First of A Thousand Autumns; IS4 none. One pick credited IS4's Winterfall from an IS6 relic about Sami warriors.
**Decision.** Other runs' IS data no longer counts as evidence (`CANON_IS_EVIDENCE=1` keeps it).
**Follow-up.** A wider candidate search with a second judge (see the 2026-09-30 entries).

### 2026-09-29: Topic summaries, first generation

**Tried.** 46 topics (18 race, 17 concept, 11 nation) chosen by how often real questions name them; summaries generated and judged sentence by sentence by Qwen.
**Result.** 529/532 sentences supported (0.994).
**Caveat.** That is an upper bound: the Sami summary stated an Ursus soldier's line ("the Sami invaded Ursus territory") as fact, and the judge passed it because the passage contains it.
**Decision.** The prompt now names the speaker of claims made in dialogue (`TOPIC_ATTRIBUTE=0` = the first prompt, prompt hash verified identical); the summaries were regenerated on 2026-09-30 (see "Topic summaries with speaker attribution").

### 2026-09-29: Short topic summaries count as none

**Result.** The 2026-09-29 judge found summaries of 214 to 389 words, except Pythia's 38, which is the generator asking for passages.
**Decision.** A summary under 100 words is treated as no summary.
**Code.** trevor/src/tools/topics.rs, `Tools::topic` (short summary)

### 2026-09-30: Topic summaries with speaker attribution

**Tried.** All 46 summaries regenerated with the attribution sentence (25 s each), judged by Qwen.
**Result.** 535/539 sentences supported (0.993), 42 of 46 fully supported, against 529/532 and 43 before. Attribution phrases ("according to", "claims", "says", "states", "describes") across the 46 summaries: 17, was 9. The case that motivated it did not move: the Sami summary still states the Ursus conflict without a speaker, now reversed ("Ursus has attempted to seize Sami territory, leading to ... the Witch"), which neither passage says in that form, and the judge passed it again. Pythia is still the generator asking for passages (1/2; the tool refuses summaries under 100 words).
**Decision.** The first run is kept in artifacts/topics/v1/. The summaries stay labelled as generated in the answer context, below the story passages in authority.
**Follow-up.** A judge that checks who says a claim is owed before the summaries could be answered from verbatim.

### 2026-09-30: Canon evidence: wider candidate search and a Qwen second judge

**Tried.** The 223 candidates of 2026-09-29 were BM25's top 12 per ending over later dated stories. A new `wide` stage adds, per ending, two hybrid searches over P4 (the ending's summary; its name plus up to 10 distinctive proper names from its script, capitalized in at least 80% of corpus uses and found in 1 to 400 non-IS chunks), fused by RRF, kept to non-IS stories released after the run began, top 50. Gemma's contrastive pick classified each passage; Qwen3.5 9B then re-asked every one of Gemma's ending picks.
**Result.** 680 new passages, 903 in all (IS2 167, IS3 174, IS4 254, IS5 200, IS6 108). Gemma's pick took 1.58 s per passage (587 in 929.5 s after a 20-passage timing of 2.00 s each). Of the 680 new passages 525 were none, 122 general and 33 named an ending; with a verbatim quote and outside IS, 21 (the old 223 gave 17). Qwen re-asked all 56 of Gemma's ending picks. By distinct later story, agreed (Gemma only in brackets):
- IS2: Grand Opening 1, Act or Die (Silent Chapter 0 (1)).
- IS3: none agreed (Precious Days (2), Stella Caerula (4)).
- IS4: none.
- IS5: The End? 2, The Masses' Travels and Unrealized Realities (Twin Crowns (1)).
- IS6: Orderly Placation, Scroll of Scars, Out of Nothingness and No Regrets 1 each, all First of A Thousand Autumns (Black-White Abstrusity (1)).

The wider search did not move IS4 (0 picks with a found quote in 254 passages) and found no later story for IS3 that survives review.
**Decision.** `referencedBy` now holds agreed stories only, `gemmaOnly` the rest. A pick answer whose quote ends in a backslash no longer crashes the parse (1 in the first 93 wide picks).

### 2026-09-30: Canon evidence read by hand; a distinctive-name anchor filter

**Tried.** 26 distinct picks read by hand with their chunks; Qwen asked with the first question and with a stricter question (an outcome unique to the ending, not a shared character, image or flashback); Gemma asked with the stricter question.
**Result.** 9 continue the ending named and 16 do not (1 unclear). Qwen agreeing with Gemma's first question: 5 right of 19; Qwen with the stricter question: 7 of 22; Gemma itself with the stricter question: 5 of 14.
**Why.** The errors are systematic in two shapes. First, a sequel event full of a run's cast: every Lucian scene in Act or Die is credited to Grand Opening (8 picks, 2 right: Lucian kills Tragodia and takes the crown, the "passing of the torch" that Grand Opening sets up), and Wang's death scene to No Regrets. Second, shared imagery: star and sky passages in Lone Trail, Chapter 13 and Chapter 15 credited to Stella Caerula, and Estelle's module ("a failed article") to Precious Days.
**Decision.** Refuted: a stricter prompt as the fix; a 9B second opinion agrees with the 12B's mistakes. The second error shape is filtered: an agreed pick must name one of the ending's distinctive names in the passage, which removes all four Stella and Precious Days stories and Silent Chapter. Switches: `CANON_WIDE=0 CANON_AGREE=0` reproduces the 2026-09-29 `is_endings.json` byte for byte (verified); `CANON_WIDE=0` alone gives the old 223 candidates; `CANON_STRICT=0` uses the first question for Qwen; `CANON_ANCHOR=0` drops the name filter.
**Result after the filter.** Right story by story: Act or Die for Grand Opening, First of A Thousand Autumns for Scroll of Scars (Zuo Le's doll from the memorial hall, "it really is her"), Orderly Placation (the Placation reported to the Sui Regulator) and No Regrets ("Wang ascended the seat of Sui"), Unrealized Realities for The End? (Tin Man with his freed revenants). Out of Nothingness is credited by three wrong picks while the passage that supports it ("a new 'Sui' is born") was not agreed; The Masses' Travels ("Kazdel doesn't need to worry about energy anymore") is unclear. Pick-level precision of the final agreed set: 6 right of 17 (1 unclear, 10 wrong).
**Caveat.** IS4's The Fleetest of Light has no script in the game data, so its summary is Gemma's guess from the title; nothing is credited to it.

### 2026-09-30: Canon evidence, story-level stage: refuted

**Tried.** A new `story` stage asks once per (run, later story) which ending's outcome the story treats as having happened, not which it resembles, from up to 8 of its passages (the passages Gemma picked first, then the story's best-ranked wide candidates, shown in story order), with the same grammar (a letter or none) and a quote check against the passages shown. 94 pairs: every story with a Gemma ending pick outside IS (36) plus every story with 3 or more wide candidates for the run (80).
**Result.** Gemma took 4.99 s per story (3.60 s on the first 5) and credited 32. Only 14 of those had a found quote: 16 quotes were copied from the ending summary in the prompt and 3 were "The". Qwen took 3.31 s per story and credited 15, 12 with a found quote. Read by hand, a credited story with a found quote of 4 or more words is right 2 of 11 times for Gemma (1 unclear), 2 of 12 for Qwen (2 unclear), and 1 of 3 where both agree. The passage-level agreed picks were right 6 of 17. By (ending, event), the level the canon tool serves, the stage is right 2 of 8 for Gemma (1 unclear), 2 of 8 for Qwen (1 unclear) and 1 of 3 where both agree, against 5 of 7 for the passage scheme (1 unclear).
**Why.** The story stage repeats the shared-imagery error at a larger scale. Both models credit Stella Caerula to Lone Trail and Chapter 13, and each adds one more (Chapter 15 for Gemma, Path of Life for Qwen). Gemma adds the Dungeon Meshi collaboration for Long into an Abyss ("that hornbeast") and a Kjeragandr folk tale for Winterfall. It does not recover Highmore's module for Stella Caerula: Gemma picked Stella Caerula there but copied its quote from the summary, and Qwen picked Precious Days with the star-voyage line. Only Out of Nothingness for First of A Thousand Autumns is right in all three schemes (the story says "a new 'Sui' is born" and "That proxy killed Sui"). Adding context does not make these models more careful: at 9B to 12B they credit a whole story as readily as a single passage.
**Decision.** Serving is unchanged: `build` fills `referencedBy` from the passage scheme and records `evidenceScheme: "passage"` in the file. `CANON_EVIDENCE=story-gemma|story-qwen|story-agreed` serves a story scheme instead. The schemes not served are kept as `passageAgreed`, `storyGemma`, `storyQwen` and `storyAgreed`, with 2 quotes per story. The default file's `referencedBy`, `quotes` and `gemmaOnly` are identical to the previous build. `CANON_WIDE=0 CANON_AGREE=0` still reproduces the 2026-09-29 file byte for byte, with or without `CANON_EVIDENCE` (verified).

### 2026-09-30: Integrated Strategies numbering corrected (round 6, issue A)

**Tried.** Integrated Strategies numbers were one low everywhere. The game's IS #1 is Ceobe's Fungimist (CN only; the EN data has it only as the untranslated `roguelike_table.json`, not in `roguelike_topic_table`), so rogue_1 Phantom & Crimson Solitaire is IS #2 and rogue_5 Sui's Garden of Grotesqueries is IS #6.
**Result.** Corrected in unit titles (`scripts/sources.py`), P4 (1,220 IS chunks re-embedded, 21,697 reused), the canon tool ("IS1" now says Fungimist is not in Trevor's data) and the numbers in the design notes (HANDOFF 4c). "Which IS2 ending is canon?" now answers Phantom.
**Decision.** Shipped. `IS_OFFSET=0` restores the old unit-title file byte for byte; `--is-numbering-v1` restores the old canon-tool numbering.

### 2026-09-30: Short-name expansion (round 6, issue D)

**Tried.** "What did bibeak design for kal": the answer chunk ranked 8th; the model did not connect "kal" with Kal'tsit. A short word that starts exactly one operator or identity name and is rare in the corpus (1 to 49 chunks, at most a twentieth of the full name) gets the full name added.
**Result.** The answer now quotes "a frilly miniskirt for Kal'tsit". It expands 1 real-probe and 0 gold questions, and 3 of 1,228 real questions (one misfire, Spanish "que" to Quercus).
**Decision.** Shipped; `--no-name-expansion` turns it off.

### 2026-09-30: Canon verdicts from all the evidence

**Tried.** Requirement: number the runs as the game does (rogue_1 is IS2; IS1, Ceobe's Fungimist, is not in the data), say which stories treat an ending as canon, and list every story that bears on an ending with a quote, not only the agreed subset. A reference reading for IS3 (the only canon ending is the one Highmore's module supports, Stella Caerula) was treated as one known answer to check against, not a rule to encode. `is_endings.json` now carries, per run, `is`, an `evidence` list and a `verdict`. The evidence is every (story, ending, quote) any model linked with a found quote of 4 or more words, from all stages and both models: 112 items over 53 (story, ending) pairs (IS2 35 items, IS3 16, IS4 4, IS5 10, IS6 47). Each item records its models, its stages and a hand grade from `data/canon_grades.json`; of the 53 pairs, 10 are right, 36 wrong and 7 unclear, and the grades are never shown to a model. A `verdict` stage gives Gemma or Qwen the run's endings and every evidence passage (quote plus 250 characters each side, merged per chunk; 1,680 to 9,324 prompt tokens) and asks which ending the stories treat as canon, with a confidence and citations. It ran in 2 prompts × 2 models × reasoning off or on, which makes 20 verdicts per prompt. v1 answered in one step. v2 makes the model judge every passage first (holds or not, and why) and decide only from the passages that hold; it names things that happen within the later story itself (a weapon firing, a vision, a legend, a backstory) as not outcomes, and sets confidence by how many different stories hold. v2 runs with a 3,000-token reasoning budget.
**Result.**

| run | right by hand | v1 Gemma off | v1 Qwen off | v2 Gemma off | v2 Gemma on | v2 Qwen off | v2 Qwen on |
|---|---|---|---|---|---|---|---|
| IS2 | Grand Opening (Act or Die, one story) | Grand Opening | Grand Opening | none | Grand Opening | Grand Opening | Grand Opening |
| IS3 | Stella Caerula (Highmore's module) | Stella, imagery | Stella, imagery | Stella, imagery | Stella, imagery plus the module | Precious Days, from the module | Precious Days, from Estelle's |
| IS4 | none | Long into an Abyss | Fleetest of Light | none | none | none | Fleetest of Light |
| IS5 | The End? (weak) | The End? | The End? | The End? | The End? | The End? | undetermined |
| IS6 | Out of Nothingness or No Regrets (the proxy becomes the new Sui) | Out of Nothingness | Out of Nothingness | Out of Nothingness | Out of Nothingness | Orderly Placation | Out of Nothingness |

All 10 answers from v1 with reasoning off were "strong", including IS4's Long into an Abyss from a Dungeon Meshi line about a hornbeast. With reasoning on, v1 produced no answer in 7 of 10 runs: the thinking ran past 7,000 tokens. Best is v2 Gemma with reasoning on (511 s for 5 runs): 4 of 5 by hand (IS2, IS4, IS5 and IS6). IS3 reaches Stella Caerula without being told, but it cites Lone Trail, Chapter 13, Path of Life and Chapter 15 for star and sea imagery next to Highmore's module, so the right answer comes from mostly wrong evidence. Its confidence is "strong" everywhere except IS4, where the hand-graded evidence supports weak to moderate (one story per run). v2 Qwen with reasoning off (97 s) gets IS2, IS4 and IS5; it finds Highmore's module for IS3 but reads it as Precious Days, and answers Orderly Placation for IS6 from the one Placation line. Gemma with reasoning on and Qwen with reasoning off agree on IS2, IS4 and IS5 (3 of 5), and disagree on IS3 and IS6. Model time for all 40 verdicts was 3,222 s (54 min).
**Decision.** Refuted: reasoning on with no budget (7 of 10 unanswered). Refuted: v1's one-step verdict (overconfident, and it credits IS4 from imagery). Serving: `verdict` comes from `CANON_VERDICT` (default `gemma-on:v2`, from `artifacts/canon/verdicts.jsonl`, every attempt kept). `verdict.second` holds `CANON_VERDICT_SECOND` (default `qwen-off:v2`) and whether it agrees. `CANON_VERDICT=0` leaves `verdict` and `evidence` out, and every earlier field is identical to the previous build. `CANON_VERDICT_PROMPT=v1` asks the first prompt. `CANON_WIDE=0 CANON_AGREE=0` still reproduces the 2026-09-29 file byte for byte (verified).
**Caveat.** Operator files, voice lines, outfits and operator records have no release date in the corpus (1,925 archive, 1,307 voice, 474 outfit, 3,342 record chunks), so they could not yet be placed after a run's start and were not in the evidence (addressed in "Canon evidence from undated sources").

### 2026-09-30: Canon evidence from undated sources

**Tried.** Operator archives, voice lines, outfits and operator records have no release date, so every dated stage left them out. A new `undated` stage runs the same two searches per ending, kept to those sources and top 30 per ending: 690 rows, 495 distinct passages (221 archive, 382 record, 59 voice, 28 outfit). Discriminate (1.74 s each, 859 s) and the Qwen second pick (23 new, 40.5 s) ran as for the rest. Picks from undated sources are marked `released: null` and `undated: true`, and they go into the evidence and the verdict only, never into `referencedBy`.
**Result.** They add 28 evidence items over 17 (story, ending) pairs (140 items in all). By hand, 3 are right: Tragodia's operator file for Grand Opening (Lucian has become Tragodia); the Sublimation outfit for Stella Caerula ("the Seaborn would ascend from Terra, their destination the infinite sea of stars"); and Nymph's operator file for Twin Crowns (her hopes made imagination manifest through the Djall). 6 are unclear: Highmore's operator file (after surgery Mizuki asks Rhodes Island to take her in "to experience life amongst people", which is the Precious Days outcome and the strongest item against the reference reading of IS3), Wang's record Final Round for three IS6 endings, Christine's record and an Amiya outfit. 8 are wrong. With the undated items, the v2 verdicts are unchanged for Gemma with reasoning on (IS2 Grand Opening, IS3 Stella Caerula, IS4 none, IS5 The End?, IS6 Out of Nothingness). Two of Qwen's verdicts change to match Gemma's: IS3 Precious Days becomes Stella Caerula (citing Path of Life, the Sublimation outfit and Skadi's voice lines), and IS6 Orderly Placation becomes Out of Nothingness. Agreement goes from 3 of 5 to 5 of 5. By hand the default is still right on 4 of 5, with IS6 defensible.
**Decision.** Shipped. `CANON_UNDATED=0` restores the earlier evidence and picks the verdict attempts made without it.

### 2026-09-30: Canon verdict confidence capped by a rule over cited evidence

**Tried.** Confidence is now the lower of the model's and a rule over the evidence the verdict cites for its own ending: no cited story none, one weak, two moderate, three or more strong only across two or more events, else moderate.
**Result.** Before and after, dated evidence only: IS2 strong to moderate (8 story ids, 1 event), IS3 strong stays (6 ids, 5 events), IS4 none, IS5 strong stays (3 ids, 2 events), IS6 strong to moderate (13 ids, 1 event). With undated evidence the rule no longer caps anything: every run with an ending is strong, because the model cites operator files and records as extra events (IS2 3 events, IS3 8, IS5 2, IS6 3), and most of those citations are graded wrong or unclear.
**Caveat.** The rule counts what the verdict cites, not whether it holds, so it curbs "strong on one story" but not "strong on wrong stories".
**Decision.** Shipped. `CANON_CONF_RULE=0` writes the model's confidence alone; the verdict keeps `modelConfidence`, `ruleConfidence`, `citedStoryIds` and `citedEvents`. `CANON_WIDE=0 CANON_AGREE=0` still reproduces the 2026-09-29 file byte for byte (verified).

### 2026-09-30: The opinion path finds a named character's file through real names and identity links

**Why.** r034 "Margaret vs Degenbrecher: Who is Stronger?" lost for two general reasons. The opinion path's `named_characters` read only the names in each file's heading. Nearl's two operator files carry codenames ("Nearl", "Nearl the Radiant Knight"), so the question's "Margaret" matched only the dossier `profile_margaret`, and that dossier did not rank in the top 60 for the per-character query, so no Margaret file was kept at all. Second, a file the model gets under a codename does not tell it that the question's name is that character.
**Tried.** A general fix. A file's names now include its operator's real name from `real_names.jsonl` (and the first word of a real name of two or more words: "Margaret Nearl" gives "Margaret") and the names of its identity links. A named character always gets its file, its first chunk when the query ranks none of it in the top 60. There are at most two files per name the question uses and six in all. A file matched through another name is labelled "this is the file of Nearl the Radiant Knight, whom the question calls Margaret".
**Result.** r034 before: "I could not find any information regarding a character named Margaret in the provided story text." After: the passages hold `archive_char_148_nearl#0000` and two chunks of `archive_char_1014_nearl2`, and the answer compares "Margaret (Nearl)" with Degenbrecher point by point (Degenbrecher's "incomparable battle prowess" and her stand against Harold's army, Margaret's Arts and her duel with the Blood Knight). Judged: responds, 9/10 supported, but not handles (the judge wants the story's "not settled" stated as such).

### 2026-09-30: IS3 canon verdict: named characters over imagery (canon prompt v3)

**Why.** The canon ending of IS3 (Mizuki & Caerula Arbor) is Precious Days, Mizuki's quiet life with Highmore, not Stella Caerula; this reading was adopted as the target. The v2 verdict said Stella Caerula, strong, because 9 of its 13 IS3 evidence passages are star, sea or Seaborn imagery (Lone Trail, Chapter 13, Path of Life twice, Chapter 15, Nasti's file, Skadi's voice lines and outfit, Estelle's module) and name none of the run's characters. The model counted them. The one passage showing a named character after the run is Highmore's operator file, where Mizuki asks Rhodes Island to take Highmore in "to experience life amongst people", and v2 discounted it. Re-graded with this reading: Highmore's operator file for Precious Days goes from unclear to right. Highmore's module goes from right to unclear for Stella Caerula (the voyage "beyond where the stars shine" is a metaphor for setting out, and "a voyage is not a farewell"), and from wrong to unclear for Precious Days. The Sublimation outfit goes from right to unclear (the flavor text of an alternate Seaborn form).
**Tried.** Prompt v3, a general rule applied to all five runs. Each ending is listed with its named characters: the speakers of its script, plus its distinctive names that speak somewhere in the corpus and appear in at most 150 non-IS chunks, which drops Seaborn and Sui. Each passage is labelled with the run's characters it names. The prompt says an ending shows in the state its characters are left in, a passage naming none of them is atmosphere and weak at most, and passages are weighed, not counted. Unlabelled passages per run: IS2 0 of 20, IS3 9 of 13, IS4 5 of 5, IS5 2 of 9, IS6 14 of 34.
**Result.** Against the targets (IS2 Grand Opening, IS3 Precious Days, IS4 none, IS5 The End? weak, IS6 Out of Nothingness or No Regrets), with the undated evidence:

| attempt | IS2 | IS3 | IS4 | IS5 | IS6 | right |
|---|---|---|---|---|---|---|
| v2 Gemma on | Grand Opening | Stella Caerula | none | The End? | Out of Nothingness | 4 of 5 |
| v2 Qwen off | Grand Opening | Stella Caerula | none | The End? | Out of Nothingness | 4 of 5 |
| v3 Gemma off | none | Precious Days | none | The End? | Out of Nothingness (from Jie's life force, a wrong reason) | 4 of 5 |
| v3 Gemma on | Grand Opening | Precious Days | none | The End? | Out of Nothingness | 5 of 5 |
| v3 Qwen off | Grand Opening | Precious Days | none | undetermined | Orderly Placation | 3 of 5 |

v3 Gemma with reasoning on is the only attempt that gets IS3 without losing another run. Its IS3 verdict cites only Highmore's operator file: model confidence moderate, rule confidence weak (one story), written as weak. Gemma and the Qwen second verdict now agree on 3 of 5 (IS2, IS3, IS4). IS5 and IS6 disagree, and in both Qwen is wrong by hand. v3 still cites Ch'en's operator file (graded wrong) for IS6. Model time for the 15 v3 verdicts was 1,521 s.
**Decision.** v3 Gemma with reasoning on is the default. `CANON_VERDICT_PROMPT=v2` restores v2 and its verdicts. `CANON_WIDE=0 CANON_AGREE=0` still reproduces the 2026-09-29 file byte for byte (verified).
**Caveat.** IS6 overflowed the 16,384 context the first time (11,627 prompt tokens plus 4,757 thinking and answer tokens). Rerun alone at 24,576 (`CANON_VERDICT_RUNS=rogue_5`), it gave Out of Nothingness. Serving IS6 with Gemma and reasoning on needs that context.

### 2026-09-30: Integrated Strategies numbering follows the game

**Tried.** The data's ids number Integrated Strategies runs as rogue_N = IS N. The game counts Ceobe's Fungimist as IS #1 (which Trevor does not have), so rogue_1 Phantom & Crimson Solitaire is IS #2.
**Decision.** Requirement: number IS runs as the game does. Default since 2026-09-30 (`ToolOptions::is_offset`); `ask --is-numbering-v1` is the kill switch and restores rogue_N = IS N.
**Code.** `src/tools/mod.rs`, `ToolOptions::is_offset`; `src/bin/ask/cli.rs`, `Args::is_numbering_v1`

### 2026-09-30: Per-run canon verdicts

**Tried.** Canon questions are answered from per-run verdicts instead of strength labels from reference counts only. `CANON_RULE` covers canon and translation questions (4 to 5% of real questions, 0.20 responded): the game text does not rule on canon.
**Decision.** Per-run verdicts are the default since 2026-09-30; `--canon-v1` answers from reference counts only, as before.
**Code.** `src/tools/mod.rs`, `ToolOptions::canon_v1`; `src/bin/ask/cli.rs`, `Args::canon_v1`; `src/bin/ask/prompts.rs`, `CANON_RULE`

### 2026-09-30: Name expansion after a short name

**Tried.** The full name is added after a short name the question uses ("kal" -> "kal (Kal'tsit)"), for retrieval and the answer.
**Decision.** Default since 2026-09-30; `--no-name-expansion` restores the earlier behaviour.
**Code.** `src/bin/ask/cli.rs`, `Args::no_name_expansion`

### 2026-09-30: Per-run canon verdicts

**Decision.** The canon tool uses the per-run verdicts (`scripts/canon_refs.py`, 2026-09-30) when every chosen run has one; else the reference counts, as before.
**Code.** trevor/src/tools/canon.rs, `Tools::canon` (verdicts)

### 2026-10-01: `update.sh` rebuilds topics and the IS ending canon when their inputs change

**Tried.** Item 3 of the work list. A new step 4b runs after P4 is rebuilt, since both pipelines read P4.
- Topics use the key rule of `scripts/incr.py`: `topics.py plan` now keeps each topic's summary, input sha and prompt sha from the old file, so `gen` redoes only the topics whose passages or prompt changed (before, plan wrote the file without summaries and gen redid all 46; `TOPIC_PLAN_FRESH=1` restores that). Plan also leaves Trevor's own topic units out of the passages: P4 has held them since the 2026-09-30 06:48 rebuild, and a plan over them picks other topics' summaries as evidence (with them in, `TOPIC_SELF=1`, 41 of 46 topics would be redone; without them, 18). `judge` and `units` were already keyed; the step rebuilds P4 only when `units.jsonl` changed, and loads Qwen only when the judge plan is non-zero.
- The canon pipeline is keyed by id at every stage ((run, chunk) and (run, story)), so it gets a stamp instead. `canon_refs.py inputs` compares the endings key (the run table, the ending scripts, every prompt and setting), one sha per candidate chunk of P4 (22,583; Trevor's dossiers, event summaries and topics are never candidates and are left out) and the release-date and name inputs with `artifacts/canon/inputs.json`, and prints none, incremental (chunks changed, removed or new) or full (endings changed). An incremental run first drops the rows of changed or removed chunks from the 6 per-passage and 2 per-story files (`prune`); a full run moves every output aside into the update log. Then the chain runs on Gemma (summarize, judge, wide, undated, discriminate, story), Qwen (second, story, verdict qwen-off) and Gemma with reasoning on in a 24,576-token slot (verdict gemma-on, IS6 needs it), then build and `stamp`.

**Result.** The stamp was written once as a backfill that vouches for the current canon files: none of their 3,593 rows names a chunk missing from the current P4. `PLAN=1 bash scripts/update.sh` (56 s, no model) now lists both: canon "plan none (nothing changed)", and topics are NOT skipped: passages changed for 14 of 46 topics and gen would redo 18 ("input changed": the 14, plus 4 whose passage texts changed), because the IS units in P4 changed after the topics run (31 IS passages enter the new plans, 10 operator-file passages leave). That is a real staleness the old script could not see, not a defect of the key. The same dry run shows 50 of 200 dossiers with changed input, which predates this change. No model was run for this item.
**Decision.** Default in `update.sh`. `SKIP_P4_MODELS=1` is the script before.
**Caveat.** refs.jsonl and picks_wide.jsonl predate the 06:48 P4 rebuild, so a text change in those chunks between their runs and the stamp would not be seen (an assumption, as with incr.py's backfill).

### 2026-10-01: Topics rebuilt for the stale inputs

**Tried.** `topics.py plan`, then `gen` on Gemma redid the 18 stale summaries (28 s each, 8 min 26 s), and `judge` on Qwen redid 17 (the 18th kept its judge key; 16 s each).
**Result.** 203 of 205 sentences supported on the redone topics (199 of 201 before), 15 of 17 fully supported (Seaborn 9/10, Anasa 12/13). All 46 topics now 539 of 543 (0.993, 42 fully supported).

### 2026-10-01: Topics chosen from game data and story text instead of question counts

**Tried.** Topic candidates come from data now, not from question counts or the CONCEPTS hand list. `topics.py mine` (no model, 12 s) takes every race, nation, birthplace, team or group and enemy race in the game tables (101 distinct names: per source race 39, nation 19, birthplace 27, team level 0/1/2 19/15/11, enemy race 11) and every capitalized name the story and operator-file text uses mid-sentence in at least 14 chunks (TOPIC_MINE_MIN, set to the 14 passages a topic is given) of at least 3 story groups (TOPIC_MINE_GROUPS), minus operator and identity-link names and names whose lower-case form is commoner (625 names; 52 overlap the tables; 674 candidates). `topics.py classify` (Gemma, one grammar-bound call each, 0.85 s, 9 min 32 s) labels each candidate. Aliases merge by the model's base name, by a demonym or plural rule (Columbian, Leithanians, Laterans, Yanese join Columbia, Leithanien, Laterano, Yan) and by the team table's short codes (L.G.D., BSW). Gemma writes a summary per topic; Qwen judges sentence support against the topic's own passages.

**Result.**
- Classes: person 205, place 125, organization 124, title 68, concept 66, race 53, nation 18, other 9, event 6. The kept share is flat across frequency bands (14 to 20 chunks 0.514, 20 to 30 0.450, 30 to 50 0.490, 50 to 100 0.577, over 100 0.804), so the cutoff is not where precision fails and a lower one would add topics at about the same precision (not tried).
- Merging: 32 names merged into 17 of the old topics (kept as `names`, so their keys, passages, summaries and judgments are unchanged: the 46 old rows are byte-identical with `names` removed), and 308 new topics (organization 117, place 105, concept 45, race 25, nation 10, event 6; 58 from the tables, 250 mined).
- The mining alone finds 14 of the 19 CONCEPTS (missing Firearms on Terra, Mobile cities and landships, Precursors, Pythia, Sankta halos, all phrased as lower-case words or not named mid-sentence 14 times); those stay through the old rows.
- 10 new topics are thin before generation (fewer than 3 of their passages name them: Half-foot, Tall-man, Anura, Izganda, Kylin, Pilosa, Northern Continent, Originium Creation, Rhodes Island-Elite Operator, Yan-Sui) and are never generated or served.
- Gemma wrote the other 298 (2 h 18 min, 28 s each, 222 to 318 words, no refusal). Qwen judged them 3,476 of 3,502 sentences supported (0.993), 273 fully supported, the same rate as the 46 (539/543); worst 5: Central Londinium 10/12, State Council 9/10, Curia 10/11, Judicial Review Office 10/11, Hannya 10/11. No topic fell under the 0.8 thin rule, so 298 are served.

**Caveat.** The judge only checks support against the topic's own passages, and some summaries are about a weak candidate (Machina is two unrelated uses of the word, Heaven is a figure of speech), which no rule here catches.

**Decision.** Reached only through `--lore v2` (see below). `TOPIC_SOURCE=v1 topics.py plan` rewrites topics.jsonl byte-identical to the file before this work (verified with cmp), and gen and judge then plan 0. In update.sh, step 4b runs mine, classify (Gemma, only new candidates; PLAN=1 lists them), thin and units; a dry run now plans mine 0 new, classify 0, plan 0 passage changes, gen 0, judge 0, and 50 stale dossiers. The overview's world unit keeps the 46 topics (OVERVIEW_ALL_TOPICS=1 takes all; 298 more would not fit its input); its PLAN shows 1 to redo, which predates this (the 18 topics redone earlier the same day).

### 2026-10-01: Dossiers v2: every operator plus the most-speaking NPCs

**Tried.** `DOSSIER_SOURCE=v2` adds every operator without a dossier (261; 71 of them speak in no story, so their dossier is written from the archive alone) and the NPCs who speak at least 145 story lines, the floor of the 200 (75: Jordi, Isidro, Adele, Woodrow, ... Golding; a few generic labels pass, Mafioso is not among them but Old Craftsman and Pole-Carrier are). Speaker labels map to an operator by a title or name tail ("Inquisitor Irene" is Irene, "Red" is Projekt Red), not by identity links. Gemma writes; Qwen judges a sample.

**Result.** Gemma wrote 336 (1 h 31 min, 16.0 s each; the 50 stale v1 dossiers were left for the next update, `DOSSIER_ONLY_NEW=1`). Qwen on a sample of 40 new ones: 520 of 542 sentences supported (0.959), 26 fully (the v1 sample of 30: 349/391, 0.893); worst Elliot 11/15, Tin Man 12/15, Lyudmila 9/11, Cora 10/12, Isidro 11/13.

**Caveat.** Identity links join Estelle to Perfumer, Shirayuki to Midnight and Mountain to "Logistics Operator", which are wrong; they are recorded, not fixed (the dossier rules do not use them).

**Decision.** Served only under `--lore v2`. `dossiers.py sets` rebuilds the v1 sets byte-identical to before.

### 2026-10-02: Race-trait audit and regeneration of the race summaries (`summaryV2`)

**Tried.** Audit of the 38 served race summaries for body-feature claims. Fix (v2 only, `summaryV2`, the v1 rows untouched): every race topic is regenerated (`topics.py retrait`, Gemma, 38 in about 18 min) with a prompt rule (a trait only when a passage gives it to the race as a whole) over its 14 passages plus up to 4 passages with a line naming the race and a body word (TRAIT_EXTRA; TOPIC_TRAIT_RULE=0 turns retrait off), then `topics.py traitcheck` (Qwen) checks each body-feature sentence against the passages it cites and drops failures (TOPIC_TRAIT_CHECK=0 keeps the generated text).

**Result.**
- Before: 18 of the 38 state a body feature and 2 were wrong, Kuranta ("horns": an uncited sentence built on a line about another person's horns beside the word Kuranta) and Elafia ("horns and fangs": the Witch King's figure of speech about Leithanien). The support judge passed both because it checks words against the 14 passages, not who has the trait.
- After: Kuranta now says the passages leave its traits open and names the red-maned Nightzmora and "Wild Mane" (no horns claim); Elafia gets horns or antlers from two passages. The 38 race v2 summaries judge 461 of 469 sentences supported, 31 fully (all 344 v1 and new rows 4,015/4,045 as before).
- Not fixed: the regeneration introduced two new misattributions that traitcheck (52 sentences checked, 2 dropped, both other sentences) did not catch, Durin "thick tails" (Inam and Tomimi's line about Tomimi's tail) and Savra "a horn" (a Savra girl about someone else's horn).

**Decision.** Shipped as `summaryV2` for `--lore v2`. `topics_v1` now keeps only the fields a v1 topic row had (the audit added summaryV2, passagesV2 and keys to old rows).

**Follow-up.** A check that reads who is speaking is owed, to fix the Durin and Savra trait sentences.

### 2026-10-02: Topic summaries regenerated by a trait audit and a race-trait rule

**Tried.** With `ask --lore v2`, topic summaries use `summaryV2` where a trait audit regenerated one (2026-10-02). With `topics_v2_text`, `summaryV2r` is used where the race-trait rule (`topics.py traitrule`, 2026-10-03, no model) wrote one.
**Code.** `src/tools/mod.rs`, `Tools::topics_v2_text`, `Tools::topics_race_rule`

### 2026-10-02: Race data from the operator files

**Tried.** Game data gives the race of each operator.
**Result.** "Do the Nearls have wings?" was declined by both lores, though the files list every Nearl as Kuranta and the Kuranta are horse-like.
**Decision.** Topics carry the race data. Names match as written (4 letters or more), folded from 6.
**Code.** trevor/src/tools/topics.rs, `Tools::game_data`

### 2026-10-02: Dossier matching by the character's own name, as written

**Tried.** Dossiers matched by identity-link names and by folded text.
**Result.** Identity-link names joined Estelle to Perfumer. Folded matching sent `g0120`'s "on the mountain" to Mountain's dossier, and "Lin Qingyan" matched Lin (`g0120`).
**Decision.** A dossier matches by its own name only, never by identity-link names, and as written (case and all, whole words: "Red", "Grace" and "Mountain" are words). A name of two or more words also matches folded ("innkeeper zheng"). A match must not be the start of a longer name.
**Code.** trevor/src/tools/topics.rs, `Tools::new_dossier`

### 2026-10-03: Speaker-aware trait check for race summaries (`topics.py traitcheck2`): refuted

**Tried.** `topics.py traitcheck2` (Qwen, new) judges each body-feature sentence of the 38 race summaries (summaryV2) against only the lines of its cited passages that name the feature, each after the line before it, with a prompt that asks who has the feature and says a person of this race talking about someone else's tail or horn does not give the race that feature. It writes `summaryV2s` and `traitDroppedS` and leaves summaryV2 alone; `ask --speaker-trait-check` (opt-in) would serve summaryV2s.
**Result.** Of 50 body-feature sentences it dropped 3, and neither target. Durin "They are characterized by having thick tails" (Inam about Tomimi's tail) and Savra "including a horn" (a Savra girl about Philae's horn) were both KEPT. The drops were Manticore "a height of 155cm and a tail length of 140cm" (one person's measurements, a fair drop), a Lupo sentence about open questions, and Perro "characterized by physical traits such as ears and tails", which is very likely true. `topics.jsonl` is byte-identical to before apart from the three new fields on the 38 race rows (checked by comparing every row with them removed).
**Decision.** A line-level judge on Qwen 3.5 9B does not separate speaker from subject either. The flag stays opt-in and should not be used; the two misattributions remain in summaryV2 (served under the default `--lore v2` for race topic routes and the game-data passage).
**Follow-up.** What would settle it: a rule that needs no judgment, where a trait counts for a race only when a cited line's speaker or narrated subject is a character whose race in `operator_attributes.jsonl` is that race, which Tomimi (Archosauria in `operator_attributes.jsonl`) and Philae (Sarkaz) would fail. Not built.

### 2026-10-03: Codename and real-name links from game text (`entities.py extract2`); Bluishsilver is Mabel, stated

**Why.** Iris's file says it outright ("Mabel, the owner of the radio, was, following confirmation, the deceased Operator Bluishsilver of the Rosmontis Squad", `archive_char_338_iris#0001`), and her voice line says "You knew her as Bluishsilver", but neither line has a cue of `entities.py` (no "real name", "codename", "also known as"), and voice lines were never in its corpus (P3a).
**Tried.** `entities.py extract2` runs the same Gemma extraction (same prompt and grammar, same verification that the quote and both names are in the lines) over P4's stories and game-text units (voice, module, IS, enemy, outfit, item; never Trevor's dossiers, summaries or topics), on lines that match the old cues or the new ones (CUE2: "knew her as", "known to us as", "the late/deceased/former/fallen Operator", "her/his name was", "true identity", "is actually", "used the name", ...), and only on lines `pairs.jsonl` lacks.
**Result.** 730 lines (0.9 s each), 61 verified links, in `pairs.v2.jsonl`. `ENTITY_V2=1 entities.py build` reads both files and writes `identities.v2.json`: 183 identities (158 before), with Bluishsilver ~ Mabel stated by Iris's file. Links that cannot be stated are kept apart as labelled inferences, never merged: 58 links whose quote does not state them (dropped by the build before, now listed), and 38 full names whose first word is the first name of exactly one identity and whose words the text never writes in lower case (so "Ch'en Estate", "Texas Family" and "Kal'tsit Hello" are out); Mabel Grimm ~ Mabel (Bluishsilver) is one, from the interlude where Iris asks for "Mabel Grimm" (`act15d5_level_act15d5_st03`). Without `ENTITY_V2` the build writes `identities.json` exactly as before: byte-identical under the same PYTHONHASHSEED (a label tie, Kinbee Harada against Kouhei Mifune at 0 speaking chunks each, follows Python's string hashing: seeds 1, 3 and 5 reproduce the file and 0, 2 and 4 flip that one label; this predates the change).
**Decision.** Nothing reads `identities.v2.json` yet but `crossref.py` with `CROSSREF_IDENTITIES=v2` (no new boss row) and `entities.py who` with `ENTITY_V2=1`, so no answer changed.
**Caveat.** `ian24` ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") is still declined by the default although Iris's file is passage [1]: the question names neither Mabel nor Bluishsilver, so the link cannot help retrieval, and the answer needs the step from "the castle's guests" to the Mabel she searches for, which Gemma does not take. Not found either way: Theresa ~ Civilight Eterna (her file calls her "Miss Theresa" only through Amiya's words; `real_names.jsonl` rejected it as "none stated").

### 2026-10-03: Ed and the Celestial Fulcrum are not in the local data

**Tried.** Searched all local text for the Ed of `ian18` ("Who is Ed?") and for "Celestial Fulcrum", including, with `rg`, the two largest old `activity_table` snapshots under `text/anon` (2.3 GB and 1.6 GB), which the coverage audit skipped.
**Result.** The only "Ed" in all local text is "Herr Ed", a critic at Dame Strollo's gathering (`gamedata/story/obt/memory/story_pianst_2_1`, two lines, both already in the corpus as `story_pianst_set_2_story_1#0008`). "Celestial Fulcrum" occurs in three places, none naming anyone: Civilight Eterna's file (Log of visit 4, "active Celestial Fulcrum signals were only transmitted from the planet's surface on one occasion"), act33side st02 ("Search: Celestial Fulcrum... No signal.") and IS endbook rogue_4_2_4 (the King of Sarkaz: "this place will be connected to the Celestial Fulcrum", the work of an unnamed "they"). "Edward" (29 hits) and "Edmund" (1) are other people. The two snapshots hold no "Celestial Fulcrum" and no "Ed".
**Decision.** The ARG's Ed is not in Trevor's local data, and `ian18`'s default answer ("could not find who Ed is; Herr Ed is mentioned as an esteemed critic invited by Dame Strollo") is the right one; `--game-text` does not touch it (it is routed to story, P3b).

### 2026-10-03: Race-trait rule: body-feature sentences in race summaries checked against game data (item 11)

**Tried.** `topics.py traitrule` (new, no model) keeps a body-feature sentence of a race's summaryV2 only when a cited line that names the feature gives it to this race. Per line, the speaker (the "Name:" label) and the subject count by their race in `operator_attributes.jsonl`. The subject is: first person ("my tail") the speaker; second person ("your tail") the previous line's speaker; otherwise the operators the line names as written, or the file's operator for an archive line without a speaker. Narration that names the race as the feature's owner ("the Savra's tail", "Durin tails", "the horns of a Sarkaz") supports it. A feature is refuted when its lines give known races, none of them this one, and no line names the race as owner; a sentence with a refuted feature is dropped. Operator names never match a body word ("Horn" is a Lupo operator; the first pass dropped Elafia horns through her).

**Result.** Of 50 body-feature sentences in 38 race summaries it drops 13 in 9 topics:
- the two it was built for: Durin "They are characterized by having thick tails" (the lines are Inam's "your thick tail" to Tomimi and Tomimi's "my tail"; Tomimi is Archosauria) and Savra "including a horn" (a Savra girl's "your horn" to Philae, Sarkaz);
- Anasa "long horns" (Sarkaz), Kuranta horns (Forte), Feline horns (Lupo), Forte tails (Feline), Savra "wrist scales" (Vulpo), Tiacauh "thick tails" (the same Tomimi scene);
- Archosauria hunting "fins" (`BODY_RE` takes the prey for a body part: a true sentence lost);
- four Elafia sentences about horns and antlers whose cited lines are about Beeswax (Caprinae), Shining (Sarkaz, "Your horns are so beautiful! Are you an Elafia?") and other non-Elafia: true of the race, but not shown by the passages cited, so they go.

`topics.jsonl` gains `summaryV2r` and `traitDroppedR` on the race rows and is otherwise identical (checked row by row). Measured on every question whose route carries a changed race topic or a game-data passage (probe set 1, held-out 1, real 2, gold 4): only r027 ("Wait, how old is Pallas?") changes, its estimate now "about 20 to 30" where the base says "about 18 to 25" (responds in both, faithful 2/3 both); `ian13`, h002 ("Do the Nearls have wings?") and the 4 gold questions are byte-identical.

**Decision.** 0 losses, so it is the default: `ask` serves `summaryV2r` for race topics and in the game-data passage since 2026-10-03. `--no-race-trait-rule` serves `summaryV2` as before.

**Caveat.** The trade: Elafia's horn sentences leave the summary served to an Elafia trait question until a summary is regenerated from passages that show Elafia antlers (not measured: no question in the five sets asks it).

### 2026-10-03: Identity notes from identities.v2.json (leftover a)

**Tried.** `ask --identity-notes` adds, for a question asking who, which, whose, a real name, a codename or an identity, a note after the passages for each stated link of `identities.v2.json` whose source story is among the passages (at most 4, relation "title" left out: "King of Sarkaz" would put Theresa's lines on Amiya), and the inferred links that share a name with one of them, labelled as inferences. Measured on the 22 questions the prefilter found.

**Result.** `ian24` gets "Bluishsilver and Mabel are the same person (former name, stated: "Mabel, the owner of the radio, was, following confirmation, the deceased Operator Bluishsilver of the Rosmontis Squad.") / Mabel may be Mabel Grimm (an inference from the names, not stated)" and still answers "The story text does not specify which Rhodes Island Operator (non-playable unit) visited Iris's castle of dreams": Gemma does not take the step from the castle's guests to Mabel. Real responds 3 to 3 of 3 changed, gold correct 7 to 7 of 8, trivia 0 changed.

**Caveat.** The prefilter undercounts (the notes fired in `ian1`, `ian3`, `ian7` and `ian18`, outside it, changing wording only), and some extracted links are noise ("Operator and Ron are the same person", from "Code Name: Operator Ron").

**Decision.** No gain and an unmeasured reach, so opt-in.

### 2026-10-03: Deepcolor and the Church of the Deep: the text says no (item 1)

**Tried.** Searched the data for evidence that Deepcolor belongs to the Church of the Deep.

**Result.** "Deepcolor" occurs only in her own content (handbook `char_110_deepcl`, voice, module, outfits, records `story_deepcl_1_1` and `_2_1`); no act18d3 file and no Church of the Deep text names her, and no line has her kill a Bishop. Her file calls her origin "shrouded in mystery" and explains the "Deep" of her codename as "boundless water" (`archive_char_110_deepcl#0000`, `#0001`); the Bishop of Under Tides (`act18d3_level_act18d3_09_end`) dedicates his life to evolution, not art. The default answer (unchanged): "The passages do not mention whether Deepcolor is part of the Church of the Deep. The story text identifies Deepcolor as a special ops Operator for Rhodes Island [6], while the Church of the Deep is associated with a bishop ...", which follows the text. Found on the way: of the 367 operator records under `obt/memory`, only `story_sqrrel_2_1` is not in the corpus.

**Decision.** The default already follows the text; no change.

**Caveat.** Not tested: the question with the user's claim written into it.

### 2026-10-03: Identity links resolved inside the passage labels, with a proper-name filter (night 8, item 2)

**Tried.** `--identity-labels` adds to the label of every passage whose story is the source of a stated identity link (not a title) "this passage states that A and B are the same person, B being the <relation> of A ("<quote>")", at most 4 links, instead of a separate note. A data rule keeps only links whose two names are proper names by the corpus: the lowercase form occurs as a whole word under 1% as often as the name ("Operator" 1,168 lowercase against 6,260, "Doctor" 676 against 4,317 and "Guard" 453 against 1,243 are dropped; "Iris" 3 against 363, "Bluishsilver" 0 against 12, "Mabel" 0 against 7 and "Ebenholz" 0 against 1,115 pass). The first candidate rule (both names speakers or in the attributes table) was measured first and refuted: Bluishsilver and Mabel are neither (no entry in `appearances.json`, not in `operator_attributes`), so it would have dropped the very link it is for. "Ron" passes the corpus rule (3 against 0) but its only links are to "Operator", which does not.

**Result.** The labels fire on 19 of the 54 identity questions by the plain-retrieval prefilter (`--detect-identity` now prints labels-p3b/labels-p4). Changed answers: real responds 3 to 3 of 3, gold correct 6 to 6 of 6 (unanswerable declined 3 to 3), trivia 1 to 1, faithfulness up (real 15/17 to 19/20, gold 5/6 to 6/6); 0 judged losses. But `ian24` ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?"), the question it was built for, goes from "The story text does not specify which non-playable Rhodes Island Operator visited Iris's castle of dreams" to "The Rhodes Island Operator who visited Iris's castle of dreams is Rosmontis [1]", with all five of Iris's file passages labelled "Bluishsilver and Mabel are the same person": Gemma reads the file's courier for the visitor. `ian1`, `ian3` and `ian22` change wording only.

**Decision.** A confident wrong name is worse than the decline, so it stays opt-in.

**Follow-up.** The two-step inference (the visitor in the interlude is Mabel Grimm; Mabel is Bluishsilver) needs the interlude passage among the passages, which neither arm retrieves.

### 2026-10-03: A place's nation derived from data (night 8, item 3)

**Tried.** `scripts/place_nation.py` (no model, 33 s) maps a place topic to a nation from two sources in order: the birthplace table when the most common non-Rhodes Island nation of the operators born there outnumbers Rhodes Island (Siesta: Columbia, 2 operators; Kazdel falls through, 2 Columbia against 11 Rhodes Island), else the nation topic that the story chunks naming the place name most, scored count x log lift (at least 3 shared chunks, lift above 1). The candidates are the nation topics plus any served topic whose name is a nation of the attributes table (Ursus is served as a race; without it Chernobog went to Yan-Lungmen, 63 chunks). `ask --place-nation` reads it; `--place-nation-summary` is the night 7 summary count and reproduces the night 7 package answers (answer and passages) on 8 of 8 (r029, r102, g0034, g0037, h004, h017, h033, `ian30`).

**Result.** 89 of 103 places are mapped. Chernobog is Ursus (98 of its 262 chunks, lift 8.5), Dossoles Bolívar (20 of 96, lift 29.7), Londinium Victoria, Yumen Yan, Zwillingstürme and Wolumonde Leithanien. Questionable: Kazdel to Nachzehrer (28 of 481, lift 6.1; Kazdel is the Sarkaz homeland and has no nation topic) and Garden of Grotesqueries to Yan (5 of 25).

Of the 56 questions the named topic fires on, 10 name a place, and the mapping differs from the night 7 one on 4 (Sarcophagus Yan-Lungmen to Ursus, Chernobog Yan to Ursus, Garden of Grotesqueries none to Yan, Kazdel none to Nachzehrer). New against old package: real responds 1 to 1 (r029; r102 unchanged), gold correct 2 to 2 (g0034, g0037), faithfulness 5/5 to 4/4. The held-out losses are not place questions (h004 Gendarmerie, h017 Rhodes Island, h033 Liberi: organization, organization and race topics), so the mapping cannot touch them. Re-measured in a fresh session, the package loses all three again (responds 3 to 0, faithfulness 5/6 to 5/6), now twice by the same judge, so they are losses of `--named-topic`/`--topic-events`. `ian30` under the package is unchanged (Bolívar both ways).

**Decision.** Chernobog now maps to Ursus, but the package still loses the 3 held-out answers, so it stays opt-in.

### 2026-10-03: Deep topic entries by map-reduce (`--deep-topics`): `ian28` is deeper, but the package loses 2 answers

**Tried.** `scripts/topic_deep.py` (new) gathers a served topic's evidence broadly from `artifacts/p4x`:
- the 10 story groups with the most chunks naming it (6 chunks each, the ones naming it most, in story order, with the group's P2 summary as uncitable orientation);
- the operator files naming it (operators of that race, nation or birthplace in `operator_attributes` first, 8 chunks);
- the other game text naming it (game-text units, IS, modules, items, enemies, outfits, 8 chunks);
- up to 6 dossiers of characters of that race, nation or birthplace;
- the storyline overviews that name it.

The map step writes cited notes per evidence set on Gemma (speakers kept, a race trait only when given to the race as a whole, or NOTHING). The reduce step writes a sectioned entry (What it is, Traits, History, Society and politics, Notable people, Key events, Open questions) cited by note, and Qwen judges each sentence against the notes it cites. `served` drops the unsupported sentences (`DEEP_DROP=0` keeps them). Cache keys: each map input with its prompt, each entry by its map keys and prompt, each judgment by entry and notes, so `update.sh` (step 4b, `DEEP_TOP=40` most asked) redoes only changed ones; `PLAN=1` lists them. `ask --deep-topics` serves the entry for the topic tool, labelled "Trevor's topic entry (generated from the story passages, operator files and records that name it; prefer the story passages)", followed by retrieval as before. Measured on the 14 topic-routed questions, judged by Qwen.

**Result.** 14 topics were built in 77 min (Sarkaz and Sami first, then the most asked: Terra, Oripathy and the Infected, Sankta, Originium, Feranmut, Firearms on Terra, Beast Lords, Sankta halos, Seaborn, Arts, Rhodes Island, Liberi), 82 to 84 chunks each against the summary's 14 passages, 628 to 883 words. The judge supported 344 of 519 sentences (0.663; Liberi 7 of 36, whose entry gave Liberi "thick tails" and "ethereal hands", so 146 words are served); served entries are 146 to 660 words. On the 14 topic-routed questions:
- 3 identical;
- held-out responds 3 to 2 of 4 changed (lost h033, Liberi feathers: the base mentioned Liberi tribes, the arm drifted to a Sami's gifts);
- real responds 6 to 5 of 6 (lost r079, black halos: the base cited halos and wings of light, the arm "black horns" of Mostima);
- faithfulness down (held-out 15/17 to 11/18, real 25/26 to 16/18).

`ian28` ("Who are the sarkaz?") goes from Kazdel, horns and mercenaries to their suffering and exile, being called devils, racial memories, Oripathy, witchcraft linked to Arts, Kazdel as "Sorrow's Ground", the King and the Royal Court of the oldest bloodlines; it keeps one error the judge passed (the Sankta named as a group within the Sarkaz). `ian29` ("What is sami guarding?") is routed to the story, not the topic tool, so the Sami entry never reaches it, and the entry itself says little about guarding (one sentence on "protecting the northern front").

After the measurements, 7 more entries were built and judged (Draco, Teekaz, Anasa, Columbia, Laterano, Leithanien, Mobile cities and landships), so 21 topics have one: 500 of 761 sentences supported (0.657), served entries 146 to 690 words. The measured answers predate them.

**Decision.** Two losses, so `--deep-topics` stays opt-in.

**Why.** The judge checks the entry against the notes, not the chunks, so a note's misreading passes.

**Follow-up.** What would fix it is not more evidence but a reduce that keeps per-claim provenance.

### 2026-10-03: Identity expansion through linked names (`--identity-expand`): `ian24` is still not answered

**Tried.** `ask --identity-expand` (opt-in): for a who/which question, each stated identity link whose source passage is among the passages (as `--identity-labels` picks them: both names proper by the corpus) and names a capitalized, proper word of the question (Iris) brings the chunks where a linked name and that word appear together, ranked by the smaller of the two counts, at most 2, placed right after the link's source passage. For `ian24` that is the interlude where Iris looks for Mabel Grimm (`act15d5_level_act15d5_st03#0006`, Mabel 3, Iris 10), never retrieved before (hybrid rank 25). It fires on 11 of 447 by the plain top 8 (`--detect-identity` prints `expand-p3b/p4`).

**Result.** Version 1:
- Alone: real responds 2 to 2 of 2 changed, gold correct 6 to 6 of 6 (unanswerable declined 3 to 3), 0 judged losses, but `ian24` goes from a decline to "Sakiko Togawa" (the act45side collaboration passages), a confident wrong name.
- With `--identity-labels`: real 2 to 2, gold 7 to 7 (unanswerable 4 to 4), faithfulness up (real 11/12 to 14/14, gold 5/6 to 6/6), and `ian24` declines again ("does not identify a specific non-playable Operator"), with the interlude and Iris's file labelled "Bluishsilver and Mabel are the same person" both in its prompt: Gemma does not take the two steps (the visitor is Mabel; Mabel is Bluishsilver).

Version 2 (same method) writes the resolved link into the added passage's own label ("this passage names Mabel, who is the same person as Bluishsilver (Mabel being the former name of Bluishsilver, as `archive_char_338_iris` states)"), so the model need not join two passages:
- Alone: real 2 to 2, gold 6 to 6 (unanswerable 3 to 3), faithfulness real 11/12 to 12/12, and `ian24` still answers Sakiko Togawa (citing the act45side collaboration passages, which the answer reads as a visit and which rank above the interlude).
- With `--identity-labels`: real 2 to 2, gold 7 to 7, faithfulness real 11/12 to 13/13, `ian24` declines.

**Decision.** No answer gains in either version, so `--identity-expand` stays opt-in (the shipped code is the second version).

**Why.** What `ian24` lacks now is not the passage or the link but the question's constraint: "Rhodes Island Operator (not playable unit)" excludes Sakiko, a guest of another world, and Gemma does not apply it.

**Follow-up.** A test that a named answer is a Rhodes Island operator by the data (`operator_attributes`, identity links) before it is given is the next step.

### 2026-10-03: Cross-references, appearances and identity links from game data

**Tried.** Three game-data tables were built on 2026-10-03: cast-wide cross-references (`scripts/crossref.py`, table name to its JSON); per-character appearances (`scripts/appearances.py`: the story groups where each character speaks and where they are only mentioned, `{"groups": {id: {name, kind, episode, order}}, "characters": {name: {forms, speaks, mentioned, namedIn}}}`, read by the `appearances` tool); and identity links ((name, other name, relation, quote, where, stated)), read by the answer category check. On 2026-10-05 the check gained `AnswerConstraint`: the category a who/which question asks its answer to belong to (`Tools::answer_constraint`).
**Code.** `src/tools/mod.rs`, `Tools::crossref`, `Tools::appearances`, `Tools::identity_links`, `AnswerConstraint`

### 2026-10-03: One-operator match for the own-file fallback

**Tried.** For the own-file fallback, a name counts as one operator's when it is part of that operator's name and is no operator's whole name ("Mutsumi" is Mutsumi Wakaba).
**Code.** `src/bin/ask/people.rs`, `one_operator_wide`

### 2026-10-04: Death confirmation from the game text instead of the wiki

**Tried.** A death is confirmed when it is reported in 2 or more story groups and the character speaks in no story released after the first report.

**Result.** Playable operators confirmed 0 of 17 both ways (agrees with the wiki on 17 of 17); the 532 events are unchanged; named characters precision 8/26 and recall 8/44 against `wiki_deceased`.

**Decision.** Shipped as the default. `DEATH_CONFIRM=wiki` reproduces the old `deaths.jsonl` byte for byte.

### 2026-10-04: Confirmed operator deaths

**Decision.** The deaths table lists playable operators whose death the game text confirms (`scripts/deaths.py build`, 2026-10-04: a death reported in 2 or more places in the text).
**Code.** trevor/src/tools/deaths.rs, `Tools::confirmed_deaths`

### 2026-10-05: Deep topic entries

**Tried.** Deep topic entries (`scripts/topic_deep.py`, 2026-10-03, night 9): topic name to the judged entry's served text. The topic tool serves deep entries for a question under lore v2 only.
**Decision.** Default since 2026-10-05; `--no-deep-topics` is the kill switch.
**Code.** `src/tools/mod.rs`, `Tools::deep_topics`; `src/bin/ask/cli.rs`, `deep_for`

### 2026-10-16: Blacksteel (`ian34`): an organization linked as a person's identity, folded into topic aliases, plus group evidence

**Why.** "Evaluate the political influence of Karlan Trade Group and Blacksteel" answered that Blacksteel is not mentioned. The prompt dump showed all 15 passages were Karlan Trade, Silverash and Kjerag; the 33k-character prompt held "Blacksteel" once, in the question. The cause is in the data:
- `artifacts/entities/identities.json` links "Blacksteel" to "Blacksteel Worldwide", an organization, as if it were a person.
- `topics.py people_names` therefore dropped "Blacksteel" (150 P3b chunks, mid-sentence in 111 chunks of 22 story groups) from mining.
- So the Blacksteel Worldwide topic kept only "Blacksteel Worldwide" (13 chunks) and "BSW" as forms.

Of the 158 identity groups, only that one has a group topic (organization, nation, place) as its label. Six others carry a race or concept word among a person's names (Lucilla: Ægir, Tessai: Oni, Gordon Tannen: Savra, Kouhei Mifune: Phidia, Sui: Feranmut, Seaborn); that separate extraction defect was left alone here.
**Tried.** Two defaults:
1. `Tools::fold_group_identities`: such a group's names become the topic's aliases, and the group leaves the person lists (`TREVOR_TOPIC_IDENTITIES=0` restores the old lists).
2. Group evidence: an opinion question naming two or more group topics (`Tools::named_group_topics`) gets each group's best 3 chunks naming it, from the topic name plus the evidence query, ahead of the candidates (`--no-group-evidence`).
**Result.** `ian34` now compares both companies with citations on each side: Karlan Trade as Kjerag's only window to the outside world, Blacksteel Worldwide as a Columbian private security contractor. The shared measurement is in the 2026-10-16 measurement entry below.
**Decision.** Both shipped as defaults; `TREVOR_TOPIC_IDENTITIES=0` and `--no-group-evidence` are the kill switches.

### 2026-10-16: Identity links that name a group, dropped at the source in `entities.py build`

**Tried.** `entities.py build` now drops a link when either name is a group word (`group_words`):
- a race, nation or birthplace of operator_attributes;
- a handbook team (organization or nation) or enemy race of the game tables;
- any served topic name or alias.

`ENTITY_GROUP_WORDS=0` keeps every link. `scripts/common.py` reads the game data from `TREVOR_GAMEDATA` when set, since the myrtle-trevor worktree has no `assets/output` of its own.
**Result.** With `ENTITY_GROUP_WORDS=0` the build reproduces the old `identities.json` byte for byte under `PYTHONHASHSEED=1`; seeds 0 and 2 differ only by the documented label tie, which is pre-existing. With the rule on, 11 links are dropped and 158 identities become 153:
- Removed: Blacksteel Worldwide (organization ~ its short name), Lucilla ~ Ægir, Gordon Tannen ~ Savra, Seaborn ~ Mysterious Seaborn, and Sui ~ Feranmut ~ proxy.
- Kouhei Mifune loses Phidia and keeps his real former name Kinbee Harada.
- Tessai loses Oni and also "Madame Oni". The only quote for "Madame Oni" ("the woman known as 'Madame Oni' silently watches on") never names Tessai; the race word was the bridge, so that link was unstated.

On all 1,017 eval questions (gold v2, real probe, held-out, trivia, the probe set, Reddit, Discord), `ask --detect-only` gives 0 differing lines between the old and new files.
**Decision.** Shipped as the default; `ENTITY_GROUP_WORDS=0` is the kill switch and reproduces the old file.
**Follow-up.** The next topic mining (`topics.py mine`) will see "Blacksteel" as a candidate, since it is no longer a person's name.

### 2026-10-16: Organization short names from a company-suffix rule (`Tools::org_short_names`)

**Why.** The Rust fold `fold_group_identities` read the bad Blacksteel link, which the source fix above removed, so Blacksteel's short name needed another origin.
**Tried.** `Tools::org_short_names`: an organization topic named with a company suffix (Worldwide, Company, Co., Ltd, Inc, Corporation, Corp, LLC, Holdings, Enterprises, Industries; not Group) also answers to its head.
**Result.** Of the 354 topics, that adds exactly "Blacksteel". Karlan Trade and Raythean already carry their heads. With the rule off, `ian34` names only Karlan Trade, as before.
**Decision.** Shipped as the default; `TREVOR_ORG_SHORT_NAMES=0` turns it off.

### 2026-10-16: Identity links that name an organization fold into topic aliases

**Tried.** An identity group whose label is the name or alias of an organization, nation or place topic gives its names to that topic as aliases and leaves the person lists. Measured on `ian34`.
**Result.** `entities.py` linked "Blacksteel" to "Blacksteel Worldwide". As a person name, "Blacksteel" (150 P3b chunks, mid-sentence in 111 chunks of 22 story groups) was never mined as a topic form, so "Karlan Trade Group and Blacksteel" named one organization. Of the 158 groups only that one has a group topic as its label.
**Decision.** Default since 2026-10-16; `TREVOR_TOPIC_IDENTITIES=0` keeps the old lists.
**Code.** `src/tools/mod.rs`, `Tools::fold_group_identities`

### 2026-10-16: Organization short names and the group topics a question names (`ian34`)

**Tried.** `ian34`, "Evaluate the political influence of Karlan Trade Group and Blacksteel", needed the organization topics a question names. An organization topic named with a company suffix gets its short name as an alias when the head is 4 characters or more and no topic already has it as a form: "Blacksteel Worldwide" is also "Blacksteel", the name the story uses (150 P3b chunks against the full name's 13). A question's group topics are the served nation, place, race and organization topics whose name or alias of 4 or more characters it names.
**Result.** Of the 354 topics the short-name rule adds exactly that one alias. "Karlan Trade CO., LTD" and "Raythean Industries" already carry their heads; "Group" is not a suffix here, so the Ursus Student Self-government Group keeps its name.
**Decision.** Shipped. It replaces, at the source, the identity link `fold_group_identities` used to read, which `entities.py build` drops since 2026-10-16 as naming a group. A test pins that an identity group whose label is an organization topic gives the topic its names. `TREVOR_ORG_SHORT_NAMES=0` leaves the topics as written.
**Code.** trevor/src/tools/mod.rs, `Tools::org_short_names`; trevor/src/tools/topics.rs, `Tools::named_group_topics`; trevor/src/tools/tests.rs, `group_identities_and_named_groups`

## Chronology

Dates, the timeline, eras, gatePos and the spoiler horizon, reading order.

### 2026-09-28: Death status follows the stories a reader has seen

**Decision.** Requirement: Arknights is live service. Death events are kept verbatim, so status follows the stories a reader has seen and a new story only adds events.
**Code.** trevor/src/tools/reading.rs, deaths section

### 2026-09-29: Reading guide leads with the game's Storylines links

**Tried.** The reading guide was changed to lead with the game's Storylines links.
**Decision.** Default since 2026-09-29; `--no-storylines` keeps the reading guide as it was before.
**Code.** `src/bin/ask/cli.rs`, `Args::no_storylines`

### 2026-09-30: reading_compare tool and Storylines zones for Episodes 15 and 16 (round 6, issue C)

**Tried.** "Can I read X after Y" and "does X spoil Y" go to a new `reading_compare` tool (release order, the game's Storylines link, what builds on what, the players' rule, and what that means for spoilers). act2mainss and act3mainss are Episodes 15 and 16 (their stages are main_15 and main_16 in zones act2mainss_zone1 and act3mainss_zone1); `reading_guide.py storylines` now maps them. The Masses' Travels probe question routed right but "Masses Travel" did not resolve; event resolution now also matches without a leading "the" and trailing "s" per word.
**Result.** The zone mapping adds the links Ending a Grand Overture before Episode 15 and Episode 15 before The Masses' Travels, and changes the reading-guide text of 5 real-probe answers. The tool now answers: release order Episode 15 (2025-09-16) then The Masses' Travels (2025-10-14), the game places Episode 15 first, 5 players say after Episode 15, so it can spoil Episode 15 and is safe after it. Route check (model) 74/76 before the resolution fix (Sami as always; Masses' Travels).
**Decision.** Shipped. `--no-reading-compare` strips the tool from prompt and grammar; `STORYLINE_ZONES=0` restores the old storylines file byte for byte.

### 2026-10-03: Chronology lines for a date question's subject (leftover c)

**Tried.** A when-question with the time rule now gets, after the timeline notes, the dated lines of `artifacts/chrono/terra_history.md` that name its subject (the question's runs of capitalized words; lines sharing more of the question's other words first, then the earliest; at most 8, listed by year) and `DATE_RULE` with them (`--no-chrono-subject`).

**Result.** It fires on 3 of 447: gold correct 2 to 2 of 2 changed (g0088, u004). `ian17` gets "1086: After excavating the Rhodes Island landship, Babel sought a skilled engineer..." and the 1090 to 1094 Babel scenes in its prompt but still answers "I could not find the specific year Rhodes Island was founded" without an estimate.

**Decision.** 0 losses, so it is the default; the estimate the rule asks for is not given.

### 2026-10-03: Time evidence as a block before the question, the estimate as the required final line (night 8, item 1)

**Why.** Under the night 7 default `ian17` ("When was Rhodes Island founded?") had the 1086 and 1090 to 1094 chronology lines in its prompt and `DATE_RULE` in the system prompt, and still declined without an estimate.

**Tried.** When `chronology_for` finds lines naming a when-question's subject, they now come as a numbered TIME EVIDENCE block (T1, T2, ...) right before the question, and the question is followed by a REQUIRED FINAL LINE ("Estimate: <a year or a range>, based on <the anchors and passages>"); the system prompt is unchanged (`--no-time-evidence` restores the CHRONOLOGY LINES block). The detector fires on 3 of 447 questions (g0088, u004, `ian17`; `--detect-only`). Measured with `--no-cache-prompt`, one Gemma session and one Qwen judge session.

**Result.** Gold correct 2 to 2 of the 2 changed (g0088 answerable, still correct, now ends "Estimate: 1095, based on ... three years prior to 1098"; u004 unanswerable, still declined, ends "Estimate: Unknown, because the passages do not provide any information"), faithfulness 1/1 to 2/3 (the estimate line is the unsupported sentence, as intended for a labelled estimate). `ian17` goes from a decline to "The exact year Rhodes Island was founded is not stated in the story text. However, the organization was established after the excavation of the Rhodes Island landship in 1086 [T1]. Records indicate that by 1090, the organization was already operating as "Rhodes Island of Babel" in Kazdel [T2]. Estimate: between 1086 and 1090, based on T1 and T2.", which is the expected shape (a labelled estimate with its evidence).

**Decision.** 0 losses, so it is the default. `--no-time-evidence` reproduces the old default's answer record (answer, passages, citations) on 3 of 3, and the new default reproduces the measured arm on 3 of 3.

### 2026-10-03: Chronology lines for when-questions, and the TIME EVIDENCE block

**Tried.** "When was Rhodes Island founded?" (leftover c) had timeline notes only for the events its passages came from, none about the founding. The dated lines of `artifacts/chrono/terra_history.md` are read as (year, line), and the dated lines of the chronology that name a when-question's subject are added. A subject counts as a proper name by the corpus (its lowercase form's occurrences). `--time-evidence` puts these lines in a numbered TIME EVIDENCE block.
**Result.** On night 8, `ian17` had the 1086 and 1090 to 1094 lines in its prompt and still declined without an estimate, which is what the numbered block addresses.
**Code.** trevor/src/bin/ask/chrono.rs, `Chrono::history`, `chronology_for`, `proper_name`, `time_evidence_block`

### 2026-10-04: Main episode dates from the game data instead of the wiki

**Tried.** `reading_guide.py dates` derives main episode dates from the game data: zone open times (Episodes 10 to 14), home-screen themes (9, 15, 16), and for 6 to 8 the Adverse stages' schedule minus the offset measured on Episodes 10 to 14 (7.25 to 7.46 days, median 7.29 over 8).

**Result.** 10 of 17 equal the wiki date, 11 within 1 day (Episode 6, 2020-07-01 against 2020-06-30). Episodes 0 to 4 are "by 2020-01-22" (the first event; the wiki's launch is 2020-01-16) and Episode 5 is undated (Adverse stages without a schedule), placed after Episode 4. Of the guide's 3,570 item pairs 1 is inverted against the wiki-dated guide (Grani and Episode 5); first appearances move for 0 of 4,310 speakers. `canon_refs`' later-story pools are unchanged (0 changed dates on or after the earliest run start, 2022-07-14), so the canon stamp was rewritten.

**Decision.** Shipped as the default. `MAIN_DATES=wiki` reproduces the old `reading_guide.json` byte for byte and `first_appearance.json` as equal JSON (key order follows `PYTHONHASHSEED`, pre-existing).

### 2026-10-04: `chrono.py build` reads only the data-picked exception groups

**Tried.** `chrono.py build` reads only the data-picked groups of `exceptions_reasoning.jsonl`, not the rows from the wiki's T1 groups.

**Result.** Same 3 candidates; `timeline_v1.json` byte-identical.

**Decision.** Shipped as the default. `CHRONO_EXC_ALL=1` reads all groups as before.

### 2026-10-07: Spoiler horizon for `ask --serve` jobs

**Tried.** A job's horizon (story or group id) keeps retrieval to chunks released at or before it, by the reading guide's 85 group positions and `spoiler.jsonl`'s `storySort`. Chunks of unplaced groups (operator files and records, voice, modules, items, enemies, outfits, IS, dossiers, summaries) are hidden, and table answers, topic summaries and entries, dossiers, the game-data passage, the overview, canon evidence, the identity and chronology notes and the evidence composition are left out (per-era rebuilds are later work).
**Result.** Three questions, passages before -> after, 0 passages after the horizon in all three:
- "Why does Ch'en leave the Lungmen Guard Department?" at main_3: 15 passages (all after main_3 or unplaced) -> 12 (main_2, main_3); declines, which is right for a reader at Episode 3.
- "What happened to Talulah?" at main_4: 17 (main_8, main_9) -> 14 (main_1, main_2, main_4); answers from her Chernobog battle only.
- `ian12`'s Mon3tr question at main_6: 24 -> 15 (act3d0, act5d0, main_6); declines, since the answer is in act42side.

Ruled out by measurement: the first cut's 8 x k (64) fused candidates before the drop kept only 1, 5 and 2 passages on the same three, since their top 64 were nearly all later stories. The filter now reads 2,000 candidates per retriever (`HORIZON_DEPTH`), which dense and BM25 score anyway. After this depth change, `ian8` and the three no-horizon runs of the horizon questions were rerun and are identical to the first run on 4 of 4 (all fields but timings).
**Decision.** Built into `--serve`; no `horizon` field is the old behavior. Tests: 2 horizon tests (lib), the request/result schema and the request reader (ask); `cargo test --release --features embed` 95 passed, clippy clean.

### 2026-10-07: Horizon from source dates

**Tried.** The 2026-10-07 horizon rule placed a story by group position, then storySort, with ties mutually visible, and used the source dates for chunks the reading guide does not place.
**Decision.** Shipped. `TREVOR_HORIZON_DATES=0` ignores the source dates, so every chunk the guide does not place is hidden, as before 2026-10-07 (the ranks stay those of the file). The old horizon of a story or group id stays available as the legacy horizon.
**Code.** trevor/src/search/horizon.rs, module doc and `GateOrder` legacy horizon

### 2026-10-07: Retrieval depth under the horizon

**Tried.** The first cut of horizon filtering took 8 x k (64) candidates, dropped those past the horizon, then kept the first `k`.
**Result.** It kept 1, 5 and 2 passages on three questions whose top 64 were nearly all later stories.
**Decision.** Filter over the full ranking, then take the first `k`: dense and BM25 score every chunk anyway, so the depth is cheap.
**Code.** trevor/src/search/runtime.rs, `Runtime::retrieve` (horizon depth)

### 2026-10-16: Time evidence as an estimate line

**Tried.** With `--time-evidence`, the chronology evidence ends in a final line ("Estimate: ..."); off, it is the CHRONOLOGY LINES block, as before.
**Code.** `src/bin/ask/cli.rs`, `Args::time_evidence`

### 2026-10-16: Horizons as gatePos

**Tried.** Since 2026-10-16 (design/trevor-contract-v1.md section 1) a horizon is an integer `horizonPos` on the gatePos order. The order is the panel prototype's (`overnight-2026-10-15/prototypes/trevor-panels/build_panels.py`, `Order`, dated 2026-10-15), ported exactly: each story gets a key, compared field by field.
**Result.** Eras pass without a boundary (22 eras on 2026-10-16). An era's `position` is the gatePos of its group's last story. A gatePos is turned back into the story at that rank, byte for byte the 2026-10-07 rule.
**Code.** trevor/src/search/horizon.rs, module doc

### 2026-10-18: A spoiler notice for an event past the horizon, and its recap within it

**Tried.** "Give me a summary of The Masses Travels" (act42side, gatePos 1613 to 1636) from a reader at gatePos 722 (Shatterpoint) answered "The provided passages do not contain a summary": retrieval within the horizon found nothing of the event, and under any horizon every table answer was dropped, so "Give me a summary of Guide Ahead" (act16side, wholly before 722) was answered from retrieval too, with passages about Eyjafjalla's guide in an operator record. Two changes, `spoiler.rs`:
- the spoiler notice: under a horizon, a question naming an event (`Tools::event_in_question`) whose first story lies after the horizon gets `prompts.rs` SPOILER_NOTICE ("The Masses' Travels comes after where you have read (you are up to Shatterpoint), so Trevor won't summarise it yet. Turn off spoiler protection to ask about it."; "answer questions about it" when the question is not a summary request), with no retrieval and no model call, in `Service::answer` (the `cpu` and `cf` rungs) and `Service::passages` (the `passages` rung; that path was not run live);
- the recap under a horizon: a recap route is served when every story of its event lies at or before the horizon (`ReleaseOrder::group_span`); every other table answer stays dropped, and an ending recap whose group id is part of another's ("main_1" in "main_10") stays dropped, since the keyword router's recap reads the ending by a substring test.
**Result.** E2B, mid, through `ask --serve --share-server --no-cache-prompt`: at 722, act42side gets the notice with 0 citations and 0 model calls; Guide Ahead gets its recap; without a horizon, act42side gets its recap. "What is the name of the npc that mon3tr suggests assassinating in masses travels" (`ian33`) at 722 gets the notice. Kill switches at 722: `--no-spoiler-notice` sends HEAD's prompt for the Masses' Travels question (byte-identical) and returns HEAD's answer; `--no-horizon-recap` answers Guide Ahead from retrieval, as before. The event detector fires on 6 questions across the eval sets' `--detect-only` outputs, all of them real event names.
**Decision.** Both on by default; `--no-spoiler-notice` and `--no-horizon-recap` are the kill switches. The notice's wording is the four SPOILER_* constants in `prompts.rs` (not sent to a model).
**Code.** `src/bin/ask/spoiler.rs` (2 tests), `src/bin/ask/serve/service.rs`, `Service::answer` and `Service::passages`; `src/search/horizon/mod.rs`, `ReleaseOrder::group_span` (1 test)

### 2026-10-18: Time-gap evidence for "how much time passes between <story A> and <story B>"

**Why.** "What is the time skip from chapter 8 to chapter 9?" and "How many years pass between chapter 8 and chapter 9" were declined (E2B, mid). The game text answers it: Kal'tsit, in Episode 9's interlude, "There hasn't been an emergency transmission to Rhodes Island's deck for eleven months and twenty-seven days, since the end of the Chernobog incident" (`main_9_level_st_09-03#0000`), and Episode 8 ends that incident. The timeline cannot: Episodes 6 to 9 state no year (all four "at most 1098"). The whole-corpus search did not reach the line, and the second wording got timeline notes and answered "about 1099 ... not specified".
**Tried.** `timegap.rs`: a question with an elapsed-time phrase ("time skip", "how long", "how many years", "years pass", ...) that names two episodes or events (`Tools::events_in_question`: "chapter 8 to chapter 9", "chapters 8 and 9", event names) gets the later story's line that states elapsed time (a count word within 3 words of a unit, and "since", "ago", "passed", "later", "been", ...) and names a capitalized word of the earlier story's event summary that is not in the later one's and does not speak in the later story ("Chernobog"); a line with "since", "ago", "passed" or "later" ranks first. The line goes first as a passage (with 1 line either side), again in a TIME GAP EVIDENCE block before the question, with a rule and a closing instruction to begin with the stated time. The model router sends such a question to retrieval before its call.
**Result.** E2B mid, `--no-cache-prompt`: both wordings go from a decline to "eleven months and twenty-seven days, ... since the end of the Chernobog incident [1]", citing `main_9_level_st_09-03#0000`. With 3 lines E2B quoted the second and third ("Several months later...", Talulah's "A few months? Half a year?"), and "Amiya" (in both episodes) counted as a mark of Episode 8 until speakers of the later story were excluded, so 1 line is kept. The detector fires on 0 questions of the probe set, real probe, held-out, gold v2, trivia, `routes.jsonl` and the 1,228 Reddit and Discord questions. `--no-time-gap` reproduces HEAD's prompts and answers on the 2 questions.
**Decision.** On by default; `--no-time-gap` is the kill switch. `--detect-only` prints `timegap=<earlier>><later>:<lines>`, `--scoped-only` the chosen line. Not measured on the 12B.
**Code.** `src/bin/ask/timegap.rs` (1 test), `src/tools/appearances.rs`, `Tools::events_in_question`; `src/bin/ask/answer/generate.rs`, TIME GAP EVIDENCE; `src/bin/ask/prompts.rs`, `GAP_RULE_TAIL`, `GAP_TAIL`

## Design inference (retired)

`design_infer.py`, art captions read for design, visual models, `--design-deduce`. The chain was retired on 2026-10-17; these entries are its record.

### 2026-10-02: Operator art captions with a local vision model: assets, model setup and caption quality

**Tried.** Art questions ("which operator has their height written in their E2 art?") have no text to retrieve, so `scripts/art_captions.py` (approved 2026-10-01) describes the images with a local vision model. Assets: `assets/output/en/textures/chararts/<char>/` holds the elite arts (`_1` = E0, 436 files; `_2` = E2, 381; the `b` files are the same picture at half size, 1024 px), `textures/skinpack/<char>/` the outfits (483 full, 471 `b`), RGBA PNGs of 1104 to 2796 px square (2048 the most common) with separate `[alpha]` companions that the exported PNGs already include. The game data adds text but no description of the picture: skin_table's DisplaySkin has the illustrator list (DrawerList), outfit name, brand group and the outfit's flavor text; `select` joins it with character_table into 379 E2 arts and 472 outfits with an existing image (2 and 2 missing). Model: Gemma 4 12B with its multimodal projector `mmproj-gemma-4-12b-it-qat-q4_0.gguf` (175,115,616 bytes, sha256 cb018338a7538a9814d994bfe54644c71eb7ed54e31eae2f721e45fd3c260da7) from the same pinned repo and revision as the GGUF, fetched by `WITH_VISION=1 ./scripts/fetch-model.sh`. llama-server 0.5.0 takes it with `--mmproj` and image_url data URIs on /v1/chat/completions. Images are cropped to their alpha box, put on mid grey and scaled to 1536 px on the long side, with `--image-min-tokens 1120 --image-max-tokens 1120` (Gemma 4's largest budget). Captions were checked by eye on 12 pilot images.

**Result.** The first run aborted on the first image (`GGML_ASSERT ... non-causal attention requires n_ubatch >= n_tokens`); `-b 2048 -ub 2048` fixes it. A request is 1,392 prompt tokens (median) and takes 8.1 s (median, 252 images, 37 min of model time for the first run). 252 of 379 E2 arts were captioned, 10 with the tiled text pass, and all 851 images with the Apple Vision cross-check (on-device text recognition, no model, 4 min). On the 12 pilot images, hair, ears, horns, tails, clothing colors, companion creatures and setting are right on all 12; objects are misnamed on 3 of 12 (Toddifons' giant crossbow called "a heavy machine gun or a tripod-mounted turret", Blaze's chainsaw called "a futuristic black rifle", Swire's white tiger called "wolf-like"); a detail that looked invented at thumbnail size, Blaze's "small black cat at her feet", is in the art. 6 of 252 captions (2.4%) ran into a repetition loop ("S.S.S.S..." until max_tokens) and are redone on resume; the schema now bounds the text lists (24 items, 120 characters), so later rows carry a different prompt hash.

**Decision.** Kept as an offline script; captions are not served (see the art-units proposal below).

### 2026-10-02: Reading visible text in operator art: whole image, tiled pass, Apple Vision

**Tried.** Three ways to read text written in the art: the whole-image caption's transcription, a `text` pass (the same transcription prompt on a 2x2 grid of overlapping full-resolution tiles), and Apple Vision on-device text recognition as a cross-check. Acceptance case: Pallas's E2, whose measurement labels read "Pallas 162cm", "Aegean 485m", "Feather Beasts 832m", about 25 px tall in the 2048 px art (her handbook height is 162cm).

**Result.**
- Whole image is not trustworthy: Pallas came back as "Pallas 82m", "Aegean 45m", "Aeolian Beasts 82m", and Toddifons' crossbow ("RHODES ISLAND") as "DRAGON SLAYER", "2018", "S.S.S.S.".
- The tiled `text` pass (13.2 s per image) read "Pallas 162cm", "Aegean 485m" and "Feather Beasts 832m" exactly, and "RHODES ISLAND" on Toddifons, returned nothing on 5 of 5 pilot images without text, and still misread faint or stylized lettering (a fourth Pallas label, "Altocumulus", came back "Atmospheric 300m"; Cairn's "SMILE" sign came back "?M.I.L.L.").
- Apple Vision found no text at all on Pallas (rotated lettering) and read a gauge on Cairn's E2 as "158", so it is a cross-check, not the source.
- Whole-image text agreed with Apple Vision on 25 images and stood alone on 36; of 2 of those 36 checked by eye, both were real (Aak's box label, Stainless's "Mission in Progress"), and repeated words across operators ("LUMINA" on Grani and Akkord, "REVENANT" on Amiya and Warfarin) look invented.
- The height question: whole-image captions found no height on the 252 E2 arts they cover; the tiled pass, which is what finds Pallas, ran on 10.

**Decision.** Visible text is to be quoted only from the tiled pass; Apple Vision stays a cross-check.

### 2026-10-02: Art units for P4: remaining cost and a proposal (not built)

**Tried.** Costing the rest of the captioning and drafting how captions would be served.

**Result.** Cost for the rest at the measured speeds: the remaining 127 E2 captions 17 min, tiles on the other 369 E2 arts 81 min, all 472 outfits 168 min (captions and tiles), about 4.4 h of model time in all. A second 40-minute run (the other 127 E2 captions, then tiles) waited until its start cutoff while another job held the model lock, and did not run.

**Decision.** Proposal, not built: one P4 unit per image, kind "art", text = operator, outfit name and brand, illustrators, the skin_table flavor text, the caption, and the tiled transcription under "Text visible in the art"; the router's source list gains "art" for questions about what an illustration, E2 art or outfit shows; the answer rule says the description was written by a model from the image and is not the game's text, quotes visible text only from the tiled pass, and says "the description may be wrong" for anything not also in the game data. Users would see a line such as "From a model-written description of Pallas's E2 art, not game text: her art carries the label 'Pallas 162cm'". The kill switch would be dropping the art units from the P4 build.

**Follow-up.** `caption --kind e2 --seed 7` then `text --kind e2 --seed 7` resume where captioning stopped.

### 2026-10-04: Design inspirations from the wiki's trivia pages, a labelled outside source (`design_basis`)

**Tried.** `ian11` ("what operator is based off an animal with phantom as part of its name") has two readings: Lucilla, based on the giant phantom jelly, and Phantom (black cat motif, via Poe's The Black Cat) with his alter Tragodia. No game-text chunk ties Lucilla to a jellyfish, so `scripts/design_basis.py` fetches each of the 407 operators' arknights.wiki.gg "<Name>/Trivia" pages through the parse API (1 request a second, raw JSON cached in `artifacts/wiki_cache/`, about 10 min from empty, never refetched) and keeps trivia sentences with a design cue ("based on", "inspired by", "motif", "modeled after", "reference", "named after", ...) outside the Distinctions and Miscellaneous sections. The basis is the linked subjects nearest the cue, the subject before "(which X is based on)" and the phrase after a strong cue; each row carries the operator's game alters (`char_meta_table` SpCharGroups).

Ask's `design_basis` tool answers before the router, with no router line. It fires on a question with a design word ("inspired by", "motif", "design inspiration", or "based on/off" at the end, before a/an/any/real, or with "animal" or "creature") and matches each content word against basis subjects (operators based on it, only for a question about an operator or character) and against operator names (their rows and their alters'), gives every reading that has rows, and opens with "From the Arknights wiki's trivia pages (arknights.wiki.gg), not the game text". Measured with `--no-cache-prompt` in one Gemma session.

**Result.** Coverage: 373 of 407 operators have a Trivia page (34 do not, or sit under another title: Haze, Aurora, GreyThroat, Leizi the Thunderbringer and 30 more), 175 of 407 have at least one basis row, 306 rows in all. Read on a sample of 10 (seed 11): all 10 sentences belong to their operator; the subject list names the right thing in 6 (Honeyberry's sugar glider, Shalem's Gorgoneion and Medusa, Surtr, Savage's Cave of the Crystals) and is clumsy in 4 (Ray gets Rim Billiton's Australia, Wang a whole clause); 3 of the 10 had outfit-tooltip text cut into the sentence, fixed by dropping data-* attributes before the split (0 rows contain it now).

The detector fires on `ian11` alone of real probe (120), gold (146), held-out (40), trivia (109) and the 32-question probe set. On the 1,228 Reddit and Discord questions it fires once (a question about faction assignments, answered with two unrelated rows, the known false positive). Earlier versions fired on 3 gold questions (g0085 "based on his inspection", g0094 "Based on the conversation between ...", u006 "Who designed the Rhodes Island company logo?") and on "Could Babel be inspired by the Tower of Babel?", which the "based on" position test, dropping bare "design"/"designed" as cues, and the operator-question test removed.

`ian11` goes from a decline ("I could not find any information in the story text ...") to Lucilla (giant phantom jelly) under the subject reading and Phantom (The Phantom of the Opera, The Black Cat as his animal motif) and Tragodia (Dionysus, labelled Phantom's alternate operator) under the name reading, each with the wiki sentence and URL. The second phrasing gives the same two readings; "What animal is Lucilla based on?" answers the giant phantom jelly where the default declined. 1 changed answer, a gain, 0 losses.

**Decision.** Shipped as the default. `--no-design-basis` reproduces the night 9 final default's `ian11` record (answer, citations, passages) byte for byte, and every other question is untouched because the detector does not fire on it. `update.sh` runs the script only with `DESIGN_BASIS=1`, so updates stay offline. (Superseded the same day: see the wiki-as-eval-reference entries below.)

**Caveat.** The answer lists rows rather than picking the animal (Phantom's third row is about his illustrator's cat), and 34 operators have no page under their game name.

### 2026-10-04: `design_infer.py`: design inspirations inferred from game evidence (generation)

**Tried.** `scripts/design_infer.py`: Gemma infers the animal and up to 2 references from each operator's game evidence, and Qwen checks the cited evidence. Generation ran for all 407 operators.

**Result.** The check reached 227 of 683 subjects before it was stopped, so the built table (`artifacts/entities/design_infer.partial.jsonl`) is not served, and the design tool fires on nothing (`ian11` declines, as before the wiki tool). In the raw output Lucilla's inferred subject is her own name (no jellyfish: the game evidence is "an umbrella with wriggling tentacles" and bubbles, and the art captions mention neither), Kirara likewise, Phantom is The Phantom of the Opera, Cliffheart a snow leopard, Honeyberry a squirrel (the wiki: sugar glider).

**Decision.** Not served until scored.

### 2026-10-04: Design inference scored against the wiki answer key: held-out precision 0.486, below the bar, nothing served

**Tried.** The Qwen check finished the remaining 456 of 683 subjects. `design_infer.py score` (Qwen compares an operator's inferred subjects with every wiki trivia row for that operator: exact, close or none) over the 175 operators the wiki covers. Before reading any score, the 175 were split by sha256(charId) parity into a held-out half (91, even) and a dev half (84), and the serving bar was fixed at held-out precision 0.600 plus 0 losses on the design questions (`artifacts/answers/wiki-ref/split.json` records both rules with their times).

**Result.** The check kept 181 of the 683, so 161 of 407 operators have a checked subject. Over all 175: exact 17, close 12, none 146 (exact or close 29 of 175 = 0.166); 107 of the 175 have no checked subject, and of the 68 that do, 29 match (precision 0.426).

On the held-out half:

| table | matches of 91 | exact | close | match rate | precision |
|---|---|---|---|---|---|
| v1, checked | 17 | 10 | 7 | 0.187 | 17 of 35 (0.486) |
| v1, unchecked (`DESIGN_CHECK=0`) | 30 | 17 | 13 | 0.330 | 30 of 91 (0.330) |

The check buys precision 0.330 to 0.486 for 13 of the 30 matching operators (Cliffheart's snow leopard, the subject the wiki names, is one it rejected).

**Decision.** Below the 0.600 bar, so nothing is served. `report` now scores the current table's own comparison text (the score file holds every version's judgments), takes `SPLIT`/`SPLIT_PART`, and prints the precision line.

### 2026-10-04: Design inference v2 prompt (`DESIGN_PROMPT=v2`): refuted on held-out

**Tried.** Reading the dev half's misses against the wiki's sentences (never a per-operator answer) gave four evidence patterns the v1 prompt ignores:
- a name is often the subject's word in another language or its scientific name (a genus as codename, a species epithet in a real name: the wiki's Cliffheart "Ensia" from Panthera uncia, Matterhorn "Yek" from yak, Astgenne "Urbica" from Delichon urbicum, Waai Fu's 槐);
- the Chinese codename carries etymology the English one drops (FEater's 食铁兽, the old name of the giant panda);
- the race gives the family (sea races are sea creatures, read by tentacles, bells, fins);
- Gemma returned the codename itself or the string "null" as a subject (Glaucus, Kirara, Eunectes, Papyrus, Jieyun).

v2 (`DESIGN_PROMPT=v2`, opt-in) adds the Chinese codename (`assets/output/cn` `character_table`) and the race's served summary (the first two sentences of `topics.jsonl` `summaryV2r`) as evidence and a three-step prompt (names, animal, references) that drops the codename and "null".

**Result.** On the held-out half it loses: checked 17 to 16 of 91 (exact 10 to 9), precision 0.486 to 16/41 = 0.390; unchecked 30 to 26 of 91 (0.330 to 0.286). It names more animals (subjects labelled animal 5 to 15 of 141) but fewer literary references (44 to 21), and Lucilla gets "Iris" and "Tidal Bore", not a jellyfish (v1: her own name, rejected by the check).

**Decision.** Refuted; the default prompt stays v1 (without the variables gen plans 0 operators and the v1 build is byte-identical).

### 2026-10-04: Animal-only pass (`DESIGN_ANIMAL=1`): refuted on the dev half

**Tried.** `design_infer.py animal` (`DESIGN_ANIMAL=1`, opt-in): one more Gemma call with the v2 evidence asking only for the species, its subject added to v1's and checked like them. Run on the dev half first under a rule fixed before its output (go to held-out only at +3 operators).

**Result.** It names an animal for 22 of 84, mostly the race's family (Fox for 6 Vulpo, Wolf for 3 Lupo, Ifrit a goat), null for Cliffheart and Matterhorn, and moves dev 12 to 13 of 84 checked (precision 0.364 to 13/43 = 0.302), unchecked 25 to 26.

**Decision.** Below the +3 rule, so it stopped at the dev half; stays opt-in.

### 2026-10-04: Best filter on the dev half still misses the bar; the design table stays unserved

**Tried.** Choosing a filter on the dev half by a rule fixed beforehand: highest dev precision with at least 20 served operators, among checked or unchecked, all or high-confidence, all or animal-only.

**Result.** The rule picks checked high-confidence subjects (dev 12/30 = 0.400); held-out 16 of 32 = 0.500, recall 16 of 91.

**Decision.** No version reaches 0.600, so the table is not served: ask's design tool fires on nothing (`ian11` declines). `update.sh` now builds `artifacts/entities/design_infer.unserved.jsonl` (`DESIGN_SERVE=1` builds the served path as before). Tables and split: `artifacts/answers/wiki-ref/` (`design_tables/`, `split.json`).

**Why.** What the game text cannot give, by a keyword reading of the 306 wiki rows: 25 of the 175 operators have only rows about outfit or E2 artwork (55 rows: Vermeer's Girl with a Pearl Earring, The Grand Budapest Hotel, Mogao murals), real weapon models (23 rows: M320, Arctic Warfare, Kulakov), memes, voice lines and Weibo intros (13 rows), all outside-world recognition the captions and files do not state. Much of the rest is literary or historical allusion (Li Bai, Salome, Dugu Qiubai, Joan of Arc) and name puns in Chinese or Japanese, which the evidence supports only through a translation step Gemma 12B does not take reliably. Lucilla's giant phantom jelly has no game-text tie (umbrella with wriggling tentacles, bubbles, Ægir; the captions mention neither).

**Caveat.** The comparison itself is generous and noisy (a row about a voice actor or an event teaser counts as a subject), so 0.187 is a floor on the animal accuracy and a ceiling on nothing.

### 2026-10-04: Gemma 4 26B-A4B design-inference pilot stopped after 2 of 84 dev operators: no score, nothing served

**Tried.** The design inference gen step on the dev half with the 26B (`DESIGN_MODEL=26b`), under a watchdog that stops the server when pressure is critical or swap grows by more than 1,500 MB.

**Result.** The watchdog stopped the server 20 s into gen, with 2 of the 84 dev operators written (`artifacts/entities/design_infer.raw.26b.jsonl`): Specter the Unchained "Stultifera Navis" and "Gramophone", where the 12B said "The Ghost Ship" and Ulpianus; Texas the Omertosa "Omertà" and "Nuova Volsinii", where the 12B said wolf and Omertà. Nothing is scored: the dev comparison (12B v1: 12 of 84 exact or close checked, precision 12/33 = 0.364; unchecked 25 of 84, 0.298), the held-out run (12B: 17/35 = 0.486 checked, 30/91 = 0.330 unchecked) and Lucilla, Phantom, Cliffheart, Honeyberry and the literary cases are all unmeasured for the 26B.

**Decision.** Parked. The table stays unserved (the bar of 0.600 is untouched, `update.sh` unchanged). `design_infer.py` takes `DESIGN_MODEL=26b` to write and read its gen rows in `design_infer.raw.26b.jsonl` (check and score files are keyed on their own input text and shared); unset, gen plans 0 operators and the build is byte-identical to `artifacts/entities/design_infer.unserved.jsonl`.

**Follow-up.** With about 15 GB free, `DESIGN_MODEL=26b python3 scripts/design_infer.py gen --ids <dev>` resumes at 82 operators (about 7 minutes at two calls at a time), then check and score on Qwen as before.

### 2026-10-04: Visual design reading of operator art (`art_captions.py design`)

**Tried.** Requirement: read an operator's design inspiration from the art itself. The motivating case is Lucilla, whose giant phantom jelly has no tie in the game text. A new stage, `art_captions.py design`, shows the 12B Gemma with its mmproj each operator's E2 art (the E0 art when there is no E2, 6 of the 175, or when the E2 reading names nothing or is low-confidence), with only the race from operator_attributes as context. Under a JSON schema it asks for the character's own body features, the other motifs, the most specific real-world animal, creature, plant or object, its group, its kind and a confidence. Output goes to `artifacts/art/design.jsonl` as compact rows (operator, image id, subject, group, kind, confidence, features, seconds) keyed on the source PNG's sha256 and the prompt hash (which includes the image size and token budget), so a rerun reads only new or changed art. It reuses `prepare()` and adds no dependency.
- The first pilot prompt carried a feature example ("bell-shaped hat with trailing translucent tendrils") that the model copied onto Specter the Unchained as a jellyfish and that encodes Lucilla's answer, so it was dropped before any score.
- On the 22 dev pilot operators, the body-and-motifs prompt (v3, the default) and a one-list prompt (v2, `ART_DESIGN_PROMPT=v2`, opt-in) each name the wiki's animal exactly or closely for 8 of the 22 dev animal cases. v2 named background creatures as the character (Cliffheart a wolf from the crystal heads behind her, Specter a whale from a skeleton).
- Image tokens 768 against 1120 on the same 22 (v2 prompt): 768 is 5.9 s against 6.4 s an image and as accurate on the wiki key (7 to 9 of 22 either way), but it read Raidian as a panda and Cliffheart as a plain cat.

**Result.** The full run read 240 images for the 175 wiki-covered operators (49 of them E0), median 8.41 s and mean 8.64 s an image, 2,073 s in all (34.6 minutes). From scratch for all 436 operators with art that is about 597 images and 86 minutes; incrementally 8.6 s per new or changed image (1.37 images per operator). Checked by eye (reading the images), of 15 readings viewed:
- 3 right: Dorothy rabbit, Shirayuki fox, Executor the Ex Foedere angel.
- 1 near: Honeyberry squirrel, where the art shows sugar gliders.
- 1 unclear: Robin bear.
- 10 wrong or missed: Kirara reads "none" (v3) or jellyfish (v2) under a visible octopus; Lucilla "none" on both arts though the phantom jelly's bell and arms are in her E2; Cliffheart a wolf beside her spotted snow-leopard tail; Flametail a fox beside a squirrel emblem; Glaucus, Thorns and Fang get invented ears and tails; Hoshiguma a unicorn (oni); Akafuyu a scorpion (spider crab legs).

Against the wiki key the visual subject is exact or close for 8 of 22 dev and 7 of 22 held-out animal cases (15 of 44, 0.341).

**Decision.** v3 is the default prompt; v2 stays opt-in. 1120 image tokens stay (`ART_IMAGE_TOKENS` keys the rows). Ruled out by measurement: the v2 one-list prompt and 768 image tokens (no accuracy gain, misreads).

**Why.** The 12B's vision misses the subjects that matter (Lucilla's jelly, Kirara's octopus, Honeyberry's gliders, Akafuyu's crab), so a stronger vision model, not more prompt work, is the next lever.

**Caveat.** The vision server held 8.6 GB resident; system free memory was 21 to 42 % while it ran (other processes, two rustc builds among them, were active; kernel pressure level 1 to 2, never 4), swap 10.3 to 10.8 GB, no thermal warning recorded at any check.

### 2026-10-04: Visual reading as design_infer evidence (`DESIGN_VISUAL=1`): held-out precision 0.486 to 0.431, not served

**Tried.** `design_infer.py` takes `DESIGN_VISUAL=1` (opt-in; `DESIGN_VISUAL_FILE` reads another reading file). The reading becomes one more evidence item labelled "visual design reading of the operator's E2 art (a vision model's reading of the image, not game text)", and the prompt gets one sentence on weighing it. Unset, gen and check plan 0 operators and the build is byte-identical to `artifacts/entities/design_infer.unserved.jsonl` (verified with cmp). Scored on the dev split (84) and once on held-out (91); serving bar: held-out precision 0.600. A second rule, written before any held-out score, picked the dev-best filter with at least 20 served operators.

**Result.**

Dev (84; base 12B text-only: checked 12/33 = 0.364, unchecked 25/84 = 0.298):
- v3 checked 18/49 = 0.367, unchecked 31/84 = 0.369.
- v2 checked 19/54 = 0.352, unchecked 27/84 = 0.321.
- v3 was kept on dev precision.
- Dev-best filter: checked, high-confidence animal subjects (dev 11/24 = 0.458; high 19/47, animal 11/25, unchecked 26/79 to 11/27).

Held-out, scored once (91; base 17/35 = 0.486 checked, 30/91 unchecked):
- v3 checked matches 22 of 91 (exact 13, close 9) at precision 22/51 = 0.431; unchecked 37/91 = 0.407 (exact 19, close 18).
- The dev-chosen filter 12/30 = 0.400.

By kind of wiki subject (hand-labelled from the wiki rows, 22 animal operators in each half): held-out animal matches go 3 to 8 checked at precision 3/6 to 8/13, and 12 unchecked (12/22), while the other 69 stay at 14 matches and their precision falls 14/29 to 14/38 (the extra served subjects there are references the visual evidence pulled in, not matches). Dev animal matches 2 to 4 checked (2/7 to 4/12), other 10 to 14.

Named cases:
- Lucilla gains nothing (vision "none" on E2 and E0; gen "Lucilla" and "Tidal Bore", both rejected or wrong).
- Phantom now gets "Black cat" (checked, judged exact against the wiki's The Black Cat) with The Phantom of the Opera.
- Tragodia's vision reads a black cat but gen names The Phantom of the Opera (wiki: Dionysus).
- Cliffheart is a snow leopard (checked, exact) from the text, not the vision, which said wolf.
- Honeyberry a squirrel (the wiki's sugar glider; judged none checked).
- Kirara has no checked subject (unchecked "Jellyfish", the wiki's mimic octopus).
- Also exact: SilverAsh snow leopard, Lappland wolf, Mousse domestic cat, Blue Poison frog, Heavyrain zebra, Pramanix snow leopard (unchecked).

**Decision.** No version reaches held-out precision 0.600, so the table is not served. The design-question A/B was not needed: the default is unchanged (ask's design tool still fires on nothing and `ian11`; the Lucilla and Kirara questions decline as before). `update.sh` is unchanged (the visual stage is not added), and `DESIGN_VISUAL` stays opt-in. Ruled out by measurement: the visual reading as evidence (held-out precision 0.486 to 0.431), the dev-chosen animal-high filter (0.400), the v2 one-list prompt (dev 0.352). Tests 78, clippy clean.

**Caveat.** The score judge is noisy at the margin (Haruka's "Fish" judged exact in one table and none in another; the high filter scores 19 where its superset scores 18), so differences of 1 or 2 operators are not signal. The per-kind animal precision of 8/13 is a post-hoc split on the answer key and cannot gate serving.

### 2026-10-04: Design-inspiration tool

**Tried.** Operator design inspirations inferred from the game data (`scripts/design_infer.py`, 2026-10-04), one row per operator, served by a design-inspiration tool to a question with a design word ("based off an animal", ...). Built for `ian11`.
**Decision.** `--no-design-basis` turns the tool off.
**Code.** `src/tools/mod.rs`, `Tools::design_basis`; `src/bin/ask/cli.rs`, `Args::no_design_basis`

### 2026-10-05: Gemma 4 26B-A4B as the design generator: dev precision 0.364 to 0.471, held-out 0.486 to 0.417, not served

**Tried.** The rule was written into `artifacts/answers/wiki-ref/split.json` (`rule_26b`) before any 26B output was scored: two variants (the v1 text evidence, and `DESIGN_VISUAL=1`), the dev winner by checked precision, held-out once with the winner only if it beats the 12B's dev 12/33 = 0.364, serve only at held-out precision >= 0.600 with 0 losses on the design questions. Gen: dev text 82 operators and dev visual 84, and held-out text 91, median 9.0, 9.9 and 9.1 s a call with two client threads on one slot (about 4.7 s an operator), 0 unparsable outputs; the Qwen check took 138, 148 and 159 subjects.

**Result.**

Dev (84; 12B v1 checked 12 of 84 at precision 12/33 = 0.364, unchecked 25/84 = 0.298; 12B visual checked 18/49 = 0.367):
- 26B text: checked matches 16 of 84 (exact 10, close 6) at precision 16/34 = 0.471; unchecked 30 of 84 (exact 18, close 12; 0.357).
- 26B visual: checked 17 of 84 (exact 10, close 7) at precision 17/39 = 0.436; unchecked 26/84 = 0.310.

Text won and beat 0.364, so it went to held-out once (91; 12B 17/35 = 0.486 checked, 30/91 = 0.330 unchecked): checked matches 15 of 91 (exact 6, close 9) at precision 15/36 = 0.417; unchecked 33 of 91 (exact 18, close 15; 0.363, precision over served 33/90 = 0.367). The 26B finds 3 more operators than the 12B without the check and 2 fewer with it, and the dev gain did not hold on held-out (0.471 on dev, 0.417 held-out).

By kind of wiki subject (labelled by a keyword pass over the wiki rows then by reading: 20 animal operators on dev, 21 on held-out; the previous hand labels, 22 and 22, were not stored, so the 12B is recomputed on these labels):

| split, kind | 12B | 26B | 26B visual |
|---|---|---|---|
| checked, matches over served: dev animal | 2/8 | 2/7 | 4/11 |
| checked: dev other | 10/25 | 14/27 | 13/28 |
| checked: held-out animal | 5/8 | 4/9 | |
| checked: held-out other | 12/27 | 11/27 | |
| unchecked, matches over the kind: dev animal | 5/20 | 6/20 | 4/20 |
| unchecked: dev other | 20/64 | 24/64 | 22/64 |
| unchecked: held-out animal | 8/21 | 9/21 | |
| unchecked: held-out other | 22/70 | 24/70 | |

So the 26B adds literary and mythological references (other), not animals.

Named cases:
- Lucilla gets only her own codename at low confidence (rejected by the check), as with the 12B.
- Phantom "The Phantom of the Opera" (checked, exact, as the 12B).
- Tragodia gains Dionysus (checked; the wiki's subject, where the 12B said The Phantom of the Opera).
- Cliffheart gets "Silverash Family" (rejected), losing the 12B's unchecked snow leopard, and with the visual reading also Kjerag.
- Honeyberry "Complete Guide to Medicinal Plants" (rejected), with the visual reading also "Squirrel" (checked, judged none against the sugar glider).
- Kirara "video games" and "jellyfish" (medium), both rejected (wiki: mimic octopus).

**Decision.** Held-out precision 0.417 < 0.600, so the table is not served. The design-question A/B (`ian11`, Lucilla, Kirara, Phantom, the jellyfish question) was not needed and those questions still decline as before. `DESIGN_MODEL=26b` stays a measurement tag, and `update.sh` is unchanged; unset, gen and check plan 0 operators and the build is byte-identical to `artifacts/entities/design_infer.unserved.jsonl` (verified with cmp). Ruled out: the 26B as the design generator (held-out 0.486 to 0.417) and the 26B with the visual reading (dev 0.436 < 0.471 text). Tables: `artifacts/answers/wiki-ref/design_tables/design_infer.{26b,26bnochk,26bvis,26bvisnochk,26bho,26bhonochk}.jsonl`, raw rows in `artifacts/entities/design_infer.raw.26b.jsonl` (259 rows). Tests 79, clippy clean.

**Caveat.** Differences of 1 or 2 operators are within the score judge's noise.

### 2026-10-05: A fair answer key for design questions: 221 of the wiki's 306 trivia rows are a design basis

**Tried.** The wiki answer key mixed design bases with trivia no deduction from the game can match (a voice actor, a gun model, a meme, an outfit's film homage). `design_infer.py label` sorts each row with Qwen (temperature 0, a grammar; keyed on the row's operator, section, sentence and subjects plus the prompt) into animal 37, creature 12, plant 5, object 41, figure 126, other 85, written to `eval/reference/design_kinds.jsonl`. A spot check of 30 rows (seed 11) by reading: 22 agree. Of the 8 disagreements, 7 are rows Qwen called a design kind that are other by the task's definition (3 outfit homages to films and murals, a skill named after a poem, a voice line quoting Rambo, a shrine) and 1 is the reverse (Astgenne's house martin labelled figure); every animal or creature label in the sample is right. So rows in an Outfit section count as other (fixes 3, breaks none; declared in `split.json` `rule_tiles` before any tiled table was scored).

`DESIGN_KEY=design` scores against the design-basis rows only (133 operators, 181 rows; dev 64, held-out 69 on the split of 2026-10-04). `DESIGN_KEY=animal` scores against animal and creature rows (43 operators; dev 20, held-out 23). Unset, the key is every row as before. The existing tables were re-scored by the same Qwen judge, by checked precision (operators with a served subject that match exact or close).

**Result.** Old key, then fair key:

| table | dev | held-out |
|---|---|---|
| 12B v1 | 12/33 = 0.364 to 7/25 = 0.280 | 17/35 = 0.486 to 14/28 = 0.500 |
| 12B v1 unchecked | 25/84 = 0.298 to 16/64 = 0.250 | 30/91 = 0.330 to 26/69 = 0.377 |
| 12B with the visual reading | 18/49 = 0.367 to 14/36 = 0.389 | 22/51 = 0.431 to 20/41 = 0.488 |
| 26B | 16/34 = 0.471 to 11/24 = 0.458 | 15/36 = 0.417 to 13/26 = 0.500 |
| 26B visual | 17/39 = 0.436 to 12/33 = 0.364 | |
| 26B unchecked | 23/64 = 0.359 | 29/69 = 0.420 |

On the animal and creature subset (dev, held-out): 12B v1 0/7 and 3/7; unchecked 3/20 and 9/23; 12B visual 3/10 and 10/13; 26B 2/5 and 1/7.

The fair key moves every table by at most 0.08 and none reaches 0.600: the low precision was not an artefact of unmatchable trivia. The 12B with the visual reading is the best checked table on animal-based held-out operators (10 of 13 served match), which is what the art shows.

**Decision.** The fair key (`DESIGN_KEY=design|animal`) became the scoring key for the design tables; nothing reached the serving bar.

### 2026-10-05: Tiled design probe: 175 operators read on 2x2 full-resolution tiles

**Tried.** `art_captions.py design_tiles` asks the 12B vision model, per tile of the E2 art (E0 without one; the text pass's tiling), which creatures, animals, plants or objects are depicted, where (body, companion, clothing, weapon, background) and their visible features, with the race as the only context. Rows aggregate per image (subject, tiles seen in, places, features) into `artifacts/art/design_tiles.jsonl`: 175 rows, 204 KB, keyed on the PNG's sha256 + prompt hash, 0 bad tiles. Median 29.2 s an image, 84.8 min in all with `-np 2` (pilot) and `-np 4` (the rest, not faster: 25 to 37 s an image either way). A rerun plans 0 images in 0.6 s; 232 images remain for the other operators (about 113 min).

A table slice was built from it by the rule declared before scoring: winner by dev precision with at least 10 served dev operators, held-out once, serve at >= 0.600 on the fair key.

**Result.** Checked by eye on 30 images (the 20 dev pilot images and 10 seeded later ones), the main creature or animal is named:
- right in 18 (Texas's wolf, Morgan's leopard, Nearl's pegasus, Estelle's crocodile, Vulcan's bull, Purestream's belugas, Phantom's cat);
- partly in 7 (body parts only, or Matterhorn's bull for a yak-like head);
- wrong in 4 (Kirara's huge translucent octopus read as jellyfish and coral, Kirin R Yato's horned beast missed, Honeyberry, Asbestos's small salamanders missed);
- 1 not judgeable.

Tiling finds small companions the whole-image reading missed, and splits large subjects across tiles (Kirara's octopus spans both top tiles; Lucilla's dark ribbon creature is read as a whale in 2 tiles).

Slices: `visual_agree` unchecked (v1 animal subjects sharing a word with the art readings) dev 1/5 and `visual_agree` checked 1/4, both under 10 served. The tiles slice (the top animal or creature seen in 2+ tiles on the body or as a companion, 104 of 175 operators): dev 12/36 = 0.333, held-out 18/41 = 0.439 (exact 9, close 9). On the animal subset its held-out precision is 10/11, but that subset is chosen by the answer key, so it says only that when the operator is animal-based the art usually shows the animal, not that a served animal is right (the false positives are operators whose basis is a figure or an object).

**Decision.** Nothing is served. `DESIGN_FILTER=visual_agree|tiles` are measurement only.

### 2026-10-05: Two-step design deduction at question time (`ask --design-deduce`)

**Tried.** The default declines every design question (the `design_basis` table is not built). With the flag, a design question the table cannot answer goes, before the router, to a deduction over `artifacts/entities/design_evidence.jsonl` (`design_infer.py evidence`, no model, 0.65 s, 407 rows: race, recruitment text, body-feature sentences of the operator file, the E2 caption, the whole-image and tiled readings).
- Forward: Gemma lists real-world candidates with group and visible features; a phrase shortlist (each phrase scores the inverse document frequency of its rarest word in the row; words in more than a quarter of rows never count) keeps 8 operators; Gemma names matches; a grounding check keeps a match only with 2 features (1 when the evidence names the candidate) sharing a rare word with that operator's evidence. The check was added after the sea-slug dev question matched Lucilla on invented "branching dorsal appendages".
- Reverse: Gemma reads the named operator's evidence, subject first. With the subject after the features, Gemma 4 12B wrote broken names at temperature 0 ("bearear", "foxrichton fox", "black catsoorted cat") in 9 of the first 15 answers, measured with the server's grammar both ways; those answers were discarded and rerun.
- Answers are labelled "Trevor's deduction from the game's art and text" and say "the game does not show it" otherwise.

Measured on 15 design questions (`ian11`, Lucilla, Kirara, Phantom, the jellyfish question, and 10 written from dev operators only; references under `eval/reference/design_questions.jsonl`, judged by Qwen with `artifacts/answers/wiki-ref/judge.py` `JUDGE_REF=...`; one Gemma session, `-np 1`, `--no-cache-prompt`).

**Result.**
- `ian11` says nothing in the game shows a match (candidates Phantom, phantom butterfly, phantom orchid; closest Phantom, Tragodia, Shamare, Shalem), so Lucilla is NOT named: her art readings show water, bubbles and a whale, not a jelly, and "What animal is Lucilla based on?" answers whale (wrong; the wiki says giant phantom jelly).
- Kirara declines (the readings call her octopus a jellyfish). Phantom answers black cat (the wiki's animal motif; the Phantom of the Opera concept is not visual). The jellyfish question declines (closest Mizuki, Kirara, Lucilla).
- Dev: FEater bear and Leto polar bear (both the right family, giant panda and Asian black bear expected), Flametail fox (red squirrel), Asbestos tiger (fire salamander), Astgenne dove (house martin); the cattle egret question names Elysium (Grain Buds); the panda, yak, sea slug and passenger pigeon questions decline.
- Pairwise: base wins `ian11`, FEater, yak, Asbestos, sea slug (5); new wins Lucilla, Flametail, Leto, egret, Astgenne (5); ties 5.
- Graded alone: the base is 1 correct, 1 partial, 2 wrong, 11 declined; the new arm 3 correct, 12 wrong (the judge counts "the game does not show it" as wrong against a reference and passed Elysium). By reading: 3 right in substance, 5 wrong, 7 honest declines.
- As a table, the reverse deduction on all 91 held-out operators ("What is X based on?") serves a subject for 47 of the 69 in the fair key at precision 12/47 = 0.255 (animal subset 4/12): below every table above.

**Decision.** Kept opt-in. Kill switch: without `--design-deduce` the new code path does not run (the flag is checked first; the base arm's answers are the default's). Ruled out: serving the deduction by default (5 pairwise losses), the tiles slice (0.439), `visual_agree` (under 10 served dev operators). Tests 80 (86 with embed), clippy clean. Outputs in `artifacts/answers/design-1005/` (answers, judgments, j3 logs) and `artifacts/answers/wiki-ref/design_tables/design_infer.{v1vis,v1visnochk,agree,agreechk,tiles,rev}.jsonl`.

**Follow-up.** What would move it: a vision reading that does not split large subjects (Kirara's octopus, Lucilla's ribbon creature) and a race-to-family prior for the reverse step (Asbestos's Savra tail read as a tiger's).

### 2026-10-17: The design-inference chain retired

**Tried.** No version reached the bar of held-out precision 0.600 (the best was 0.486), and its outputs fed only the unserved table and the opt-in `--design-deduce`.

**Decision.** Removed: `scripts/design_infer.py`, `scripts/design_basis.py` (the refresher of the wiki trivia answer key it was scored against), the `design` and `design_tiles` passes of `scripts/art_captions.py`, and the design steps of `update.sh`'s step 4b (Gemma gen, Qwen check, the vision tile pass, evidence and build). The results above are the record; the last outputs stay on disk (`artifacts/entities/design_*.jsonl`, `artifacts/art/design.jsonl` and `design_tiles.jsonl`, `eval/reference/design_basis.jsonl`).

2026-10-19: the Rust consumer is removed too (`src/tools/design.rs`, the `design_basis` tool, its pre-router detector and `--no-design-basis`), since `artifacts/entities/design_infer.jsonl` no longer exists and the load log printed `design_infer.jsonl: missing` on every start; `--route-only` over routes and the five eval sets is byte-identical to the build before.

## Embeddings

Embedder models, ONNX exports, INT8 and fp32, fine-tuning, re-embedding. The fp32 re-embed and its guard are documented in full in [trevor-reembed.md](trevor-reembed.md).

### 2026-10-16: Every corpus on the fp32 export; query model must match the vectors'

**Tried.** The tokenizer may never differ from the corpus's. The ONNX file was allowed to differ until 2026-10-16 (INT8 corpus built on the development machine, fp32 queries on x86, cosine 0.997). Since then every corpus moves to the fp32 export, and long re-embedding runs are paced and resumable (`design/trevor-reembed.md`).
**Decision.** `check_query_model` refuses a query model other than the vectors' own. Setting `ALLOW_QUERY_MODEL_MISMATCH_ENV` to `1` is the kill switch: the refusal becomes a warning, and query and corpus models may differ, as before 2026-10-16.
**Code.** `src/corpus/embed.rs`, module doc (pacing), `QueryEmbedder`, `ALLOW_QUERY_MODEL_MISMATCH_ENV`, `check_query_model`

### 2026-10-16: Query ONNX file follows the vectors' model

**Decision.** `auto` is the default query ONNX file since 2026-10-16 (design/trevor-reembed.md section 7). `RuntimeArgs::allow_query_model_mismatch` allows a query model other than the one the vectors were built with (logged), as before 2026-10-16.
**Code.** trevor/src/search/runtime.rs, `RuntimeArgs::query_onnx`, `RuntimeArgs::allow_query_model_mismatch`

### 2026-10-16: Paced, resumable corpus embedding

**Decision.** Since 2026-10-16 (design/trevor-reembed.md) `embed-corpus` runs by default with two ONNX threads, a resource guard, a 0.6 duty cycle and a checkpoint. `--threads 0 --no-guard --duty 1 --checkpoint-every 0` is the run before (`--threads 0` is ONNX Runtime's own default, `--checkpoint-every 0` is no checkpoint).
**Code.** trevor/src/bin/embed-corpus.rs, module doc, `Args::threads`, `Args::checkpoint_every`

## Eval

Gold sets, judges, the real probe, the probe set, sign tests and the eval harness.

### 2026-09-30: Gold set v2 on the model router

**Tried.** Gold set v2 (146 questions, 23.5 s each) answered with the model router and compared with the keyword-era run (`gs2_p3b_new`, 84/131).
**Result.** 145 answers byte-identical to the keyword-era run, so their judgments were reused; the one change is g0048, where the router added the Seaborn topic summary as passage [1]. Judged: still correct, 2/2 sentences supported. Totals identical to the keyword-era run: 84/131 correct (0.641), 18 answerable declined, unanswerable 15/15, faithfulness 207/215 (0.963).
**Caveat.** The gold set asks almost nothing a table or topic answers (1 of 146 routed differently), so it cannot see the router; the real probe is the measurement that moved.

### 2026-09-30: Four more questions added to the probe set (ian6 to ian9)

**Tried.** Four questions were added to `eval/ian-questions.jsonl` as ian6 to ian9 with their evidence, and answered with the defaults.
**Result.**
- "What operator declared that they want to pee in public?" is declined ("I could not find any information ... pee in public"), wrong: Chiave's "see who can piss farther" (`act15d5_level_act15d5_st02#0000`) is not in the passages, a vocabulary miss.
- "Who is Flamebringer romantically attracted to?": "The passages do not mention any romantic interests or attractions for Flamebringer", right but without the Perfumer friendship (`archive_char_1022_flwr2#0006` is not retrieved).
- "What is Insider's favorite food?": "extremely fond of sweets ... his handmade frosted donuts ... ice cream donuts", right in substance, opening with "the story does not settle on a single favorite food".
- "Is Ulpianus talkative or untalkative?": "a man of few words", right, although Amaia's "less and less tight-lipped" line is not in the passages.

With `--question-form` the Ulpianus answer opens "relatively untalkative" and quotes his "......" lines; the other three read the same.

### 2026-10-01: Held-out character trivia set (`eval/trivia.jsonl`)

**Tried.** A held-out trivia set in the style of the probe set (`scripts/trivia.py`, `eval/trivia.jsonl`): 160 sampled chunks, one per operator, gave 110 questions and Qwen kept 109. Answered by `ask` with the new defaults, and without the form call.
**Result.** The generator skewed to personality (75) and hobby (31), with 1 each for food, habit and relationship, so the set under-covers the kinds the probe set asks about. With the new defaults: 74/109 correct (0.679), 6 declined, faithful 301/320 (0.941). Without the form call also 74/109 (4 gained, 4 lost), faithful 220/230. By source: operator archives 31/37, dossiers 14/15, records 10/15, scenes 11/18, voice lines 8/24. The 35 misses: 14 answer step (source chunk in the passages), 7 source chunk in the raw question's top 8 but displaced from the passages (route, form or budget), 8 at rank 9 to 40, 3 at 41 to 100, 2 beyond 100, 1 name resolution. On the 9 probe-set questions with the new defaults, the Ulpianus answer quotes Nightblade's file ("quiet warrior ... overlooked") as Ulpianus: the opinion path adds other operators as candidates for a question about one character, and the answer misattributes.
**Follow-up.** Open, in order of size: the answer step on present evidence (14), displaced top-8 evidence (7), voice lines (8/24), the one-character opinion misattribution.

### 2026-10-01: The 14 answer-step and 7 displaced trivia misses, read one by one

**Result.** The 14 trivia misses with the source chunk in the passages are 5 judge or reference errors, 1 borderline and 8 real answer errors.
- Judge errors: t008 (Kroos: the tape and hollow-cake pranks are in the answer, cited), t039 (Flint: "headstrong prick" quoted, from passage 16 of 17), t047 (Harmonie: "fashion magazines"), t071 (12F: "the day they came to this world", plus a March 3rd detail from another passage).
- Reference error: t091, the "hard to hide my excitement" line is Paryaqaqa's, not Logos's, so the decline is right.
- Borderline: t060 answers Leizi's downtime from her file (reading in the library) instead of the voice line at passage 7 ("idle days ... thorough study").
- Real errors: 6 are one-character questions the form call classed as opinion (t009 Nian "subjective", t034 Passenger gets Aosta's and Zebra's files attributed to him, t040 GreyThroat's line at passage 8 unused, t096 "no definitive statement" beside "I don't like you", t103 Jieyun's line misread with invented affection, t106 Yu's scene at passage 13 of 17 unused); 1 is the keyword OPINION_RULE on a fact form (t064 Blaze: "does not settle on a single favorite" before the right saying, the same opener as the probe set's Insider answer); 1 is a decline with the evidence at passage 1 (t053 Aroma: "watch" against "seeing bubbles"). Evidence ranked late is a factor in 2 (t040, t106), both inside the opinion pattern.

The 7 displaced misses share a different cause: 6 are voice lines (P4 rank 1 to 6) on questions the router sent to source "story", so `ask` retrieved from P3b, which has no voice lines; t051 (P3b rank 5) was displaced by the yes/no evidence interleave. The Ulpianus misattribution is the same opinion pattern: "Is Ulpianus talkative or untalkative?" gathered Ifrit's, Gummy's and Nightblade's files as candidates.
**Follow-up.** The corpus choice for voice-line questions is open.

### 2026-10-03: Eleven new probe questions and cross-session determinism of the default

**Tried.** `eval/ian-questions.jsonl` now holds 25 questions: `ian15` to `ian25` are eleven new questions with expected answers and the evidence found for them (Swire's kidnapping and grandfather, every boss who joins us, Rhodes Island's founding, Ed, Kristen and Friston, the Abyssal Hunter project, Aroma's grandmother, Mutsumi's plant helpers, Lava's idol, the operator who visited Iris's castle, and the brownie prompt injection). Every A/B in this round was answered with `--no-cache-prompt`, base and arm in one Gemma session, judged by Qwen in one session; the base arm is the default binary with the age estimate (frozen copies ask0 and ask2).
**Result.** The default answers of two sessions (ask0 and ask2) are identical on all 11 new questions, and a third binary's default answers (ask3) are identical to ask2's on 12 of 12 re-asked questions.
**Decision.** A later arm may be compared with an earlier session's base answers. Answers, judgments, prompt dumps and the scripts that made them are in `artifacts/answers/night6/` (r1: the first arms, r2: the second versions, r3: the third origin version and the kill-switch check).

### 2026-10-03: Kill switches verified for the four new defaults, and the probe set (25 questions) before and after

**Tried.** In one Gemma session with the new binary, every question the four new defaults changed was re-answered: 17 for the game text, the typed fallback and the race rule; 12 for the origin chain.

**Result.** `--no-game-text --no-typed-fallback --no-race-trait-rule` reproduces the old default's answer record (answer, passages, citations) on 17 of 17, and adding `--no-origin-chain` on 12 of 12. The new default reproduces each arm's answer on 17 of 17 (`ian11` and g0084 are the old default's, as the own-unit check intends; r084's answer was unchanged and only its passages differ, as in the arm), and the origin chain alone reproduces the version-3 arm on 12 of 12.

The 25 probe questions under the new default against the old: unchanged on 20 (`ian1` to `ian7`, `ian9` to `ian14`, `ian16`, `ian18`, `ian19`, `ian21` to `ian24`). `ian15` goes from partial (the grandfather only) to the whole expected chain (the kidnapping, the officer, the L.G.D., the grandfather); `ian20` stays right and adds "endless debates"; `ian17` stays a decline without a year; `ian8` keeps the substance and drops the childhood detail; `ian25` keeps the decline with no recipe.

By the reference answers: `ian15` partial to right. Still missing under the default: `ian16` (bosses), `ian17` (founding year), `ian19` (Kristen and Friston, muddled: "his daughter" from the dream, then "not my sunshine"), `ian22` (Mutsumi's robots), `ian24` (Bluishsilver). `ian16` and `ian19` are answered by opt-in flags (`--cross-ref`, `--relation-rule`) that lost elsewhere. `ian18` (Ed) is right as far as the data goes; `ian20`, `ian21` and `ian23` were right before and stay right; `ian25` declines without a recipe but does not say that Trevor answers lore only.

Tests pass, 74 without embed and 78 with (2 new: the `cross_ref` tool and the off-topic name test in `tools.rs`, the rule triggers and typed-unit owners in `ask.rs`), and clippy is clean.

**Follow-up.** In order of what it would buy:
- a date question needs the chronology searched for its subject (`ian17`);
- a one-operator question whose answer cites only that operator's file should get the typed fallback too (`ian22`, Mutsumi's module);
- routing additions keep costing unrelated routes, so `--cross-ref` wants a trigger outside the router prompt (a keyword or form test for "every/all ... boss ... join");
- the relation rule's second wording needs a re-measure on more relation questions (its one loss, r108, is judge noise by reading);
- the off-topic guard needs the sentence placed where Gemma follows it (it ignores an appended rule that conflicts with the base decline sentence);
- `identities.v2.json` is not read by `ask` yet;
- the outfit captions are a third done (164 of 472, no tiled text).

### 2026-10-03: Night 7 batch: the 32-question probe set and the firing-only method

**Tried.** `eval/ian-questions.jsonl` now holds 32 questions: `ian26` to `ian32` are seven new questions with evidence checked in the data (Deepcolor and the Church of the Deep, IS4 Ending 2 with its endbooks, the Sarkaz, what Sami guards, Dossoles's three factions, the Khaganquest, Elysium's chapters). Every new mechanism reads the question or game data only, never the router prompt (each router line added that week moved 20 to 34 unrelated routes), and each has a kill switch. Firing was listed without a model over all 447 questions of the five sets and the 32 probe questions (`ask --detect-only --batch`, hidden; `--detect-identity` adds the plain hybrid top 8 for the identity notes). The arms were answered only on the questions they fire on, with `--no-cache-prompt`, base and arms in one Gemma session (the base arm is the new binary with every new switch on), and judged by Qwen in one session.

**Result.** Kill switches: the new binary with all of them reproduces the old default's answer record (answer, passages, citations) on 32 of 32 probe questions (frozen copy `ask0`, the binary of 12:05).

**Decision.** Answers, judgments and scripts are in `artifacts/answers/night7/`.

### 2026-10-03: Night 7 kill switches verified, and the 32-question probe set before and after

**Tried.** With the final binary (topic parts and identity notes opt-in), all new switches off against the old default (`ask0`), and the final default against each measured arm.

**Result.** `--no-appearances --no-cross-ref-trigger --no-is-ending --no-term-rule --no-chrono-subject --no-own-file-fallback` reproduces the old default's answer record (answer, passages, citations) on 32 of 32 probe questions (against `ask0`), and the base arm's on the 6 changed trivia and 2 changed gold questions. The final default reproduces each measured arm on the questions it changed (`ian17` the chronology arm, `ian28` and `ian29` the term arm, `ian22` the own-file arm).

The 32 probe questions under the new default against the old: unchanged on 24 (`ian1` to `ian15`, `ian18` to `ian21`, `ian23` to `ian26`, `ian30`); changed on 8. By the reference answers:
- `ian16` (bosses who join): decline to the 12 operators;
- `ian22` (Mutsumi's helpers): wrong to right;
- `ian27` (IS4 Ending 2): "Trevor does not have that text" to the ending and its four endbooks;
- `ian32` (Elysium): 5 of the 9 chapters with text to all 9 plus Seaside Serendipity, with the paradox simulation missing (no script in the data);
- `ian17` (founding year) still declines without an estimate;
- `ian28` (Sarkaz), `ian29` (Sami) and `ian31` (Khaganquest) change wording only and stay shallow;
- `ian26` (Deepcolor) is right in both;
- still missing under the default: `ian30` (answered with `--named-topic --topic-events --place-nation`), `ian24` (declines even with `--identity-notes`), `ian19` (muddled without `--relation-rule`).

Tests pass, 75 without embed and 80 with (new: the question-detector test in `tools.rs`, the chronology and wide-operator test in `ask.rs`), and clippy is clean.

**Follow-up.** In order of what it would buy:
- a topic answer needs passages that hold the topic's substance, not one more summary (the named topic and event summaries cost three held-out declines, and the Sarkaz answer did not deepen);
- a term the question names should pull P4x game text when it is defined only there (Nightzmora);
- identity notes need Gemma to take a two-step inference it skips, or the link written into the question's retrieval;
- the date rule's estimate is ignored on `ian17`;
- outfit captions (308 left).

### 2026-10-03: Night 8 batch: tests and the probe set against the night 7 default

**Result.** Tests pass, 75 without embed and 80 with (the chronology test now also checks the TIME EVIDENCE block, the place-nation test the data mapping when `place_nation.json` exists), and clippy is clean. The probe questions under the night 8 default against night 7's: `ian17` goes from a decline to "Estimate: between 1086 and 1090, based on T1 and T2"; the rest are untouched by the default (base answers equal the night 7 final default on the 6 probe questions re-run).

**Follow-up.** Still missing: `ian24` (identity labels give a wrong name), `ian30` (only with the opt-in topic package), `ian19` (muddled without `--relation-rule`).

### 2026-10-03: Question-only detector prefilter

**Tried.** `--detect-only` with `--batch` prints, per question, which of the 2026-10-03 question-only detectors fire, without a model.
**Decision.** Used as the prefilter of the A/B runs.
**Code.** `src/bin/ask/cli.rs`, `Args::detect_only`

### 2026-10-03: Route-only batch in one process

**Decision.** `route_only` routes a batch in one process (2026-10-03); the per-question process loads the tools each time.
**Code.** trevor/src/bin/ask/batch.rs, `route_only`

### 2026-10-04: Night 9 batch: the plan and the method

**Tried.** Four requirements for the batch (run 2026-10-03 to 2026-10-04): deeper topic summaries (the Sarkaz and Sami summaries were judged too thin), the Khaganquest (`ian31`), Iris's castle of dreams (`ian24`) and the outfit text pass. The 32 probe questions were answered first with the night 8 default (`ask --no-spawn --no-cache-prompt`, `artifacts/answers/ian-baseline-night9.jsonl`) for the verifier, which has since added a `verified` field to `eval/ian-questions.jsonl` (`design/archive/trevor-ian-questions-verified.md`: 12 right, 12 partial, 1 wrong, 7 declined under that default). As in the night 7 and night 8 batches, each mechanism reads the question or data, never the router prompt; firing was listed without a model (`--detect-only --batch`, `--detect-identity --batch` over the 447 questions of the five sets, the probe set included); base and arms were answered only where they fire, with `--no-cache-prompt`, in one Gemma session, and judged by Qwen in one session. The base arm is the new binary with every new switch off. The deep topics fire only on topic routes, so the 105 questions naming one of the 40 most asked topics were routed first (frozen old binary, `--route-only`).

**Result.** The base arm reproduces the step-0 answers (answer, passages, citations) on the 4 probe questions it re-answered (`ian24`, `ian28`, `ian29`, `ian31`). Of the 105 routed questions, 18 went to the topic tool, 14 of them to a topic with a deep entry.

**Decision.** Answers, judgments and scripts are in `artifacts/answers/night9/`.

### 2026-10-04: Kill switches and the 32-question probe set before and after (night 9 final default)

**Tried.** The final default (term lookup on) answered the 32-question probe set with `--no-cache-prompt` (`eval/ian-questions.jsonl` re-read, with the verifier's `verified` field), compared record by record with the step-0 answers and graded against the verifier's notes.

**Result.** 30 answer records (answer, passages, citations) are byte-identical to the step-0 answers, so the p4-art rebuild changed none of them through the typed fallback. The 2 that changed are `ian28` and `ian31`, the two term lookup changes, and each equals the measured arm. `--no-term-lookup` reproduces the step-0 record on both (2 of 2), and every new opt-in switch off reproduces it on the 4 re-answered in the A/B run.

Graded: the counts stay 12 right, 12 partial, 1 wrong (`ian19`), 7 declined.
- `ian31` stays partial but moves from "a rite of the Kuranta" to the rite of the Nightzmora from its record (still repeating the corpus's "demonic army", which the verifier corrects).
- `ian28` stays partial (Arts talent added; kings, civil war and Londinium still missing under the default; `--deep-topics` adds the King, the Royal Court, Oripathy and exile but also one judge-passed error).
- Under opt-ins: `ian14` answers Pallas 162cm with `--art`; `ian24` has no correct answer under any arm.

The verifier corrected these references ([archive/trevor-probe-questions-verified.md](archive/trevor-probe-questions-verified.md)): `ian11` (the answer is Lucilla, giant phantom jelly, not in game text), `ian31` (Nightzmora are an Elder subgroup of the Kuranta, Kuranta are horse-like, Kharanduu is one Khagan), `ian20` (no halt-and-revival), `ian18` (Ed is a director, from the ARG), `ian14` (unverifiable), `ian16` and `ian10` (need explicit lists).

Tests pass, 76 without embed and 82 with (new: the deep-entry test in `tools.rs`, the term-lookup helper test in `ask.rs`), and clippy is clean.

**Follow-up.** Still missing: `ian24` (the answer model ignores "not playable unit"), `ian29` (routed to the story, so no topic entry reaches it, and the Sami entry says little about guarding), deeper default topic answers (the deep entries lose 2 answers as served), `ian30` (topic package, opt-in), `ian19`.

### 2026-10-04: The wiki becomes an eval reference only: audit and move of every wiki copy

**Tried.** Requirement: the wiki is used as a reference, not served. Audit of every wiki use in `scripts/` and `src/`:
- Serving or building: `design_basis.py` and the `design_basis` tool (the trivia table answered `ian11`); `reading_guide.py`'s main-episode dates (`main_release.json`, read by the reading guide, first appearances, `canon_refs.release_dates` and the tool texts "from the Arknights wiki"); the hardcoded prequel paragraph of `reading_chronological` (community guides and wiki); `deaths.py`'s corroboration (`wiki_deceased.jsonl` and `wiki_status.jsonl`, also used as known names) and the death tool texts; `update.sh`'s `DESIGN_BASIS` step; and, indirectly, `chrono_exceptions_reasoning.py` (it also ran on the wiki's T1 groups, whose rows `chrono.py build` read).
- Eval only: `chrono.py`'s `reference()`, flashback and derive evals (`reference.json`, `timeline.wikitext`), `deaths.py eval`, `wiki_status.py`, and `operator_status.py`'s docstring.
- `data/reading_community.json` is Reddit placements (the wiki was one corroborating source in the research doc).

Every wiki copy moved to `eval/reference/` (git-ignored): `wiki_cache` (407 pages), `design_basis.jsonl`, `wiki_deceased.jsonl`, `wiki_status.jsonl`, `main_release.wiki.json`, `reference.json`, `reference.raw.json`, `timeline.wikitext`, `timeline.wiki.json` (an unread wiki timeline that sat in `artifacts/chrono`). Only `src/reference.rs` names that directory; a new test (`serving_code_never_opens_eval_reference`) scans `src/` and loads the tables from a root holding a reference file. Tool texts say "from the game data"; `ask --wiki-legacy` restores the wiki answers and texts (measurement only).

**Result.** The replacements for each served use are in the entries that follow (episode dates, death confirmation, chrono exceptions, design inference). Tests 78 (84 with embed), clippy clean.

**Decision.** Wiki data is eval-only; serving code never opens `eval/reference/`, enforced by the test.

**Follow-up.** At this point no answer A/B had run (route-only covered 296 of 447 questions; 7 reading routes found so far) and there was no score against the wiki table. The plan: `python3 scripts/design_infer.py check` on Qwen (456 left), `build`, `score` on Qwen, then the A/B (base: the old tables in a scratch root with `ask --wiki-legacy`; arm: the default) on the questions routed to `reading_guide`, `reading_event`, `reading_chronological`, `reading_compare`, `first_appearance`, `dead_operators`, `death_of`, `appearances` and `design_basis`. Both were done the same day (entries below).

### 2026-10-04: The wiki-removal A/B: 0 losses on reading routes, 3 losses on design questions, which only the wiki answered

**Tried.** The remaining 151 of 447 questions were routed (`--route-only --batch --no-cache-prompt`), so all 447 are in `artifacts/answers/wiki-ref/routes.txt`. Base: `ask --wiki-legacy --no-cache-prompt` run from a scratch root whose `reading_guide.json`, `first_appearance.json`, `deaths.jsonl` and `appearances.json` are the `eval/reference/legacy` copies; new: the default with `--no-cache-prompt`; both on Gemma in one session, judged by Qwen pairwise in both orders (a side wins only when both orders agree) and graded alone. Set: 7 of the 8 routed questions plus `ian11` and 4 extra design questions (x0 to x3).

**Result.** Routes over 447: 399 retrieve, 30 topic, 5 `reading_guide`, 2 `reading_event`, 3 `operator_attribute`, 2 recap, 1 each `timeline_all`, `operator_filter`, `is_ending`, `cross_ref`, canon and `appearances`, and 0 to `reading_chronological`, `reading_compare`, `first_appearance`, `dead_operators` or `death_of`; the detectors add `appearances` on `ian32` and `design_basis` on nothing (no table).
- r064, r065, r073, r097 and r113 (`reading_guide`) change only in the release-order lines (Episodes 0 to 4 "by 2020-01-22" against the wiki's 2020-01-16; Episode 5 undated and placed after Episode 4, so before Grani and the Knights' Treasure, against the wiki's 2020-02-26 after 2020-02-05; Episodes 6 to 8 "about"): all 5 ties, both arms graded correct.
- r069, r071 (`reading_event`) and `ian32` (`appearances`) are byte-identical.
- The design questions lose where the wiki answered: `ian11` (graded correct to wrong: the wiki gave Lucilla's giant phantom jelly and Phantom's Phantom of the Opera and Black Cat, the default declines), x0 "What animal is Lucilla based on?" and x3 "What animal is Kirara based on?" (correct to declined); x1 "What is Phantom based on?" is a tie (the default answers his abilities from retrieval); x2 is unchanged.

On the 32-question probe set: `ian32` unchanged, `ian11` lost as above, and the other 30 route to tools and detectors other than the changed ones, so they were not re-answered (not measured beyond the routing). Tests 78 (84 with embed, `serving_code_never_opens_eval_reference` kept), clippy clean.

**Decision.** Wiki serving was not restored (`ask --wiki-legacy` remains measurement only). The design losses are left as an open decision.

**Caveat.** The Episode 5 inversion is not fixable from the game data: no EN excel table holds a timestamp next to a `main_05` or `main_5` reference (all of them searched), so it stays placed after Episode 4.

### 2026-10-04: The wiki is an eval reference only

**Decision.** No serving code names the wiki's directory, and loading the tables never opens it; a test enforces this.
**Code.** trevor/src/tools/tests.rs, `serving_code_never_opens_eval_reference`

### 2026-10-05: A/B method for the 2026-10-05 answer changes

**Tried.** The backlog, in order: a general check of a named answer against the question's category (Bluishsilver, `ian24`); the two deep-topic losses, and routing world questions such as "What is sami guarding?" to the deep entry; a rarity order for the term-lookup glossary; and, with time left, the relation rule's r108 and the topic package's held-out h004 and h017. Method:
- Every A/B ran with `--no-cache-prompt` in both arms in one Gemma session per binary (frozen copies of the binary), on the questions each change touches as the detectors list them (`ask --detect-only --batch` now also prints `constraint=`, `broad_topic` and `relation-group`).
- Changed answers were judged by Qwen twice: by the set judges of earlier runs (responds and faithfulness for real probe, held-out and the extra relationship questions; correct for gold v2 and trivia), and by the pairwise judge of the wiki-ref A/B (`artifacts/answers/wiki-ref/judge.py`: both orders, a side wins only when both agree; the probe-set questions get their `verified` note as the reference).
- A pairwise base win counts as a loss.

### 2026-10-05: Kill switches and the 32-question probe set before and after the 2026-10-05 changes

**Tried.** With the final binary in one Gemma session: the three kill switches together against the base (the old default, run first with `--no-glossary-rarity` on the binary that had the other two opt-in) on 20 targeted questions (the 14 topic-routed questions with an entry, `ian17`, `ian24`, `ian28`, `ian29`, `ian31` and the strongest-playable question), comparing answer, passages and citations. Then the 32-question probe set (`eval/ian-questions.jsonl`) under the final default against night 9's final records (`artifacts/answers/night9/ans-ian-final.jsonl`).

**Result.**
- `--no-answer-check --no-deep-topics --no-glossary-rarity` reproduce the base on 20 of 20.
- The final default equals the measured arms on all 20 (19 equal to the base arm with rarity, `ian28` equal to the gated deep arm).
- Probe set: 30 byte-identical, `ian28` and `ian31` changed, each as measured. Graded against the verifier's notes the counts stay 12 right, 12 partial, 1 wrong (`ian19`), 7 declined. `ian28` stays partial but deeper (racial memories, witchcraft, discrimination, the King and Royal Court); `ian31` stays partial (the glossary now holds Kuranta, which the answer does not use).
- Under opt-ins: `ian24` goes from wrong (Sakiko Togawa) to declined with `--identity-expand` (the check is default); `ian19` goes from wrong to partial with `--relation-rule`; `ian29` reaches the Sami entry with `--named-topic` (partial both ways) at the cost of `ian17`'s pairwise loss.

**Caveat.** Free memory 70 to 75 % at every check, swap 8.6 GB used throughout, no thermal or performance warning recorded; one model at a time, the server and lock released after each of the four jobs. Tests 79 (85 with embed), clippy clean. Answers, pairs and judgments are in `artifacts/answers/night10/`.

**Follow-up.** Still missing: `ian24`'s answer (Bluishsilver, Mabel Grimm), `ian29`'s Collapsals, Myrkwood and Aethergate, a Nightzmora topic for `ian31`, and `ian19` by default.

### 2026-10-05: Cumulative regression check of the default after a week of default changes: real probe responds 73 to 92 of 120 with 1 loss, gold v2 unchanged on 50

**Tried.** The current binary (`target/release/ask`, frozen copy) answered with `ask --no-cache-prompt` and no other flag, in one Gemma session (-np 2, real probe and gold in two parallel batches; gold was stopped at 50 of 146 so that the real probe could finish in the time available). Judged by Qwen with `CORPUS=artifacts/p4x` (`real-probe.py judge`, `answer-eval.py`); the first judge run failed on an 8,017-token request against `-np 2 --kv-unified-per-slot 4096` and was rerun at 16384. Baselines, the earliest comparable judged runs: `artifacts/answers/real-probe-rmodel.judged.jsonl` (2026-09-29 23:19, the model router before the question form) and `gs2_p3b_rmodel.judged.jsonl` (2026-09-30 02:35, 84 of 131 correct, faithfulness 0.963 on the full set); the older `real-probe.judged.jsonl` (09-28) and `gs2.judged.jsonl` (09-27, stories-only corpus) predate the router.

**Result.** Real probe: responds 73 to 92 of 120 (0.608 to 0.767), 20 gained and 1 lost; faithfulness 233/268 = 0.869 to 355/403 = 0.881 (answers are longer: 403 sentences against 268); handles 40 to 37 of 43; not responding 47 to 28; routed to a table 8 to 9; 37 of 120 answers are byte-identical.

Per category (base to new, responds):

| category | base | new | of |
|---|---|---|---|
| A | 5 | 6 | 8 |
| B | 4 | 5 | 6 |
| C | 5 | 5 | 6 |
| D | 6 | 6 | 6 |
| E | 4 | 5 | 6 |
| F | 4 | 6 | 7 |
| G | 5 | 6 | 7 |
| H | 7 | 7 | 10 |
| I | 2 | 2 | 4 |
| J | 1 | 5 | 7 |
| K | 2 | 3 | 5 |
| L | 3 | 4 | 5 |
| M | 5 | 7 | 8 |
| N | 5 | 5 | 7 |
| O | 3 | 3 | 5 |
| Q | 2 | 5 | 7 |
| R | 8 | 10 | 11 |
| S | 2 | 2 | 5 |

No category lost a question on responds except G, which gained 2 and lost r030. By reading, the 20 gains are 7 substantive (r033; r048 now summarizes Dorothy's "Where is the Friend's House?"; r058; r062; r082; r090 names Skadi's first record "Endless Journey"; r114 serves the event summary of Such is the Joy of Our Reunion), 8 declines that now add related context the judge credits (r021, r044, r046, r051, r074, r098, r102, r106) and 5 judge noise (r028, r045, r078 and r094 still decline in substance, r093 had content in both arms). By the form field, 9 of the 20 were sorted opinion, 5 yes_no, 5 fact and 1 went to a table. The one loss is r030 (see the next entry).

Gold v2, g0001 to g0050 only (35 single_fact, 15 entity, all answerable): correct 33 to 33 of 50, 0 gained, 0 lost, 43 of 50 answers byte-identical, faithfulness 66/70 = 0.943 to 72/77 = 0.935, declined 10 to 9.

**Decision.** A measurement of the default only; the one regression is attributed to the question form's opinion branch (next entry).

**Caveat.** g0051 to g0146 (96 questions, 81 answerable) were not answered, so the full-set 84 of 131 is not re-measured. Free memory 28 to 30 % with both batches running and 71 to 75 % between jobs, swap 8.57 to 8.88 GB, pressure level 1 (normal), no thermal or performance warning at any check; one model at a time, each server and the lock released after its job. Outputs in `artifacts/answers/regress-1005/` (answers, judgments, routes, detector lines, the 4 no-form answers).

### 2026-10-05: Gold v2 on the full set: 146 of 146 answered with the current default

**Tried.** The 96 questions the earlier regression run of 2026-10-05 did not reach (`g0051` to `g0146` and the 15 unanswerable) were answered by the default binary of 14:23 (which differs from the 02:28 binary only in the opt-in `--design-deduce`) with `ask --no-cache-prompt` in two streams (about 59 s a question per stream), judged by Qwen with `answer-eval.py` and `CORPUS=artifacts/p4x`, and merged with the earlier run's 50 judged rows. Compared against `gs2_p3b_rmodel.judged.jsonl` (2026-09-30).

**Result.**
- Full set correct 84 to 86 of 131 (0.641 to 0.656): 2 gained (`g0117`, `g0129`), 0 lost.
- Answerable declined 18 to 15; unanswerable declined 15 of 15 both.
- Faithfulness 207/215 = 0.963 to 220/233 = 0.944 (13 unsupported sentences of 233 against 8 of 215; answers are longer).
- Gold chunk among passages 106 and cited 96 of 131.
- On `g0051` to `g0146` alone: correct 51 to 53 of 81, faithfulness 141/145 = 0.972 to 148/156 = 0.949.

**Decision.** The default holds on the full gold set. Outputs: `artifacts/answers/regress-1005/gold146.jsonl`, `gold146.judged.jsonl` and `gold146.judge.out`.

### 2026-10-05: The probe set under the 2026-10-05 defaults

**Tried.** The probe set (`eval/ian-questions.jsonl`, 33 questions with `ian33`) answered under the new default and compared with the previous round's records (night 10).
**Result.** 27 of 31 unchanged questions byte-identical and 4 (`ian2`, `ian13`, `ian15`, `ian28`) reworded with identical passages (concurrent decoding; `ian13` still ends "Estimate: about 15 to 18"). Changed by the new defaults: `ian33` declined to right (Idida), `ian19` wrong to partial and `ian23` a pairwise gain (relation rule). `ian24` is still declined, and `ian12` ("During which event Mon3tr suggested assassinating a child", the same scene) still declines because it names no event to scope to. Counts against the verifier's notes: 13 right, 13 partial, 0 wrong, 7 declined of 33 (`ian19` leaves wrong; `ian33` is new and right).

### 2026-10-06: The probe set under the 2026-10-06 defaults

**Tried.** The probe set (`eval/ian-questions.jsonl`), final default against the old default, answered serially with one client.

**Result.** 32 of 33 answers byte-identical and 99 of 101 prompts identical; the 2 differing prompts are `ian12`'s (declined to right). The 2026-10-05 count against the verifier's notes was 13 right, 13 partial, 0 wrong, 7 declined of 33; with `ian12` moving from declined to right it would be 14, 13, 0, 6, but the other 32 were not regraded.

**Caveat.** The serial old-default answers differ in wording from the night-11 records (`artifacts/answers/night11/ian.new.jsonl`, answered with three clients sharing the server) on 6 of 31 (`ian2`, `ian13`, `ian15`, `ian19`, `ian23`, `ian28`).

### 2026-10-16: Measurement of group evidence, the identity fold and the operator list check

**Tried.** Firing counted without a model (`--detect-only`) on 450 questions. The firing questions plus `r117` (the one other Blacksteel question), 26 in all, were answered with all three switches off and with the defaults, `--no-cache-prompt`, one client, and judged with Qwen pairwise.
**Result.**
- Group evidence fires on 14 of 450 questions (gold v2 6, real 2, held-out 4, probe set 2); the operator gate on 11 (gold 4, real 3, held-out 2, trivia 1, probe set 1).
- Switches off reproduced the old binary byte for byte on `ian34` and `ian35`.
- 4 answers changed: `ian34`, `ian35`, `h005` (both decline, worded differently), and `r055`. `r055` ("Which operator/Rhodes Island NPC ...") was a false trigger on "Dr. Kal'tsit"; the NPC exception fixes it, and `r055` is byte-identical to the base again.
- The Qwen pairwise judge answered "A" in both orders on all 4 changed answers: position bias, not a verdict, so it settles nothing either way.
- Tests: 98 pass, clippy clean.
**Decision.** The three defaults stand, each with a kill switch verified identical to the old binary.

### 2026-10-16: Three-way rung evaluation harness: frozen binary, metered tokens, memory watchdog

**Tried.** A harness to compare the models and profiles that could answer each live rung, all under identical conditions.
- Binary: `target/release/ask` as built 2026-10-16 on the trevor branch (no file under `src/` newer than it; sha256 `0172c1bd86ba768f...`), frozen as `artifacts/answers/eval3way/bin/ask` so that concurrent builds could not change it mid-run. Every arm ran `ask --batch <set> --out ... --no-cache-prompt --profile full|slim` as one client against one llama-server that the arm's script started alone under the model lock file, with `-np 1 -fa on -cram 512 --reasoning off -ngl all --metrics`, and `--kv-unified-per-slot 16384` for the full profile and 8192 for slim. No horizon, so every answer takes the one-shot path that the service takes without a horizon.
- Sets: gold v2 (146 questions: 131 answerable, 15 unanswerable), judged by `scripts/answer-eval.py`, and the real probe (120), judged by `scripts/real-probe.py judge`. Both judged by Qwen3.5 9B with `CORPUS=artifacts/p4x`, after all answer arms, one model loaded at a time (Qwen `-np 2`, two judge processes). The gold verdict is binary (correct or not), so "partial" is not measured. No pairwise judging was used.
- Tokens: llama-server's `/metrics` counters (prompt tokens processed, tokens predicted) read every 0.5 s and attributed to the answer appended in between. With `--no-cache-prompt` every prompt token is processed, so tokens in are the whole prompts of every call an answer makes (router, question form, answer, retries).
- Latency: the wall time between consecutive answers. `ask`'s own `ms` field times only the answer call: 23.8 s of the 33.0 s on the full 12B.
- Watchdog (`artifacts/answers/eval3way/watch.py`), every 5 s: `kern.memorystatus_vm_pressure_level`, the `memory_pressure` free percentage, swap used, server RSS. It kills the client and then the server by PID at pressure level 4, free memory under 10 %, swap grown more than 3 GB over its value at the arm's start, free disk under 12 GB, or any thermal warning.

**Result.** Records in `artifacts/answers/eval3way/`: per arm `gold.jsonl`, `real.jsonl`, `*.judged.jsonl`, `*.judge.out`, `tokens.log`, `mem.log`, `run.log`, `server.log`; `TRIP.1` and `TRIP.2` for the 26B; `summary.json` from `summarize.py`; `compare.py` and `declines.py` for the per-question comparisons; `run_arm.sh`, `run_arm2.sh`, `judge_all.sh`, `watch.py`.

### 2026-10-16: answer-eval.py's decline pattern undercounts E2B's declines

**Tried.** The "declined" column was counted twice: by `answer-eval.py`, and under a broader phrase list (`artifacts/answers/eval3way/declines.py`), shown in parentheses in the rung table.
**Result.** The script's `DECLINE` pattern knows "does not contain" but not "do not contain" or "do not state", which is how E2B words a decline ("The provided passages do not contain information about ..."). So the script scored E2B's declines on unanswerable questions as answers (5 of 15) where 13 of 15 decline in substance (u002, u003 read).
**Caveat.** A decline counts as not correct either way, so correct counts and the real-probe judge (which reads the whole answer) are unaffected.

## Serving and integration

Which model answers each rung, llama-server and the model lock, the worker, profiles, the power guard, caching, the backend and frontend integration, the VPS, timing, memory and heat.

### 2026-09-30: Thinking off in every template render, and `ask` waits for the model lock

**Why.** `ask` hung: it connected to whatever answered on :8081, and at the time that was the canon job's Gemma with `--reasoning on --reasoning-budget 3000`. Two causes. First, the client renders prompts through the server's `/apply-template`, and a reasoning-on server renders Gemma's template with `<|think|>` in the system turn, so the answer begins with a thought channel: run on such a server with the old request, Insider's answer printed "<|channel>thought * Question: ..." and took 46 s. Second (a hypothesis for the hang itself): the server has one slot, so `ask` queued behind the canon job's long reasoning requests.
**Tried.** The client now sends `chat_template_kwargs: {"enable_thinking": false}` with every render. And `ask` no longer shares a server another job owns: while the model lock file names a process that is not `ask` or one of its ancestors, it prints "the model is busy with <label> (pid N); waiting" and waits. `--no-wait` fails at once with that message (verified with a fake lock), `--ignore-lock` restores the old sharing, `--lock` names the file, and a server `ask` starts itself is written to the lock and removed with it.
**Result.** The new request renders byte-identically to the plain request on reasoning-off Gemma and on reasoning-off Qwen, and on reasoning-on Gemma it renders byte-identically to the reasoning-off prompt. On the reasoning-on server the Insider answer is now byte-identical to the reasoning-off answer (28 s). Route check under a wrapper that holds the lock: model 75/76 (Sami), keywords 61/61.
**Decision.** Both are the default. `TREVOR_TEMPLATE_THINK=1` sends the old request; `--ignore-lock` restores server sharing.

### 2026-09-30: The shared model lock

**Tried.** The shared model lock holds `<pid> <label>`. While another job holds it, `ask` waits for it instead of sharing that job's server. A lock held by an ancestor of the process (the script that started `ask`) counts as its own, and `ask` writes the lock itself for a server it starts.
**Result.** `ask` reused a canon job's reasoning-on Gemma and hung.
**Decision.** Waiting is the default since 2026-09-30. `--ignore-lock` shares the server even when another job holds the lock (before 2026-09-30). `--no-spawn` fails instead of starting a llama-server when none answers at `--server`, as before. `--share-server` shares the server under the model lock, never starts one (as `--no-spawn`) and never stops it; off, each process waits for the lock and starts its own server when none answers, as before.
**Code.** `src/bin/ask/cli.rs`, `Args::lock`, `Args::ignore_lock`, `Args::no_spawn`, `Args::share_server`

### 2026-09-30: Thinking off per request

**Result.** A server another job started with `--reasoning on` rendered Trevor's prompts with the thinking turn open, and `ask` hung on it.
**Decision.** Thinking is turned off per request, whatever the server was started with. On a `--reasoning off` server the rendered prompt is byte-identical with or without this (verified for Gemma 4 12B and Qwen3.5 9B). `TREVOR_TEMPLATE_THINK=1` sends the request as before.
**Code.** trevor/src/llm/mod.rs, `Llm::apply_template`

### 2026-10-02: Prompt nondeterminism is llama-server's KV-cache reuse; `--no-cache-prompt` added

**Tried.** `ask --dump-prompt FILE` (new, debug only) writes every request's rendered prompt with the server's `prompt_n`, `cache_n` and output. The 14 probe-set questions were answered twice under `--lore v1` with the same binary in one Gemma session (runs A and B). `ask --no-cache-prompt` (new, opt-in) sends `cache_prompt: false`; runs C and D in the same session, run F in another server session with the new binary and `--lore v1`.

**Result.**
- A against B: 45 of 46 requests had byte-identical prompts, and the one that differed (ian3's answer) differs because the router's OUTPUT differed upstream, `{"source": "any"}` in A and `{"source": "story"}` in B, from an identical router prompt.
- The recorded `promptTokens` is the server's `prompt_n`, the tokens it evaluated, which excludes the prefix it reused from the cache; `cache_n` ran from 0 to 7 tokens per request (226 of 333,762 prompt tokens, 0.068%) depending on what the previous requests had been, so ian1 read 8,820 then 8,816 and ian2 8,718 then 8,716 with identical prompts. That explains the 1 to 2 token gaps seen on 2026-10-02 (ian10 9,139 and 9,137): they are not prompt differences.
- The wording differences are real and have the same cause: a reused prefix shifts the batch split of the rest of the prompt, the logits change in the last bits, and greedy decoding takes another token. In A against B 43 of 46 outputs were identical, and ian3's router flip changed its whole answer. So the 2026-10-01 finding "ian3 moves its source from any to story under --overview" was at least partly this noise: the same binary and prompt gave both sources.
- With `--no-cache-prompt`: C and D were identical in 46 of 46 prompts and 46 of 46 outputs; F equals C in every answer record field but `ms` (14 of 14) and in 46 of 46 prompts and outputs, so outputs are a function of the prompt across sessions and binaries.
- Cost: nothing measurable, 21.47 and 22.80 s per answer without the cache against 21.45 and 22.63 s with it.

**Decision.** `--no-cache-prompt` kept opt-in and used for all A/B measurement from here on, so a route that moves is the prompt's doing, not the cache's.

### 2026-10-02: Unordered iteration on the answer path replaced with BTreeMaps

**Tried.** Trevor's own code was audited for unordered iteration. `Tools::groups` and `Chrono::groups` were HashMaps that `named_group`, the event lookup by name and `latest()` iterated with `find`/`max_by_key`, so a tie (two groups with the same release time or name length) could resolve differently per run.

**Result.** The 14 probe-set questions did not touch a tie: A (HashMap binary) and C (BTreeMap binary) have 46 of 46 identical prompts. Every other HashMap and HashSet on the answer path is a lookup or counter (named_characters, the opinion candidates' per-name counts, short_names), and serde_json maps are BTreeMaps (no preserve_order).

**Decision.** Both are BTreeMaps now.

### 2026-10-02: Run-to-run determinism: prompt dump, no prompt cache, ordered groups

**Tried.** `--dump-prompt` appends every model request's rendered prompt, `prompt_n`, `cache_n` and output to a file (JSON lines), to check that two runs send the same prompts; debug only, the answers are unchanged. `--no-cache-prompt` sends every request with `cache_prompt: false`, so no prompt reuses KV cache from an earlier one and the same prompts decode to the same answers whatever ran before (slower by the reused prefix). Chronology v1 groups by id (`artifacts/chrono/timeline_v1.json`) are now an ordered map.
**Result.** Lookups by name and the latest release iterate the groups, and a HashMap made their ties depend on the run.
**Decision.** `--no-cache-prompt` is off by default. The groups stay ordered.
**Code.** `src/bin/ask/cli.rs`, `Args::dump_prompt`, `Args::no_cache_prompt`; `src/tools/mod.rs`, `Tools::groups`

### 2026-10-02: The prompt cache changes the decode

**Result.** Measured 2026-10-02: the prompt cache changes the logits in the last bits, so the same prompt can decode to other words.
**Decision.** `Llm::without_prompt_cache` exists for runs that need the same prompt to decode the same way.
**Code.** trevor/src/llm/mod.rs, `Llm::without_prompt_cache`

### 2026-10-04: Gemma 4 26B-A4B: fetch, load, speed and memory on the development machine

**Tried.** A bigger local model was approved for the design inference. The file is Google's own QAT Q4_0 GGUF, the same publisher as the 12B: `google/gemma-4-26B-A4B-it-qat-q4_0-gguf` at revision `d1c082be9cf3c8a514acf63b8761f4b41935842e`, `gemma-4-26B_q4_0-it.gguf` saved as `models/llm/gemma-4-26b-a4b-it-qat-q4_0.gguf`, 14,439,363,584 bytes, sha256 `3eca3b8f6d7baf218a7dd6bba5fb59a56ee25fe2d567b6f5f589b4f697eca51d` (verified; downloaded in 22.5 minutes at about 10.7 MB/s). `WITH_LLM_LARGE=1 ./scripts/fetch-model.sh` fetches it pinned and hash-checked (a rerun reports "ok (cached)"). llama-server 0.5.0 loads it with `-np 2 --kv-unified-per-slot 8192 -fa on -cram 512 -ngl all --reasoning off`.

**Result.** Load in 27.0 s. Generation runs at 77.7 tokens/s on a 9-token prompt and 70.6 tokens/s after a 2,518-token prompt; prompt processing at 1,092 tokens/s. A `design_infer` gen call took 9.3 s and 10.5 s (the 12B took a median 20.2 s, two calls at a time in both).

Memory is the problem. The model's Metal buffers are not counted in its RSS (1.1 GB) or footprint (1.5 GB), but system free memory fell from 69 % to 8 % on load (about 14.6 GB of the development machine's 24 GB) and swap grew from 8,620 MB to 12,558 MB in under a minute, with the kernel's pressure level at 2 (warning, never 4, critical), as the other open processes (three node processes at 1.0 to 1.2 GB each, rust-analyzer 0.9 GB, a browser) were paged out. After the stop free memory was back at 70 % and swap stayed at 12.1 GB; thermal state normal throughout (no warning recorded).

**Decision.** By the memory rule (swap growing sharply means stop and report), the server was stopped and not restarted.

**Follow-up.** The 26B needs about 15 GB free before it loads: close the dev servers, rust-analyzer and the browser first (or accept a one-time swap growth of about 4 GB with pressure at warning).

### 2026-10-05: Loading Gemma 4 26B-A4B on the development machine: swap growth, a CPU-only fallback, and the load that worked

**Tried.** Four loads of the 26B-A4B for the design-inference run, under a watchdog that stops the server on critical pressure or swap growth over 2,000 MB.

**Result.**
- Load 1, `-np 2` (free memory 74 %, swap 6,714 MB): swap grew to 9,834 MB within 12 s (+3,120 MB, pressure 2), and the watchdog stopped it before any call.
- Load 2, with the swap ceiling anchored at the 8,162 MB reported just before the run (its own start was 9,254 MB): came up at 8 % free and was stopped at 10,221 MB (+967 MB over its own start, +2,059 MB over the anchor). The server hung in "cleaning up before exit" for 20 s and was killed by PID; swap read 29,628 MB during that teardown and fell back to 12,133 MB within 60 s (pressure 1, free 78 %). Its client, orphaned because the runner killed only the client's shell, exited on its own; the runner now stops the shell's children by parent PID.
- Load 3, CPU-only (`-ngl 0 -dev none`, mmap): never grew swap (11,965 to 8,034 MB) but streamed weight pages from disk (free memory swinging 32 to 72 %): prompt processing 21 to 25 tokens/s, generation 2.7 to 8.6 tokens/s, 110 s and 165 s for the first two operators, about 95 minutes for dev. Stopped by PID and its 2 rows deleted, keeping every scored row from the Metal model.
- Load 4, Metal with `-np 1 --kv-unified-per-slot 8192` once free memory was back at 77 %: came up in 4 s at 16 % free with swap unchanged at 8,078 MB, because the earlier loads had already paged the other apps out.
- While the 26B ran: pressure 1 to 2 (never 4), free memory 10 to 16 %, swap 5,994 to 7,950 MB; every exit after the second load was clean; no thermal or performance warning was recorded at any check.

**Decision.** The design run used the fourth load (Metal, `-np 1 --kv-unified-per-slot 8192`). Ruled out: CPU-only serving of the 26B on the development machine (about 95 minutes for 84 operators).

### 2026-10-05: Memory during the design vision and text runs

**Tried.** Seven design jobs run one model at a time, the server and the model lock released after each (the labelling job was stopped by PID after the labels, its re-scoring redone in the last job).

**Result.** Memory pressure 1 to 2, never critical; swap 13.3 to 16.4 GB total during the vision run (a check at 14:11: llama-server resident 10.9 GB, `-np 4`). No thermal or performance warning at any check.

**Decision.** Because of the swap during the vision run, the text runs after 14:15 used `-np 1`.

### 2026-10-05: Kill-switch verification for scoped scenes and the relation rule; concurrent decoding breaks byte-identity

**Tried.** On the final binary in one Gemma session: `--no-scoped-scenes --no-relation-rule` against the old default (`target/release/ask` of 14:23) on `r030`, `ian19` and `ian33` (serial, `-np 1`); and the round 3 binary with the same switches on 6 questions.

**Result.** Final binary: 10 of 10 rendered prompts and 3 of 3 answers identical, and the default answers `r030` as the old default (the gate is opt-in). Round 3 binary: 20 of 20 prompts identical, answers identical except `r014` once, whose identical prompt decoded differently while three clients shared the server (a serial rerun matched).

**Decision.** With `-np 2` and a third client, `--no-cache-prompt` is not enough for byte-identical answers, so A/Bs compare prompts first.

### 2026-10-05: Memory watchdog for concurrent ask clients

**Tried.** One model at a time throughout, with memory pressure and swap checked.

**Result.** Pressure level 1 at every check, no thermal or performance warning recorded; free memory 19 to 43 %. Swap was 10.9 GB at the start and 15.5 GB at 23:16 while a third ask client and an unrelated backend `cargo test` (rustc 2.3 GB, not Trevor's) ran.

**Decision.** The third client was stopped by PID, and a watchdog (swap over 16,000 MB or pressure critical stops llama-server and the clients by PID) ran from then on; it never fired. Tests 82 + 4 (86 with embed), clippy clean.

### 2026-10-06: Kill-switch verification for character scenes and the narrow opinion gate

**Tried.** Every A/B ran with one ask client against one llama-server (`-np 1`, `--no-cache-prompt`). `--no-character-scenes --no-opinion-gate` on the final binary against the old default (the night-11 binary of 01:33) on `ian12`, `g0049`, `r030` and `r116`.

**Result.** 15 of 15 rendered prompts and 4 of 4 answers identical (also on the first version's binary).

**Decision.** Both kill switches reproduce the old default. Tests 82 + 3 + 1 (86 with embed; `rule_triggers_fire_on_their_questions_only` gained the `asks_scene` asserts), clippy clean.

### 2026-10-06: Memory and heat during the character-scene and opinion-gate runs

**Tried.** The runs of 2026-10-06 with the swap watchdog of 2026-10-05 in place.

**Result.** One model at a time (Gemma 12B for answers, Qwen 9B for judging, four runs), memory pressure level 1 at every check, no thermal or performance warning recorded, free memory 33 to 75 %, swap 12.8 GB at the start and 12.4 GB at the end (under the 16 GB stop; the watchdog never fired).

### 2026-10-06: Memory watchdog during the line-pool and answer-rule runs

**Tried.** A watchdog stops a model session when swap grows by more than 2 GB or memory pressure turns critical.
**Result.** The first Gemma load (kv 16,384) grew swap from 14,245 MB to 16,414 MB in two minutes with pressure at warning, and the watchdog stopped it. Reloaded with `--kv-unified-per-slot 13312`: swap stayed between 16,374 and 17,260 MB (+0.9 GB), pressure 1 to 2, free memory 25 % while Gemma ran and 71 % at the end. Qwen judged at 16,327 to 16,374 MB. No thermal or performance warning was recorded.
**Decision.** The 2026-10-06 measurements ran with Gemma at `--kv-unified-per-slot 13312`.

### 2026-10-07: `ask --serve`: one resident process answering jobs over HTTP, and the worker skeleton

**Tried.** `ask --serve` (design/trevor-integration.md section 8 step 1) loads the tools, the P3b indexes and the llama-server once and answers jobs as library calls over a minimal HTTP JSON API on 127.0.0.1: `POST /v1/ask {question, horizon?, server?, flags?}` and `GET /health` (`src/bin/ask/serve.rs`). The result carries the answer, citations as `{storyId, chunkId, lineStart, lineEnd}`, the passages, the route, the job flags, the form, `server` "en" and `corpusVersion` (`en-e79e364cfbdeeffa` at the time: 16 hex of the sha256 over the P3b and P4x chunks shas). Without a horizon a job takes the one-shot path: route first, P4x swapped in as the main corpus when the route or the source keywords ask for it, then `answer_one`. `ask --worker URL` is the pull-worker skeleton of section 4 (long-poll `worker/next`, `worker/result`, `worker/heartbeat`, `x-service-key` from `TREVOR_SERVICE_KEY`), built but never run against a backend.
**Result.** Equivalence holds: `ian1` to `ian10` through `ask --no-cache-prompt "q"` and through `--serve --no-cache-prompt` in one Gemma session are byte-identical on 10 of 10 (answer text plus the cited `[n] id` lines), with 8 story routes and 2 P4x routes (`ian6` voice, `ian8` operator_file) among them and no table route. Throughput did not improve: serve's 10 jobs took 347.1 s (router 64.8 s, answer 235.1 s) against 321 s of wall clock for the 10 one-shot processes including their loads, 26 s slower.
**Decision.** Built; no `horizon` field is the old behavior, verified by the 10 of 10.
**Follow-up.** The 26 s is unexplained (system free memory was 28 % during the run, against 64 % before it). A repeat with `memory_pressure` sampled would decide whether the resident second corpus costs it.

### 2026-10-07: Hourly question limit

**Decision.** A signed-in user may ask 30 free questions per clock hour (`ASK_PER_HOUR`, set 2026-10-07).
**Code.** backend/src/app/services/trevor/mod.rs, `ASK_PER_HOUR`

### 2026-10-16: Gemma 4 E2B fetched as Q4_K_M for the CPU rung

**Tried.** `WITH_E2B=1 ./scripts/fetch-model.sh` (new) downloads unsloth/gemma-4-E2B-it-GGUF at `0314792d7f1f7e229411f620751375812bb9faf2`, file `gemma-4-E2B-it-Q4_K_M.gguf`, saved as `models/llm/gemma-4-e2b-it-q4_k_m.gguf`.
**Result.** 3,106,738,272 bytes, sha256 `740185b21d22ceb83a11c3aa62ad5842ef32c70f6096d756bbee85a1e4ec34b8` (verified by the script). Disk was 27 GiB free before the download and 24 GiB after.
**Why.** Google publishes E2B only as a QAT Q4_0 (`google/gemma-4-E2B-it-qat-q4_0-gguf`). Q4_K_M is the quant the VPS CPU benchmark used, so it is the one measured.

### 2026-10-16: Which model answers each live rung: 12B full, 12B slim, E2B slim, 26B-A4B

**Tried.** Four arms through the harness above: (a) the 12B on the full profile (the default), (b) the 12B on the slim profile, (c) Gemma 4 E2B on the slim profile, (d) the 26B-A4B on the full profile. Gold v2 and the real probe, judged by Qwen3.5 9B.
**Result.** Gold columns are of 131 answerable; real columns of 120; tokens and latency are per answer over all 266.

| arm | gold correct | declined | wrong | unanswerable declined | gold faithful | real responds | handles | real faithful | latency median / p90 | tokens in / out (mean) | Neurons mean (p90) | peak memory |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a. 12B full (default) | 86 | 15 (18) | 30 | 15 of 15 | 219/233 = 0.940 | 91 | 40 of 43 | 351/402 = 0.873 | 33.0 / 41.1 s | 12,418 / 123 | 116.3 (127.5) | free 17 %, swap +3,266 MB (tripped), retry: free 20 %, +879 MB, level 2, server RSS 9.2 GB |
| b. 12B slim | 40 | 52 (67) | 39 | 15 of 15 | 145/157 = 0.924 | 62 | 42 of 43 | 181/224 = 0.808 | 5.1 / 6.6 s | 1,818 / 47 | 17.8 (21.9) | free 38 %, swap +0, level 1, RSS 8.1 GB |
| c. E2B slim | 42 | 10 (33) | 79 | 5 (13) of 15 | 185/206 = 0.898 | 44 | 37 of 43 | 138/164 = 0.841 | 1.0 / 1.6 s | 1,814 / 39 | 17.6 (20.9) | free 58 %, swap +0, level 1, RSS 4.0 GB |
| d. 26B-A4B full | stopped for memory at load, twice | | | | | | | | | | | free 9 % then 8 %, swap +2,030 MB then +174 MB, level 2 |

- E2B on the slim profile ties the slim 12B on gold correctness (42 against 40 of 131) but answers wrong without declining on 56 of 131 where the slim 12B does so on 25, and it responds to 44 of 120 real questions against 62 (slim 12B) and 91 (the current full 12B).
- The slim profile itself is the larger cost: on the 12B it takes gold correct from 86 to 40 of 131 and real responds from 91 to 62 of 120, because the gold chunk reaches the prompt for 55 questions instead of 106. It buys 6.5 times the speed (median 33.0 s to 5.1 s an answer) and 6.8 times fewer prompt tokens (12,418 to 1,818).
- The 26B-A4B arm was stopped by the memory watchdog at load, twice (free memory 9 % and 8 %), so its quality on the development machine is not measured.
- The Neuron price of a Cloudflare answer, measured from the same prompts: 116.3 Neurons for a full answer (77 answers in a 9,000-Neuron day), 17.8 for a slim one (505).

**Decision.** Recommendations:
- Label the CPU rung (E2B, slim) "quick answer", not "answer". It matches the slim 12B on gold correctness but gives a confident wrong answer about twice as often (56 against 25 of 131) and answers fewer real questions (44 against 62 of 120); against the current answer (the full 12B, 86 of 131 and 91 of 120) it gets about half.
- Keep the Cloudflare rung on the full profile, as contract v1 has it: slim would raise the free day from about 77 to about 505 answers but halve gold correctness on the 12B (86 to 40). If the daily cap binds, a middle profile (k 8, a 4,000 to 5,000-token budget, one call) is the next thing to measure, not slim.
- Measure the 26B (full and slim) before labelling the Cloudflare rung "answer": on the development machine only after the browsers, Discord and other desktop apps are closed, with the same script (`run_arm2.sh d-26b-full models/llm/gemma-4-26b-a4b-it-qat-q4_0.gguf full cf 16384 1`).

### 2026-10-16: E2B against the 12B, both slim: the model alone

**Tried.** Per-question comparison of arms b (12B slim) and c (E2B slim), same retrieval.
**Result.**
- Gold correct 40 to 42 of 131 (6 gained, 4 lost), the same per stratum within 3 (multi_hop 5 to 8, single_fact 13 to 12, entity 10 to 9). The gold chunk is among the passages for 55 of 131 in both, as the retrieval is the same.
- The difference is what each does without the evidence: the 12B declines (67 broad declines, 25 wrong answers that do not decline), E2B answers anyway (33 declines, 56 wrong answers that do not decline).
- A seeded sample of 6 of E2B's 56, read: g0058 says Exusiai is "an owner-founder of Apple Pie Logistics in Laterano" (the 12B: Penguin Logistics, correct); g0089 has Siege send Dagda back to camp (the 12B also wrong); g0045, g0095 and g0097 answer confidently from a part of the passages that does not hold the answer; g0030 names the same village as the 12B (both judged wrong).
- Real questions: E2B opens 67 of 120 answers with a decline and stops (4 of those 67 judged responds), where the slim 12B opens 89 with a decline and adds related context on 36 of them. Responds 62 to 44 (8 gained, 26 lost), handles 42 to 37 of 43, faithfulness 0.808 to 0.841 (fewer, shorter sentences: median answer 114.5 characters against 407.5 for the full 12B).
- Speed on Metal on the development machine: 1.0 s an answer against 5.1 s. On the VPS CPU, the measured 1,814 prompt and 39 output tokens at the benchmarked 49.5 and 4.7 tokens/s on one thread come to about 37 s of prefill and 8 s of generation, about 45 s an answer.

**Caveat.** The VPS figure is an estimate from the measured tokens, not a VPS measurement.

### 2026-10-16: Slim against full on the 12B: the profile alone

**Tried.** Per-question comparison of arms a (12B full) and b (12B slim).
**Result.**
- Gold correct 86 to 40 of 131 (3 gained, 49 lost). Every stratum loses: multi_hop 17 to 5, temporal 11 to 3, aggregation 7 to 2, single_fact 24 to 13, entity 17 to 10, causal 10 to 7.
- Gold chunk among passages 106 to 55; answerable declined 15 to 52; unanswerable declined 15 of 15 in both; faithfulness 0.940 to 0.924.
- Real responds 91 to 62 (5 gained, 34 lost), handles 40 to 42 of 43, faithfulness 0.873 to 0.808.
- Price: prompt tokens 12,418 to 1,818 an answer (the slim profile makes one model call: no router or question-form call; metered tokens in equal the answer prompt on 146 of 146 gold answers), output 123 to 47, latency 33.0 to 5.1 s.

**Why.** The loss is mostly retrieval reach (k 5 and a 2,000-token passage budget against k 8 and 9,000), not the model: E2B on the same passages scores the same gold correct.
**Follow-up.** A middle profile (more passages, still one call) was not measured here (see the middle-profile entry of 2026-10-17).

### 2026-10-16: The 26B-A4B arm does not fit in memory beside open desktop apps

**Tried.** Arm d, the 26B-A4B on the full profile, once both preconditions held (the fp32 re-embed had written `reembed.done`, and `pgrep -l cargo rustc llama-server embed-corpus` was empty).
**Result.**
- First load (free memory 72 %, swap 15,164 MB): 10 s later free memory was 9 %, pressure level 2, swap +2,030 MB, so the watchdog killed the server before the first answer.
- One retry after memory recovered (free 76 %): the model loaded in 3 s (its pages were resident) and free memory fell to 8 % within 5 s (server RSS 14,304 MB, swap +174 MB), and the watchdog stopped it again. Not retried further.

**Why.** With desktop apps open (a browser, Discord, a desktop chat app and five CLI sessions; free memory 72 to 77 % with no model loaded), the 26B's 14.4 GB of weights plus its KV leave under 10 %. The 2026-10-05 pilot ran at 10 to 16 % free only after the Rust and Node processes had been closed.
**Decision.** Parked. The 26B quality question is open; on the development machine it needs those apps closed first.

### 2026-10-16: The 26B arm runs through the cf rung's own code path, with a 16384-token slot

**Tried.** Arm d was set up to be the cf rung's code path rather than the plain llama backend: `ask --backend openai` with `TREVOR_CF_BASE_URL=http://127.0.0.1:8081/v1/chat/completions` and `TREVOR_CF_MODELS=@cf/google/gemma-4-26b-a4b-it` is `OpenAiConfig::cloudflare_from_env`, the configuration the worker's `cf` rung uses (`serve.rs`).
- The chat template is applied by the server as Cloudflare does; thinking is turned off by the request's `chat_template_kwargs`; the 120 s timeout and the model failover are the cf configuration's.
- The router, question-form and flag calls become validated JSON prompts with the keyword router and the fact form as fallbacks, because Cloudflare takes no GBNF grammar. The plain llama backend would have measured the 26B through grammar-bound calls that the Cloudflare rung never makes.

**Result.** The arm used `--kv-unified-per-slot 16384`, not the 8192 in the 26B notes of the developer guide at the time: full-profile answer prompts reach 10,318 tokens (arm a; passage budget 9,000 plus instructions, and up to 400 output tokens), so 8,192 would fail the call on any question that fills its budget and abort the batch. The difference is the KV of the global-attention layers only, an estimated 0.3 GB.
**Caveat.** What stays unfaithful either way: the weights (Google's QAT Q4_0 locally; Cloudflare's serving precision is not published on any page read) and sampling defaults.
**Decision.** The choice stands for a later 26B run.

### 2026-10-16: The cf path's JSON router and form prompts, measured on the 12B

**Tried.** `ask --json-check` runs only the router and question-form calls of each question. It ran over all 266 questions twice on the 12B: on the grammar path (`--backend llama`) and on the cf path (`--backend openai` with `TREVOR_CF_BASE_URL` at the same server). No 26B needed.
**Result.**
- The JSON path made 574 JSON calls with 0 second asks and 0 failures.
- It routes 260 of 266 questions the same (differing: g0117, u004, r033, r068, r070, r095) and gives the same form label on 266 of 266 (gold 146 fact; real 63 fact, 37 opinion, 20 yes_no).
- Its router and form calls (2 a question, 3 when an opinion form gets its second look) cost 3,600 prompt tokens a question against 3,556 (+44, +1.2 %, about +0.4 Neurons) and 30.1 output tokens against 29.7.

**Decision.** The Neuron price of a full answer on the cf path is the grammar path's within 1 %: about 116.7.

### 2026-10-16: What a day of Cloudflare traffic costs in Neurons

**Tried.** Neuron price per answer from the metered tokens of arms a and b, at 9,091 Neurons per million input tokens and 27,273 per million output tokens.
**Result.**
- Full pipeline: 116.3 Neurons an answer on the mean (median 114.7, p90 127.5; input is 112.9 of it), so the 10,000-Neuron free day buys 86 answers and the backend's 9,000 stop 77.
- Slim profile: 17.8 (p90 21.9), so 562 and 505.
- The 2026-10-16 estimates were 123 (full) and 50 (slim at about 4k tokens in). The measured slim prompt is 1,818 tokens because the slim profile makes one call with a 2,000-token passage budget.

**Caveat.** The full-pipeline figure assumes the 26B's prompts tokenize as the 12B's do (both Gemma 4) and its answers run near the 12B's 123 output tokens. Output is 3.4 of the 116 Neurons, so even twice as long answers add about 3.4. Cached and FAQ answers cost nothing and are not in these numbers.

### 2026-10-16: Machine conditions during the rung runs

**Result.**
- Thermal: no warning recorded at any check.
- Disk: 28.3 GB free at the start, 22 to 27 GiB through the run (swap files grew with swap).
- Arm a's first start (15 minutes) tripped the swap rule at +3,266 MB after 28 answers with the re-embed still running (free memory 17 to 29 %, level 2). Its one retry resumed by qid and ran to the end with swap +879 MB at most.
- The Qwen judge ran at 32 % free or more, level 1, no swap growth.
- llama-server aborts on SIGTERM in ggml-metal's teardown (`GGML_ASSERT` rsets count, `ggml-metal-device.m:1025`); the runner's SIGKILL fallback was not needed.
- Every server, client and watchdog was stopped by PID; the model lock was released after each arm.

### 2026-10-16: Passage text trimmed at a line, with a word-break rule

**Tried.** `trim_text` uses `trim_at_line` with the word-break rule chosen.
**Result.** Without the word-break rule, an operator file (a title line, then one paragraph of 1,000 characters) kept only its 25-character title: on P4x, 785 of the 19,604 chunks over 600 characters kept under 100 of them and 2,021 under 300.
**Code.** `src/bin/ask/serve/service.rs`, `trim_text`

### 2026-10-16: Streaming answer deltas to the backend

**Tried.** In `--worker` mode, answer deltas are streamed to the backend.
**Decision.** On by default; `--no-stream` sends the result alone, as before.
**Code.** `src/bin/ask/cli.rs`, `Args::no_stream`

### 2026-10-16: BM25-only serving and a shared query embedder

**Tried.** `ask --retrieval bm25` loads a runtime without the dense index and the embedder. Separately, `ian7` showed that the first `ask --serve` job that loaded P4x or the art corpus spent 21.2 s building a second embedder.
**Decision.** A query embedder is shared across runtimes, reused only for the same model directory, ONNX file and thread count, and only when the new corpus's vectors were made with the tokenizer it was checked against. `TREVOR_SHARE_EMBEDDER=0` loads one per runtime, as before.
**Code.** trevor/src/search/runtime.rs, `Runtime::bm25_only`, `SharedEmbedder`

### 2026-10-16: OpenAI-compatible chat backend

**Decision.** The model client is a local `llama-server` (the default backend, unchanged) or an OpenAI-compatible chat endpoint (Cloudflare Workers AI), added on 2026-10-16 by design/trevor-integration-plan-v3.md phase B step 5.
**Code.** trevor/src/llm/mod.rs, module doc; trevor/src/llm/openai.rs

### 2026-10-16: Streaming answer calls to the worker

**Decision.** When a stream sink is set (`Llm::set_stream`, the `ask --worker` job loop), a call with no grammar and no stop strings (an answer call) streams (`stream: true`) and sends its text pieces to the sink as they arrive; the returned `Completion` is the same. Without a sink every request is sent exactly as before.
**Code.** trevor/src/llm/mod.rs, module doc, `Llm::set_stream`

### 2026-10-16: Worker mode against the myrtle backend

**Decision.** `--worker URL` pulls jobs from a myrtle backend by design/trevor-contract-v1.md section 3 (2026-10-16), starting its own model server when none answers, as before.
**Code.** trevor/src/bin/ask/serve/mod.rs, module doc

### 2026-10-16: Reader unchanged with the Trevor flag off

**Result.** Checked once by hand on 2026-10-16: the reader (the tutorial fixture) and the chapter sheet rendered with the flag off are byte-identical to their render before Trevor existed (24,216 and 14,222 bytes of `document.body.innerHTML`, `cmp` equal).
**Decision.** CI keeps the structural half of that check. The Trevor sheet is built from the kit (`Sheet`, `Tabs`, `Badge`, `Button`) and its tokens only (663d6d3f), so mockups can restyle it without touching the data flow.
**Code.** frontend/src/components/story/trevor/flag.test.tsx (lines 6-10 as of c50d26e1); frontend/src/components/story/trevor/TrevorSheet.tsx (lines 16-17)

### 2026-10-17: Middle profiles for the CPU rung: retrieval budget, not the model, is the lever

**Tried.** The rung comparison found that slim loses to full mainly because the gold chunk reaches the prompt far less often (55 against 106 of 131), and that E2B scores like the 12B on the same passages. So two "middle" arms were run on E2B, both with one model call. Each takes the full pipeline's answer call with the keyword router, `--n-predict 250` and slim's switches off (no typed fallback, answer extension, answer check or operator-list check), raises k to 8 and widens the context budget. Conditions: `--no-cache-prompt`, one client, the same Qwen judge and scripts as the rung comparison (`artifacts/answers/eval3way/e-e2b-mid36`, `f-e2b-mid6k`; `run_arm3.sh` passes the flags). Run time about 1 h 26 min.
**Result.**

| E2B arm | context budget | gold correct / declined / wrong (of 131) | gold chunk in prompt | gold faithful | real responds / handles | tokens in (mean) | Neurons (mean) | median per answer (development machine) | server RSS |
|---|---|---|---|---|---|---|---|---|---|
| slim (rung comparison) | 2,000, k 5 | 42 / 10 / 79 | 55 | 0.898 | 44 / 37 of 43 | 1,814 | 17.6 | 1.0 s | 3,983 MB |
| mid 3.6k | 3,600, k 8 | 57 / 4 / 70 | 82 | 0.900 | 46 / 38 | 3,463 | 32.6 | 2.0 s | 4,019 MB |
| mid 6k | 6,000, k 8 | 73 / 3 / 55 | 101 | 0.930 | 58 / 39 | 5,832 | 54.2 | 3.1 s | 4,013 MB |
| (12B full, rung comparison) | 9,000, k 8, 3 calls | 86 / 15 / 30 | 106 | 0.940 | 91 / 40 | 12,418 | 116.3 | 33.0 s | 9,221 MB |

E2B's declines are undercounted by the script's phrase list (see the decline-pattern entry); correct counts are unaffected.

- On E2B, widening the passages from 2k to 6k tokens takes gold correct 42 to 73 and real responds 44 to 58, and faithfulness rises with it (0.898 to 0.930). That is 85% of the current full 12B gold score from one call on a 2B-class model.
- It still answers wrongly instead of declining far more often than the 12B (55 against 30), so "quick answer" stays the right label.
- Slot sizes: the 6k arm's first run overflowed an 8,192-token slot at 8,596 tokens, because the budget counts passage tokens only. It ran with `--kv-unified-per-slot 12288`. The 3.6k arm fits the VPS unit's `-c 4096`; 6k needs `-c 12288` there, and E2B's RSS barely moves (about 4.0 GB at either size).
- Cost on the VPS CPU (not measured there): prefill dominates. At the benchmark's 49.5 to 94.6 prompt tokens/s on 1 to 2 threads, 5,832 tokens take about 1 to 2 minutes, against roughly 20 to 40 s for slim.
- Cost on Cloudflare: a 6k-style answer costs about 54 Neurons, about 166 answers under the 9,000 stop, against 77 for the full pipeline (assuming the 26B tokenizes like the 12B).

**Decision.** Recommendation: for the CPU rung, replace slim with a mid profile, 6k if the 1 to 2 minute wait is acceptable, else 3.6k. The contract's `slim` profile stays as built until a profile is chosen; a `mid` profile is a small addition to `apply_profile`.
**Follow-up.** For the Cloudflare rung when the daily cap binds, measure a mid profile on the 26B (not run: the 26B did not fit beside the open desktop apps).

### 2026-10-17: Pipeline profiles: full, slim, mid and mid6k

**Tried.** The pipeline profile (`design/trevor-contract-v1.md` section 2; also per worker job). `full` changes nothing; `slim` is the contract v1 CPU pipeline; `mid` and `mid6k` are exactly the flags of the eval arms `e-e2b-mid36` and `f-e2b-mid6k` (`artifacts/answers/eval3way/queue_mid.sh`): `--router keywords --k 8 --n-predict 250 --no-typed-fallback --no-answer-extend --no-answer-check --no-operator-list-check --max-context-tokens 3600|6000` on top of `--profile full`. One answer call, no router call.
**Result.** Gold 57 and 73 of 131.
**Decision.** `mid` (contract v1.1 section 4, the eval's middle arm `e-e2b-mid36`) is the cpu rung's default since 2026-10-17.
**Code.** `src/bin/ask/cli.rs`, `Args::profile`, `Profile::Mid`, `apply_profile`

### 2026-10-17: Stale model lock from a dead process

**Tried.** `read_holder` treats a lock left by a process that died without its cleanup as stale.
**Result.** Seen 2026-10-17: pid 98205 held the lock with no llama-server running.
**Code.** `src/bin/ask/llama_server.rs`, `read_holder`

### 2026-10-17: Resource guard thresholds on macOS

**Tried.** The guard reads memory pressure and thermal state.
**Result.** An idle development machine on 2026-10-17: three Notes, no warning. Level 2 (warning) is the machine's everyday state: it never trips.
**Decision.** The guard trips on critical pressure and thermal warnings only.
**Code.** `src/host.rs` tests, `guard_parses_proc_and_pmset`; `src/bin/ask/tests_worker.rs`, `guard_mac_signals_trip_on_critical_pressure_and_thermal_warnings`

### 2026-10-17: Two workers on `-np 2`

**Tried.** Measured on an Apple M5 Pro (Metal) with E2B and the `mid` profile, 10 gold questions, two runs each.
**Result.**
| setup | answers a minute | median per job |
|---|---|---|
| one worker, `-np 1` | 26.4 and 28.8 | 2.1 to 2.3 s |
| two workers, `-np 2` | 33.8 and 33.4 | 3.6 to 3.7 s |

So +16% to +28% throughput for +60% latency.
**Caveat.** A VPS core without a GPU is not measured.
**Code.** trevor/src/bin/ask/serve/mod.rs, module doc

### 2026-10-17: Question normalization: contract v1.1 and full NFKC

**Decision.** Since 2026-10-17 `normalize_question` runs full NFKC (`unicode-normalization`) before `nfkc_fold`; a decomposed accent composes onto its letter and keys like the precomposed letter (tests pin both). `normalize_question_v1` is the contract v1 normalization (the question as the cache saw it before contract v1.1), kept verbatim for `TREVOR_NORMQ=0`.
**Caveat.** Most of the fold's arms are NFKC mappings already applied; the zero-width/soft-hyphen arm is the one that apparently still changes output. An equivalence proof is still owed before any arm is deleted.
**Code.** backend/src/app/services/trevor/normq.rs, `normalize_question_v1`, `nfkc_fold`, tests

### 2026-10-17: Accounts without read rows ask at gatePos 0

**Result.** 2,628 of the 2,630 local accounts on 2026-10-17 had stage progress but no game read rows: the rows only exist since a sync after 2026-09-24.
**Decision.** An account with stage progress but no game read rows and no site marks would ask at gatePos 0, so `read_stories` handles that case.
**Code.** backend/src/app/services/trevor/ask/progress.rs, `read_stories`

### 2026-10-17: Admission lock and the twin join

**Tried.** `tests/trevor_load_test.rs` sends 2,000 concurrent asks over 50 keys.
**Result.** Without the admission lock all 50 jobs went on a cpu rung capped at 24. Without the twin join under the lock, 24 of 50 keys in a 2,000-ask burst also got a needless deferred job.
**Decision.** Asks are admitted under `admission_lock`, and a twin ask joins the existing job under it.
**Code.** backend/src/database/queries/trevor/admission.rs, `admission_lock`; backend/src/app/services/trevor/ask/mod.rs, `ask` (twin join)

### 2026-10-17: Question chip row

**Decision.** Added 2026-10-17 (bf1553a0). `VITE_TREVOR_CHIP_ROW=0` drops the row, so the chips live in the empty thread only, as before 2026-10-17.
**Code.** frontend/src/components/story/trevor/QuestionChipRow.tsx (lines 12-13 as of c50d26e1)

### 2026-10-17: Stalled answer streams

**Result.** A stalled stream left the answer frozen mid-sentence with nothing to finish it.
**Decision.** `watch.ts` handles a stalled stream since 5d579830 (2026-10-17).
**Code.** frontend/src/lib/trevor/watch.ts (lines 15-16 as of c50d26e1)

### 2026-10-17: Server-function errors returned as data

**Result.** An error thrown inside a server function reaches the browser as a bare `Error` with only its message: TanStack Start's `$TSR/Error` serializer drops `status` and `kind`. Measured in the preview: a 503 arrived as "Trevor is busy" with no status.
**Decision.** The authed calls return a refusal as data (`settle`) and the browser side rethrows it as the `APIError` it was (`unwrap`) (7a47aef8).
**Code.** frontend/src/lib/api/trevor.ts (lines 18-24 as of c50d26e1)

### 2026-10-17: Contract v1.1 response fields and the dev preview

**Decision.** The v1.1 fields of an ask's response were read whether or not the generated type had them yet (removed in 07893abd). The backend names the deferred job `deferredJobId` beside the quick passages job; the dev preview's mock (`scripts/trevor-preview.mjs`) answers the passages inline and flags the ask's own job `deferred`. The preview serves canned, labelled answers with `TREVOR_PREVIEW_REAL=0`.
**Code.** frontend/src/lib/trevor/askState.ts (line 88 as of c50d26e1); frontend/scripts/trevor-preview.mjs (lines 44-47, now split with canned.mjs)
