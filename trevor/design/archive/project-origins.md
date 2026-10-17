# Trevor: project origins and early decisions (2026-09-22 to 2026-09-25)

This is the record of how Trevor started: the requirements it was built to, the research conclusions, the corpus and backend API facts, the Phase 0 build with its measurements, the retrieval baseline and eval harness, the automated gold-set design, and the questions that were still open on 2026-09-25. It replaces a verbatim build log and keeps every number, file name, API shape and decision from it. Times are UTC. Everything after 2026-09-25 is in [`../experiment-log.md`](../experiment-log.md); the state at the end of this period is in [`HANDOFF.md`](HANDOFF.md).

## Goals and constraints

Trevor is a lore model for Arknights: it analyzes the story, answers questions, gives summaries and understands the lore. It is a new sibling sub-repo of myrtle.moe (`myrtle/trevor/`).

- Requirement: zero cost and fully local. No external LLM that requires an API, no provider keys, no hosted inference.
- Requirement: the work started as design only. No edits were made until the approach had been theorized, and the two design branches first proposed were both costed before the zero-cost requirement was set.
- Requirement: the surfaces are a myrtle.moe web chat, a Discord bot command, and an enriched version of the existing story reader.
- Requirement: spoilers are gated on the user's synced story progress.
- Requirement: research mode. Verify everything against source code, docs and measurements, and do not write from recollection.
- Requirement: keep the development machine clean. Stop every background process started for a step, delete temp logs, and leave no scheduled or recurring jobs.
- Requirement: git commits carry no co-authorship or attribution trailers of any kind.
- Requirement: the house rules apply: prose paragraphs, no em-dashes, a number on every claim, report what did not move, record refutations, and every new parameter gets a kill switch that is verified inert.
- Requirement: the gold-set review is automated rather than done by hand (2026-09-24).

## Research conclusions

- The design is retrieval-augmented generation over the EN story corpus (1,862 scripted stories), with no knowledge fine-tuning: the research found that continued pretraining or SFT on facts hurts.
- A behavior-only LoRA (RAFT-style) is an optional later step.
- RRF's k is a config knob with a default of 20. k=20 is NOT evidence-backed. `trevor-research.md` carries an RRF correction banner.
- llama-server facts for P1 and P2:
  - `-c` is the total context divided by `-np`; use `--kv-unified-per-slot`.
  - `--mlock` does not exist on master.
  - Set `-cram 1024`; the 8 GB default risks OOM.
  - Requests queue when slots are busy, so bound client concurrency with a semaphore.
  - A single decode error returns 500 for every in-flight request.
  - Use `/completion` to get `stop_type`.
- Eval design: 150 gold questions anchored on (story_id, line_start, line_end), not on chunk_id. The closed-book control gate is at 30%. FactCG/HHEM serve as the judge.
- Build order: build the measuring stick before improving anything, so every later change comes with a number. The order set on 2026-09-24 was (1) the search baseline, (2) the eval harness, (3) the question set, then (4) Phase 1 onward: a short context note added to every chunk (about 3.5 hours overnight), then summaries and a character index, each kept only if the eval shows it helps.

## Corpus and backend API facts

- `GET /api/story/index` returns StoryIndex {groups[].stories[], records[].stories[], totals{stories, withScript, words}}, camelCase.
- `GET /api/story/{id}` returns StoryScript {id, name, groupId, commands[], assets, wordCount}.
- StoryCommand is {kind, args (BTreeMap), text?, line}.
- PROSE_KINDS are [name, text, multiline, narration]. PROSE_ARG_KINDS are [subtitle, sticker], whose prose is in args["text"].
- Scene boundaries:
  - The backend's BACKGROUND_KINDS (background, backgroundtween, largebg, gridbg, verticalbg), named through `image` OR `imagegroup`. The strip kinds use `imagegroup` only.
  - `header` counts, and there is exactly one per story.
  - `[Image]` is NOT a boundary; it is a CG overlay.
  - Soft boundaries: decision, curtain.
