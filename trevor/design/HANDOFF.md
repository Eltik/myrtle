# Trevor: handoff to Claude Code (2026-09-25)

This is the state of Trevor when the Cowork task that built it handed over to Claude Code. The Cowork task lost its link to Ian's Mac after the Mac was removed and re-enrolled in the Claude desktop app, which is why the work moved here. Everything below was true at handoff; verify anything that matters before acting on it.

## 1. What Ian asked for, in order

1. "Create an LLM that can analyze the Arknights story, answer questions, give summaries, and understand the lore", integrated into myrtle.moe as a new sub-repo. Surfaces he chose: myrtle.moe web chat, a Discord bot command, and enrichment of the existing story reader. Spoilers: gate on the user's synced game progress.
2. Zero cost and no external LLM APIs.
3. Extensive research first (done: `trevor-research.md`), then implementation in "research mode".
4. Phase 0 (ingest, chunk, embed) built, then run for real on his Mac.
5. The search baseline and the retrieval eval harness, then the gold question set.
6. "Is it possible to automate the process instead of manually reviewing?" Yes: the review is automated (`trevor-goldset-automation.md`). He wants no manual review step.
7. Continue here and keep going through the rest of the phases with as much context as possible.

`conversation-log.md` has every message of the Cowork session's second half verbatim, and a detailed summary of the first half.

## 2. Built and verified

**P0 corpus, on the Mac, from the live backend (2026-09-24).** `artifacts/chunks.jsonl`: 13,837 chunks from 1,862 scripted stories (the live index reports `totals.withScript` = 1,862 of 1,887; the "1,797" in backend comments is stale). 0 lossless-reconstruction failures, 0 word-count mismatches. Chunks target 400 to 600 model tokens with a scene cap of 3; 0.43% exceed 600 (single speaker turns are never split); the longest is 1,355. Stored `tokenCount` equals the full encode for every chunk. `unresolved.jsonl` holds 25 stories from two events never extracted by the unpacker: `act24side` "A Flurry to the Flame" (19) and `act13mini` "It's Been A While" (6). That gap is in the assets pipeline, not Trevor.

**Embeddings.** gte-modernbert-base, official INT8 ONNX, CLS pooling, unit length, one text per call. 13,837 x 768 in 1,691 s on the M5 Pro (8.2 chunks/s, about 5 cores, 650 MB), no thermal warnings. Against the fp32 export: cosine 0.997 on Apple Silicon, 0.94 on x86.

**Search.** BM25 (tantivy, lore analyzer that keeps `Kal'tsit`, `Ch'en`, `Ifrit-Nian` whole), exact dense search, RRF fusion (k = 20, unswept), optional Ettin-17m cross-encoder rerank with the scoring head applied in Rust (66,305 parameters; matches sentence-transformers to 6.4e-6 on the Mac). Latency on the Mac: hybrid 15 ms, rerank of 40 passages 550 ms.

**Eval harness.** Line-span anchors, recall@10 as the single gated metric, paired bootstrap, paired t, McNemar and Wilson (all matched to scipy), `eval/runs.jsonl`, tier A golden file `eval/golden-a.json` (recorded on the Mac, reproduced exactly on a re-run).

**First numbers** (dev set of 47 questions Claude wrote, not the gold set; BM25 finds the source first for 27 of 47, so it favours BM25): hybrid recall@10 0.894, BM25 0.883, dense 0.702, hybrid + rerank 40 0.872. Hybrid beats dense clearly (+0.202, paired CI [+0.096, +0.319]); hybrid vs BM25 and the reranker are within noise. Full table: `trevor-retrieval-baseline.md`.

**Local LLM stack.** `brew install llama.cpp` gave 0.5.0 (build 11146; it also upgraded `openssl@3` to 3.6.4). GGUFs in `models/llm/`, sha256-verified: `gemma-4-12b-it-qat-q4_0.gguf` (generator) and `Qwen3.5-9B-Q4_K_M.gguf` (judge).

