# trevor

Trevor is myrtle.moe's Arknights lore assistant: it answers story questions from retrieved passages, with citations,
using local models and no paid LLM API. This crate holds the corpus build (ingest, scene-aware chunking, embeddings),
hybrid retrieval (BM25 and dense, fused by RRF, optional cross-encoder rerank), the table tools for structured questions
(reading order, canon, deaths, topics, operators), the `ask` answerer with its HTTP and worker modes, and the
retrieval and answer evals.

Design notes are in `design/` (start at [design/README.md](design/README.md)); the experiments behind every default are
in [design/experiment-log.md](design/experiment-log.md). Installing on a server: [deploy/README.md](deploy/README.md).
The backend's Trevor settings are in [../backend/README.md](../backend/README.md#trevor).

All commands run from this directory. Anything that embeds, reranks or answers needs `--features embed`, which
downloads ONNX Runtime 1.28 on the first build. To use a system ONNX Runtime (1.23 or newer) instead, build with
`--features embed-dylib` and set `ORT_DYLIB_PATH`.

## Build and test

```sh
cargo build --release --features embed --bins
cargo test --release --features embed
cargo clippy --release --all-targets --features embed
./scripts/fetch-model.sh                 # embedder + reranker, pinned and sha256-checked (~220 MB, once)
WITH_LLM=1 ./scripts/fetch-model.sh      # + the Gemma 4 12B answer model and the Qwen3.5 9B judge (GGUF)
WITH_FP32=1 ./scripts/fetch-model.sh     # + the fp32 embedder export (portable across CPUs)
```

## Build the corpus

The myrtle backend must be running (default `http://127.0.0.1:3060`).

```sh
target/release/build-corpus --tokenizer models/gte-modernbert-base/tokenizer.json
target/release/embed-corpus --model-dir models/gte-modernbert-base
target/release/build-index
```

Everything lands in `artifacts/` (git-ignored): `chunks.jsonl`, `spoiler.jsonl`, `unresolved.jsonl`,
`manifest.p0.json`, `vectors.bin`, `vectors.meta.json` and `bm25/`. `build-corpus` exits nonzero if any story failed
to fetch, failed lossless reconstruction, or disagreed with the API's word count, and it resumes from where a killed
run stopped. Every derived index records the sha of the `chunks.jsonl` it was built from and refuses a stale one.
`embed-corpus` reuses vectors whose text is unchanged; long runs are paced and resumable (`--threads`, `--duty`,
`--max-minutes`, `--checkpoint-every`; see [design/trevor-reembed.md](design/trevor-reembed.md)).

The derived corpora `ask` reads (`artifacts/p3b`, `artifacts/p4x` and the table-tool data) are built by
`scripts/update.sh`, which redoes only what changed after a game update (`PLAN=1` prints the plan).

Measured on the live backend: 1,887 stories, 1,862 with a script, 13,837 chunks, 0 lossless failures. On an Apple M5
Pro the embed takes 28 minutes and the BM25 index 2.4 seconds.

## Search

```sh
target/release/search "Why does Ch'en leave the Lungmen Guard Department?"
target/release/search "Who is Enciodes?" --mode bm25 --k 5
target/release/search "What is Oripathy?" --rerank-top 40
```

`--mode` is `hybrid` (default), `dense` or `bm25`. `--rerank-top N` reranks the fused top N with ettin-reranker-17m;
0 (the default) turns it off. On an Apple M5 Pro a hybrid query takes 15 ms and a 40-deep rerank 550 ms.

Use the fp32 embedder for the corpus and the queries: the INT8 export's vectors depend on the CPU and the ONNX Runtime
build (cosine 0.94 to 0.95 across them), fp32 vectors do not. `search` and `ask` refuse a query model other than the
one the vectors were built with (`--allow-query-model-mismatch` overrides).

## Ask