- Corpus counts:
  - 5,092 files on disk (4,860 on 2026-09-22). The [uc]info swap is fixed on disk.
  - The index has 1,887 entries with 1,887 distinct ids; 1,862 resolve to a script.
  - 4,360,146 prose words by the backend's word_count.
  - 1.581 tokens per word, 3.809 characters per token, 6,892,833 real tokens.
  - The 25 missing stories are two events the assets unpacker never extracted: act24side "A Flurry to the Flame" (19 stories) and act13mini "It's Been A While" (6). This gap is in the unpacker, not in Trevor, and it affects the story reader too. A rerun after extraction picks them up.
- The live API reports `withScript: 1862`. The 1,797 figure in a backend code comment (`services/story.rs`) is stale.
- The zero-chunk video story `main_14_level_main_14-20_beg` is always refetched by design.
- The backend runs on port 3060 and authenticates service calls with the `x-service-key` header.

## Phase 0 design

Phase 0 (P0) is ingest, chunking and embedding. It produces `chunks.jsonl`, `spoiler.jsonl`, `unresolved.jsonl`, `manifest.p0.json`, `vectors.bin` and `vectors.meta.json`.

### Crate

`trevor/Cargo.toml` at the end of Phase 0:

```toml
[package] name="trevor" version="0.1.0" edition="2024" rust-version="1.86"
[dependencies]
anyhow="1"; clap={version="4",features=["derive"]}; futures="0.3"
reqwest={version="0.13",default-features=false,features=["json"]}
serde={version="1",features=["derive"]}; serde_json="1"; sha2="0.10"
tantivy="=0.26.2"; thiserror="2"
tokenizers={version="=0.23.2",default-features=false,features=["onig"]}
tokio={version="1",features=["rt-multi-thread","macros","fs","sync","time","io-util"]}
fastembed={version="=7.0.1",default-features=false,optional=true}
[features]
default=[]
embed=["embed-core","fastembed/ort-download-binaries-rustls-tls"]
embed-dylib=["embed-core","fastembed/ort-load-dynamic"]
embed-core=["dep:fastembed"]
[lib] name="trevor" path="src/lib.rs"
[[bin]] name="build-corpus" path="src/bin/build-corpus.rs"
[[bin]] name="embed-corpus" path="src/bin/embed-corpus.rs" required-features=["embed-core"]
```

`src/lib.rs` declares `pub mod corpus; pub mod search;`. `src/corpus/mod.rs` declares `chunk`, `ingest`, and `embed` behind `#[cfg(feature = "embed-core")]`.

### Chunking (`src/corpus/chunk.rs`, 21 tests)

- Exports StoryCommand (camelCase Deserialize/Serialize), PROSE_KINDS, PROSE_ARG_KINDS and Chunk {chunkId `"{story_id}#{ordinal:04}"`, storyId, groupId, ordinal, sceneOrdinal, lineStart, lineEnd, text, speakers, onScreen, background, tokenCount, contentSha (the first 16 hex of a sha256)}.
- ChunkConfig is {target_min_tokens 400, target_max_tokens 600, max_scenes_per_chunk 3, overlap_turns 0, sequence_overhead_tokens 0, joiner_tokens 0}.
- `hard_boundary` reads `image` or `imagegroup` on the BACKGROUND_KINDS; `[Image]` is not a boundary; `header` opens a scene.
- Turns merge consecutive same-speaker lines within a scene and render as `"{speaker}: {body}"`. Masked speakers are "", ?, ?? and ???.
- on_screen is running state built from charslot (a slot map), character (name/name2, which replaces), and charactercutin (separate state). interlude is excluded: 258 of its 377 named uses are sprites.
- Packing groups speaker turns into 400 to 600 tokens with a scene cap of 3 and never splits a turn:

  ```rust
  let mut total = cfg.sequence_overhead_tokens;
  let join = if end > start { cfg.joiner_tokens } else { 0 };
  let next_total = total.saturating_add(join).saturating_add(costs[end]);
  ```

  A chunk breaks at the ceiling, at the scene cap, or at a seam (at least the minimum at a new scene or after a soft boundary). `debug_assert!(end > start)` replaced a dead oversize branch. `token_count` is `count_tokens(&text).saturating_add(cfg.sequence_overhead_tokens)`.
