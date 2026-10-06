# Trevor: retrieval baseline and eval harness

Written 2026-09-25. Covers the search stack and eval harness built after P0, and the first numbers. Code is in `myrtle/trevor/` (not committed); how to run it is in the README there.

## 1. What exists

Hybrid retrieval over the 13,837 P0 chunks: BM25 through tantivy with the lore analyzer, exact dense search over `vectors.bin`, reciprocal rank fusion (k = 20, a knob), and an optional cross-encoder rerank with ettin-reranker-17m-v1. Each stage has an off value that reproduces the pipeline without it: `--mode dense` or `--mode bm25` return that retriever's ranking untouched, and `--rerank-top 0` (the default) returns the fused ranking untouched. `search` and `eval` load through the same runtime, so the eval measures the path users run. Every derived index records the sha of the `chunks.jsonl` it was built from and refuses to load against another.

The eval harness follows `trevor-eval-spec.md`: gold items anchored on line spans and resolved to chunks by overlap; recall@10 as the one gated metric, with hit@1/5/10/20, precision@5 and nDCG@10 (diagnostic) reported pooled and per stratum; Wilson intervals, a seeded 10,000-sample paired bootstrap, the paired t-test and McNemar's exact test, all matched to scipy to 1e-10 in unit tests; a run manifest in `eval/runs.jsonl` with per-question detail; FAIL only when the whole 95% CI of the paired recall@10 difference is below zero; and tier A golden files (corpus facts plus exact top-10 ids for 20 fixed queries), recorded on the Mac as `eval/golden-a.json`. A re-run on the Mac reproduced it exactly, reranker included, and in a cloud test two planted changes were both caught.

## 2. The reranker head, verified

The Ettin ONNX export outputs only `last_hidden_state`, so the scoring head is applied in Rust from the three safetensors files: CLS, Dense 256 to 256 without bias, exact-erf GELU, LayerNorm (eps 1e-5), Dense 256 to 1 with bias. That is 66,305 parameters; implementation notes section 5 said 66,561, which was wrong by 256. Against `sentence_transformers.CrossEncoder.predict` on 33 real (question, chunk) pairs, including the 1,355-token chunk, the largest difference is 4.5e-6 on x86 and 6.4e-6 on the Mac. Pair tokenization matches `transformers` on 33 of 33. The model's context is 7,999 tokens, so chunks are scored whole; the 256-token pair cap in the eval spec's manifest example is not used, and the cap is a flag.

## 3. First numbers, and what they can and cannot say

On the Mac, INT8 query embeddings, over the 47 answerable items of `eval/dev-set.jsonl`:

| config | recall@10 [95% CI] | hit@1 | hit@10 | p50 latency |
|---|---|---|---|---|
| BM25 | 0.883 [0.787, 0.957] | 0.574 | 0.894 | 1 ms |
| dense | 0.702 [0.574, 0.830] | 0.319 | 0.723 | 15 ms |
| hybrid | 0.894 [0.809, 0.968] | 0.511 | 0.915 | 15 ms |
| hybrid + rerank 40 | 0.872 [0.766, 0.957] | 0.532 | 0.872 | 548 ms |

Hybrid beats dense clearly: +0.202 recall@10 with a paired CI of [+0.096, +0.319], 9 questions gained and 0 lost (McNemar p = 0.004). Hybrid against BM25 is +0.021 with a CI of [-0.064, +0.106]: no detectable difference. The reranker is -0.021 against hybrid, CI [-0.106, +0.053], WARN; it rescued 1 question from outside the top 10 and pushed 3 out, and it costs 36 times the latency. Nothing here justifies turning it on yet.

