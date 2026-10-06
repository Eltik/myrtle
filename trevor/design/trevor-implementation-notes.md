# Trevor: verified implementation notes

Written 2026-09-23 against docs.rs, crates.io, Hugging Face and llama.cpp `master`. The
tantivy section was **compiled and run**, not read; its claims are marked [measured].
**Updated 2026-09-24 with §8: the chunker run over the real corpus through the backend's own
parser.**

**Read this before acting on `trevor-corpus-spec.md` or `trevor-eval-spec.md`.** Section 1
lists places where those specs are wrong. Three of the corrections are things that would
have failed silently rather than loudly.

---

## 1. Corrections to the specs I already wrote

| # | Spec said | Verified truth | Severity |
|---|---|---|---|
| 1 | RRF "measured best at k=20, not the paper's k=60" | **No study establishes k=20.** The claim traces to a 2026 dev.to post that gives an analytic argument and no retrieval measurement. The one real sweep (k=1…1000, 300 trials) found **k=32 peak at nDCG@10 0.899, k=60 at 0.881**, with everything from 10 to 100 between 0.868 and 0.899. A gentle hill. | **I overstated evidence.** Make `k` a config value, default 20, tune against the gold set. |
| 2 | Embedding model unspecified pooling | `gte-modernbert-base/1_Pooling/config.json` is `pooling_mode_cls_token: true`. **CLS, not mean.** | **Silent quality loss** if you mean-pool |
| 3 | `--mlock` in the llama-server flags | **`--mlock` and `--no-mmap` do not exist on llama.cpp master.** The server was refactored; `tools/server/utils.hpp` is gone. | Startup failure |
| 4 | `-fa on` implied a boolean flag | `-fa` is now tri-state `[on\|off\|auto]`, **default `auto`**, and the old bare boolean is gone. `auto` silently degrades to off on a device mismatch. Pass `-fa on` so failure is loud. | Silent perf loss |
| 5 | `-cram -1` (unlimited host prompt cache) | Default is already **8192 MiB**, and that default is the documented #1 cause of "llama-server leaks RAM" reports. On 24 GB, `-cram -1` is actively harmful. Use `-cram 1024`, and `-cram 0` on the long-context instance. | **Would have caused OOM** |
| 6 | `--cache-reuse 256` | Verified worth ~nothing here. It reuses KV across *mid-prompt* insertions via KV shifting; your divergence is at the end of the prompt, which the plain longest-common-prefix already handles. | Harmless, just useless |
| 7 | "byte-identical system prompt so slot matching hits" | The rule is right, the mechanism I cited is not. `-sps` (slot-prompt-similarity) defaults to **0.10**, and it divides the common-prefix length by the *incoming prompt length*. A 100-token prefix on a 2,000-token prompt scores 0.05, below the floor, so **similarity matching never fires** and selection falls through to LRU. Keep the prompt byte-identical anyway (the cache is a strict token-level LCP), but the throughput win is batching, not slot affinity. | Reasoning was wrong |
| 8 | `ort`: `SessionBuilder::with_disable_per_session_threads` | **Does not exist in 2.0.0-rc.13.** The polarity is inverted: enabling a global pool on the environment makes all sessions share it, and a session opts *out* with `with_independent_thread_pool()`. | Compile error |
| 9 | "ONNX `Session` is `Send + Sync`, one session shared" | In rc.13, `Session::run` takes **`&mut self`** — the docs cite EP allocators and statistics trackers not being thread safe. You cannot call it from `&self` in an axum handler. Needs `Mutex<Session>` or a pool. fastembed hides this behind internal locking, which is a real argument for using it. | **Design error** |
| 10 | Ettin reranker: use it via ONNX | The published `onnx/model*.onnx` **appear to contain the ModernBERT backbone only**, with the 4-module scoring head living in separate safetensors. Size arithmetic on two model sizes supports this: 17m ONNX exceeds the backbone by 135,312 bytes while the head's own weights are 265,604. High confidence, not proven. §5 has the one-command check. | **Blocks the reranker** until checked |

---

## 2. The story API wire contract, verified

`GET /api/story/index` → `StoryIndex`, serde `rename_all = "camelCase"`, exported through
`ts-rs`:

```
StoryIndex   { groups: StoryGroup[], records: OperatorRecordGroup[],
               storylines: Storyline[], totals: StoryTotals }
StoryTotals  { stories, withScript, words }          // distinct story ids, deduped
StoryGroup   { id, name, category, entryType, actType,
               displayType?, coverUrl?, coverKind?, ... }
StoryEntry   { id, name, code?, sort, avgTag?, groupId,
               hasScript, wordCount, hasVideo, requiredStages: string[] }
```