- `chunk_story(story_id, group_id, commands, cfg, count_tokens: &dyn Fn(&str)->u32) -> Vec<Chunk>` documents that `count_tokens` must count WITHOUT special tokens.
- Token accounting: the per-turn count excludes special tokens. `sequence_overhead_tokens` (2 for gte-modernbert) and `joiner_tokens` (1 per `\n`) are calibrated from the tokenizer.
- Lossless reconstruction is verified on every story.

### Lexical search (`src/search/tokenizer.rs`, 7 tests)

- tantivy 0.26.2 with the RegexTokenizer `[\p{L}\p{N}]+(?:['\u{2019}\-.][\p{L}\p{N}]+)*`, a LowerCaser, and a LoreNormalizer that folds U+2019 and strips possessives. No stemmer.
- Exports LORE_TOKENIZER "lore", LORE_PATTERN, the LoreNormalizer filter, `lore_analyzer()`, `lore_schema()` (chunk_id STRING|STORED; body WithFreqsAndPositions, not stored), `open_index(path)` (registers the analyzer), `analyze(index, text)` and `rrf(lists, k)`.
- Every index open goes through `open_index`, because custom tokenizers are lost on reopen.
- `TopDocs::with_limit(n).order_by_score()` is required in 0.26.
- QueryParser cannot parse a bare apostrophe.

### Ingest (`src/corpus/ingest.rs`, 5 tests)

- Wire types: StoryIndex, StoryTotals, StoryGroup (category as Value, startTime i64, chapterNumber), OperatorRecordGroup, StoryEntry, StoryScript.
- `plan(&index) -> (Vec<Job>, Vec<StoryEntry>)` sorts groups by (startTime, id), stories by (sort, id) and records by charId, dedupes by id (first wins), and sets unscripted stories aside.
- `recover(path)` cuts a torn final line, drops every line of the LAST story so it is refetched, and returns the done set plus Recovery {tornBytesDiscarded, droppedLastStory, linesDropped}.
- `reconstructs(commands, chunks)` is an exact word-sequence check that strips the longest speaker prefix.
- The fetcher uses reqwest with a 45 s timeout and `x-service-key` from `TREVOR_SERVICE_KEY`. It retries 5xx and transport errors for 4 attempts with a backoff of 500 ms << n; a 4xx fails without retry.
- `run(cfg, count_tokens: Arc<dyn Fn(&str)->u32+Send+Sync>) -> Manifest`:
  - writes `spoiler.jsonl` (SpoilerRow raw fields, no invented chronology) through tmp+rename;
  - fetches with `futures::stream::iter(...).buffered(concurrency)` so file order is deterministic;
  - chunks in `spawn_blocking` and writes each story in one `write_all`;
  - writes `unresolved.jsonl`;
  - writes `manifest.p0.json` {phase, builtAtUnix, backendBase, totalsFromApi, scriptedStories, unscriptedStories, storiesSkippedAsDone, storiesFetched, storiesFailed, chunksWrittenThisRun, chunksInFile, losslessFailures, wordCountMismatches, recovery, tokenizerSha, chunkConfig};
  - `Manifest::clean()` decides success.
- A panic placeholder Job was replaced by jobs carrying (id, gid, Result<Outcome,String>).

