# Trevor self-contained deployment: one box, no model vendor, two years unattended (2026-10-19)

A design for running Trevor with no external model service and no second machine: one server does the serving, the
nightly data update and, paced in the background, the model stages. The trade is time, not money: answers in seconds,
new content fully absorbed over days instead of hours. This doc records the decision, what the box must do, what
exists, what has to be built, and the limits that no design removes. It does not change the contracts
(`trevor-contract-v1.md`, `-v1.1.md`); it chooses a deployment shape within them.

## 0. Goals

1. A new instance comes up from a clone and one bootstrap command, with no hand steps and no accounts beyond the server.
2. No model runs anywhere but on the deployer's own server: no Cloudflare Workers AI, no API keys, no home machine.
3. The instance serves and keeps ingesting game updates for two years with no attention, and every failure leaves the
   last good state serving and posts one alert.
4. Scale is the architecture's, bounded only by the boxes the deployer runs: the serving design of contract v1.1
   (cache first, admission control, fair claiming across workers) is kept as is.

## 1. The decision

Every model Trevor needs runs on a CPU. The fp32 embedder and the Ettin reranker already do; E2B serves the `cpu`
rung today; the 12B runs on a CPU at a few tokens a second. The pipeline's model stages are incremental and resumable
(`scripts/incr.py`, the embed checkpoints), so a stage that takes three days is acceptable as long as nothing waits
on it. Therefore:

- **Serving uses two rungs: cache and `cpu`, with passages as the floor.** The `cf` rung stays in the code and is
  never configured. Contract v1.1 section 1 measured the `cpu` rung at roughly 1,000 to 1,900 answers a day on one
  VPS core with E2B at the `mid` profile; with a good bank, novel live questions are a small fraction of asks.
- **The nightly update stays deterministic and fast**, and it is the part that must never lag: a new episode is
  searchable the morning after it ships (passages, BM25, dense, the appearance index, dates, identities).
- **The model stages run continuously in the background, paced**, on the 12B at CPU speed, one unit at a time,
  at a duty cycle that leaves the cores to serving. New content gets its summaries, dossiers, topics, deaths, canon
  and bank answers a few days after each game update instead of the next morning.
