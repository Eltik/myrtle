# Trevor: Phase 0 status

Written 2026-09-24, updated the same day after the live run (section 3). This is the current state of P0 and supersedes `trevor-ingest-codex-brief.md` (implemented directly, with the changes listed in section 4) and items 6 and 7 of section 7 in `trevor-implementation-notes.md`. Section 8 of those notes used a chars/4 token proxy; the numbers here use the real gte-modernbert tokenizer.

## 1. Where it is and how to run it

The crate is at `myrtle/trevor/` (14 files, not committed). The README there has the three commands: `./scripts/fetch-model.sh` (pinned revision `e7f32e3c`, every file sha256-checked), `cargo run --release --bin build-corpus -- --tokenizer models/gte-modernbert-base/tokenizer.json` with the backend running on port 3060, then `cargo run --release --features embed --bin embed-corpus -- --model-dir models/gte-modernbert-base`. `cargo test` runs 26 tests; `cargo clippy --all-targets` is clean with and without the embed feature.

## 2. What is verified, and how

Ingest and chunking ran end to end against a mock of the two story routes that serves the real EN corpus through the backend's own parser and serde derives. Results: 1,887 stories in the index, 1,862 with a script, 25 without, 13,837 chunks, 0 lossless failures, 0 word-count mismatches, 15 seconds wall time. A second full run after the last code change produced `chunks.jsonl`, `spoiler.jsonl` and `unresolved.jsonl` byte-identical to the first. Earlier runs verified resume after a torn final line, 503 retries, a 404 on a story the index claims is scripted, and independence from the order groups arrive in.

Token accounting is exact: the stored `tokenCount` equals the full encode with special tokens for 13,837 of 13,837 chunks. The earlier version counted special tokens per turn and missed the newline joins, undercounting by 2 to 47 tokens and putting 26.1% of chunks over the 600 ceiling. With a calibrated 2-token sequence overhead and 1 token per join, 0.43% exceed 600, all of them single speaker turns that are never split. Corpus totals on the real tokenizer: 6,892,833 tokens, 1.581 tokens per word, 3.809 characters per token. The longest chunk is 1,355 tokens.

The embedder ran over all 13,837 real chunks with two synthetic ONNX encoders, before the real weights could be downloaded here. Both use the real tokenizer files and ONNX Runtime 1.25. The first returns each token's row of a random table, so CLS pooling must give one identical vector for every chunk: maximum deviation 0.0, 1 distinct row. The second makes position 0 depend on every token in the sequence, and the stored vectors match a Python reference built from `tokenizers` 0.23.2 to within 1.2e-6 (minimum cosine 0.99999987). That one check confirms CLS pooling, that `[CLS]` is prepended (13,837 of 13,837), that no chunk is truncated, that rows are in `chunks.jsonl` order, and that vectors are unit length (maximum norm error 1.8e-7). A tampered `tokenizer.json` is refused and leaves the existing `vectors.bin` untouched. Throughput on the toy is 1,130 chunks per second, which says nothing about the real model; section 3 has the real figure.

Two corrections to the implementation notes fall out of this. Section 4 suggested `with_max_length(512)`; that would silently truncate every chunk over 512 tokens, so the code uses 8,192, the model's trained context. Item 6 of section 7 is closed: fastembed normalizes in `text_embedding/output.rs`, and the code asserts unit norm per vector anyway.

## 3. The live run on the Mac, 2026-09-24

P0 has run for real on the M5 Pro against the live backend (the existing debug build, started for the run and stopped after it).

The live API reports `totals.withScript` = 1,862 of 1,887, so the 1,797 figure in `services/story.rs` and `docs/story-reader.md` is stale. Ingest took seconds: 1,862 fetched, 0 failed, 13,837 chunks, 0 lossless failures, 0 word-count mismatches. Every chunk is field-for-field identical to the mock run; only row order differs, because the live index adds 315 operator-record groups (367 stories, all already listed under event groups and deduplicated by id) and real group metadata that moves plan order.

The `embed` feature built on the Mac with ONNX Runtime 1.28 downloaded by ort-sys, so that path is now verified. Embedding took 1,691 s (28.2 min) at 8.2 chunks per second on about 5 cores and 650 MB resident, with no thermal or performance warning from `pmset` at any check. Output: 13,837 x 768, maximum norm error 1.37e-6, model sha `bae96b27`, tokenizer sha `6c8aaa9a`.

**Finding: the INT8 model is accurate on Apple Silicon and not on x86.** Against the official fp32 export on 40 random chunks, the Mac's INT8 vectors score cosine 0.9940 minimum, 0.9966 mean. The same INT8 file on x86 (a Xeon with AVX-512 VNNI and AMX) scores 0.9174 minimum, 0.9410 mean, and the result is identical on ONNX Runtime 1.25 and 1.28 and for the `model_uint8.onnx` variant, so it is the x86 kernel path, not the version or the file. The Rust embedder and a Python reference agree to cosine 1.00000 on x86, so the pipeline is not at fault. This refutes the VNNI gate in implementation notes section 4: VNNI is present here and INT8 is still off by 6 points of cosine. Consequence: never embed queries with INT8 on an x86 host and compare them to these vectors. A VPS should embed queries with the fp32 model (596 MB), which sits within 0.997 of the stored vectors. The gate for any new host is the fp32-vs-INT8 cosine on 20 chunks, not a CPU flag.

A dense-only smoke test with fp32 queries: name queries land well ("Who is Kal'tsit?" returns three Kal'tsit passages at 0.82 to 0.84; "Who is Enciodas Silverash?" returns his own introduction), while "why" questions do not ("Why does Ch'en leave the Lungmen Guard Department?" returns an unrelated side story second). That is expected of dense retrieval alone on dialogue and is what BM25, contextual prefixes and the reranker in P1 are for. Retrieval quality is judged by the eval, not by this.

**The Ettin reranker check is done.** `onnx/model.onnx` (opset 18) outputs `last_hidden_state [batch, seq, 256]`, not `logits`: the scoring head is missing, as predicted. The head ships as `2_Dense`, `3_LayerNorm` and `4_Dense` safetensors (262,232, 2,200 and 1,172 bytes), so the Rust head from implementation notes section 5 is the path. There is also a `model_qint8_arm64.onnx`, which would need the same fp32 comparison before use.

Artifacts on the Mac: `trevor/artifacts/` 74 MB, `trevor/models/` 148 MB, `trevor/target/` 793 MB, all git-ignored.

## 4. Where the implementation departs from the ingest brief

- Per-turn counting is WITHOUT special tokens, charged once per chunk through a calibrated overhead. The brief's "count special tokens on every encode" rule is what caused the 26.1% overshoot.
- Resume drops every line of the last story in the file, not only a torn line, because a story is written in one call and a crash can leave it partly flushed at a line boundary.
- `spoiler.jsonl` carries the index's raw fields (`startTime`, `sort`, `requiredStages`) and no invented `globalOrdinal`; chronology is a P1 decision.
- Retries are 4 attempts (3 retries), and fetch results are written in plan order, so the file is deterministic regardless of which request finishes first.
- `build-corpus` exits nonzero on any fetch failure, lossless failure or word-count mismatch.
- The story `main_14_level_main_14-20_beg` is a video with no prose; it yields zero chunks and is therefore refetched on every resume. This is harmless and documented in the code.