`src/bin/build-corpus.rs` takes `--base` (default `http://127.0.0.1:3060`), `--out` (`artifacts`), `--tokenizer` (required), `--concurrency 4`, `--limit`, `--min-tokens 400`, `--max-tokens 600` and `--max-scenes 3`. It sets `TOKENIZERS_PARALLELISM=false`, computes the tokenizer file's sha256, and runs `calibrate(tok)`, which derives the overhead from a probe string and the joiner cost from 4 fixed pairs and bails if they disagree. It prints a summary and exits nonzero if `!m.clean()`.

### Embedding (`src/corpus/embed.rs`)

- Embedder: gte-modernbert-base, the official `onnx/model_int8.onnx` (150,218,016 bytes), CLS pooling, through fastembed 7.0.1, which pins ort =2.0.0-rc.13. fastembed L2-normalizes the output (`output.rs`).
- Dynamic quantization is batch-dependent, measured: cosine 0.999977 against the text embedded alone, 0.999823 with a mate 5 times longer, and bit-identical when embedded alone twice. Decision: embed one text per call, for documents and queries alike.
- MODEL_MAX_TOKENS is 8192. EmbedConfig is {model_dir, onnx_file, corpus, intra_threads}.
- It refuses to run if the tokenizer.json sha does not match `manifest.p0.json` tokenizerSha, and bails if any chunk's token_count is over 8192.
- It builds `UserDefinedEmbeddingModel::new(onnx, TokenizerFiles{tokenizer_file, config_file, special_tokens_map_file, tokenizer_config_file}).with_pooling(Pooling::Cls).with_quantization(QuantizationMode::Dynamic)`, then `InitOptionsUserDefined::new().with_max_length(8192)`, optionally `with_intra_threads`, then `TextEmbedding::try_new_from_user_defined`.
- Each chunk is embedded alone: `embedder.embed([c.text.as_str()], None)`. It checks that the dimension is consistent and bails if |norm - 1| > 1e-3.
- It writes f32 LE row-major vectors to `vectors.bin` (tmp+rename) and `vectors.meta.json` {modelFile, modelSha, tokenizerSha, chunksSha, dim, count, dtype "f32-le", layout, pooling "cls", normalized true, batchSize 1, maxNormError, builtAtUnix, seconds, chunkIds}.
- An earlier note said to cap input at 512 tokens. That would have silently cut every chunk longer than 512 (the longest is 1,355 tokens); the code uses the model's full 8,192.

`src/bin/embed-corpus.rs` takes `--model-dir`, `--onnx-file` (default `model_int8.onnx`), `--corpus` (`artifacts`) and `--threads`, and prints progress every 500 chunks.

### Probe workspace (outside the repo)

The measurements above were made in a scratch probe workspace, not in the repo:

- `probe/`: the unpacked corpus plus the backend source. `probe/runner/` holds the real `parser.rs` with the ts-rs/utoipa derives stripped, plus chunker copies (chunk_old, chunk_new, chunk_fix and an instrumented chunk.rs).
- Probe bins: probe, diag, sweep, imgab, bgargs, census, ab, verify64, realtok, fixcheck and mockapi. `mockapi` serves the real corpus at `/api/story/index` and `/api/story/{id}`, with fault switches MOCK_FLAKY (503 twice), MOCK_404 and MOCK_SHUFFLE.
- `models/gte/`: tokenizer.json (3,583,228 bytes), tokenizer_config.json, special_tokens_map.json, config.json and 1_Pooling/config.json. `models/ettin/config.json`.
- `e2e/` held the run outputs a, b, c, d and f.
- A 14.9 MB probe tarball was left at `scratchpad/_to_delete/trevor-corpus-probe.tar.gz` in the repo for deletion.

### Errors fixed and claims refuted during Phase 0