The set cannot settle any of that, for three reasons. It has 47 scored items, so a paired difference must be roughly 9 to 18 points before it clears noise. It was written by Claude reading each sampled chunk, and the questions carry the chunk's proper nouns, so BM25 puts the gold chunk first on 27 of 47; the spec's filter drops items like that from the gold set, and 57% is near its 60% ceiling for a set that can tell retrievers apart. And every item is `review: pending`. What it does show: dense retrieval alone is the weakest leg on this corpus by a wide margin, entity questions are where dense fails worst (it finds 4 of 9), and in the cloud run one question (Siesta's Catastrophe Messenger) was missed by every configuration.

Cloud (x86, fp32 queries) gave the same picture: hybrid 0.904, dense 0.702, BM25 0.883, rerank 0.872, with rerank at 2.5 s per query.

Two numbers in the eval spec were stale: the corpus has 5,911 distinct speakers, not 5,808; and the golden file now records the measured figure.

## 4. Gold set v1, first real run (2026-09-25, on the Mac)

`scripts/goldset-run.sh` ran end to end on the Mac: Gemma 4 12B QAT Q4_0 generates, Qwen3.5 9B Q4_K_M judges, llama.cpp 0.5.0 build 11146, `-np 2`. Wall time 10:53 to 12:14 JST, of which about 30 minutes was the Mac asleep mid-`finish` (requests stalled with 0 tokens decoded and resumed on wake at 12:00:50); awake compute was about 50 minutes. `eval validate` passes: 57 items, 63 gold chunks, sha `76c6bc39aa890615`.

**Metal check (llama.cpp PR #28748).** A fixed 300-token chat completion at temperature 0 on a real chunk is byte-identical with and without `GGML_METAL_TENSOR_DISABLE=1`, on both models, and two default runs are identical to each other. The run used the default (tensor path on). The tensor path is worth keeping: prompt processing 466 vs 227 tok/s on Gemma and 526 vs 317 on Qwen; generation 31.2 vs 33.7 tok/s on Gemma and 45.3 vs 45.4 on Qwen, unchanged within noise.

**Probes.** Both models produce grammar-valid JSON on the first sample item and get both self-containedness verdicts right (true, then false).

**Throughput.** Generation: 280 items in 1,431 s (11.7 per minute on 2 slots, falling from 13.9 as swap grew). Judging: 88 items in 372 s. Anchor completion: about 214 judge calls per minute.

**Funnel.**

| stage | items |
|---|---|
| sampled (round 1) | 280 |
| passed deterministic filters | 88 |
| judged self-contained | 77 |
| passed every judge check | 42 |
| selected | 42 answerable + 15 unanswerable = 57 |

The filter is where the set dies. Of 280 candidates, 132 have BM25 rank the source first (the sole reason for 82), and 101 have an evidence quote that is not in the passage (the sole reason for 48). Deictic wording 9, abstained 7, copied wording 4, unparsed 2. Past the filter, the judge drops 25 because the source does not answer, 11 as not self-contained, and 10 multi-hop items because one passage alone answers them.

**The evidence failures are a generator defect, not bad questions.** Of the 101 failed quotes, 60 are paraphrases or fabrications, 26 run away (the model keeps quoting across speaker lines, appends "Note: ...", or repeats), 13 match for at least 60% of their length and then degenerate (for example "the Wisteria Knight" followed by 70 digits), and 2 differ only in case. 50 of 353 evidence strings (14.2%) end exactly at the grammar's 300-character cap, meaning the model never closed the string on its own. Ruled out by measurement: a double quote in the passage being blocked by the grammar (0 of the 50 capped quotes come from a passage that contains one). The likely levers are a lower evidence cap (the median quote is 68 characters), a generation temperature below 0.7, and matching a quote's longest verbatim prefix. None is built yet, and each changes a prompt or grammar hash, so it starts gold set v2.

**Judge calibration.** On random passages from other stories the judge said "answers it" 0 of 39 times (Wilson 95% upper bound 0.090). On the source passages it said yes 31 of 39 times (0.795), so it under-accepts rather than over-accepts. It passed 0 bad control questions and failed 1 good one. The design called for 60 calibration trials; the run had 39 because calibration draws only from passing single-passage items, and the filter left 39. Anchor completion added 6 extra gold chunks to 4 items.

**Closed-book.** Gemma answered 5 of the 42 selected answerable questions correctly without context (0.119), under the 0.30 gate, so no re-selection was needed. BM25 puts the source first for 0 of the selected single-passage items, by construction of the filter.

**Shortfall.** 93 of the 150 quota: single_fact 25, entity 19, causal 14, multi_hop 15, temporal 11, aggregation 9. Aggregation has 1 item. Ambiguous-entity (10) and spoiler-boundary (5) remain deferred to hand construction.

### 4.1 The evidence fix, measured (2026-09-25)

Two generation knobs were built, both off by default: `goldset generate --evidence-max N --evidence-one-line` rewrites the evidence rule of both grammars (cap N characters, no `\b \f \n \r \t` escapes), and `--temperature T` overrides 0.7. Off returns the v1 grammar bytes (a unit test pins shas `54a50ef9b9f8a61b` and `5dc0bddf0e5670ab`), and rows carry no `temperature` key.

A/B on the same 99 round-1 items (same seeds), one Gemma load:

| variant | evidence fails | quotes at cap | quotes with a newline | unparsed | filter pass |
|---|---|---|---|---|---|
| v1 (cap 300, T 0.7) | 36 | 16 / 125 | 12 | 1 | 30 |
| cap 200 + one line, T 0.7 | 31 | 19 / 120 | 0 | 6 | 31 |
| cap 200 + one line, T 0.3 | 31 | 19 / 120 | 0 | 5 | 29 |

Ruled out by measurement: the grammar cap, the one-line rule and a lower temperature as levers. Filter passes moved by +1 and -1, inside the serving noise measured next, and unparsed multi-hop rows rose from 1 to 6 and 5 (runaway to the 700-token limit). The knobs stay built and off.

Serving is not bit-reproducible: with every knob off, 3 of 20 regenerated rows differ from v1 despite the identical grammar sha, seed and prompt. Two slots batching concurrent requests is the likely cause (unverified). Any A/B on generation carries roughly 15% per-row churn.

What the failures actually are: the model quotes a real stretch of the passage, then continues into the next speaker's line without the "Speaker:" label, glues fragments with "...", or runs off into digits. So the fix belongs in the filter. `goldset filter --evidence-min-prefix N` accepts a quote whose longest verbatim prefix (normalized, cut back to a whole word) has at least N characters, and carries that prefix forward as the evidence (`groundedEvidence` in `filtered.jsonl`, used by the closed-book judge and written to the gold item). Absent, the filter is v1: re-running it on v1's candidates gives a `filtered.jsonl` with the same sha256 (`362c6bf3...`). The price on v1's 280 candidates:

| min prefix | quotes rescued of 101 | items newly passing the filter |
|---|---|---|
| 20 | 72 | 29 |
| 30 | 66 | 28 |
| 40 | 58 | 25 |
| 60 | 50 | 23 |

40 is a TRADE, shipped knowingly: about seven words is long enough that a chance match is implausible, and going to 20 buys 4 more items for much weaker grounding. The curve is monotone, so this is a choice, not an optimum. At 40 the filter passes 113 of 280 instead of 88.

## 5. Numbers on gold set v1

On the Mac, INT8 query embeddings, 42 scored answerable items (15 unanswerable skipped):

| config | recall@10 [95% CI] | hit@1 | hit@10 | recall@20 | p50 latency |
|---|---|---|---|---|---|
| BM25 | 0.694 [0.571, 0.810] | 0.310 | 0.810 | 0.802 | 2 ms |
| dense | 0.437 [0.310, 0.567] | 0.214 | 0.548 | 0.472 | 75 ms |
| hybrid | 0.710 [0.591, 0.821] | 0.310 | 0.833 | 0.813 | 73 ms |
| hybrid + rerank 40 | 0.639 [0.508, 0.766] | 0.452 | 0.738 | 0.706 | 1,248 ms |

Hybrid beats dense clearly: +0.274 recall@10, paired CI [+0.155, +0.405], 12 questions gained and 0 lost (McNemar p < 0.001). BM25 also beats dense: +0.258 [+0.119, +0.405]. Hybrid against BM25 is +0.016 [-0.060, +0.103], 2 gained and 1 lost: no detectable difference, the same verdict as the dev set. The reranker against hybrid is -0.071 [-0.210, +0.071], WARN, 2 gained and 6 lost. It does lift hit@1 from 0.310 to 0.452 (6 more questions at rank 1), so it orders the head well while pushing other gold passages out of the top 10. At 17 times the latency it stays off.

Every absolute number dropped from the dev set (hybrid 0.894 to 0.710, BM25 0.883 to 0.694, dense 0.702 to 0.437). That is what the filter is for: the dev set let BM25 find the source first on 27 of 47, gold set v1 on 0 of its single-passage items. The weakest strata under hybrid are entity (n = 6, recall@10 0.472) and single_fact (n = 10, 0.700), though at these sizes a stratum is an anecdote. Latencies are higher than the dev-set runs (hybrid 73 vs 15 ms) because the machine was under memory pressure from another build, so this is not a like-for-like latency comparison.

What 42 items can settle: a paired difference has to be roughly 12 points of recall@10 before its CI clears zero, so dense against the rest is decided, and hybrid against BM25 and the reranker are not. Until the shortfall is filled, the set gates large effects only.

## 6. Gold set v2 (2026-09-25): the gate from here on

`eval/goldset_v2.jsonl`, 146 items (131 answerable + 15 unanswerable), 181 gold chunks, sha `f12a1600499c9995`, `eval validate` passes. Built as: v1's round 1 (sample, generations, closed-book answers and verdicts copied, since the models, prompts and grammars are unchanged and `generationVariants` in the meta is null), plus a round 2 of 560 passages (`goldset sample --scale 2`, excluding round 1's chunks), with `goldset filter --evidence-min-prefix 40`. Command: `WORK=artifacts/goldgen-v2 OUT=eval/goldset_v2 FILTER_ARGS="--evidence-min-prefix 40" ./scripts/goldset-run.sh`.

Run: 13:19:58 to 18:08:16 JST, with the Mac asleep for most of 14:00 to 16:01 (it was on battery at 16%, where `caffeinate -is` does not hold; once plugged back in the run resumed on its own). Generation 560 in 3,572 s including stalls (8.5 to 9.2 per minute awake), closed-book 245, judging 245 in 820 s, anchor completion at about 369 calls per minute. No thermal warning at any check; memory pressure reached warning level 2 at times while another session compiled Rust, never critical.

**Funnel.** 840 generated, 333 passed the filter (39.6%, as priced), 296 judged self-contained, 156 passed every judge check, 131 selected. Filter losses (first reason): BM25 rank 1 330, evidence not in passage 117, deictic 25, abstained 17, copies wording 15, unparsed 3. The prefix rule trimmed evidence on 150 rows, 54 of which pass the filter, and 24 of the 146 final items carry trimmed evidence.

**Shortfall.** Only causal, 4 short (16 of 20). Every other stratum is full: single_fact 35, entity 25, multi_hop 30, temporal 15, aggregation 10, unanswerable 15. Ambiguous (10) and spoiler (5) are still deferred to hand construction.

**Judge calibration.** 60 random-passage trials, 0 false positives (Wilson 95% upper bound 0.060); source passages accepted 48 of 60 (0.800, v1 0.795); 0 bad control questions passed, 1 good one failed. Anchor completion added 20 gold chunks to 13 items.

**Closed-book.** 0.141 of candidates and 0.130 of selected (17 of 131) answered correctly without context, under the 0.30 gate; no re-selection. BM25 puts the source first for 0 of the selected single-passage items.

**Numbers**, on the Mac, INT8 queries, 131 scored items, no memory pressure during scoring:

| config | recall@10 [95% CI] | hit@1 | hit@10 | recall@20 | p50 latency |
|---|---|---|---|---|---|
| BM25 | 0.681 [0.606, 0.752] | 0.221 | 0.748 | 0.784 | 1 ms |
| dense | 0.566 [0.487, 0.643] | 0.237 | 0.649 | 0.651 | 14 ms |
| hybrid | 0.747 [0.678, 0.812] | 0.336 | 0.817 | 0.833 | 16 ms |
| hybrid + rerank 40 | 0.656 [0.583, 0.729] | 0.382 | 0.740 | 0.746 | 538 ms |

Hybrid beats dense: +0.181 [+0.124, +0.242], 22 gained, 0 lost. Hybrid now beats BM25 too, which neither the dev set nor v1 could show: +0.066 [+0.003, +0.134], 14 gained and 5 lost, gate PASS. It is marginal: the bootstrap CI clears zero by 0.003, and paired t p = 0.051 and McNemar p = 0.064 sit on the line. BM25 beats dense, +0.115 [+0.025, +0.202]. The reranker FAILS the gate against hybrid: -0.090 [-0.173, -0.009], 9 gained and 19 lost, while lifting hit@1 from 0.336 to 0.382. It orders the head better and loses recall below it, the same shape as v1, now outside noise. It stays off, and it is refuted as a recall@10 stage in its current form (40 candidates, Ettin 17m, passage scored whole).

Hybrid per stratum: causal 0.875 (n 16), aggregation 0.800 (10), temporal 0.800 (15), entity 0.773 (25), multi_hop 0.700 (30, hit@10 0.967: one of the two passages is usually found), single_fact 0.671 (35, the weakest). A repeat hybrid run (`gs2-hybrid-strata`) gave identical numbers.

Resolution: a paired recall@10 difference of about 7 points is now at the edge of detectability, against about 12 on v1. P1 and later phases are gated on this set, against the hybrid row.

## 7. What did not move, and what is next

Unchanged by this work: all retrieval code, the dev-set runs (section 3) and the v1 runs (section 5), which stay in `eval/runs.jsonl` (not re-run after these changes). Gold set v1 is left as built. RRF k and candidate depth are still unswept, and reranker batching is untouched (moot unless a different reranker setup passes the gate).

Next: P1 contextual prefixes, measured against hybrid 0.747 on gold set v2, with the 4 missing causal items noted. Optional: the 20-item spot-check in `eval/goldset_v2.spotcheck.md` would bound the automatic review's error rate.

## 8. P1a: deterministic contextual prefixes (2026-09-25)

**Built.** `target/release/build-prefixes` writes a corpus directory (default `artifacts/p1a`) whose `chunks.jsonl` is P0's with a `prefix` on every chunk: group, `StoryCode`, story name and `AvgTag`; "Story N of M" in the group; the chunk's speakers (up to 10); and the game's own story synopsis. The synopsis is the `StoryInfo` archive blurb from `story_review_table`, found for 1,861 of 1,861 chunked stories (median 194 characters). No model is involved. Prefix median 325 characters, max 634; 0 chunks lack story facts or a synopsis. Location is left out because the background plates are mostly generic. `Chunk.prefix` is optional: absent, `indexed_text()` returns the text unchanged and the key is not serialized. The embedder and BM25 index `prefix + "\n\n" + text`; display, reranking and anchors use `text`. `embed-corpus --ignore-prefix` embeds raw text in a prefixed corpus. `build-prefixes --no-story|--no-position|--no-speakers|--no-synopsis` drops single parts.

**Kill switches verified.** A P0 BM25 index rebuilt through the new code reproduces the tier A golden file ("identical", 4 checked). `--ignore-prefix` on the P1a corpus writes a `vectors.bin` with the same sha256 as P0's (`2b002251...`).

**Numbers on gold set v2 (131 scored), recall@10:**

| config | recall@10 [95% CI] | hit@1 | vs P0 same mode, paired |
|---|---|---|---|
| P0 hybrid (baseline) | 0.747 [0.678, 0.812] | 0.336 | |
| P1a BM25 | 0.709 [0.636, 0.780] | 0.328 | +0.028 [-0.003, +0.065], 4 gained, 1 lost |
| P1a dense | 0.478 [0.399, 0.559] | 0.206 | **-0.087 [-0.162, -0.015], FAIL**, 8 gained, 20 lost |
| P1a hybrid (prefix embedded and indexed) | 0.714 [0.644, 0.781] | 0.359 | -0.033 [-0.097, +0.031], WARN |
| P1a split (BM25 prefixed, vectors raw) | 0.765 [0.697, 0.828] | 0.374 | +0.018 [-0.014, +0.053], 4 gained, 1 lost |
| P1a hybrid + rerank 40 | 0.651 [0.578, 0.724] | 0.389 | |

On the dev set (not filtered by BM25), P1a BM25 recall@10 is 0.883 (P0 0.883) with hit@1 0.574 to 0.660; split hybrid is 0.904 (P0 0.894) with hit@1 0.511 to 0.468.

**Ruled out by measurement:** prefixing the embedded text. gte-modernbert with a story-level prefix loses the gold chunk (dense hit@10 0.649 to 0.557) and also the gold story (the gold story in dense top 10 falls 0.763 to 0.679), while the share of the top 10 from the gold story does not rise (0.226 to 0.220). So the harm is not within-story blurring, which was the first hypothesis and is refuted by that last number; the likely mechanism, unverified, is that synopses naming the same main characters pull vectors of different stories together.

**Selection-bias control.** Gold set v2 dropped questions P0 BM25 ranked first, so any BM25 change could regress upward. A placebo corpus with each chunk given the prefix of a random other story scores BM25 0.625, down 0.056 from P0; the real prefix scores 0.709. Arbitrary perturbation does not ride the selection upward.

**What carries the BM25 gain (BM25 recall@10, full P1a 0.709, P0 0.681):** leave one out: no story 0.716, no position 0.709, no speakers 0.705, no synopsis 0.682. Only one part: synopsis 0.709, story 0.692, speakers 0.686, position 0.677. The synopsis carries all of it; the rest is inside noise.

**Verdict.** Not kept as the default: the spec's bar is a paired CI excluding zero, and the best form (split, +0.018) does not clear it. `artifacts/p1a` and `artifacts/p1a-split` stay on disk, and `--corpus artifacts/p1a-split` selects the split form. The official synopses are a free input to P2 (story synopses) and to any P1b prompt.

**P1b (model-written per-chunk referents) is priced, not run.** At gold set v1's measured throughput (280 prompts of about 880 tokens in 1,431 s on 2 slots), 13,837 chunks take about 19.6 hours of Gemma 12B, before `-np` tuning. Given that embedding a prefix hurt dense retrieval here, P1b would be measured split (BM25 only) as well as embedded.

### 8.1 P1b pilot: model-written per-chunk context (2026-09-25), refuted

**Design.** A mini-corpus of 1,793 chunks: every chunk of the stories holding the gold chunks of 40 seeded gold set v2 questions (seed 7), the union of the top-20 chunks from P0 hybrid, P0 BM25, P0 dense and P1a split for those questions (hard distractors), and 300 random chunks (seed 11). Every chunk got a context line from Gemma 4 12B (temperature 0, one line of 40 to 420 characters under a grammar; prompt: the P1a lines plus the passage; instruction: name who is present, resolve pronouns, epithets and "???" to names where the passage or synopsis makes it clear, say where, say what happens in this passage). Prefixes were applied to every chunk regardless of gold status, and every arm was scored on the same 40 questions and the same corpus. Pilot tooling was scratch Python; nothing of it is in the crate.

**Throughput.** 2 concurrent requests: 2.62 s per chunk (573 prompt tokens, 60 generated); 4 concurrent on `-np 4`: 2.51 s, and 2.38 s at steady state over 1,593 chunks. The GPU saturates at 2; 4 buys 4 to 9%. The full corpus would take 13,837 x 2.38 s = 9.1 hours, half the earlier estimate from question generation. Output quality read well: the lines name who is present and what happens in that passage.

**Numbers (mini-corpus, 40 questions, recall@10):**

| arm | BM25 | dense | hybrid |
|---|---|---|---|
| P0 | 0.758 | 0.673 | 0.773 |
| P1a split | 0.771 | | 0.856 |
| P1b split (P1a + context line, BM25 only) | 0.796 | | 0.856 |
| context line only, split | 0.783 | | 0.856 |
| P1b embedded | 0.796 | 0.629 | 0.819 |
| context line only, embedded | 0.783 | 0.606 | 0.869 |

Paired: P1b split against P1a split on hybrid +0.000 [-0.075, +0.075], 1 gained and 1 lost; on BM25 +0.025 [+0.000, +0.075], 1 question. Embedding the context line hurts dense again (P1b embedded dense -0.044 against P0, 2 gained and 5 lost; hybrid -0.037 against P1b split).

**Ruled out by measurement:** the model-written context line as a retrieval gain over the free P1a prefix. The mini-corpus inflates effects (P1a split is +0.083 against P0 here, against +0.018 on the full gold set), so a gain that is zero here is not hiding on the full set. And, a second time, prefixing the embedded text: chunk-specific context hurts gte-modernbert dense retrieval just as the story-level prefix did. The 9.1-hour full run is not spent. Caveat: 40 questions is low power; the refutation rests on the point estimate being exactly zero under conditions that favour prefixes, not on a tight interval.

## 9. P2: summaries (2026-09-25 to 26)

### 9.1 Scene summaries as index units: refuted

Scenes are chunk-sized (13,700 scene boundaries against 13,837 chunks), so the P1b pilot's per-chunk context lines are the spec's one-to-two-sentence scene summaries. Added as separate units in the same flat index (each carrying its chunk's story and line span, and mapped back to that chunk, deduplicated, before scoring), on the P1b mini-corpus (1,793 chunks, 40 questions): P0 hybrid 0.773 to 0.781, +0.008 [-0.071, +0.087], 3 up and 2 down; P0 dense 0.673 to 0.698 and BM25 0.758 to 0.783, one question each; on top of P1a split, hybrid 0.856 to 0.806, -0.050 [-0.125, +0.000], 0 up and 2 down. The mini-corpus favours additions, and even there the units do not pay. Ruled out by measurement: scene summaries as a retrieval lever; the roughly 9-hour generation is not spent.

### 9.2 Story synopses: the game already ships them

`story_review_table` `StoryInfo` points at an archive blurb for 1,861 of 1,861 chunked stories (median 194 characters). They are teasers ("Elysium is in a town working to become a Messenger, but danger creeps near."), not plot summaries, so the reader deliverable is both: the official blurb, and a model paragraph of about 130 words from the full script.

### 9.3 Group summaries: pilot of three methods

449 groups, but 364 are operator-record groups of 1 to 3 short stories, where the story summaries are the summary; 85 main and event groups are the work, median 58,127 tokens, 76 of them over 32k, `main_14` 148,713. So the full-text route is map-reduce: a story summary from each story's full script (largest story 12,928 tokens, so a 16k slot fits all), then a group summary from the story summaries in order (method B). Method A reduces from the official blurbs instead; method C (added after the pilot) from both side by side.

Pilot on `main_1` (8,469 tokens), `main_5` (24,536), `act11mini` (33,770), `act13d5` (58,127, 19 stories), Gemma 4 12B generating, Qwen3.5 9B judging. Map: 54 stories in 454 s, 8.4 s per story, 125,024 prompt and 9,130 generated tokens. Reduce: 13 to 24 s per group.

Instrument, and one refutation of it: faithfulness was first measured by having Qwen decompose each summary into atomic claims under a JSON-array grammar. It produced fragments ("#user", single words) and, once, a single claim for a 323-word summary, so those numbers ("4/46 supported", "1/1") were measuring the decomposer and are void. The judge now splits sentences deterministically (abbreviation-aware after a second bug: "Mr." was ending sentences) and judges each against the top 5 BM25 chunks of the source plus neighbours. A control judges 10 sentences from another group's summary against this group's evidence. Coverage has two parts: each story's official blurb judged as an event the summary covers, and how many of the 5 most frequent playable operators the summary names (a first operator list from the gold-set tooling held 315 names and lacked Amiya, so it was replaced by `character_table`, 1,053 names; NPC leads such as Talulah are not counted).

| method (4 groups) | sentences supported | official beats covered |
|---|---|---|
| A, from official blurbs | 53 / 72 = 0.736 | 51 / 54 = 0.944 |
| B, from model story summaries | 64 / 72 = 0.889 | 27 / 54 = 0.500 |

Control false positives: 1 of 40. Story summaries (12 sampled): 77 of 82 sentences supported (0.939), mean 128.5 words; two of the five misses were the "Mr." splitting bug.

The trade: A covers the official beats by construction but pads vague blurbs with detail the script does not support; B is grounded but, compressing 19 stories into 274 words, drops half the beats. Method C gives the reducer both inputs for the same cost. The full run writes A, B and C for every multi-story group, and the method shipped as the default is chosen from the judged sample in 9.4, not from this pilot.

### 9.4 The full run and the choice of method (2026-09-26)

**Run.** `scripts/p2-run.sh` (Gemma 4 12B, `-np 2`, 16k per slot; pauses on battery, logs thermals every 10 minutes): 1,861 of 1,861 story summaries in 21,682 s (11.2 to 11.8 s per story, slower than the pilot's 8.4; cause not established), 6,842,644 prompt tokens; then 264 group summaries (88 multi-story groups x methods A, B, C) by 05:59, prompt shas `ec2defec3214ba32` (story) and `30983030cd804936` (group). On AC throughout, no thermal warning at any 10-minute check, memory pressure between 1 and 2, never critical. Story summaries: 108 to 175 words (median 132), 0 at the 320-token cap, 0 degenerate or preambled, one model and one prompt sha on every row. Output: `artifacts/p2/stories.jsonl` (official blurb and model summary per story), `artifacts/p2/groups.jsonl` (`method` A, B or C). The group stage sends one request at a time and leaves a slot idle; parallelising it would roughly halve its 75 minutes.

**Judge.** `scripts/p2-judge.py` on Qwen3.5 9B, seeded sample of 8 groups (every method each) and 30 stories. The first launch at 15 groups and 60 stories ran at about 3.7 minutes per group unit (up to 15 evidence chunks, about 7k tokens, per sentence verdict), about 3.5 hours in all, so it was stopped after 8 units and restarted on the first 8 and 30 of the same seeded shuffles; rows already judged stayed valid. 07:37 finish.

| method (8 groups) | sentences supported | official beats covered | top-5 operators named | mean words |
|---|---|---|---|---|
| A, official blurbs | 96/144 = 0.667 | 83/108 = 0.769 | 24/35 | 315 |
| B, model story summaries | 106/122 = 0.869 | 29/108 = 0.269 | 31/35 | 300 |
| C, both | 140/163 = 0.859 | 48/108 = 0.444 | 31/35 | 386 |

Control false positives: 1 of 80 (method B units). Story summaries (30): 190/196 = 0.969 sentences supported, 60/79 top operators named.

**Decision: method C is the default group summary.** It keeps B's faithfulness (0.859 against 0.869, within a few sentences) and covers 19 more official beats (48 against 29 of 108). A's beat coverage is inflated by construction (the beats are its own input), and a third of its sentences are not supported by the script. Per group, C is at or near the best faithfulness in 6 of 8 and never the worst on beats.

**Limits, stated.** Beat coverage of C is 0.444: a summary of 300 to 400 words over a 19-story event omits most story-level beats; a longer group summary is the obvious lever and is untested. Faithfulness is measured against 5 BM25 chunks plus neighbours, which undercounts support for sentences that synthesize across stories, so all three methods' faithfulness is a lower bound. Operator coverage ignores NPC leads. No human spot-check yet: the spec asks for 30 to 50 human-judged summaries to anchor the judge (kappa), which needs Ian.

**Deferred:** the "what you need to know before starting" variant needs a chronology decision (release order, in-world order, or per arc), which is Ian's call.

## 10. P3a: operator archives as a lore source (2026-09-26 night)

`target/release/build-archives` writes `artifacts/p3a`: the P0 `chunks.jsonl` byte for byte (source sha `3d7133d2...`), then the operator archives from `handbook_info_table` (basic info, profile, physical exam, clinical analysis, archive files 1 to 4, promotion record), each operator's sections packed in order into chunks of at most 600 tokens with section titles kept: 417 operators, 1,925 chunks, 861,531 tokens, largest 611 (the "Operator archive: name" header is counted after packing; far under 8,192). Token counts are the tokenizer's full encode. BM25 2.6 s; embedding 15,762 chunks in 1,760 s.

**No regression where it is served.** Gold set v2 (131 story questions) on P3a against P0: hybrid 0.747 to 0.747 (+0.000, 0 gained, 0 lost), dense 0.566 to 0.566 (+0.000), BM25 0.681 to 0.650 (-0.031 [-0.061, -0.008], 3 lost, gate FAIL). Archive rows compete for the same names in BM25 alone; fused with dense they never push a story passage out of the top 10.

**The archives are findable.** An archive slice built with the same automated pipeline (`WORK=artifacts/goldgen-arch OUT=eval/goldset_arch CORPUS=artifacts/p3a`; 80 sampled archive chunks, seed 20260926): 15 of 80 pass the filters, 44 failing first on "BM25 ranks the source first": archive facts sit under the operator's name and are keyword-easy. Judge calibration 0 of 15 false positives, 15 of 15 sources accepted; closed-book 0.067. On the 15 hard questions that remain (8 entity, 7 single fact): hybrid recall@10 0.867 [0.667, 1.000] (hit@1 0.600), BM25 0.683, dense 0.683. Without the archives these questions have no answer in the corpus at all.

Verdict: ready to adopt as the serving corpus (hybrid unchanged on stories, archive lore now reachable); the default `--corpus` stays P0 until Ian decides. The runner fix found on the way: `goldset-run.sh` validated a new set against the P0 corpus; it now validates against `$CORPUS` (default unchanged).

## 11. P3: who is who (identity links), 2026-09-26 night

`scripts/entities.py` (corpus `artifacts/p3a`): lines stating an identity ("real name", "codename", "also known as", "call me", "born", "stage name", "under the name", ...; 2,004 lines) go to Gemma 4 12B with a grammar; a link is kept only if its quote is in the lines and both names appear there. `build` unions links into identities keyed by the speaker label with the most chunks; `who NAME` prints the identity and its evidence.

Two recall failures found on the acceptance case (Ian's "Haruka" question) and fixed. The first cue list lacked "born" and "stage name", so the archive line "Haruka, born Haruka Shino, ... performing under the stage name Momoka Hanyuu" was never read. With the cues widened, Gemma still returned an empty list for that line; one worked example in the prompt (a profile-style sentence and its two links) fixed it, and on 7 test lines kept every link the old prompt found and added Catapult ~ Arleta. Final run (prompt `35d65d46bf9402f2`): 311 verified links from 2,004 lines (83 rejected by the checks), 183 identities from 474 names, 167 labelled by a speaker. `who Haruka` -> Momoka (speaks in 66 chunks): Haruka, Haruka Shino, Momoka, Momoka Hanyuu.

Precision, 25 seeded links read by hand: 18 correct (Siege ~ Vina, Figurino ~ Luchino de Montano, Pohl ~ Mitm, Allerdale ~ Duke of Cumberland, Mercia ~ Merry, ...), 2 wrong (a form of address "Shih-fu"; "child of the House of Rostov"), 5 uncertain: 0.72 strict. Union-find amplifies single bad links: 2 identities exceed 6 names (the Kal'tsit group absorbed Raidian and Serafina; the Fiammetta group collects nicknames). `who` should rank direct links first; not yet done.

Ruled out on the acceptance case: alias expansion of the search query. "What happened to Haruka's parents?" on P3a already finds `act44side_05_beg#0004` (rank 2) and `#0003`, the parents chunk (rank 9), where P0 found only `#0004`; appending "Momoka" pulls in general Ato scenes and drops `#0003`. One case, so an anecdote, but it points the identity index at answering "who is X", not at rewriting queries.

## 12. P3 dossiers (2026-09-27 night)

`scripts/dossiers.py` (prompt `001ae818c327e653`): 200 characters, the speakers with the most story chunks among operators, identity labels and names speaking in 4+ groups, minus generic role labels (a few borderline labels remain: "Shieldguard", "High Priest", "Doc"). Inputs per character: the operator archive when there is one (127 of 200), the P2 summaries of the 12 stories they speak most in, ordered by chronology v1, and their known names. Gemma 4 12B, 16.1 s each, 200 in about 54 minutes; median 280 words. Output `artifacts/dossiers/dossiers.jsonl`.

Faithfulness (Qwen3.5 9B, each sentence against the top 5 BM25 passages of the character's own story chunks and archive, plus neighbours; seeded sample of 30): 349 of 391 sentences supported (0.893); 10 of 30 dossiers fully supported; worst Specter 9/14, Toland 9/13, Czerny 8/11. The unsupported set mixes likely-true sentences outside the evidence window (Toland's specific Near Light chapter roles, drawn from story summaries) with likely conflation (Specter "trained by Gladiia", "met Skadi, who taught her to dance"). As with P2 this is a lower bound, and no human has checked a dossier yet.

Instrument note: the sentence splitter also broke at "Mt." (a dossier sentence was judged as the fragment "... at Mt."), so the 0.893 slightly understates the dossiers. "Mt.", "No.", "Vol." and a duplicate "Ms." guard are now in both `dossiers.py` and `p2-judge.py`; the numbers above were measured before the fix and are left as measured. `design/review-sheet.md` holds 32 items (dossier sentences with the judge's verdict, identity links, derived years, two event summaries) for a human check of the automatic judges.

## 13. Ian's review of the judges and outputs (2026-09-27)

`design/review-sheet.md`, filled in by Ian; blanks are items he was unsure of.

**Dossier sentences (judge vs Ian):** 6 of 12 marked, and Ian agrees with the judge on all 6 (2 judged not supported, 4 supported). Six is small; it says nothing against the judge.

**Identity links:** 6 correct, 2 wrong (Allerdale = Duke of Cumberland; Miss Bagpipe = "Victoria's nomadic villages"), 2 unsure: 0.75 on the decided 8. Relation labels are unreliable: Nearl = Radiant Knight of Kazimierz is right as a link but is a title, not a former name. Fiammetta genuinely went by many codenames, so "more than 6 names" is not an error signal and is no longer reported as one.

Fixes from the review, both applied at build time without re-extraction:
- A name must be a name: two or more lowercase words make it a description ("Victoria's nomadic villages", "child of the House of Rostov"). 18 links dropped.
- The quote must itself state the link, with an identity cue or both names. "I can see you, Raidian." only addresses someone, and had merged Raidian and Serafina into Kal'tsit. 54 links dropped.
- Result: 159 identities from 383 names (was 183 from 474), none with more than 6 names. On Ian's 10 reviewed links: both wrong ones dropped, 5 of 6 correct kept, the unsure ones split; the correct one lost is Fiammetta = Hardship Overseer ("knows my little Fia as the Hardship Overseer": a nickname, no cue), a trade. These rules were checked on the same items that prompted them, so this is a sanity check, not a held-out measurement. `who Haruka` still -> Momoka; Kal'tsit's group is now AMa-10, Calcite, Dr. Louisa, Kal'tsit, Louisa.

## 14. The answering step (2026-09-27)

**Corpus switch.** Search, eval and ask now default to `artifacts/p3a` (stories plus operator archives); `--corpus artifacts` is the stories alone. Gold set v2 validates on P3a (146 items). The P0 golden file moved to `eval/golden-a.p0.json` and still reproduces with `--corpus artifacts`; `eval/golden-a.json` was re-recorded on P3a and reproduces. The gold-set pipeline pins its own corpus (`CORPUS`, default `artifacts`), so its datasets stay reproducible.

**`target/release/ask`.** Hybrid retrieval (k 8), each hit widened by one neighbouring chunk on each side in the same story (`--neighbors`, 0 turns it off), capped at 9,000 passage tokens, numbered passages labelled with event and story; Gemma 4 12B at temperature 0 answers only from them, cites [n], and declines when they do not hold the answer. `--batch` answers a gold set. About 16 to 21 s per question on the Mac.

**Eval** (`scripts/answer-eval.py`, Qwen3.5 9B judging; gold set v2, 131 answerable and 15 unanswerable):

| measure | result |
|---|---|
| answerable, correct | 81 / 131 = 0.618 (closed-book, same model, no retrieval: 17 / 131 = 0.130) |
| unanswerable, declined | 15 / 15 |
| answer sentences supported by the given passages | 203 / 212 = 0.958 |
| citations pointing at no passage | 0 |
| gold chunk among the passages given | 105 / 131; cited 94 |

By stratum (first scoring): aggregation 7/10, causal 10/16, entity 16/25, multi_hop 14/30, single_fact 22/35, temporal 10/15. Of the 50 misses, about half are retrieval (the gold passage never reached the model: 24 in the first scoring) and half generation (gold present, answer wrong or declined).

Eval defect found and fixed: the first scoring counted any answer containing "does not state ..." as a decline and forced it wrong (0.603). A decline now has to open the answer (first sentence) and be confirmed by the judge; re-judging the 23 affected answers gave 0.618.

**Refuted: `ask --partial`** (answer the covered part instead of declining; off by default). On 47 items: of 10 over-cautious declines with the gold passage present it rescued 1; of 22 correct answers it lost 1; and 2 of 15 unanswerable questions were no longer declined. Net worse, and answering unanswerable questions is the costlier error; it stays off.

Next levers, by size: retrieval misses (about 24 of 131: the gold passage never reaches the model; k, neighbours and the P1a split all bear on this) and multi-hop questions (14 of 30).

## 15. Answering fixes and primers (2026-09-27)

**Fixes for two failures Ian found.** (1) "Rank the most powerful Sui" declined: no passage ranks them, and `ask` could not search the dossiers or event summaries. (2) Asked for years, `ask` gave none: it never saw Trevor's chronology, and Episodes 0 to 9 state no year anywhere in the script (0 year facts, 0 dated lines apart from a passing 937 in Episode 6); the main chapters were also missing from the timeline, whose backbone keys on release dates they do not have.
- `target/release/build-profiles` writes `artifacts/p3b`: P3a plus 288 profile units (200 dossiers, 88 method-C event summaries), 16,050 rows.
- Timeline v1 now places main chapters by chapter order (in-world order); Episodes 0 to 9 are bounded "at most about 1098, before Episode 10; the script states no year", 15 and 16 "at least about 1100".
- `ask` routing (`--no-route` turns it off): time questions get timeline notes listed in in-world order (event, main-story episode, storyline year or bound with its basis, the story's dated and derived lines); comparison and overview questions get a synthesis instruction (say what the story states and what is inferred). `ask` also starts and stops its own server when none runs (`--no-spawn` restores the old error).

**Measured, gold set v2:** retrieval on P3b (story passages only) 0.747 to 0.732 (-0.015 [-0.038, +0.000], 2 lost, WARN): profiles displace story passages for 2 questions. Answers on P3b with routing: 84/131 correct (0.641, was 0.618), paired +0.023 [-0.015, +0.061], 5 gained, 2 lost; unanswerable still declined 15/15; faithfulness 0.966. By stratum: entity, multi_hop and single_fact +1 each; aggregation, causal, temporal flat. Positive, not significant, no measured cost.

On Ian's two questions: the ranking question now answers instead of declining, but reads "Sui" as the entity rather than the siblings (retrieval returns act49side's passages about It, not the siblings' profiles). The years question now gives the Sui at about 980 to 1000 (the wiki's "beginning of Sui's revival" is 982) and bounds the two deaths at most about 1098; whether it orders FrostNova before Patriot depends on whether retrieval returns Episode 6 as well as Episode 7 (it did on P3a, not on P3b).

**Primers ("what to know before starting", `scripts/primers.py`):** 105 targets (main chapters and multi-story events); prior context = the six earlier groups in in-world order sharing the most recurring speakers, from their method-C summaries, plus the target's official opening blurb. Judge sample of 20: faithfulness to the given summaries 199/215 (0.926); the leak judge flags 61/215 sentences (0.284) in 17 of 20 primers as revealing the target's own plot. Reading 10 flagged sentences: about 4 genuine spoilers (act7d5 "cross paths with Blaze"; Near Light's primer on Margaret entering the Major), about 3 the target's opening situation, about 3 generic recaps, so the real rate is lower than 0.284 but still too high for a spoiler-free primer. Likely cause: "earlier in in-world order" admits groups released after the target, whose summaries recap it. Next fix: prior context must be both earlier in-world and released no later than the target.

**Known failure class, reported by Ian (2026-09-27): corpus-wide enumeration.** "What playable operators are confirmed to be dead?" was answered from 10 retrieved passages: it named Miarow (a character in Operation Originium Dust, not a playable operator) and ended "No other playable operators are explicitly stated to be dead in the text", a completeness claim no passage sample can support. Three defects: (1) enumeration over the whole corpus cannot come from top-k retrieval; it needs a structured per-character fact table (playable or not, from `character_table`; status with evidence, extracted offline from dossiers and story summaries) that `ask` answers "list all" questions from; (2) the model is not told which characters are playable operators; (3) the prompt does not forbid completeness claims ("no other ...") drawn from a sample. (3) is a one-line rule; (1) and (2) belong with P3's entity index. Not yet fixed.

Second example of the same class (Ian, 2026-09-27): "Give me every story in chronological order" listed six items from 16 retrieved passages, several of them sub-stories of one event (Rewinding Breeze), and presented that as the order. The complete answer already exists as data: chronology v1 orders every event and main chapter (449 groups) with a year or bound and its basis. The fix is a route in `ask`: whole-corpus listing questions ("every story", "all events", "full timeline", "which operators ...") skip generation and answer from the structured table (timeline v1 now; a per-character fact table for status questions, still to build), saying which years are estimates. Not yet built.

**Primers v2 (release rule + stricter prompt), 2026-09-27.** Prior context now also requires release no later than the target (13 of 85 prior sets changed, so release order was not the main leak source); the prompt (`e25de30925a96ee6`; `PRIMER_PROMPT=v1` restores the first) confines the target to its opening blurb and asks for past-tense sentences about earlier events only. Paired on the same 20 primers: faithfulness 199/215 (0.926) to 243/253 (0.960); judge-flagged leaks 61 (0.284) to 79 (0.312).

The leak number is an instrument defect, not a finding. Of 8 v2 sentences the judge flagged, about 5 are the target's own official opening blurb, which a primer is meant to state (Episode 5's primer quotes its blurb "Ch'en, who just woke up from a nightmare, finds herself inside the Rhodes Island landship ..." verbatim), about 2 recap earlier events, and about 1 may be genuine (Babel's primer names three deaths). The leak judge sees the target's summary but not its blurb, so blurb content reads as a leak. Next: give the leak judge the blurb and exclude blurb content, re-judge v1 and v2 paired (about 20 minutes), and only then decide whether v2 is spoiler-safe. v1 is kept in `artifacts/primers.v1/`.

**Leak judge with the blurb, paired re-judge (2026-09-27).** The leak judge now sees the target's opening blurb and is told blurb content is not a leak. Same 20 primers: v1 flags 66/215 sentences (0.307), v2 62/253 (0.245); faithfulness unchanged (v1 0.926, v2 0.960). The judge still over-flags: of v2's 62 flagged sentences, 60 are also judged supported by the prior summaries, which by the release rule come only from groups released no later than the target, so they recap earlier content the target's summary repeats (Blaze and Kal'tsit sentences flagged were blurb text, still). Leak rate by the defensible measure, flagged and not supported by earlier material: v1 3/215 (0.014), v2 2/253 (0.008). v2's two are genuine Near Light leaks (Maria defeating Olmer Ingra; Sona and Greynuty Kaliska against the Armorless Union). v2's 10 unsupported sentences also hold inventions ("the death of Emperor", "the tragic sacrifice of Allerdale Cumberland"). Decision: v2 primers enter the answer bank with only the sentences the support judge accepts (`answer_bank.py primers`; `PRIMER_FILTER=0` keeps whole primers), which removes both leaks and both inventions in the sample. The leak judge as an instrument is refuted for this use: without the prior summaries it cannot tell a recap from a leak.

## 16. Corpus-wide routes (2026-09-27 night)

**Listing route ("every story in chronological order", "full timeline", "all events").** `ask` answers from chronology v1 without generation: the main story as its own sequence (17 episodes, in-world order = episode order), then the 85 events in estimated in-world order with year or bound and basis, and a count of the 364 operator records not listed. The main story is kept apart because Episodes 0 to 9 carry only an upper bound ("at most about 1098"); sorted among events by year they would land after every 1097 side event, an order the script never states. Retrieval gave six items for the same question. `--no-route` restores retrieval.

**Completeness rule.** The system prompt now forbids claiming a list is complete or that nothing else exists (Ian's Miarow answer ended "No other playable operators are explicitly stated to be dead"). Measured with the gold v2 run below.

**Operator status table: refuted as built.** `scripts/operator_status.py`: 407 playable operators (`character_table`, not TOKEN or TRAP, not IsNotObtainable), 296 candidate sentences from story summaries and dossiers that name the operator next to a death word. v1 asked Qwen per (operator, sentence) "does the named character die?": 50 operators flagged, and the judge said true for anyone's death in the sentence (Skadi flagged because Ulpianus died, Amiya because Theresa did). v2 has the model list who dies in each sentence and matches names in code, a leading title stripped ("Captain Ulpianus" is Ulpianus; "Maria Nearl" never matches Nearl): 19 flagged. On reading, at most about 5 of the 19 are defensible. The rest are extraction errors (Delphine for her mother, Eyjafjalla for her parents, Qanipalaat for a fowlbeast, Jessica, Ascalon), a codename shared across people (the previous Platinum), a different character with the same name (Master Lin), or deaths the summaries themselves invented (W and Ines, both alive). Two defects stack: summaries are second-hand and a 9B extractor confuses subjects. Decision: the operator route serves only rows marked `verified` (none are), and otherwise says Trevor cannot list this reliably and suggests asking about one operator, which retrieval answers with citations. A usable table needs each candidate checked against the script passages of its story, not the summaries; not built.

**Measured, gold set v2 on P3b (`gs2_v4`, with the completeness rule and both routes):** 84/131 correct (0.641), identical to the previous run; paired 1 gained, 1 lost; declined 18 answerable, as before; unanswerable declined 15/15; faithfulness 204/210 (0.971, was 0.966); invalid citations 0. No gold question triggered a table route (0 of 146), so the routes are untested by this set and cost nothing on it; the completeness rule did not move correctness. Answers containing "no other", "nothing else" or "the only": 1 (was 0), read as a legitimate use. What did not move: everything. The routes are checked only by Ian's two example questions; the gold set has no whole-corpus listing items, which the hand-built items still owed should add.

## 17. P4 answer bank (2026-09-28 night)

`scripts/answer_bank.py` builds `artifacts/bank/bank.jsonl`: 3,107 entries. Seeded without generation: 200 "Who is X" dossiers (117 alias questions from the current identity links; the dossiers' own frozen name lists held 155, including Kal'tsit as "Raidian" and "Serafina"), 1,861 story summaries, 88 event summaries, 83 primers cut to the sentences the support judge accepts (1,090/1,123 kept, 0.971; `PRIMER_FILTER=0` keeps whole primers). Generated: 900 questions, one per chunk, round-robin over 405 groups, from script and archive chunks only (model-written profile and summary units excluded; stories anchored by gold v2 excluded), 1.7 s each on Gemma; answered by `ask` on P3b at about 18.6 s each (4 h 42 min); 875 kept, 25 dropped as declined or uncited.

Spoiler gate per entry: `gateStories` (source story plus every cited story; for a dossier its 12 stories; for an event all its stories; for a primer its prior groups' stories), the union of their `requiredStages` (1,872 entries carry at least one), and `gateChars` for operator archives used. 0 entries reference a story missing from spoiler.jsonl. 59 generated answers cite more than one story, so their gate is wider than their source, which is correct: an answer may only be shown to a reader who has cleared everything it draws on.

Faithfulness of generated entries (Qwen, sample of 60): 131/135 sentences supported by the passages `ask` was given (0.970), 56/60 entries fully supported, in line with `ask` on gold v2 (0.971). The 4 unsupported sentences are over-readings, not inventions of plot (for example Zofia "refers to her as her niece", Jesselton's "likely no returns" quote attributed across a scene). Correctness of generated entries is not measured: no reference exists for a generated question; gold v2's 0.641 is the nearest estimate, and the bank keeps only answers that cite, which removes declines but not wrong answers. Short of the 5,000 target by design; grow from miss logs.

## 18. Death events (2026-09-28)

Ian asked for a verified dead-operator table, verification against the Arknights wiki, and "What non-playable characters died in x story", noting that Arknights is live service so a status can change with each new story. Deaths are now events, not statuses: `scripts/deaths.py` writes one event per (story, character, kind) with a verbatim quote, so status as of a reader's progress is the latest event in the stories they have read, each event is its own spoiler gate, and a new story only adds events.

**The wiki cannot verify operators.** `scripts/wiki_status.py` read the infobox status of all 407 playable operators' `/Story` pages: 375 have no status, 40 have no Story page (alter operators), and none is "Deceased" (Beagle and Necrass carry a death inside a living status; Kal'tsit is "Unknown"). An operator page describes the operator, not the person: Civilight Eterna reads "Active", Theresa's own page "Deceased". The wiki does mark 83 named characters deceased (85 pages with 2 list pages), which is the reference used to measure recall. The verify stage written for operator_status.py would have confirmed 0 operators; it was removed unrun.

**Method.** Gemma reads every script passage with a death word (4,700 of 13,837), lists who dies with kind ("dies" in the story's present, "dead" before it) and a quote; the quote must appear verbatim in the passage with at least 3 words; names in capitals or known as speakers, identities, operators or wiki titles are named, "my son" and "the others" are counted unnamed; a strict second question per named mention ("does this passage show this character actually dying?") drops lost-contact, figurative and vision cases. Titles never resolve to a person ("King of Sarkaz" gave Yliš's death in 898 to Amiya in the pilot). Extraction 1.41 s per passage with 2 slots (about 1 h 45 min), the check about 12 min.

**Results.** 938 mentions: 246 unnamed; dropped 5 for a quote not in the passage, 7 for a quote under 3 words, 81 by the strict check (607/690 confirmed). 550 named events (276 dies, 274 dead). Recall against the wiki: 39 of 82 deceased characters found (0.48); the rest die off-screen, before the story, in stories outside the EN corpus, or only by action. Precision on a sample of 25 named non-playable "dies" events, by reading: about 19 right (0.76); errors are right-person-wrong-line (Kreide), a line that does not show the death (Scorpion), and ambiguous transformations (Priestess, the Sui).

**Operators: refuted as confirmable from the script.** 22 playable events. "Reported in 2+ stories" as confirmation was refuted: W and Ines, both alive, are each reported dead in two stories (in-story rumors), and Ebenholz inherited the previous Graf Urtica's death through the title. Only the wiki confirms an operator death now (`DEATH_CORROB_STORIES=1` restores the story rule), so the confirmed list is 0 and the operator answer says the script also reports 16 more unconfirmed and points to the per-operator question, which shows the lines with a warning.

**Routes in `ask`** (after table routes, before retrieval; `--no-route` off): "What (non-playable) characters died in <event or Episode N>" lists named deaths in that group with quotes, role labels after, and earlier deaths recalled; "Which operators are dead" as above; "Does X die?" lists X's events in release order with the corpus's latest event as the as-of point. Each is dated by corpus, since a later story can change it.

**Identity fix found on the way.** "Code Name" had become a node joining archive codenames: Amiya's identity held Almond and Mulberry, Istina's held "codename", and a "???" link made Officer Swire Clovisia. `entities.py build` now rejects placeholder names: 159 to 158 identities, 3 changed, all corrections.

## 19. Real names and the listing guard (2026-09-28)

Ian's "List all the known real names of every operator" got 7 names from 17 retrieved passages: two were codenames (Rose Salt, Titi), one was not an operator, and nothing said the list was partial.

**Real-names table** (`scripts/real_names.py`, `artifacts/entities/real_names.jsonl`): for each of 407 playable operators, Gemma reads the operator's archive, up to 4 story passages naming them near a naming word ("real name", "full name", "née", ...), and the passages behind their identity links, and gives a real name with a quote. Kept when the quote is verbatim in those passages, contains the name, the name is not the codename (an alter's base codename included: "Hoshiguma the Breacher" is not a real name), and a second question confirms the quote shows it is this operator's own name. The first check prompt ("states or plainly shows") rejected 2 of 8 correct names in a 16-operator pilot (Gummy = Lada, whom a friend addresses by it; Ch'en = Ch'en Hui-chieh); it now also accepts self-use and being addressed by the name. Full run: about 9 s per operator with 2 slots, 62 min. Result: 121 of 407 with a real name; rejected 266 none stated, 7 same as codename (then 8 after the alter fix), 7 name not in quote, 3 check false, 2 quote not in passages. Precision by reading a sample of 30: 29 right, the one error being the alter codename now fixed. Recall is not measured: a name revealed only in a story passage without a naming word near the codename is missed (Lappland Saluzzo in the pilot).

**Routes.** "Real names of every operator" lists the 121 with their quotes and says how the table was built and what it can miss; "What is X's real name?" answers from the table, or falls back to retrieval when the table has none (Amiya). **Listing guard:** any "list all / every / which operators" question that no table answers now opens with "Trevor has no complete table for this question, so this list comes only from the N passages it read and is not complete." ("List all the operators from Kazimierz": 4 names from 19 passages, flagged.) Operator faction, birthplace, race and infection status are structured in the game data (`handbook_info_table`) and would answer questions like that one completely without a model; not built.

## 20. Game-data attributes (2026-09-28)

`scripts/attributes.py` (no model, under a second) writes `artifacts/entities/operator_attributes.jsonl` for the 407 playable operators: rarity, class, branch and affiliation (nation 383, group 77, team 54 set) from character_table and handbook_team_table; gender, place and date of birth, race (400 each), height (401) and infection status from each operator's handbook Basic Info. Infection is read from the free text: 216 not infected, 176 infected, 15 left unset where the game says Unknown, Undisclosed or redacted (Ch'en "Unknown.", Tin Man "(REDACTED)"); the first parser missed "Infection confirmed by medical examination" and left 72 unset.

`ask` answers roster questions from it before retrieval: filters for place (affiliation or birthplace, both shown per operator because "from X" is ambiguous in the data), race, class, rarity, gender and infection, listed with a count ("List all the operators from Kazimierz": 17, where retrieval found 4; "Which operators are Sarkaz?": 27; "How many 6-star Casters are there?": 19; "Which female Sankta operators are infected?": 0, checked against the data). One operator's attribute ("What race is Texas?", "Where was Amiya born?") comes from the same table. A roster question must leave no content word once its filters and question words are removed: the first guard allowed one, and "Which operators betrayed Rhodes Island?" listed all 84 Rhodes Island operators; with zero it goes to retrieval, as does "Which operators from Rhodes Island went to Londinium?". A word that is both a place and a race ("Ursus") filters as the place only (8 with both filters, 14 as a place). Not handled: superlatives ("the tallest"), branches as filters, and non-playable characters (the game data has no handbook for them).

**Routing regression set (2026-09-28).** The table routes are keyword rules, and two wrong routes had been found by hand, so `eval/routes.jsonl` (20 questions) records the route each must take and text a table answer must contain; `scripts/route-check.py` runs them through `ask --route-only` (no model, no index) and `update.sh` runs it after every update. First run: 19/20. Ian's own ordering question ("In chronological order, order the death of FrostNova, death of Patriot, and the awakening of the Sui") was caught by the death route, which would have listed death events instead of ordering them with years; time and ordering questions now skip it and keep the timeline notes. "Real names of every operator from Kazimierz" had ignored the place; it now applies the attribute table's place filter. 20/20 after both fixes. `ask` also loads the index only after the table routes, so a table answer takes 1.1 s instead of waiting on the ONNX load.

**Superlatives and branches (2026-09-28).** The attribute route answers "the tallest / the shortest" over operator-file heights (401 of 407), combined with any filter ("Who is the shortest Sarkaz operator?"), top 5 with heights; and filters by the game's 71 branches ("Which operators are Fortress defenders?": Ashlock, Firewhistle, Horn), leaving a branch named like its class ("Medic") to the class filter. Route check: 26/26, including "Who is the tallest person Amiya fought?" staying with retrieval.

## 21. Fine-tuning the embedder on Trevor's corpus (2026-09-29)

Ian chose the embedder fine-tune as the next retrieval experiment (the research survey's evidence: GPL, up to +9.3 nDCG@10 from generated in-domain pairs).

**Data.** 5,000 generated questions (`scripts/embed_pairs.py`, Gemma, 1.7 s each, 2 slots, about 2 h 15 min): one per passage from the story scripts and operator archives, round-robin over groups, every story gold set v2 anchors to excluded (168 stories), in two styles alternately: the answer bank's full fan question and a short casual chat question. The first casual prompt allowed questions with no name ("how much money did the worker get", 118 dropped); the second requires naming who or what it is about. Plus the answer bank's 900 questions: 5,900 pairs, 5,605 trained, 295 held out. The gold set generation files were not used, since gold set v2 was selected from them.

**Training** (`scripts/embed_finetune.py`, sentence-transformers 6.1, cached multiple-negatives ranking loss at batch 64, 1 epoch, lr 2e-5, max 512 tokens, MPS): 88 steps in 974.8 s (about 11 s per step), held-out loss 0.255. The first run trained in float16, because the checkpoint's config sets `torch_dtype: float16` and the loader followed it; with no loss scaling every one of the 134 weight tensors came out non-finite (held-out loss NaN from step 20). Loading in float32 fixed it.

**Export** (same script): torch.onnx at opset 14 with the shipped file's signature, then onnxruntime dynamic quantization of the MatMuls only. Quantization settings, mean cosine to the fp32 export over 6 texts: unsigned per-tensor (the default) 0.949; signed per-tensor 0.993; signed per-channel 0.9988; the shipped INT8 file 0.9967. Signed per-channel is used for both the fine-tuned model and the original re-exported as the fair baseline (`models/gte-requant`); the tokenizer files are the pinned ones. INT8 embedding runs at about 0.10 s per chunk, the same as the shipped file.

**Results, gold set v2 (131 answerable), recall@10, P3a:**

| model | dense | hybrid |
|---|---|---|
| shipped INT8 | 0.566 | 0.747 |
| original, same export pipeline | 0.558 | 0.714 |
| fine-tuned | **0.665** | **0.754** |

Paired against the same pipeline: dense +0.107 [+0.042, +0.172], paired t p = 0.002, hit@10 gained 19 lost 5 (McNemar p = 0.007); hybrid +0.040 [-0.006, +0.088], p = 0.094, gained 7 lost 2. Against the shipped file, hybrid +0.008 [-0.031, +0.046], p = 0.70. The same original model moved 0.033 in hybrid recall from the quantization settings alone, so the hybrid comparison sits inside that noise; a full-precision comparison (both models fp32) is running to settle it.

**Independent check.** The dev set (49 questions written by Claude Opus, not Gemma) leaks: 21 of its items target a passage that was a training positive, 39 a story that appears in training. On the 28 items whose passage was not trained on (26 scored): dense 0.692 shipped, 0.731 same pipeline, 0.769 fine-tuned; hybrid 0.923 for all three (ceiling). The dense gain holds in direction on questions from a different writer; at 26 items it is an anecdote.

Caveat: gold set v2's questions were also written by Gemma from passages, so the fine-tuned model saw the same kind of text in training; the gold set may favor it.

**Full precision settles it.** Both models embedded and queried at fp32 (no quantization in either; 1,729 s and about 1,900 s for 15,762 chunks): dense 0.573 original, 0.669 fine-tuned, +0.095 [+0.034, +0.160], paired t p = 0.004, hit@10 gained 17 lost 5; hybrid 0.735 original, 0.747 fine-tuned, +0.011 [-0.031, +0.053], p = 0.60, gained 6 lost 3. The fine-tune improves dense retrieval (consistent across INT8 +0.107, fp32 +0.095, and the clean dev items), but the hybrid pipeline `ask` uses does not gain from it: BM25 already recovers what the better dense ranking adds, and RRF fusion saturates. **Decision: `ask` and the index keep the shipped model.** The fine-tuned export stays at `models/gte-trevor` (`model.onnx` fp32 for an x86 host, `model_int8.onnx` for the Mac) for a use where dense retrieval stands alone, such as a semantic answer cache at serving time; `--model-dir models/gte-trevor` selects it. Evaluation corpora, the re-quantized baseline, the training environment and the PyTorch checkpoints were deleted afterwards; retraining needs `models/ft-venv` rebuilt (uv, torch, sentence-transformers, accelerate) and the pinned PyTorch weights re-downloaded.