Three things fall out of this that simplify the corpus build:

- **`hasScript` tells you in advance whether `GET /story/{id}` will 404.** Filter on it and
  the 25 unresolved paths never become error handling.
- **`requiredStages` is already on `StoryEntry`.** The spoiler metadata needs no separate
  pass; it comes with the index.
- **`wordCount` is precomputed** by the same rule as `StoryScript.wordCount`, so it is a
  free cross-check against the chunker's own token accounting.

One discrepancy is narrowed but not closed. `services/story.rs` and `docs/story-reader.md`
both say **1,797 scripted EN stories**; the table resolves **1,862 of 1,887** on the current
tree. Three explanations are ruled out by measurement (§8.4). Log `totals.withScript` on the
first ingest run and treat the API as authoritative.

---

## 3. tantivy 0.26.2 — five footguns, all [measured]

The plan survives: `Kal'tsit`, `Ch'en`, `Ifrit-Nian`, `W` and `Dr.` all tokenize intact.
But two of the five failure modes are silent.

**3.1 `TopDocs::with_limit(n)` alone no longer compiles.** `TopDocs` stopped implementing
`Collector` in 0.26. You must write `TopDocs::with_limit(50).order_by_score()`. Every code
sample predating 2026-03 is wrong. [measured] `error[E0277]: the trait bound
TopDocs: Collector is not satisfied`.

**3.2 Custom tokenizers are silently lost on `Index::open_in_dir`.** `open_from_metas`
assigns a fresh `TokenizerManager::default()` unconditionally; nothing about analyzers is
persisted in `meta.json`. The failure modes differ by path [measured]:

| path | behavior |
|---|---|
| writing without registering | hard error, `SchemaError("Error getting tokenizer for field")` |
| `QueryParser` after reopen | hard error, `UnknownTokenizer { tokenizer: "lore" }` |
| **hand-built `TermQuery` after reopen** | **no error, returns wrong results** |

Since the design builds terms by hand rather than using `QueryParser` (see 3.4), this is the
dangerous path. **Funnel every open through one `fn open_index(path) -> Result<Index>` that
registers the analyzer, and never call `Index::open_in_dir` anywhere else.**

**3.3 `TextFieldIndexing::default()` gives `IndexRecordOption::Basic`, not
`WithFreqsAndPositions`.** [measured] `TEXT` gives positions; the builder default does not.
Phrase queries then fail loudly, but **BM25 degrades silently** without term frequencies.
Always chain `.set_index_option(IndexRecordOption::WithFreqsAndPositions)`.

**3.4 `QueryParser` cannot parse a bare apostrophe.** [measured] `parse_query("kal'tsit")`
→ `SyntaxError("kal'tsit")`. Escaped `kal\'tsit` and quoted `"kal'tsit"` both work. Since
the corpus's most important token class is exactly the one the parser chokes on, bypass
`QueryParser` entirely: run query text through the registered analyzer yourself and build
`TermQuery`/`BooleanQuery` from the tokens. That is the better design for a retrieval
backend anyway.

**3.5 Curly and straight apostrophes are different terms.** [measured] a query for
`kal’tsit` (U+2019) against a corpus spelled with ASCII `'` returned **0 hits**. Game script
dumps mix both. Fold U+2019 to `'` in a token filter *and* normalize at ingest. Related:
`Amiya's` stays one token and will not match `amiya` without a possessive-stripping filter.

**Measured numbers worth having.** Index size over 14,000 synthetic docs, body not stored:
`Basic` 273 KB, `WithFreqs` 485 KB, `WithFreqsAndPositions` **2.78 MB**, and 8.46 MB if the
body is `STORED`. So positions cost single-digit MB and the doc store costs 3x what
positions do: turn positions on, keep the text in your own store. Fuzzy latency over 80,004
distinct terms: exact `TermQuery` 0.035 ms, fuzzy d=1 **0.310 ms**, d=2 **2.158 ms**,
prefix d=2 10.577 ms. `distance >= 3` is a runtime error, not a compile error. And the
useful accident: **fuzzy d=1 matches `kaltsit` to `kal'tsit`**, which is a free safety net
for users who type proper nouns without punctuation.

Writer arena is in **bytes and is the total across threads**, with a hard floor of
15,000,000 per thread. [measured] `index.writer(3_000_000)` errors. Use
`index.writer(50_000_000)`, then `wait_merging_threads()` at the end of a one-shot build.

