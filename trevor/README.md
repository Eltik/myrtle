# trevor

Trevor is a local, zero-cost Arknights lore assistant: retrieval over the EN story corpus plus a local model, with no external LLM API. This crate holds the corpus build (ingest, scene-aware chunking, dense embeddings), hybrid retrieval (BM25 + dense, fused by RRF, optional cross-encoder rerank) and the retrieval eval harness. Design notes are in `design/` (start with `design/HANDOFF.md`), copied from the claude.ai project docs.

All commands run from this directory. Anything that embeds or reranks needs `--features embed`, which downloads ONNX Runtime 1.28 on first build. To use a system ONNX Runtime (1.23 or newer) instead, build with `--features embed-dylib` and set `ORT_DYLIB_PATH`.

## Build the corpus

The backend must be running (default `http://127.0.0.1:3060`).

```sh
./scripts/fetch-model.sh     # embedder + reranker, pinned and sha256-checked (~220 MB, once)
cargo run --release --bin build-corpus -- --tokenizer models/gte-modernbert-base/tokenizer.json
cargo run --release --features embed --bin embed-corpus -- --model-dir models/gte-modernbert-base
cargo run --release --bin build-index
```

Everything lands in `artifacts/` (git-ignored): `chunks.jsonl`, `spoiler.jsonl`, `unresolved.jsonl`, `manifest.p0.json`, `vectors.bin`, `vectors.meta.json` and `bm25/`. `build-corpus` exits nonzero if any story failed to fetch, failed lossless reconstruction, or disagreed with the API's word count; it resumes from where a killed run stopped. Every derived index records the sha of the `chunks.jsonl` it was built from and refuses to load against a different one.

Measured on the live backend: 1,887 stories, 1,862 with a script, 13,837 chunks, 0 lossless failures. The embed takes 28 minutes on an M5 Pro; the BM25 index 2.4 seconds.

## Search

```sh
cargo run --release --features embed --bin search -- "Why does Ch'en leave the Lungmen Guard Department?"
cargo run --release --features embed --bin search -- "Who is Enciodes?" --mode bm25 --k 5
cargo run --release --features embed --bin search -- "What is Oripathy?" --rerank-top 40
```

`--mode` is `hybrid` (default), `dense` or `bm25`. `--rerank-top N` reranks the fused top N with ettin-reranker-17m; 0 (the default) turns it off. On the M5 Pro a hybrid query takes 15 ms and a 40-deep rerank 550 ms.

On x86 hosts, fetch the fp32 embedder (`WITH_FP32=1 ./scripts/fetch-model.sh`) and pass `--query-onnx model.onnx`: the INT8 export drifts to cosine 0.94 from fp32 on x86, against 0.997 on Apple Silicon.

## Evaluate

```sh
# Is the gold set consistent with the current chunks?
cargo run --release --features embed --bin eval -- validate --goldset eval/dev-set.jsonl
# Score a configuration; --baseline last compares with the previous run on the same set.
cargo run --release --features embed --bin eval -- run --goldset eval/dev-set.jsonl --include-unreviewed \
  --mode hybrid --label my-change --baseline last
# Tier A: corpus facts and exact top-10 ids for eval/golden-queries.txt.
cargo run --release --features embed --bin eval -- golden          # check
cargo run --release --features embed --bin eval -- golden --update # re-record after an intended change
```

The primary metric is recall@10. A run against a baseline is judged on the paired difference: FAIL when the whole 95% bootstrap CI is below zero, WARN when the estimate is negative but the CI reaches zero. `--gate` makes FAIL, or hit@10 at or below 0.40, exit nonzero. Runs append to `eval/runs.jsonl`; per-question detail goes to `eval/runs/` (git-ignored).

`eval/dev-set.jsonl` is a 49-question development set written by Claude from sampled chunks, every item `review: pending`. It is not the gold set: its questions borrow names from their source passages, which favours BM25 (the gold passage is BM25's first hit for 27 of 47). The 150-question gold set replaces it.

## Layout

- `src/corpus/`: `chunk.rs` (scenes, speaker turns, token packing), `ingest.rs` (fetch, resume, manifests), `embed.rs` (embedder and query embedder).
- `src/search/`: `store.rs` (the chunk rows every index shares), `bm25.rs`, `dense.rs`, `rerank.rs` (ONNX backbone plus the safetensors scoring head), `pipeline.rs` (stages and their off switches), `runtime.rs` (loading, shared by `search` and `eval`), `tokenizer.rs` (the lore analyzer and RRF).
- `src/eval/`: `goldset.rs`, `metrics.rs`, `stats.rs`, `run.rs` (scoring, run records, the gate), `golden.rs` (tier A).
- `tests/rerank_parity.rs`: the Rust reranker against sentence-transformers (max difference 6.4e-6 on 33 pairs); skipped unless `TREVOR_RERANK_*` point at a reference.

`cargo test` runs 40 tests; `--features embed` adds 2 more plus the parity test.