```sh
target/release/ask "Who is Kal'tsit?"                   # one answer with citations
target/release/ask --batch eval/real-probe.jsonl --out artifacts/answers/run.jsonl
target/release/ask --serve --port 8090                   # local HTTP JSON API: POST /v1/ask, GET /health
TREVOR_WORKER_KEY=... target/release/ask --worker http://127.0.0.1:3060   # pull jobs from the backend
```

`ask` starts a llama-server with the Gemma 4 12B model on `--server` (default `http://127.0.0.1:8081`) when none
answers there, and stops it afterwards (`--no-spawn` fails instead). One model runs at a time: `ask` takes a model lock
(`--lock`). `--profile full|slim|mid|mid6k` picks the pipeline (the backend's `cpu` rung serves `mid`),
`--retrieval bm25` skips the dense index, and `--backend openai` answers through an OpenAI-compatible endpoint
(Cloudflare Workers AI for the `cf` rung). Every default added since the first version has a kill switch (mostly
`--no-...`); `ask --help` lists them, and the experiment log records why each exists.

## Evaluate

```sh
# Retrieval: is the gold set consistent with the current chunks? Then score a configuration.
target/release/eval validate --goldset eval/goldset_v2.jsonl
target/release/eval run --goldset eval/goldset_v2.jsonl --corpus artifacts/p3a --mode hybrid --label my-change --baseline last
# Tier A: corpus facts and exact top-10 ids for eval/golden-queries.txt (--update re-records).
target/release/eval golden
# Answers: run ask over a set, then judge them with the Qwen model on :8081.
python3 scripts/answer-eval.py eval/goldset_v2.jsonl artifacts/answers/run.jsonl
# Routing: which questions a table answers, no model needed for the keyword router.
python3 scripts/route-check.py
```

The retrieval metric is recall@10. A run against a baseline is judged on the paired difference: FAIL when the whole
95% bootstrap CI is below zero, WARN when the estimate is negative but the CI reaches zero. `--gate` makes FAIL, or
hit@10 at or below 0.40, exit nonzero. Runs append to `eval/runs.jsonl`; per-question detail goes to `eval/runs/`
(git-ignored).

Question sets in `eval/`: `goldset_v2.jsonl` (146 generated and judged questions with anchors), `real-probe.jsonl`
(120 real player questions), `ian-questions.jsonl` (the 35-question probe set with verified answers),
`lore-heldout.jsonl`, `trivia.jsonl` and `routes.jsonl` (the routing regression set). `dev-set.jsonl` is an early
49-question development set written by a model from sampled chunks; its questions borrow names from their passages,
which favours BM25 (the gold passage is BM25's first hit for 27 of 47), so the gold set replaced it.

## Layout

- `src/corpus/`: `chunk/` (scenes, speaker turns, token packing), `ingest/` (fetch, resume, manifests), `embed/`
  (corpus and query embedders, pacing, checkpoints).
- `src/search/`: `store.rs`, `bm25.rs`, `dense.rs`, `rerank.rs` (ONNX backbone plus the safetensors scoring head),
  `pipeline.rs`, `runtime.rs`, `tokenizer.rs` (the lore analyzer and RRF), `horizon/` (the spoiler horizon).
- `src/tools/` (table tools), `src/router/` (model and keyword routers), `src/llm/` (llama.cpp and OpenAI-compatible
  clients), `src/eval/` (gold sets, metrics, statistics, run records, tier A), `src/goldgen/` (gold-set generation).
- `src/bin/`: the corpus builders, `search`, `eval`, `goldset/` and `ask/` (the answerer; `serve/` holds `--serve` and
  `--worker`).
- `prompts/`: every prompt and grammar, compiled in and hashed into outputs.
- `scripts/`: the Python stage scripts and the update pipelines; `deploy/`: systemd units and the env example.
- `tests/rerank_parity.rs`: the Rust reranker against sentence-transformers (max difference 6.4e-6 on 33 pairs);
  skipped unless `TREVOR_RERANK_DIR`, `TREVOR_RERANK_PAIRS` and `TREVOR_RERANK_REF` point at a reference.
