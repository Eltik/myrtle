# Trevor: research findings for an Arknights lore model

> **Partially superseded.** `claude/trevor-implementation-notes.md` (2026-09-23) corrects
> several claims here against verified APIs. One correction is mine to own: the RRF k=20
> claim below overstated its evidence.

Six parallel research threads, 2026-09-22. Constraint set: no external LLM API of any
kind, zero recurring cost. Hardware measured, not assumed: MacBook Pro **M5 Pro, 15 cores
(5P + 10E), 24 GB unified, 307 GB/s, 114 GB free disk**, no ML tooling installed. VPS
**3 vCPU, 10 GB RAM + 4 GB swap**, seven tenants, one recorded kernel OOM kill at
`anon-rss: 9.3 GB`.

Corpus, measured on disk: **4,860 script files, 80,434,038 bytes, 4,260 unique by content
hash**. Against `story_review_table`: **451 groups, 1,887 stories, 25 unresolved paths
(1.3%), 4,108,615 prose words**. Two independent token estimates agree within 2.3%
(1.45 tok/word gives 5.96M, chars/4 gives 6.10M). Largest group `main_14` is 93,221 words
(~135k tokens); median group 3,042 words (~4.4k tokens). 502,367 prose lines, 5,808
distinct speaker strings.

---

## 1. Training: the knowledge question is closed, and the answer is no

Do not train the model on the lore. This is not caution, it is four separate measurements
pointing the same direction.

**Raw continued pretraining on a corpus this size is measured to be net-harmful.** The
closest published analogue is EntiGraph (ICLR 2025), whose source corpus was 1.3M tokens,
same order as yours and the same shape: a small closed corpus where facts appear once.
QuALITY closed-book accuracy on Llama 3 8B: base **39.49%**, raw CPT on the corpus
**38.15%**. The training made it worse. To reach the regime where it helps they needed
**460x synthetic expansion to 600M tokens**, which got 56.42%. For your 6.0M tokens that
ratio is 2.8B tokens, which on this Mac is roughly **162 days of generation plus 65 days
of training**. And the payoff once retrieval is present was **60.35% -> 62.73%, +2.38
points**. Off by three orders of magnitude on effort for two points.

**Scale arithmetic says the same thing without any experiment.** 6.0M tokens against an 8B
model is **0.00075 tokens per parameter**. Successful domain-adaptive pretraining in the
literature runs at 3.2 to 8 tokens per parameter. You are 1,000x to 10,000x below the
floor. Allen-Zhu's knowledge capacity work needs **~1,000 exposures per fact** for full
extraction; a story beat narrated once in one phrasing gets 2 to 4.

**The most damaging finding for a story corpus specifically:** Allen-Zhu & Li (ICML 2024)
showed models pretrained on un-augmented biography data then instruction-tuned for QA
reach **0% accuracy**. Quote: "knowledge may be memorized but not extractable." Screenplay
scripts are exactly un-augmented single-phrasing prose. Memorizing them installs nothing
retrievable.

**And fine-tuning on facts the base model does not know actively increases hallucination.**
Gekhman et al. (EMNLP 2024): regression of test accuracy on fraction-fit gives coefficient
**-8.3 for unknown examples vs +7.3 for known, R² = 0.86**. A 2025 follow-up measured
**-56.40% same-type QA accuracy** when training on 100% unknown knowledge, with marked
degradation visible at **5-10% unknown content**. Ovadia et al. compared directly: Mistral
7B on new knowledge scored base 0.481, fine-tuned 0.588, **RAG 0.875**, and FT+RAG 0.830,
i.e. fine-tuning made RAG worse in all three models tested.

### What training IS worth doing: one adapter, for behavior

RAFT (2024) trained Llama2-7B on (question + retrieved passages including deliberate
distractors -> chain-of-thought answer with citations) and measured **+35.25% over
7B + RAG on HotpotQA**. The chain-of-thought in the target is load-bearing: **+9.66%**
on HotpotQA, **+14.93%** on HuggingFace, when included versus omitted. Models trained with
distractors held performance **flat across top-k = 1 through 10**; golden-only-trained
models did not.

Fold two more behaviors into the same dataset at no extra cost. R-Tuning (NAACL 2024)
teaches abstention and it **generalizes as a meta-skill**: out-of-domain gains of **+10.03
to +16.94 AP**, and refusal rates on genuinely unanswerable questions of **95.45% to
99.18%**. FRONT (ACL 2024) on ~8,000 examples moved citation recall **67.67% -> 77.70%**
and precision **63.67% -> 69.89%** over a vanilla-SFT 7B baseline, beating ChatGPT.