## 3. The automated gold set

**Status 2026-09-25 (Claude Code):** run on the Mac. v1 (57 items) exposed a generator defect in evidence quotes; `goldset filter --evidence-min-prefix 40` fixes it in the filter (grammar and temperature knobs were built, measured and refuted, and stay off). `eval/goldset_v2.jsonl` (146 items, causal 4 short) is the gate from now on: hybrid recall@10 0.747. Metal check passed (byte-identical with and without `GGML_METAL_TENSOR_DISABLE=1`). Everything is in `trevor-retrieval-baseline.md` sections 4 to 7. The history below is kept as written.

### As handed over: built but never run on real models

`src/goldgen/`, `src/bin/goldset.rs`, `prompts/`, `scripts/goldset-run.sh`. The design is in `trevor-goldset-automation.md`. It was run end to end in the cloud with a 0.8B stand-in model to prove the plumbing (its output passes `eval validate`; the questions were nonsense, as expected from 0.8B). It has not run on the Mac.

Things to check on the first real run, in order:

1. **The M5 Metal bug from the research doc.** llama.cpp PR #28748 (open as of 2026-09-18): on M5, tensor slices past 2 GB could produce wrong matmuls. Before trusting a long run, run one fixed prompt at temperature 0 with and without `GGML_METAL_TENSOR_DISABLE=1` and diff. If they differ, keep the env var set for the run and say so.
2. **The probes.** `goldset probe` renders a prompt through the model's own template, generates one question under the grammar, and asks two self-containedness verdicts (the first should be true, the second false). Check the generator's JSON parses and the judge gets both verdicts right before the full run.
3. **Judge calibration** (`goldset calibrate`, run by the script before judging). If the judge says "answers it" for many random passages from other stories, anchor completion is untrustworthy; report the false-positive rate and its Wilson interval before using the set.
4. **Closed-book rate.** Above 30% of selected questions answered correctly without context means the eval measures pretraining; the pipeline then re-selects preferring unknown items and records both rates.
5. **Shortfall.** Quotas are single_fact 35, entity 25, causal 20, temporal 15, aggregation 10, multi_hop 30, unanswerable 15. If a stratum falls short, `goldset sample` appends a new round; rerun the stages (each resumes).

Then score bm25, dense, hybrid and hybrid + `--rerank-top 40` on `eval/goldset_v1.jsonl`, and write the results into `trevor-retrieval-baseline.md` (the design docs here are copies of the claude.ai project; Ian may want them updated in both places).

## 4. The roadmap after the gold set

From `trevor-research.md` and `trevor-corpus-spec.md`, each phase measured against the eval before it is kept:

- **P1, contextual prefixes. DONE 2026-09-25, not kept:** deterministic prefixes (P1a) +0.018 hybrid only when BM25-only, CI spans zero; prefixing the vectors hurts dense; the model-written P1b pilot adds +0.000 over P1a, so the 9.1-hour run was not spent. See `trevor-retrieval-baseline.md` section 8. Original plan: About 100 tokens per chunk (group and story, position in arc, location from the background asset, speakers present, who "he"/"she"/"the Doctor" refer to, one clause of what happens), prepended before both embedding and BM25. Keep it only if recall@10 improves with a paired CI that excludes zero. Needs a re-embed (28 min) and a BM25 rebuild.
- **P2, summaries. DONE 2026-09-26:** 1,861 story summaries (0.969 of sampled sentences supported) plus the official blurbs, and group summaries for all 88 multi-story groups in methods A, B, C, with C (official blurb plus model summary as input) the default: 0.859 supported, 0.444 of official beats covered. Scene summaries refuted as a retrieval lever. 'Before starting' variant waits on a chronology decision. See `trevor-retrieval-baseline.md` section 9. Original plan: Scene summaries (feed P1), story synopses (1,887; this is the story-reader enrichment deliverable), group summaries (451) plus a "what to know before starting" variant, all embedded into the same flat index. `main_14` is about 135k tokens and needs a long-context pass. Add a coverage counter; faithfulness alone is gameable.
- **P3 identity links + dossiers built 2026-09-27** (`scripts/entities.py`, `scripts/dossiers.py`): 311 verified identity links (0.72 strict precision on 25 read), `who Haruka` -> Momoka; 200 character dossiers, 0.893 of sampled sentences supported. See `trevor-retrieval-baseline.md` sections 11-12. - **P3a DONE 2026-09-26: operator archives indexed** (`artifacts/p3a`, 1,925 chunks): hybrid on story questions unchanged (0.747), archive questions 0.867; ready to adopt as serving corpus, Ian to decide. - **P3, entity index and dossiers.** The speaker index is free and exact (5,911 distinct speakers).
- **P4, answer bank seed of 5,000,** grown later from real miss logs. Do not enumerate 50,000 (19 hours, 62% of the whole build).
- **P4 status (2026-09-28 night, `scripts/answer_bank.py`, results in `trevor-retrieval-baseline.md` sections 16 and 17):** bank at `artifacts/bank/bank.jsonl`, every entry gated by its source stories (`gateStories`, their `requiredStages`, `gateChars` for operator files). Seeded without generation: 200 "Who is X" dossiers (aliases from the current identity links), 1,861 story summaries, 88 event summaries, 83 primers cut to the sentences the support judge accepts (1,090/1,123). Generated: 900 questions from script and archive chunks outside the gold set's stories, answered by `ask` and kept when they cite something and do not decline. 3,107 entries (875 generated, faithfulness 0.970 on a sample of 60), short of the 5,000 target by design; grow from miss logs.
- **Updates (2026-09-28, `design/trevor-updates.md`):** `scripts/update.sh` refreshes the corpus, reuses unchanged vectors and redoes only model units whose inputs changed; `PLAN=1` is a dry run. Death events per story (`scripts/deaths.py`, baseline section 18) replaced the refuted operator table.
- **Real questions and the weak categories (2026-09-28/29, `design/trevor-questions.md` sections 6 to 11):** 1,102 real lore questions (757 r/arknights Lore posts via the Arctic Shift archive, 345 from Ian's Discord lore channel) classified into 20 categories; 54% answerable from Trevor's sources. `ask` now defaults to P3b (P4 for questions naming modules, voice lines, IS, enemies, outfits or items), and answers reading-order questions from a guide (`scripts/reading_guide.py`, wiki episode dates plus the community's essential events and placement rules in `data/reading_community.json`), plot recaps from summaries, roster questions from game data (`scripts/attributes.py`), real names (`scripts/real_names.py`), deaths (`scripts/deaths.py`), with canon, opinion and source policy lines; 61 routing checks (`scripts/route-check.py`). Real probe responds 0.550 (P3a, old) to 0.583; gold set v2 unchanged within noise. Refuted: the decline retry (0 answered in both forms), the answer bank as a serving rung for real phrasing (0.5% of real questions within cosine 0.95), fine-tuning on this Mac (real prompts run out of memory; about 15 to 22 h for a run at 3,000-token prompts; `design/trevor-improvement-research.md`, last section).
- **Corpus-wide routes in `ask`:** the story listing answers from chronology v1; the operator-status route serves only verified rows and otherwise declines, because the summary-built death table was refuted (19 flagged, at most about 5 defensible). A verified table needs each candidate checked against its story's script passages.
- **Answering step built 2026-09-27 (`target/release/ask`, see `trevor-retrieval-baseline.md` section 14):** 0.618 correct on gold set v2 (closed-book 0.130), 15/15 unanswerable declined, 0.958 of answer sentences supported. Its place in this plan: the offline generator for the P4 answer bank and a local tool; the serving design below is unchanged (no live generation on the site's critical path). Still owed from the plan: P4 itself, P2's "what to know before starting" (chronology is now decided: in-world order), spoiler gating, and the hand-built ambiguous-entity and spoiler-boundary gold items.
- **Serving.** No live generation on the critical path: the VPS cannot generate fast enough, and the laptop cannot serve. Ladder: exact cache, semantic cache at cosine 0.95 or higher, precomputed answer bank, retrieval-only answers with citations, then a static "not covered yet". The VPS embeds queries with the fp32 gte export. Spoiler gating filters by the user's synced progress using `spoiler.jsonl` (`requiredStages`, `storySort`, `startTime`; chronology is still an open decision).
- **Integration.** Web chat on myrtle.moe, a Discord command, story-reader enrichment (synopses and "previously on").
- **Behavior-only LoRA** is optional and last; fine-tuning on facts was researched and rejected.

Throughput assumptions in the research doc were for Gemma 4 26B-A4B; with the 12B and 9B on this Mac, measure tokens per second on the first run and re-budget.

## 4b. In-world chronology (2026-09-26)

Ian: chronology is in-world order, not release order; the AI identifies it, framed on explicit years, to answer "timeline history of Terra" and "was X before Y". State in `trevor-chronology.md`: explicit-year Terra history (153 events, 0.926 year-corroborated by the fan wiki), verified time facts for all 1,861 stories, backbone ordering (release order with dated anchors, 0.830 on the wiki reference), reasoning-mode exceptions recorded as annotations (moving them costs 0.019), story flashback tags (0.632 recall, 0 false 'depicts' in 91). The wiki timeline is an eval-only reference.

## 4c. Tools, canon, topics (2026-09-29 to 09-30 night)

State in `trevor-questions.md` section 12. `ask` routes through tools with structured arguments (`src/tools.rs`), picked by one Gemma call (`--router model`, default; 71/72 route check, 0.7 s); `--router keywords` is the kill switch (byte-identical to the old binary); `--router knn` and `--router hybrid` exist but lose route-check cases (30/72 and 61/72). Real probe on the same binary: responds 0.608 against 0.575 for keywords (4 gained, 0 lost); gold set v2 unchanged at 84/131 (the gold set barely touches tables). Paraphrase augmentation for kNN refuted. Topic summaries (46) serve as a labelled passage [1]; the attribution prompt moved attribution phrases 9 to 17 but did not fix the Sami case. Canon: passage-level contrastive picks with Qwen agreement serve (`evidenceScheme: passage`, about 5 of 7 served story pairs right by hand); the wide search and a story-level stage were both refuted (6 of 17 and 2 of 8 right by hand); IS3 and IS4 have no agreed later story; the remaining lever is a larger judge model. The reading guide leads with the game's Storylines links (18). Since 2026-09-30 the model router also names the source of a retrieval question (story, operator file, module, voice, skin, IS, enemy, item, any), replacing the keyword source list: real probe +7/-0 on the moved questions, gold +1/-0 (`--no-router-source` restores the previous default byte for byte).

## 5. Open items

- Canon evidence needs a stronger judge than the 9B to 12B local models (both refuted schemes fail on sequel events full of a run's cast and on shared imagery); IS3 lost its agreed stories under the agreement filter although Highmore's module was graded right.
- Topic summaries state character claims as fact (Sami); a judge that checks who says a claim is owed.

- `-np` scaling on the M5 Pro is unmeasured (watch `llamacpp:n_busy_slots_per_decode` or run `llama-batched-bench`).
- RRF k and candidate depth are unswept; scene cap {3, none} and tail merge are priced but unchosen (`trevor-implementation-notes.md` section 8.5). All wait for the gold set.
- Reranker batching (fp32, so batch-safe) if the gold set shows it earns its place.
- Ambiguous-entity (10) and spoiler-boundary (5) gold items need hand construction.
- Closed-book Arknights knowledge of the answering model is unmeasured and is a model-selection criterion.
- The VPS CPU's INT8 behaviour: measure fp32 against INT8 cosine on 20 chunks there before choosing.