---

## 4. Embeddings: use fastembed, and no export is needed

**fastembed 7.0.1 can load arbitrary local ONNX.** The recommendation survives. The API is
`UserDefinedEmbeddingModel::new(onnx_bytes, TokenizerFiles { tokenizer_file, config_file,
special_tokens_map_file, tokenizer_config_file })`, then `.with_pooling(Pooling::Cls)` and
`.with_quantization(QuantizationMode::Dynamic)`, into
`TextEmbedding::try_new_from_user_defined(model, InitOptionsUserDefined::new()
.with_max_length(512).with_intra_threads(2).with_session_config("session.intra_op.allow_spinning","0"))`.
It pins `ort = "=2.0.0-rc.13"` exactly, so adding `ort` at that same exact version gives one
shared ONNX Runtime in the tree. Build it with `default-features = false` to drop `hf-hub`.

**The INT8 ONNX already exists officially.** `Alibaba-NLP/gte-modernbert-base` ships
`onnx/model_int8.onnx` at **150,218,016 bytes**, plus all four tokenizer files the
`TokenizerFiles` struct requires, with no `auto_map` so `trust_remote_code` is not needed.
The entire Optimum export-and-quantize pipeline in the corpus spec is unnecessary for the
embedder. That also means the ModernBERT export traps (`reference_compile=False`,
`attn_implementation="eager"`) never have to be dealt with.

**`token_type_ids` is not an input.** Both models declare
`model_input_names: ["input_ids", "attention_mask"]`; ModernBERT has no segment embeddings.
Passing a third input errors. Defensive pattern: read `session.inputs()` at startup and
build the input set from the actual names.

**INT8 is only worth it with VNNI.** Without `avx512_vnni` or `avx_vnni`, ONNX Runtime's
INT8 GEMM uses the saturating `VPMADDUBSW` path and you need `reduce_range=True`, which
quantizes weights to 7 bits. Measured: Cascade Lake with VNNI 31.46 ms → 19.76 ms (1.59x);
Ryzen 3700X without VNNI 38.05 ms → 35.08 ms (**1.08x**). So the gate before committing:

```bash
grep -qE '\bavx512_vnni\b|\bavx_vnni\b' /proc/cpuinfo && echo VNNI || echo NO-VNNI
```

No VNNI means INT8 buys 1.1 to 1.6x and costs accuracy; prefer fp32 with `--optimize O2`.
Note ORT's CPU EP does not compute in fp16, so fp16 buys memory, not speed.

---

## 5. The reranker needs one command run before its design is settled

`cross-encoder/ettin-reranker-17m-v1` is a 5-module sentence-transformers pipeline:
Transformer → CLS Pooling → Dense(256→256, GELU, no bias) → LayerNorm(256) →
Dense(256→1, bias, Identity). Its `config.json` says `architectures: ["ModernBertModel"]`,
which is the backbone, not a sequence-classification head, and Optimum would export
`last_hidden_state` from that rather than `logits`. fastembed's `TextRerank` fetches the
output by the literal key `"logits"`, so a backbone-only graph fails.

**Run this first. It decides the whole reranker design:**

```bash
pip install onnx huggingface_hub
python - <<'PY'
import onnx
from huggingface_hub import hf_hub_download
p = hf_hub_download("cross-encoder/ettin-reranker-17m-v1", "onnx/model_qint8_avx512_vnni.onnx")
m = onnx.load(p, load_external_data=False)
print("opset:", [(o.domain or "ai.onnx", o.version) for o in m.opset_import])
print("outputs:", [(o.name, [d.dim_param or d.dim_value for d in o.type.tensor_type.shape.dim])
                   for o in m.graph.output])
PY
```

- Output `logits [batch, 1]` → my inference is wrong; use `TextRerank` unchanged. Note the
  repo has **no `special_tokens_map.json`**, so pass `special_tokens_map_file: b"{}".to_vec()`
  (fastembed iterates the map and skips invalid entries, so an empty object is safe).
