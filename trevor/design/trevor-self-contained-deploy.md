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
| 2 | CI releases of the binaries (`trevor` bins and the asset unpacker and downloader); the server never builds Rust. A stale hand-built unpacker deleted a game table on 2026-10-18 | 1 to 2 days |
| 3 | **Corpus snapshot**: one tarball of the derived artifacts (chunks, vectors, indexes, summaries, dossiers, topics, entities, chronology, appearances, bank) with a manifest and checksums; a publisher and an installer, one command each. A fresh deploy starts from the snapshot and only ever runs incremental updates: building the tables from nothing on a CPU is weeks | 1 to 2 days |
| 4 | **Bootstrap**: clone, one command: release binaries, pinned models, migrations, the snapshot, the units and timers, the first nightly update. Documented in `deploy/README.md` as the only path | 1 to 2 days |
| 5 | **The paced stage runner** (`trevor-stages`): section 4's priority order, duty cycle, lock and pressure pauses, gated publish | 3 to 5 days |
| 6 | The nightly re-answer job (yesterday's live questions on the 12B into the bank; the bank's keep rules of contract v1.1 section 3a apply) | 1 day |
| 7 | The missing-input sweep: every reader of a game table or derived file keeps its last good output and logs it; a test per reader with the input removed | 1 to 2 days |
| 8 | A chaos run of the nightly update: a skipped table, an unreachable CDN, a failed gate, a full disk; each must leave the last publish serving and post one alert | 1 day |
| 9 | Retention: update logs, `eval/runs`, the spool, `trevor_asks`, the asset download; disk alerts | half a day |
| 10 | The `cf`-less verification of section 3, and the return runbook: what to read after two years, in one page | half a day |

Two to three focused weeks. Steps 3 and 5 are the real work; the rest is wiring that the last 24 hours of operation
showed to be necessary.

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