- `TantivyError::IoError` expects an Arc: fixed with `.map_err(|e| TantivyError::IoError(std::sync::Arc::new(e)))`.
- Test expectations went stale after same-speaker merging was added; fixed.
- on_screen was computed from the chunk's own line range, but charslot comes before the dialogue. It is now running state.
- The probe's "114 lossless failures" were a check bug (it stripped text after any ": "). The exact check shows 0 of 1,862.
- An earlier A/B, "widening the background kinds costs +8 scenes", read only `image`, so it was invalid. The `imagegroup` finding superseded it.
- A CG-count comment said 960; the correct number is 986 (the first A/B built its CG-name set incrementally).
- Token undercount: per-chunk counts were 2 to 47 tokens low, and 26.1% of chunks were over 600. With the calibrated overhead and joiner the stored count equals the whole encode, and 0.43% of chunks are over 600. Each of those is a single speaker line, which is never split.
- The test helper `small()` lacked the new fields; it now uses `..ChunkConfig::default()`.
- A cargo build timed out at 2 minutes; builds use a 600 s timeout.
- Two delivery fixes: a failed re-read of `chunks.jsonl` could have reported an empty corpus as a clean build, and `vectors.bin` could end up paired with the metadata file from an older run. Both are fixed.
- Refuted by measurement: "half of backgrounds are fades" (822 of 15,039 = 5.5%); "scenes are about 290 tokens, smaller than chunks" (13,700 boundaries against 13,837 chunks); the 1,797-story explanations (tree age, a 16-byte has_script probe, index filters), settled by the live API's 1,862.
- The project doc `trevor-corpus-spec.md` was corrected: image is not a boundary, there are 5 background kinds, measured counts, the missing events, and the music-pair rule dropped.

## Phase 0 build and measurements

### Verification before the live run (2026-09-24)

- A full ingest against `mockapi`, serving the real corpus through the real parser, produced 13,837 chunks from 1,862 scripted stories. Reconstruction from the chunks was lossless and every story's word count matched. A second full run after the last code change gave byte-identical output. Ingest was verified deterministic, resumable, crash-safe, order-independent, and correct on retries and 404s.
- Every chunk's stored token count matches the real tokenizer exactly.
- The embedder was tested over all 13,837 real chunks with two toy ONNX encoders (the real gte tokenizer files plus a synthetic fp32 encoder with inputs `input_ids` and `attention_mask` and output `last_hidden_state` through a masked Gather over a 50368 x H table), loading libonnxruntime from the pip onnxruntime 1.25.0 wheel through `ORT_DYLIB_PATH` with `--features embed-dylib`. Both confirmed CLS pooling (every vector equals the normalized [CLS] row), no truncation, rows in chunk order, unit norms to within 1.8e-7, the `vectors.bin` layout and meta, and the tokenizer-sha refusal, which leaves the old vectors alone.
- The `--features embed` build downloads the ONNX Runtime from `cdn.pyke.io` while compiling, which the build sandbox blocked, so it compiled first on the development machine. The `embed-dylib` variant differs only in how the library is loaded; the README has the fallback.
- Build-sandbox egress at the time: allowed were the huggingface.co API and small files, pypi.org, and index.crates.io/static crates through cargo; blocked were `us.aws.cdn.hf.co` (the HF CDN for weights), `cdn.pyke.io` (ort binaries) and the crates.io API. Allowing `*.hf.co` fixed the model download. The local backend was not reachable from the sandbox.
- The crate was delivered to `myrtle/trevor/` (14 files, uncommitted). The project doc `trevor-p0-status.md` was added and the ingest brief marked as replaced.

The run commands, from `trevor/`:

```sh
./scripts/fetch-model.sh
cargo run --release --bin build-corpus -- --tokenizer models/gte-modernbert-base/tokenizer.json
cargo run --release --features embed --bin embed-corpus -- --model-dir models/gte-modernbert-base
```

`fetch-model.sh` downloads directly and checks each file's hash.

### Live run on the development machine (2026-09-24, 13:08 to 13:47)