Format adaptation is cheap. LIMA's ablation is the scale marker: **30 multi-turn examples
moved "excellent response" rate from 45.2% to 76.1%** and dropped failures from 35.7% to
2.2%. Scaling their data 2K -> 16K plateaued. Diversity, not volume.

**Recipe.** ~3,000 grounded examples generated locally, always with the source passage in
context, never closed-book. Composition: 55% question + golden passage + 3-4 distractors
-> CoT answer with inline citations; 25% distractors only, no golden passage -> explicit
refusal; 10% multi-hop across two passages; 10% in-voice style. Filter with an NLI model
before training: Self-Instruct's expert audit found only **54% of self-generated examples
correct in all fields** without that step. LoRA rank 16-32 over 16-32 layers, seq 2048,
batch 2, grad checkpointing, 2-3 epochs, **early stop** (Gekhman's damage appears
specifically at convergence: 43.0% -> 38.8% over 50 epochs). MLX-LM measured ~4.1 it/s for
an 8B SFT at 6-10 GB on an M4 Pro 24 GB, the same memory class. Generation ~2 hours,
training ~3 hours.

**The honest gate:** compare the adapter against prompt-engineered RAG on the same base
model with no adapter. If it does not beat that, RAG alone was sufficient and the adapter
is dead weight. Say so and drop it.

**Ruled out by measurement:** raw CPT on the corpus (38.15% vs 39.49% base); synthetic CPT
at EntiGraph scale (~7-8 months of compute for +2.38 points post-RAG); closed-book
knowledge SFT (-8.3 coefficient, -56.40% in the replication); full fine-tuning of a 3B
(~48 GB for AdamW states, does not fit); LoRA-based CPT (matched full FT at 5x the data).

---

## 2. Retrieval: route before you retrieve

**The corpus is small enough that retrieval is the wrong default for most questions.**
Median group is 4.4k tokens, mean 12.2k, and only `main_14` exceeds 100k. On NarrativeQA,
long-context beat RAG **558 correct vs 405**; across 9 datasets with Gemini the averages
were **LC 49.70% vs RAG 37.33%**. Self-Route (ask from chunks, fall back to the full
document on "unanswerable") recovered LC accuracy at **38.6-61.6% of the tokens**. So
"summarize chapter 8" and "what do I need to know before this event" should load the
group, not query an index. A SetFit classifier trains on **8 examples in 30 seconds** and
runs in 16-100 ms; add a deterministic pre-check on named group plus token count.

**Chunk structurally, not semantically.** Three independent studies find semantic chunking
does not pay: fixed-size beat breakpoint and clustering chunkers across 10 retrieval
datasets; build times were **fixed-size <1 s, recursive semantic 4.90 min, LumberChunker
8.37 h, DenseX 15.05 h** with no meaningful effectiveness gain. Meanwhile SECOM (ICLR 2025)
measured dialogue specifically: topically-coherent **segment-level** memory units scored
GPT4Score **69.33** versus 57.99 turn-level and 51.18 session-level. Your scripts already
carry scene and speaker-turn boundaries for free. Chunk at 400-600 tokens on turn
boundaries, never splitting a turn, 0-10% overlap, with the speaker name kept verbatim in
the chunk text so BM25 can hit it.

**Contextual retrieval is the highest-value single item, and it is not free here.**
Anthropic measured failure rate (1 - recall@20) going **5.7% -> 3.7%** with contextual
embeddings, **-> 2.9%** adding contextual BM25, **-> 1.9%** adding reranking, a 67%
reduction. The usual framing prices this at $1.02 per million document tokens, about $5.60
for your corpus. **You have no API, so the real price is Mac time:** ~14,000 chunks at
~100 generated tokens each is 1.4M tokens of generation, roughly **7 hours**. That is a
real line item and it roughly doubles the v1 build. It is still worth it, because the
prefix is where you resolve who "he" and "the Doctor" refer to, and NoLiMa measured that
conversion precisely: questions with literal lexical overlap score **98.5% at 32k context**
while latent-association questions score **56.2%**. Screenplay dialogue is the worst case
for latent association.