- **Quality has two tiers in time, not two vendors:** E2B answers a novel question now; the 12B's answer to the same
  question lands in the bank later (overnight for repeat questions, days for a new episode's), and the next asker
  gets it.

What this costs: a bigger server than the serving alone needs (section 5), and derived tables that lag the game by
days. What it buys: nothing outside the repository and the server can change price, retire a model, or need a key.

## 2. Topology

One server, four services under systemd (`deploy/`):

| service | does | today |
|---|---|---|
| backend (myrtle) | owns asks: cache lookup by normalized question, the horizon from the profile, admission control, rung choice, streaming, Discord alerts, the stale sweep | exists |
| `trevor-worker` | `ask --worker` on E2B `mid`, fp32 vectors, the model lock, heartbeat, power guard, spool | exists |
| `trevor-update` (nightly timer) | fetch assets, extract, corpus, embed changed chunks, indexes, deterministic derived tables, gates, publish, rollback, bank regen for changed chunks, warm-up | exists as `scripts/trevor-vps-update.sh daily`; the model-stage parts move to the runner below |
| `trevor-stages` (continuous, paced) | the model stages of `scripts/update.sh` as a resumable background service with a priority order and a duty cycle; takes the model lock per unit so the worker and the runner never load two models at once | to build (section 6) |

A second box is a second `trevor-worker` (and optionally a second runner) pointed at the same backend with its own
worker key; contract v1.1's fair claiming spreads the jobs. That is the whole horizontal story: more of the same box,
not a model API that grows on demand.

## 3. Serving

Unchanged from contract v1.1 except that `cf` is never configured:

1. cache (the bank plus every answered ask, carried over across publishes when the cited chunks' shas are unchanged);
2. `cpu` (E2B `mid` on the worker; deferred jobs and the quick look when the slots are busy);
3. passages, when no worker heartbeats.

Two things to verify once, with a test that stays in the suite: rung selection falls straight from cache to `cpu`
when `TREVOR_CF_*` are unset (no job may wait on a rung that does not exist), and no UI copy promises the `cf` rung.

The bank is the lever that replaces the paid rung. It grows three ways: the pre-generated questions per story (as
today), the nightly re-answer of the previous day's live questions on the 12B (new, section 6), and the carry-over.

## 4. The update, split in two

**Nightly, deterministic, minutes to an hour.** `trevor-vps-update.sh daily` as it is, minus its model-stage calls:
fetch, extract, `build-corpus --refresh`, `embed-corpus` for changed chunks (paced; `trevor-reembed.md`), indexes,
prefixes, archives, chronology, dates, identities (v1 and v2), deaths, attributes, typed sources, reading guide,
source dates, gate order, label renames, appearances, bank seed and merge, gates, publish, warm-up. Every reader of a
game table survives a missing or stale table the way `reading_guide.py dates` and `gametext.py` do since 2026-10-18
(keep the last good output, say so in the log), because a skipped table is the most common upstream failure
(`assets/output/*/gamedata/.skipped.json`, backend `core/gamedata/stale.rs`).

**Continuously, paced: the model stages.** In priority order, each unit resumable and recorded by `incr.py`:

1. story summaries for new or changed stories (`p2-summaries.py story`, then `group`);
2. the nightly re-answer: the previous day's live questions through the 12B at the `full` profile into the bank;
3. bank answers for new pre-generated questions (`answer_bank.py answer`);
4. dossiers, real names, primers, entity and death cues and extracts;
5. topics (mine, classify, plan, gen, judge, retrait), deep topics;
6. canon (`canon_refs.py`), the judge passes on Qwen.

Each stage already redoes only changed units. The runner adds: one unit per model load where the scripts allow it,
a duty cycle (run, then idle, like `embed-corpus --duty`), a pause when the worker is busy (the model lock) or the
host is under memory pressure, and a publish of the derived tables through the same gates as the nightly update when
a stage completes. A game update's model work took about 7 GPU-hours on an Apple M-series machine; at CPU speed the
estimate is 3 to 6 days, inside the roughly two-week EN update cadence. The estimate is replaced by the measurement
of section 5 before the runner is built.

## 5. Hardware, and the two measurements that come first

- Resident memory: the 12B at q4 about 8 GB, E2B about 3 GB, the fp32 embedder, the reranker and the indexes a few
  GB more, beside the backend (its pm2 limit is 3 GiB) and Postgres. Plan for 32 GB and 8 or more cores; measure.
- Disk: the models (about 10 GB with the 12B and Qwen), the corpora and vectors, the update logs and `eval/runs`,
  the spool, the asset download. Retention for logs, runs and asks is part of the build (section 6).

Before anything else is built, two numbers on the target cores, with serving running beside them:

1. E2B `mid` answers per hour (contract v1.1 measured 1,000 to 1,900 a day on one core; confirm on the target).
2. 12B tokens per second for generation and for prompt processing (the stages' prompts are 5,000 to 10,000 tokens),
   and the hours one game update's stages take at the chosen duty cycle.

**Measured 2026-10-19 on the development machine as the proxy** (Apple M5 Pro, `-ngl 0`, 8 threads; llama.cpp build
11146). The caveat first: with `-ngl 0` the layers run on the CPU, but Apple's Accelerate BLAS and the unified memory
bandwidth make these numbers an upper bound for a 3 vCPU x86 slice, by roughly 2 to 4 times on generation; the VPS
numbers must be taken on the VPS.

| measurement | result |
|---|---|
| E2B `mid`, the 35 probe questions, one client | 1,249 s wall; model time median 38 s, mean 39 s (26 to 54 s); prompts about 3,500 tokens; about 93 answers an hour at full duty, roughly 2,200 a day per 8-thread box |
| 12B Q4_0 `llama-bench` | prompt processing 36.4 tokens/s (pp4096), generation 17.4 tokens/s (tg128) |
| E2B Q4_K_M `llama-bench` | prompt processing 92.7 tokens/s, generation 48.0 tokens/s |

Consequences: a 7,000-token summary prompt plus 450 output tokens is about 3.7 minutes on the 12B here, so one game
update's roughly 400 model calls is 1 to 3 days on this CPU and longer on the VPS; and the 12B does not fit the
current VPS's 11 GB beside serving at all, so on that box the stages run on E2B (2.6 times faster, lower quality,
unmeasured) or the box grows. Contract v1.1 section 1's "1,000 to 1,900 a day on one VPS core" for E2B `mid` is not
supported by this measurement (93 an hour on 8 fast threads); it needs a correction note there.

Everything in section 4 is an estimate until these exist; if the 12B is too slow for the box, the stages fall back
to E2B for all but the bank re-answer, at a measured quality cost.

## 6. What exists and what has to be built

Exists: the serving contract and its backend, the worker, the power guard and alerts, the admission control, the
cache carry-over, the gates and publish with rollback, the VPS runbook and units, the incremental scripts, the
pinned model fetch (`scripts/fetch-model.sh`, sha256-checked), the CI that builds the game-data artifact.

To build, in order:

| step | what | size |
|---|---|---|
| 1 | The two measurements of section 5 on the target box | hours |
| 2 | Done 2026-10-19: the Release Bundle workflow (`.github/workflows/release.yml`) builds the 12 `trevor` bins with `--features embed` beside the asset unpacker and downloader it already shipped, and `scripts/fetch-release.sh --components trevor <sha>` installs them into `$TREVOR_DIR/target/release` (sha256-checked, renamed into place, `ask --help` before and after). ONNX Runtime 1.28 links statically: ort-sys's download path emits `rustc-link-lib=static=onnxruntime`, the macOS download holds only `libonnxruntime.a`, and `otool -L` on the Mac's `ask` lists no ONNX library while `nm` finds `OrtGetApiBase`; the CI job fails if `ldd` shows a dynamic one. Unverified until the workflow first runs on main. A stale hand-built unpacker deleted a game table on 2026-10-18 | 1 to 2 days |
| 3 | Done 2026-10-19 (`scripts/snapshot.py`, see "Step 3 built" below; the fp32 corpora refreshed the same day, see "For step 4"). **Corpus snapshot**: one tarball of the derived artifacts (chunks, vectors, indexes, summaries, dossiers, topics, entities, chronology, appearances, bank) with a manifest and checksums; a publisher and an installer, one command each. A fresh deploy starts from the snapshot and only ever runs incremental updates: building the tables from nothing on a CPU is weeks | 1 to 2 days |
| 4 | **Bootstrap**: clone, one command: release binaries, pinned models, migrations, the snapshot, the units and timers, the first nightly update. Documented in `deploy/README.md` as the only path. llama.cpp (`llama-server`) is still installed by hand (`deploy/README.md` step 2); pinning a release and its sha256 belongs here | 1 to 2 days |
| 5 | **The paced stage runner** (`trevor-stages`): section 4's priority order, duty cycle, lock and pressure pauses, gated publish | 3 to 5 days |
| 6 | The nightly re-answer job (yesterday's live questions on the 12B into the bank; the bank's keep rules of contract v1.1 section 3a apply) | 1 day |
| 7 | The missing-input sweep: every reader of a game table or derived file keeps its last good output and logs it; a test per reader with the input removed | done 2026-10-19 for the Python stages. `common.read_derived` raises on a missing derived input instead of reading it as empty (the root of most silent overwrites), in 15 scripts: label_renames, entities build (pairs, pairs.v2), deaths build, chrono derive/flashbacks/build, answer_bank seed and merge, topics, primers, topic_deep, reading_guide, sources, attributes, source_dates, gametext, canon_refs (`rd()` no longer turns a missing table into a full run) and incr.py. `common.write_keep_last` writes 20 outputs atomically (6 more files are now written through a rename only) and keeps the old file when the new one is empty or lost more than `INCR_MAX_DROP` percent (20) of its rows; `incr.py` exits 3 instead of dropping every row (simulated on a scratch copy: an empty corpus would have emptied all 1,930 rows of p2/stories.jsonl; now kept, exit 3); `common.excel_table` raises on a missing table and logs a stale one from `.skipped.json` once, and the six table readers keep their last output on a missing table; answer_bank merge aborts without p4x chunks or gate_order.json instead of dropping every ask entry; update.sh's full canon run copies the canon files aside and clears the resume files only after `endings` succeeds, leaving is_endings.json served. With inputs present, 20 no-model stages rewrote 24 files in a scratch tree byte-identical to the code before, except first_appearance.json, whose key order follows the string hash seed (the old code run twice differs the same way; parsed, all 4,615 entries are equal). `scripts/tests/test_missing_inputs.py`: 53 tests, all pass (41 fail on the code before); kill switches `INCR_ALLOW_SHRINK=1`, `KEEP_LAST=0`, `TABLES_REQUIRED=1`, each tested. Rust: done A, B 2026-10-19. A: `build-corpus` (resume and `--refresh`) stops before writing anything when the index plans no scripted story or loses more than `TREVOR_INGEST_MAX_DROP` percent (20) of the previous `spoiler.jsonl`'s stories (`TREVOR_INGEST_ALLOW_SHRINK=1`: as before); the fake-backend test refuses a 5-to-3 shrink with all four files byte-identical, and with the guard off the same refresh removes 2 stories. B: the 24 serving artifacts of `Tools::load`, `load_new_dossiers`, `load_chrono`, `load_names` and the label renames load through `util::load_json_opt`/`load_jsonl_opt`/`load_text_opt`, which log one line per artifact (missing, unreadable, unparsable, or N malformed lines skipped; `TREVOR_LOAD_LOG=0` silences) and list it in `util::load_failures()`; on this tree the one line is `design_infer.jsonl: missing`; `--route-only` over routes and the four eval sets is byte-identical (79, 35, 120, 40, 109 lines). Left: the failures in the heartbeat or `/status` (needs a backend field), and a shrink gate in gates.py |
| 8 | A chaos run of the nightly update: a skipped table, an unreachable CDN, a failed gate, a full disk; each must leave the last publish serving and post one alert | 1 day |
| 9 | Retention: update logs, `eval/runs`, the spool, `trevor_asks`, the asset download; disk alerts | half a day |
| 10 | The `cf`-less verification of section 3: verified by reading 2026-10-19. The `cf` rung exists only while a worker heartbeat reports `cf:true`, which needs the Cloudflare keys and a passed canary (`trevor/src/bin/ask/serve/worker.rs`, `openai.rs cloudflare_from_env`); with them unset admission goes `cpu`, deferred `cpu`, passages (`backend/.../trevor/rung.rs`), `settle_rung` steps a stray `Cf` down to `Cpu`, Neurons accrue only on cf results so the budget alert never fires, and no frontend string names the rung. Done 2026-10-19: `heartbeat_reports_no_cf_rung_without_cf_models` (`cf:false` beside a fresh passing canary; the same canary with the keys gives `cf:true`), the three cf keys commented out with "leave unset" in `deploy/trevor.env.example`, and the dev preview's canned answers, jobs and status default to `cpu` with `rungs: {cf:false, cpu:true}`. Still to do: the return runbook | half a day |

Two to three focused weeks. Steps 3 and 5 are the real work; the rest is wiring that the last 24 hours of operation
showed to be necessary.

### Step 3 built (2026-10-19): `scripts/snapshot.py`

`snapshot.py pack` writes one tarball whose first member, `manifest.json`, records the schema, the creation time, the
git commit, every file's size and sha256, each corpus's `chunks.jsonl` sha (sha16 and sha256) with whether its BM25
index and vectors were built from that file, and the embedding model files the vectors were made with
(`vectors.meta.json` `modelFile`, `modelSha`, `tokenizerSha`). `install` checks every member against the manifest
before writing anything, extracts into `<dir>.tmp` and renames it into place; an existing tree is refused unless
`--force`, which keeps the old one as `<dir>.old-<stamp>`. `check` is the verification alone.

What it holds, measured on the development machine's tree: 246 files, 1,176,975,963 bytes, from 30 top-level entries.

| part | bytes |
|---|---|
| vectors, INT8 export (P0, p3a, p3b, p4, p4x, p4-art) | 370,968,576 |
| vectors, fp32 export (p3b-fp32, p4x-fp32, p4-art-fp32) | 197,932,032 |
| `chunks.jsonl` of those 9 corpora | 391,836,237 |
| BM25 indexes of those 9 corpora | 116,507,189 |
| `vectors.meta.json` (the reuse keys) | 11,873,574 |
| stage outputs and resume files (sources 14.7 MB, bank 11.8, art 11.3, entities 11.2, gametext 10.2, chrono 9.4, topics 4.7, p2 3.5, canon 2.5, dossiers 1.8, appearances 1.1, the rest under 1) | about 88,000,000 |

Both vector kinds are f32 rows of 3,072 bytes; the name is the ONNX export that made them. The tar.zst is 215,714,764
bytes: zstd level 3, one thread, a 2^29 window. The window is what matters, because the corpora repeat each other's
chunks and reused vectors byte for byte: the default window gives 766 MB, 2^29 gives 216 MB (588 MB peak memory to
write, 542 MB to read) and 2^30 gives 206 MB for twice the memory. Vectors are 148 MB of the 216. On the M5 Pro, pack
takes 2.9 s, check 1.1 s and install 2.6 s. Without zstd on PATH it writes tar.gz: 777,605,858 bytes in 46 s.
`--include-models` adds the embedder and the reranker (821 MB raw; 740 MB as tar.zst in all); the GGUFs are never
packed, `scripts/fetch-model.sh` fetches them pinned.

Left out: raw game data, `eval/runs`, eval outputs (`answers`, `trivia`, `goldgen-arch`, `goldgen-v2`), update logs,
the spool, experiments (`p1a-split`, `p3b-lore`, `p4-lore`, `embedft`, `primers.v1`), the query-embedding cache
(`router`), instance state (`publish`, `canary`), scratch files, and what the nightly update rebuilds in seconds
(`p1a`, `panels`, `sources/all_units.jsonl`, `chunks.prev.jsonl`, `changes.json`): `build-prefixes` rewrote a
byte-identical `p1a/chunks.jsonl` in 0.5 s, and `panels.py` built 1,931 panels from the installed tree in 16 s. A
top-level entry the script does not know is reported and left out.

Verified: `pack` into a scratch directory, `check`, `install --into` a scratch directory, then `cmp` of all 246
manifest files against the source (0 differ) and `diff -rq` of every installed entry (20 lines, all files skipped on
purpose: 18 empty tantivy lock files, `art/server.log`, `sources/all_units.jsonl`). A BM25 search on the installed
p3b and p4x returns the same hits as on the source. A truncated archive and an archive with one altered member both
fail `check` with exit 1.

For step 4: the development machine serves INT8 vectors, which no other host can query (`trevor-reembed.md` section
6), so a snapshot for another host must carry the `-fp32` corpora, and they must be current. Resolved 2026-10-19:
the refresh of `trevor-reembed.md` section 4 (copy each INT8 corpus's chunk files and BM25 index into its `-fp32`
directory, then `embed-corpus --onnx-file model.onnx` with `--reuse-from` chained p3b, p4x, p4-art) embedded 1,063
new texts for p3b-fp32 and 238 for p4x-fp32 in 4.7 minutes and reused the rest; the three fp32 corpora now hold the
live chunk shas and row counts (16,752; 25,514; 23,765) under the same model and tokenizer hashes, so they stay
interchangeable with the earlier fp32 set. The snapshot packed from that tree is 219,235,313 bytes and checks clean.
Rule: run that refresh before every `snapshot.py pack` meant for another host; `trevor-vps-update.sh` refuses INT8
vectors under `EMBED_ONNX=model.onnx`, and its `EMBED_FULL=1` re-embeds every text without reuse.

## 7. Fail safe, never fail silent

The two-year limit is the same for every deployment shape and has nothing to do with where the model runs:

- **Upstream format changes** (a FlatBuffer layout the unpacker cannot verify, a changed CDN or encryption) stop the
  ingest. Twice in October 2026 the EN data changed shape. The posture: the nightly update keeps the last good
  tables, the gates refuse a worse publish, one Discord message says which step and why, and Trevor keeps answering
  about everything it already knows. Expect this to happen within months, not years, and to need about an hour of a
  person's time each time.
- **Model and runtime drift** (a llama.cpp or ONNX Runtime release, an OS update) is held off by pinning: release
  binaries, pinned model files with checksums, the runtime versions in the bootstrap. Nothing on the box updates
  itself.
- **Resource drift** (disk, database growth, certificates) is covered by retention and alerts.
- **Automatic self-repair by a model with deploy rights is excluded.** Its failure mode is a confident wrong fix
  applied to production with no one watching. The system heals the failure classes it knows (restart, rollback,
  keep the last good table, skip a unit) and alerts on the rest.

The operator's duty is one hour a quarter: read the alerts.

## 8. Open decisions

1. The target box: the current VPS with more memory, or a second box for the runner. Section 5 decides.
2. The 12B or E2B for the stages, per stage, from the measurement.
3. Snapshot hosting: a release asset of the repository, or a file on the operator's own storage. It carries game
   text, as the site does.
4. Whether the live questions re-answered nightly are limited to those asked more than once, to keep the 12B's hours
   for questions that recur.
5. An in-browser quick rung (a small model over WebGPU, run by the user's own device) would scale with users at no
   server cost but asks each user for a gigabyte-plus download; an experiment, not part of this plan.

## 9. Kill switches

The deployment shape introduces no new answer behaviour. Its services are turned off by their units; `TREVOR_MODE`
(contract v1.1) remains the master switch; the runner's duty cycle at 0 stops the model stages without stopping
serving; a publish is rolled back with `trevor-vps-update.sh rollback`.