- All five model files verified.
- The backend's debug build was started for the ingest and stopped right after; port 3060 was free again and the temp logs were deleted.
- The live API reports 1,862 scripted stories of 1,887. All 1,862 were fetched, none failed, giving 13,837 chunks; reconstruction was lossless and every word count matched.
- Every chunk is identical to the mock run; only the row order differs, because the live index includes operator records and real group metadata.
- Embedding: 13,837 x 768 in 28.2 minutes at 9.9 chunks/s on about 5 cores, using about 650 MB of RAM. macOS reported no thermal or performance warning at any check.
- All six output files are in `trevor/artifacts/`. `trevor/` took about 1 GB, mostly the 793 MB `target/` folder; all of it is git-ignored and deleting `target/` is safe.
- Reranker check: the Ettin reranker's ONNX file has no scoring head (it outputs last_hidden_state, not logits). The head ships as three small separate files, so Phase 1 adds that step in Rust.
- INT8 portability: on the development machine (Apple silicon, arm64) the INT8 vectors match the full-precision model at cosine 0.997. The same INT8 model on an Intel/AMD CPU drifts to 0.94, even on a CPU with the instruction support the earlier notes expected to fix it. Decision: if the site's server is x86, it embeds questions with the full-precision model (596 MB), or questions and stored vectors will not line up.
- Quick dense search test: "Who is Kal'tsit?" returns three Kal'tsit passages. A "why" question about Ch'en pulled in an unrelated side story, which is what Phase 1's keyword search and reranker are for.

## Search baseline and eval harness (2026-09-24)

- Search combines BM25 keyword and dense vector matching (hybrid, RRF) with an optional reranker: `cargo run --release --features embed --bin search -- "your question"`. A normal search takes 15 ms on the development machine, 550 ms with the reranker on.
- The reranker's scoring head, written in Rust, matches the reference Python library to within 0.0000064 on 33 real question/passage pairs, on both the development machine and the build sandbox.
- The eval harness scores a search setup against a question set and compares it with the previous run. It flags a real drop in quality but does not fire on random noise; its statistics code matches scipy's results. A second, fixed check records the exact top-10 results for 20 queries and flags any change: a rerun on the development machine reproduced them exactly, and it caught both of 2 planted changes. It needs no model and runs in seconds.
- First numbers, on a practice set of 47 questions written from random passages (recorded in [`../trevor-retrieval-baseline.md`](../trevor-retrieval-baseline.md)):

| Setup | Right passage in the top 10 |
|---|---|
| Keyword only | 88% |
| Vector only | 70% |
| Hybrid | 89% |
| Hybrid + reranker | 87% |

- Vector search alone is the weakest. Hybrid fixed 9 of the questions vector search missed and lost none.
- Keyword against hybrid is a tie as far as this set can tell.
- The reranker did not help: it rescued 1 question and lost 3, and it is 36 times slower. Decision: it stays off by default.
- The practice set favors keyword search, because questions written while reading a passage reuse its names: 27 of the 47 hit the right passage first by keyword alone. That is why the real question set matters.

## Gold-set automation (2026-09-24 to 2026-09-25)

The original plan had a local LLM write about 225 candidate questions, each tied to the passage it came from, reviewed by hand down to 150 (2 to 3 hours of review), with Gemma 4 26B-A4B (about 16 GB). Decision: automate the review instead. Each job of a human reviewer has an automatic replacement:

- Rejecting bad questions. Filters drop a question if its supporting quote is not really in the passage, if it says things like "the speaker" or "this scene" and so cannot stand on its own, if it copies the passage's wording, if keyword search alone already finds its passage first, or if it duplicates another question. A second model then gives yes/no verdicts on whether each question makes sense on its own.
- Confirming the passage answers the question. The judge checks it; for two-passage questions it also confirms that neither passage answers it alone.
- Finding other passages that also answer it. For every question the judge reads the top search results and adds any that also answer it.
- The judge is from a different model family from the writer, so nothing grades its own work: Gemma 4 12B writes, Qwen3.5 9B judges.
- Judge calibration before it reviews anything: 60 questions paired with passages from unrelated stories, where any "yes" is a known error, and 20 questions of known quality. Its error rate is recorded alongside the question set. An optional 10-minute spot check of 20 random questions exists; nothing waits on it.
- Model choice: the development machine has 24 GB of unified RAM, with 7.6 GB of swap already in use and 58 GB of free disk at the time, so the 14.4 GB Gemma 4 26B-A4B would have pushed it into heavy swapping. The two smaller models (7 GB and 5.7 GB) load one at a time, in `trevor/models/llm/`, downloaded and verified. llama.cpp is installed through Homebrew, which also upgraded `openssl@3` as a dependency.
- Two spec mistakes caught while building: the GBNF output format in the spec did not load in llama.cpp (fixed), and the check for questions that copy the passage's wording could never trigger because of how its score was calculated (replaced with one that does).
- The full pipeline ran end to end in the build sandbox with a tiny stand-in model, and its output passes the eval's format check. That proves only that the parts connect.
- `scripts/goldset-run.sh` runs the pipeline. It probes each model and prints the result before the full run, keeps one model loaded at a time, and shuts the model server down when it finishes, even after an error. It was estimated at about an hour, untimed. Its first step needs the backend for story and group names, which help the model write questions that stand on their own. The design and status are in [`../trevor-goldset-automation.md`](../trevor-goldset-automation.md).
- Build and run: `cargo build --release --features embed --bins`, then `./scripts/goldset-run.sh`. The run reports the funnel, the judge calibration and the closed-book numbers in `eval/goldset_v1.meta.json`.

On 2026-09-25 the remote link to the development machine was lost (the machine was re-enrolled), so the gold-set run moved to a local development session. The code went across as `trevor-crate.tar.gz`, checked by sha256; extracting it over `trevor/` only adds and updates files and does not touch the models, `artifacts/` or the eval runs. [`HANDOFF.md`](HANDOFF.md) records the state at that point.

## Project documents of the period

- [`trevor-research.md`](trevor-research.md) (with the RRF correction banner)
- [`../trevor-corpus-spec.md`](../trevor-corpus-spec.md) (corrected as listed above)
- [`../trevor-eval-spec.md`](../trevor-eval-spec.md)
- [`trevor-goldset-generation.md`](trevor-goldset-generation.md) (prompt and GBNF)
- [`trevor-p0-codex-brief.md`](trevor-p0-codex-brief.md) (superseded)
- [`trevor-ingest-codex-brief.md`](trevor-ingest-codex-brief.md) (superseded by the implementation)
- [`../trevor-implementation-notes.md`](../trevor-implementation-notes.md) (sections 1 to 7 corrections, section 8 real-corpus measurements)
- [`../trevor-p0-status.md`](../trevor-p0-status.md), [`../trevor-retrieval-baseline.md`](../trevor-retrieval-baseline.md), [`../trevor-goldset-automation.md`](../trevor-goldset-automation.md)
- Code bundles: `trevor-scaffold.tar.gz` (chunker and tokenizer, 20 tests; stale, without the accounting fix, ingest or embed), `trevor-p0.tar.gz` (Phase 0) and `trevor-crate.tar.gz` (with the gold-set pipeline).

## Open questions at the time (2026-09-25)

- The 25 missing stories of act24side and act13mini wait on the assets unpacker.
- The backend comment in `services/story.rs` still said 1,797 scripted stories.
- VNNI on the VPS, and the query embedder for an x86 server (full precision, per the INT8 finding).
- `-np` scaling of llama-server on the development machine.
- The chunking knob sweep (scene cap {3, none} x tail merge) is deferred to the eval.
- The gold-set pipeline had not yet run on real models; its runtime was unmeasured.