Late chunking is the cheaper alternative and loses on both axes here. Head-to-head it
trails contextual retrieval (NDCG@5 0.309 vs 0.317), and it caps at the encoder's context
window (8,192 tokens), which covers your median group but not your mean or the 124k
outlier.

**Hybrid, and expect BM25 to matter more than usual.** Start with RRF. *(Correction, 2026-09-23: I wrote "measured best at k=20" here and that overstated the evidence. No study establishes k=20; the one real sweep found k=32 peaking at nDCG@10 0.899 against k=60's 0.881, with 10 through 100 all inside 0.868-0.899. Make k a config knob, default 20, and do not spend much on it.)* Move to convex combination once you have ~40 labeled queries:
Bruch et al. (TOIS 2023) found CC beats RRF in-domain and out-of-domain and needs only a
small tuning set, and Elastic measured tuned weighted sum at **+6% nDCG@10 over the strong
single retriever versus RRF's +1.4%**. Your proper nouns are invented tokens with no
pretraining support, and "every time X is described as Y" is an enumeration query that
BM25 owns.

**Reranking is the only component whose removal was catastrophic in a proper ablation.**
Full pipeline nDCG@10 0.644; remove the reranker and it collapses. A separate head-to-head
measured no-reranker Hit@1 **62.67% -> 83.00%** with one. Critically the size curve is
nearly flat: the **Ettin 17M** reranker scores MTEB nDCG@10 0.5576 / NanoBEIR 0.6746, and
the 1B version only reaches 0.6114 / 0.7237. The 17M beats ms-marco-MiniLM-L6-v2 (0.5082 /
0.6312) at **2x the throughput**. Take the 17M.

**Do not build a knowledge graph.** The narrative-specific numbers are consistent and
negative: on NovelQA's detail subset plain RAG scored **55.28% vs Community-GraphRAG
46.88%**, HippoRAG2 on NarrativeQA gained **+0.2 F1**, and on STAGE's screenplay QA
benchmark **Hybrid RAG 70.2% Pass@5 vs GraphRAG 52.0%**. Build the cheap piece instead:
your 5,808 speaker strings are exact, zero-error structured metadata, so a speaker-to-chunk
inverted index costs nothing and has no extraction error. E²GraphRAG showed the light
version wins anyway, beating GraphRAG-local on NovelQA **45.60% vs 43.34%** with indexing
**1,397 s vs 13,794 s**.

**Shallow summary tree, flat index.** RAPTOR measured NarrativeQA **+7.3 ROUGE-L** over
BM25 and QuALITY **+20.3 points**, and found the **collapsed tree consistently beat tree
traversal**. So embed the 451 group summaries and the scene summaries in the same flat
index as the leaf chunks rather than building a deep recursive structure. Those 451
summaries are also, directly, the answer to "what do I need to know before this event."

**No vector index.** 14,000 vectors at 256 dims is **14.3 MB**; at 768 dims, 43.0 MB. An
exact SIMD scan costs **0.50 ms** and **1.5 ms** respectively at recall 1.0. An ANN index
would buy 1.4 ms in a pipeline whose reranker costs 280 ms, at the price of 4% recall plus
an index build plus a dependency. The crossover where ANN starts to matter is 300k to 1M
vectors. Skip pgvector entirely: at 14,000 rows its iterative scan degenerates into the
flat scan you would have written, with a planner, a WAL and an extension upgrade treadmill
in the way.

---

## 3. Integration: the stack, and one disqualifying Postgres finding

**Postgres full-text search cannot index `Kal'tsit`.** Confirmed from the pgsql-hackers
archive: `to_tsvector('peter o''toole')` yields `'o':2 'peter':1 'tool':3`. The apostrophe
splits the token **and then the English stemmer mangles the remainder**. Applied to your
corpus that turns your single most important token class into garbage. The parser cannot be
fixed without a C extension. Tantivy's default `SimpleTokenizer` has the same problem, but
tantivy lets you replace the tokenizer, which is the point: a `RegexTokenizer` on
`[\p{L}\p{N}]+(?:['’\-.][\p{L}\p{N}]+)*` with `LowerCaser` and **no stemmer** keeps
`Kal'tsit`, `Ch'en`, `W's` and `Ifrit-Nian` intact. Fuzzy distance 1-2 then rescues
`Kaltsit` -> `Kal'tsit`.

Recommended crate stack, versions verified 2026-09-22:

| Concern | Choice | Note |
|---|---|---|
| Embedding | `fastembed = "=7.0.1"` | Wraps `ort`; absorbs its RC churn. `ort` is still 2.0.0-rc.13 with breaking changes in every RC |
| Lexical | `tantivy = "0.26.2"` | Mmap-backed, so the 12-25 MB index is reclaimable page cache, not anonymous RSS |
| Vectors | none | `Vec<f32>` behind `ArcSwap`, normalized at build time |
| Tokenizing | `tokenizers = "0.23.2"` | Same `tokenizer.json` on Mac and VPS or chunk sizes silently drift |
| Hot swap | `arc-swap = "1.9.2"` | Matches the existing `ArcSwap<GameData>` pattern |
| Reranker | Ettin 17M, INT8 ONNX, 256 tokens, batched top-40 | ~280 ms on 2 cores |

**Rejected:** `candle` (measured **8.15 ms vs ort's 1.10 ms** single-query CPU, 7.4x
slower); `embed_anything` (44,999 lifetime downloads); `instant-distance` (**no release
since 2023-06-26**); pgvector; Outlines for constrained decoding (**3% schema coverage on
GitHub-Hard, 3-8 s grammar compile** versus llama.cpp GBNF at 39% and 0.05 s).

**Two memory hazards specific to your OOM history.** First, glibc arena hoarding around
ONNX Runtime is measured and large: a 19-20 MB model consumed ~120 MB RSS, with
`mallinfo2()` showing **~40 MB in use versus ~188 MB hoarded**. Lowering the mmap threshold
to 32 KB cut hoarding **188 MB -> 4-5 MB**, and disabling ORT's internal arena saved
another 50 MB. Second, `intra_op_num_threads` defaults to one thread per core **per
session**, and `allow_spinning` defaults to on, so two sessions on a 3-vCPU box gives you
six busy-waiting ORT threads competing with tokio. Use a global ORT thread pool,
`intra_threads = 2`, spinning off, every `run()` inside `spawn_blocking`, and a
`Semaphore::new(1)` in front of the reranker.

Estimated steady-state RSS for the lean configuration (gte-modernbert-base 149M embedder,
30k-vocab tokenizer, 256-dim vectors, Ettin 17M reranker, all INT8): **~515 MB tuned,
~695 MB untuned**. Set `MemoryHigh=900M` and `MemoryMax=1200M` on the systemd unit with
`OOMScoreAdjust=500`. That single line converts "the new service OOMs the box and takes
down all seven tenants" into "the new service gets throttled and logs."

Query latency budget on 2 cores: tokenize <1 ms, embed 15-25 ms, BM25 1-5 ms, flat scan
0.5 ms, fusion <1 ms, rerank ~280 ms. **p50 ≈ 300-350 ms**, all of it retrieval, none of
it generation.

**Constrained decoding for the offline build is free quality.** JSONSchemaBench measured
that grammar-constrained decoding **improves** downstream accuracy by up to 4 points with
no observed declines (GSM8K 80.1% unconstrained -> 82.4% with llama.cpp GBNF). Drive it
over HTTP to `llama-server` with `reqwest`; keep `llama-cpp-2` (currently 0.1.156,
releasing every 1-3 weeks) out of the VPS build graph entirely.

---

## 4. Hosting: your constraint eliminates live generation, and that is fine

State the constraint's consequence plainly. The free inference tiers are real and
generous, Cloudflare Workers AI computes to **~740 answers/day free** on Llama 3.1 8B with
the cleanest data terms of any provider, Cerebras gives 1M tokens/day, Groq 200k. **All of
them are external APIs.** Your rule rules them all out. What remains is your VPS, your
laptop, or rented hardware.

**The VPS cannot generate.** Projected from published benchmarks via the bandwidth model, 3
shared vCPUs give roughly 5-12 tok/s on a 1.7B at Q4 and 1-2.5 tok/s on an 8B. But decode
is not the blocker: **prefill is, and almost everyone forgets it.** A 2,000-token RAG
prompt at 50-200 tok/s for a 1.5B means 10-40 s before the first token. **Total
time-to-answer 30-90 seconds**, while saturating the cores that serve your website, on a
box that has already been OOM-killed once. One user asking a question makes the site slow
for everyone.

**The laptop cannot serve.** Cloudflare Tunnel itself is genuinely free and excellent,
unmetered, with TLS at the edge and no open inbound ports. The problem is the origin: lid
closes, campus Wi-Fi to café to tethering, Kyoto egress to a global audience, and a single
point of failure that travels in your backpack. Realistic availability lands around 50-80%,
which for a public feature is worse than not shipping. It also conflicts with your standing
preference against leaving long-running background jobs on the Mac. The laptop's correct
role is batch precompute while plugged in.

**Dedicated GPU is the only live-generation option, and it is not zero.** Cheapest
sustained 24/7 is Vast.ai interruptible RTX 4000 Ada at **~$101/month**, on-demand ~$130,
Hetzner GEX44 at **€184 (~$210)**.

**So the architecture is: no live inference on the critical path.** Precompute on the Mac,
serve lookups and retrieval from the VPS. The degradation ladder, each rung independently
testable:

1. Exact cache hit, sub-10 ms
2. Semantic hit at **cosine ≥ 0.95**, 20-40 ms
3. Precomputed answer bank hit
4. Retrieval only: top-3 passages with citations and no generated prose
5. Static "not covered yet," page renders normally

Rung 4 is the most valuable and most often skipped. Retrieval alone is genuinely useful on
a lore site, and it is honest.

**Use 0.95, not 0.90.** Measured false-positive rates: low single digits at 0.95, **8-15%
at 0.90**, catastrophic at 0.85. A false positive here means confidently serving the wrong
lore answer to the wrong question, which is the one failure a fan site cannot afford.

Published semantic cache hit rates: open-ended chat 10-20%, production RAG ~20%, bounded
FAQ/knowledge-base **40-70%**. A lore site sits at the high end for structural reasons: the
answer space is finite and enumerable, question phrasing collapses hard ("who is Amiya" /
"Amiya's backstory" / "tell me about Amiya"), and fan traffic is event-driven, so thousands
ask near-identical questions in the 48 hours after a chapter drops. No published
question-distribution data exists for game-wiki Q&A specifically; that is a genuine gap,
not something to extrapolate from web-search Zipf literature.

**The miss path is a content pipeline, not an error.** Log it, batch-generate overnight,
and the log tells you exactly which lore to prioritize.

---

## 5. Verification: build the retrieval eval, and nothing else, first

One measured fact dictates the build order. **Synthetic questions are valid for retrieval
work and invalid for generation work.** Kendall τ between synthetic-benchmark and
human-benchmark rankings was **0.44-0.75 for retriever configurations** but ranged
**-1.00 to +1.00 for comparing generators**, with several metrics inverting model
preferences. Your only free labeling source is trustworthy for exactly one thing.

Retrieval is also where the headroom is: oracle context scores **76.1-88.6%** against
closed-book **31.5-56.1%**. No prompt change moves a 40-point band. And retrieval metrics
need no model at all, so they run in CI in seconds and never flake.

**The mandatory control, and it is a bigger risk for you than for most.** Run every gold
question closed-book with no retrieval and record the score. Standard benchmarks leak
badly: GPT-5 scored **HotpotQA 50%, 2WikiMultihopQA 62%, QASC 75%** with no context at all,
and when leakage was removed the spread between RAG methods widened from **3% to 10-18%**.
Arknights is a popular game with a large wiki footprint in pretraining data, so a local
Qwen or Gemma may already know a fair amount of it. **If closed-book accuracy exceeds
30-40%, your eval is not measuring retrieval** and no number from it means anything. Five
lines of harness code, highest-value validity check available.

**Gold set: 150 questions.** That number converges from three independent directions:
Sakai's minimum-detectable-difference of 0.10 gives 75, ARES's prediction-powered inference
floor gives 150, and Wilson 95% CI half-width at ±7.3 pp gives 150. At n=50 the half-width
is **±12.3 pp**, which is why absolute-threshold gates flap. Build by known-item generation
(generate the question from a chunk, that chunk is gold), then spend human time only on
rejecting bad questions and on the ~40 queries where retrievers disagree: ~2-3 hours rather
than the ~9 hours full pooled judging would take. Drop any question with trigram overlap
above 0.3 against its source chunk, and any question BM25 already answers at rank 1: those
are too extractive to distinguish retrievers.

Stratify across factoid, multi-hop, entity-centric, temporal, causal, aggregation,
unanswerable, ambiguous-entity and spoiler-boundary. Report per stratum, **gate on the
pooled number only** (8 strata x 4 metrics at α=0.05 gives ~1.6 false alarms per run).

**No paid judge is needed, and this is not a compromise.** On LLM-AggreFact balanced
accuracy, **FactCG-DeBERTa-v3-L at 0.4B scores 77.2 against GPT-4o's 75.9**. A 0.4B
encoder beats GPT-4o at groundedness. HHEM-2.1-Open is 0.11B, runs in **<600 MB and ~1.5 s
per 2k tokens on CPU**, and beats GPT-3.5-Turbo by **+18.12 pp** on RAGTruth-QA. These are
encoder forward passes with argmax, so the judge itself never flakes in CI.

Use a small local generative model as a judge **only** for binary rubric-anchored verdicts,
where 16 SLMs across 0.6B-14B reached **89.55% accuracy**. Never for pairwise preference or
1-5 scoring: the best binary judge collapsed to **Spearman ρ = 0.21** on MT-Bench, and
Qwen 3 8B showed **position bias of 0.192**. Also report Cohen's κ rather than raw
agreement, because exact match **overstates chance-corrected agreement by 33.8-41.3 pp**.

**Temperature 0 is not deterministic.** 1,000 completions at temperature 0 from
Qwen3-235B produced **80 unique outputs**, first diverging at token 103, caused by
batch-size-dependent floating-point reduction order rather than sampling. Run evals at
**batch = 1** and pin model SHA, quantization, engine version, prompt hash, index build ID
and eval set version in every run record.

**Gate on the paired difference, not an absolute threshold.** Run current and baseline on
the same frozen set, take per-query paired differences, paired t-test plus a 10,000-sample
bootstrap CI, fail only when the 95% CI on the difference lies entirely below zero. Keep
one loose absolute gate (`hit@10 > 0.40`) that only fires on real breakage like an index
that did not build.

**Spoiler filter: do not try to prove it by attribution.** A 2026 evaluation of
ContextCite, AttriBot, TracLLM and TokenShapley found they cannot disentangle in-context
from in-weight contributions. Use a counterfactual probe set instead: N questions whose
answers appear only past a boundary, run closed-book, filtered, and oracle-leak, and detect
leaks with the same 0.4B entailment model (premise = answer, hypothesis = known spoiler
claim). Report **both** the leak rate and the over-filtering rate, and state the leak rate
as a Wilson upper bound: with 50 probes and zero leaks the honest claim is **≤7.1%**, not
"0%". With 150 probes and zero leaks, ≤2.5%.

Use the ALCE definitions verbatim for citation accuracy; their automatic procedure agrees
with humans at **κ = 0.698 / 85.1% accuracy for recall** but only **κ = 0.525 / 77.6% for
precision**, so treat precision deltas below ~5 pp as noise. Add a free per-commit
pre-filter with no model at all: every citation ID must exist in the retrieved set and the
cited span must be byte-identical to corpus text.

---

## 6. Build budget, and the finding that changes the phasing

Generation totals at ~55 tok/s single-stream (Gemma 4 26B-A4B measured 58-59 tok/s on an
M5 Pro), assuming `-np 4` batching delivers the measured ~2x aggregate:

| Artifact | Output tokens | Hours |
|---|---|---|
| Scene, story and group summaries | 1.2M | 3.0 |
| Contextual chunk prefixes (14k x ~100 tok) | 1.4M | 3.5 |
| Entity dossiers (~1,000 x 800 tok) | 0.8M | 2.0 |
| SFT training set (3,000 x 400 tok) | 1.2M | 3.0 |
| **Answer bank at 50,000 questions** | **7.5M** | **19.0** |
| Prefill across all passes (~15M tok at ~500 tok/s) | — | 8.3 |

**The answer bank is 62% of the entire build.** That is the phasing finding. Do not
enumerate 50,000 questions upfront. Seed 5,000 (1.9 hours) and grow the bank from real miss
logs, which is both cheaper and better targeted than any enumeration you would guess at.

Which means **v1 is one night, not a weekend**: summaries plus prefixes plus prefill is
roughly **10 hours**, and that alone ships reader enrichment (per-story synopsis, "previously
on this arc", speaker identity) with zero live inference, zero chat UI and zero abuse
surface.

**Serving stack: llama.cpp `llama-server`, not MLX.** MLX wins single-stream (Qwen3-8B 79.9
vs llama.cpp 76.9 tok/s) but loses everything that matters for a batch job. Concurrency
measured on an M5 Pro with an 8B: `mlx_lm.server` went 81 -> 94 tok/s at concurrency 4
because it serializes, while `llama-server` went **74 -> 218 tok/s, ~2.9x**. MLX's prefix
cache reuse is **broken for exactly the model families you want**, silently falling back to
full recompute on sliding-window and SSM-hybrid architectures, which covers Qwen 3.5 all
sizes, GPT-OSS, and Gemma 3. And MLX runs ~50% slower past 30k context. Your workload is
5:1 input-heavy with a shared system prompt, so prefix caching is the single biggest lever
and MLX cannot pull it.

**Two-tier model choice.** Bulk work on **Gemma 4 26B-A4B at Q4** (~16-17 GB, 3.8B active,
58-59 tok/s measured on M5 Pro, KV only ~20.8 MB per 1k tokens). The 135k-token passes and
anything shipped verbatim on **Qwen3.5-9B at Q8_0** (~10 GB, 4.42 GB fp16 KV at 135k). The
reason for the second tier is measured: 4-bit quantization loses **up to 23% at 128k
tokens** while 8-bit loses **0.2-0.8%**. A smaller model at 8-bit beats a bigger model at
4-bit for long-context faithfulness.

**Before committing to a multi-hour run, three checks that take under an hour:**

1. **Correctness under an open Metal bug.** llama.cpp PR #28748 is still open as of
   2026-09-18: when the offset between a slice address and the tensor base reaches 2 GB,
   the M5 Metal Tensor API returns an address off by -4 GB from int32 wrap, producing
   **wrong matmul outputs and invalid embeddings**. Reproduced on M5. With a 16-17 GB model
   this boundary is well within reach. Run a fixed prompt at temp 0 with and without
   `GGML_METAL_TENSOR_DISABLE=1` and diff. Also require build **b10734 or newer**
   (2026-09-01), which fixed M5 startup failures on macOS 26.2-26.4.
2. **Does `-np` actually scale?** The 2.9x figure is contested; one five-backend comparison
   found all backends flat under concurrency. `llama-batched-bench` at 1, 2, 4, 8 settles
   it in ten minutes. If flat, budget ~1.7x the hours above.
3. **Does the VPS CPU have VNNI?** `grep -o 'avx512_vnni\|avx_vnni' /proc/cpuinfo`. INT8
   quantization gives **4.0x single-thread speedup at -0.20% quality** with VNNI; without
   it, ONNX Runtime falls back to 7-bit weights and most of the gain evaporates, in which
   case use fp32 with a smaller model.

Also: `sudo pmset -a powermode 2 disablesleep 1` (measured **+52% to +89%** on dense models
on M5 Pro), `sudo sysctl iogpu.wired_limit_mb=19456`, `caffeinate -dimsu`, write every
result to disk with an idempotent job ID so a crash costs one unit rather than the run, and
restart `llama-server` every ~2,000 requests. Expect **8-15% sustained thermal derate**;
the 14" chassis held 92% of peak over a 30-minute sustained load but the top-end parts in
that chassis degrade 13.8-19.0% within a single run.

---

## 7. What I did not verify

- **Nothing was benchmarked on your hardware.** Every throughput figure is derived from
  published benchmarks on comparable chips, several of them from SEO-generated sites that
  contradict roofline physics. The roofline filter is
  `tok/s ≈ (bandwidth ÷ active-weight GB) × 0.60-0.70`; one widely-cited claim of 70-80
  tok/s for a dense 27B on this class of Mac computes to 12-18 and is simply wrong.
- **Closed-book Arknights knowledge of the candidate models is unmeasured**, and it is the
  single largest unknown in the plan. It determines whether the eval is valid and whether a
  spoiler filter is even achievable.
- **CN remains unverified**: 5,388 script files on disk but `docs/story-reader.md` records
  only 1 of 2,009 `StoryTxt` paths resolving. I did not re-check that this session.
- **`handbook_info_table.json` (5.7 MB) and `charword_table.json` (11.1 MB)** are
  lore-bearing, already parsed, and not in the 4,108,615-word count. The corpus is larger
  than stated by an amount I have not measured.
- **No public benchmark isolates abstractive summarization faithfulness** for any of the
  candidate models. That gap is real and only your own eval closes it.
- One directly on-topic paper, "Self-Study Reconsidered: The Hidden Fragility of Learning
  from Self-Generated QA" (arXiv 2606.32002), was rate-limited and could not be read. It
  bears on the SFT-data-generation step and is worth reading directly.