- Output `last_hidden_state [batch, seq, 256]` → the head is missing. Implement it in Rust:
  it is **66,561 parameters** total, loaded from the three small safetensors files with the
  `safetensors` crate (already in fastembed's tree). That is roughly 40 lines and cheaper
  than maintaining a Python export step.

Either way the scoring convention is the same: **one logit, Identity activation, higher is
better, unbounded.** Not a two-class softmax. Sort on the raw value; sigmoid only for
display, and it is monotonic so it cannot change the ranking.

Pair encoding uses the tokenizer's own post-processor, not manual `[SEP]` concatenation:
`(&str, &str)` implements `Into<EncodeInput>` as the `Dual` variant. Truncate with
`TruncationParams { max_length: 256, strategy: TruncationStrategy::OnlySecond, .. }` so the
query survives and the document is cut. **`OnlySecond` errors if the query alone exceeds the
budget** — clamp query length upstream.

---

## 6. llama-server, for the corpus build

**`-c` is the total KV pool and is divided by `-np`** unless unified KV is on. Verified in
`llama-context.cpp`: `n_ctx_seq = n_ctx / n_seq_max`, padded to 256. So `-c 32768 -np 4`
gives 8,192 per slot. The modern, explicit way to say what you mean is
**`--kv-unified-per-slot N`**, which sizes the shared pool to `n_parallel * N`. Also note
that passing `-np` explicitly leaves `kv_unified` false, while omitting it (auto) sets
`n_parallel = 4` *and* `kv_unified = true` — two different regimes.

**A shared system prefix is cached once per slot, not once.** Each slot is its own
`llama_seq_id` and there is no content-addressed dedup. The cost is `n_slots x prefix_len`
tokens on the first N requests and then free, which is trivial. The real win from `-np` is
that N requests prefill and decode in the same `llama_decode` batch.

**Corrected flags for the two instances:**

```bash
# bulk: 35k short-to-medium requests
llama-server -m MODEL.gguf --port 8080 --host 127.0.0.1 \
  -np 4 --kv-unified-per-slot 8192 \
  -b 2048 -ub 512 -fa on \
  -ctk q8_0 -ctv q8_0 \
  -cram 1024 --no-cache-idle-slots \
  --no-webui --metrics -to 3600

# long-context: the single 135k-token document
llama-server -m MODEL.gguf --port 8081 --host 127.0.0.1 \
  -np 1 -c 139264 -b 2048 -ub 512 -fa on \
  -cram 0 --no-webui --metrics -to 3600
```

Two instances means two copies of the weights in 24 GB of unified memory. **That is probably
the binding constraint** — run them sequentially if both do not fit. Quantized V cache
requires flash attention, which is why `-fa on` and `-ctv q8_0` travel together.

**Client behavior the Rust driver must implement:**

- **All slots busy queues, it does not reject.** Firing 35,000 requests would queue 35,000
  tasks in RAM with 35,000 open connections. Bound concurrency client-side with a
  `tokio::sync::Semaphore` sized to `-np`. `GET /slots?fail_on_no_slot=1` returns 503 and is
  the backpressure probe; `/health` returns 200 even with every slot busy and is useless for
  this.
- **One bad decode 500s every in-flight request** and clears every processing slot's KV. So
  a 500 does not mean your request was bad. Treat it as retryable with backoff, and
  pre-check lengths with `POST /tokenize` so it never fires.
- **`json_schema` can silently fail open.** Schemas that expand past
  `MAX_REPETITION_THRESHOLD` (2000) log a warning, **return 200 OK, and generate
  unconstrained output**. Always validate the returned JSON in Rust; never trust the
  constraint. Prefer raw GBNF over `json_schema` for this reason.
- **Invalid GBNF is a 400 on current master** but older builds logged an error and continued
  unconstrained at 200. Verify your build with a deliberately broken grammar before trusting
  it.
- Use `POST /completion`, not the OpenAI-compatible path: only `/completion` returns
  `stop_type`, so only it lets you distinguish "hit the stop token" from "hit `n_predict`".
  The most likely grammar failure mode is truncation at `n_predict`, so budget it generously
  and check `stop_type` on every response.

**GBNF repetition:** `{m}`, `{m,}` and `{m,n}` are supported; **`{,n}` is not** — write
`{0,n}`. `max_times > 2000` is **silently converted to unbounded**. Expansion is linear in
`n`, so `char{1,100}` creates ~100 rules and `{1,1000}` is expensive. The grammar is
re-parsed on **every request** with no cache, so keep it small and flat. The `qstring
::= "\"" qchar{20,180} "\""` in the gold-set grammar is fine; nothing nested.

**Two live measurements, free in every response.** `timings.cache_n / (cache_n + prompt_n)`
is the actual prefix-cache hit fraction, and the Prometheus gauge
`llamacpp:n_busy_slots_per_decode` is the batching-efficiency number: **if it sits near 1.0
with `-np 4`, batching is not happening and `-np` is buying nothing.** That single gauge
answers the contested 2.9x question without a separate benchmark.

**Stability for a multi-hour run.** A genuine ~3-6 MB per-request growth is reported (not
confirmed on Metal) that plateaus rather than growing unbounded; combined with the 8 GiB
`-cram` default this is what people report as a leak. Monitor RSS, set `-cram` explicitly,
and make the driver resumable so a server restart costs one unit. Grammar handling can abort
the process in edge cases, so supervise it.

---

## 7. Still unverified, in priority order

> **Items 1, 5 and 6 are blocked from this workspace.** `huggingface.co` is denied by the
> account's egress allowlist on both the cloud side and your computer's shell (the proxy
> answers 403 to CONNECT). They need either your own terminal, which is not under that
> policy, or `huggingface.co` added to the allowlist (Admin settings, Capabilities). The
> model files and the gte-modernbert tokenizer all live there, which is also why §8 uses a
> chars/4 token proxy.

1. **Whether the Ettin ONNX contains the scoring head.** §5 settles it in one command. Until
   then the reranker design has two branches.
2. **Whether `-np` scales on this machine.** `llama-batched-bench` at 1, 2, 4, 8, or simply
   watch `llamacpp:n_busy_slots_per_decode` during the first hundred requests. No published
   Apple Silicon batched-throughput numbers exist; every figure I have seen is CUDA.
3. **Whether the VPS CPU has VNNI.** One grep, and it decides INT8 versus fp32.
4. **The 1,797 vs 1,862 scripted-story discrepancy.** Narrowed in §8.4; one API call closes it.
5. **Opset versions** of both ONNX files. Same script as §5 prints them.
6. **Where fastembed applies L2 normalization** — confirmed it is *not* in `pooling.rs`.
   Check before assuming unit-norm vectors, since it decides whether you store raw dot
   products or cosine.
7. **`EnvironmentBuilder::commit()` returning `bool` rather than `Result`** — docs.rs says
   `bool` and the official example calls it bare. If the compiler disagrees, add `?`.

---

## 8. Measured on the real corpus, 2026-09-24

**Method.** The backend's `core/story/parser.rs` copied verbatim with only its `ts-rs` and
`utoipa` derive attributes removed; its own test suite passes on the copy. The file probe
copied verbatim from `core/story/mod.rs`. Every story in `story_review_table` resolved and
chunked once. Tokens counted as chars/4, because the embedder's tokenizer sits behind the
egress block; on 2026-09-22 chars/4 agreed with 1.45 tok/word within 2.3%, so boundaries
here are approximate by a few percent, not wrong in shape. The corpus snapshot was staged
from the repo as one tarball, left at `scratchpad/_to_delete/trevor-corpus-probe.tar.gz`
(14.9 MB, git-ignored) for you to delete.

### 8.1 The corpus

- **5,092 script files on disk**, up from 4,860 on 2026-09-22.
- **The `[uc]info` swap is fixed on disk.** 0 of 1,862 resolving stories come from
  `[uc]info/`, matching the `mod.rs` note of 2026-09-23.
- **1,887 table entries, 1,887 distinct `StoryId`, 0 listed twice.** 1,862 resolve.
- **4,360,146 prose words** by the backend's own `word_count`. The 4,108,615 of 2026-09-22
  used a regex prose rule on an older tree; the backend's rule also counts 32,292 bare
  `text` lines.
- **The 25 unresolved are two whole events never extracted, not a probe bug:**
  `act24side` "A Flurry to the Flame" (19 of 19 stories) and `act13mini` "It's Been A While"
  (6 of 6), with zero matching files anywhere under `gamedata/story`. It's an extraction gap,
  and until it is fixed the lore model will not know those stories exist.

### 8.2 What the chunker got wrong, and the fix

Before and after, same corpus, same parser, same token proxy:

| | before | after |
|---|---|---|
| chunks | 13,578 | **13,396** |
| scene boundaries | 15,521 | **13,700** |
| tokens p10 / p50 / p90 / max | 289 / 534 / 594 / 1,513 | **309 / 541 / 595 / 1,513** |
| under 400 / over 600 | 16.9% / 0.4% | **15.4% / 0.5%** |
| chunks whose `background` names a CG | 986 of 13,439 | **64 of 13,245** |
| chunks with empty `on_screen` | 320 | **242** |
| exact prose reconstruction failures | 0 | **0** |

What moved it:

1. **`[Image]` was treated as a scene boundary. It is a CG overlay.** The reader doc: a new
   `[Image]` fades in over the still-opaque scene and a bare one fades it out; the resolver
   classes an `[Image]`-written plate as a CG even when it is named `bg_*`. The error opened
   1,973 scenes and put a CG name in the `background` field of 986 chunks (7.3%), which the
   contextual-prefix pass would have read as the location. The remaining 64 are real
   backgrounds: exactly three plate names are used both ways (`ac5_2_on`, `bg_0_ori`,
   `bg_towerinside`), and **0 chunks carry a background that no background command set.**
2. **Strip backgrounds were read as fades.** `gridbg`, `largebg` and `verticalbg` name their
   plate with `imagegroup` only (86, 60 and 17 uses, never `image`). The chunker now reads the
   backend's five `BACKGROUND_KINDS` and both argument names.
3. **`charactercutin` joins the stage** without clearing it (393 named uses). `interlude`
   stays out: it names a sprite in only 258 of 377 named uses.
4. **A dead branch removed.** The `end == start` oversize path never ran: every break is
   guarded by `end > start`, so an oversized turn closes through the ceiling branch. Output
   was correct; the code claimed a path that did not exist. 73 of 13,396 chunks are oversized
   single turns.

**Did not move:** 1,224 of 1,862 stories produce byte-identical chunk text; the largest
chunk stays 1,513; p90 moves by 1; total tokens move by 1,242 of 6.59M (0.02%), from fewer
speaker prefixes where same-speaker lines now merge inside a longer scene.

### 8.3 Refuted, with what killed each

- **"About half of `[Background]` commands are fades, so breaking on all of them doubles the
  scene count."** Killed by the real parser: 822 of 15,039 name no plate, 5.5%.
- **"Scenes average ~290 tokens, so they are packed, not split."** Killed: 13,700 scene
  boundaries against 13,396 chunks. Scenes are roughly chunk-sized.
- **"114 stories fail the lossless check."** That was the check: it stripped words after any
  `": "`, eating text from unattributed lines containing a colon. Exact reconstruction, which
  rules out loss *and* duplication, holds for 1,862 of 1,862.
- **"Widening to all five background kinds costs nothing (+8 scenes)."** Not recorded as a
  result: that A/B read `image` only, so it could never see the three strip kinds. The
  measurement tested an incomplete rule.

### 8.4 The 1,797 figure, narrowed

Ruled out by measurement: the older tree (1,862 resolved on the 2026-09-22 tree too); the
16-byte `has_script` probe (it also resolves 1,862, with 0 files whose first `[` sits past
byte 16); and filtering in the index builder (it takes every table entry plus operator
records, with no filter that drops stories). The figure is most likely stale. **AMBIGUOUS
until one call to `/api/story/index` on the current tree reports `totals.withScript`.**

### 8.5 The two chunking knobs, priced but not set

Full sweep, chars/4 proxy:

| scene cap | tail merge | chunks | p10 | p50 | p90 | under 400 | over 600 |
|---|---|---|---|---|---|---|---|
| 2 | off | 14,553 | 191 | 510 | 594 | 26.8% | 0.4% |
| 2 | on | 13,733 | 257 | 529 | 597 | 21.4% | 4.0% |
| **3** | **off** | **13,578** | **289** | **534** | **594** | **16.9%** | **0.4%** |
| 3 | on | 12,890 | 373 | 547 | 597 | 11.9% | 4.3% |
| 4 | off | 13,344 | 330 | 539 | 595 | 13.8% | 0.5% |
| 4 | on | 12,683 | 403 | 550 | 597 | 8.9% | 4.5% |
| 5 | off | 13,292 | 344 | 540 | 595 | 13.0% | 0.5% |
| 5 | on | 12,637 | 406 | 551 | 597 | 8.2% | 4.5% |
| none | off | 13,243 | 354 | 541 | 595 | 12.3% | 0.5% |
| none | on | 12,602 | 408 | 552 | 597 | 7.7% | 4.5% |

(Swept before the §8.2 fix; the bold row is the old default.) Tail merge folds a sub-floor
final chunk into its predecessor when the pair stays within 750. It trades under-floor for
over-ceiling roughly one for one: at cap 3, under-400 falls 5.0 points and over-600 rises
3.9. The scene cap flattens fast: 3 to 4 removes 3.1 points of under-400, 4 to none 1.5
more. **No knob change is justified by this sweep.** Chunk size is a proxy and retrieval
recall is the objective, which needs the eval. The eval should A/B scene cap {3, none} x
tail merge {off, on}, four configs, already priced here.
