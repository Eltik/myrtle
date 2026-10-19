# Trevor on the VPS: runbook (2026-10-16)

How Trevor's content is built, gated, published and rolled back on the myrtle.moe VPS, and what happens when
something breaks. The install steps are `deploy/README.md`; the interface is `trevor-contract-v1.md` (binding); the
build order is `trevor-integration-plan-v3.md`. Nothing here has run on the VPS yet: every number below was measured
on the development machine (Apple M5 Pro, 24 GB) unless it says otherwise. The variables the backend reads, with their
defaults, are documented in [backend/README.md, Trevor](../../backend/README.md#trevor); this runbook names them and
links there. The variables the worker and the scripts read are documented here and in `deploy/trevor.env.example`.

## 1. What runs where

| unit | what | when | limits |
|---|---|---|---|
| `trevor-llama.service` | `llama-server`, Gemma 4 E2B Q4_K_M, `-t 1 -c 4096 -np 1 --no-repack --poll 0 --cache-ram 256`, port 8081 | always | `AllowedCPUs=2 CPUWeight=10 Nice=19 MemoryMax=4G`, no CPUQuota (it stalls llama.cpp's threads) |
| `trevor-worker.service` | `ask --worker http://127.0.0.1:3060 --retrieval $TREVOR_RETRIEVAL --server http://127.0.0.1:8081 --no-spawn --ignore-lock` | always | `CPUQuota=100% MemoryMax=1536M Nice=10` |
| `trevor-update.timer` / `.service` | `trevor-vps-update.sh daily`: the deterministic update | 19:30 UTC | `Nice=19`, IO idle, `CPUQuota=100% MemoryMax=2G` |
| `trevor-summaries.timer` / `.service` | `trevor-vps-update.sh summaries`: summaries for new stories, then rebuild and publish | 23:00 UTC | `Nice=19`, IO idle, `AllowedCPUs=0-1 CPUWeight=10 MemoryMax=4G` |
| `trevor-worker-restart.path` | restarts the worker when the live release changes | on change | |

The worker ignores the model lock (`--ignore-lock`): it only talks to the serving llama-server, and a worker that
waited for an overnight summary job would be offline for hours. The lock (`/run/trevor/llm.lock`, ask's `<pid> <label>`
convention) serializes the batch model jobs: the gate canary and the overnight summary server.

## 2. Layout and the release switch

```
/var/www/myrtle.moe/trevor/            code: scripts prompts data eval deploy target/release models
/var/www/myrtle.moe/trevor/artifacts -> /var/lib/trevor/current            (made once by `trevor-vps-update.sh init`)
/var/lib/trevor/current -> /var/lib/trevor/releases/<stamp>-<version>/artifacts   (flipped by the update job)
/var/lib/trevor/stage/trevor/          the working copy every build runs in (code copied, artifacts a real copy)
/var/lib/trevor/state/                 active.json, versions/<v>/activate.json, history.jsonl, releases.jsonl,
                                       stage-base, worker-restart, gates/, update.lock
/var/lib/trevor/logs/<stamp>-<cmd>/    run.log, changes.txt, summaries-*.json, publish-dry/ (30 days)
```

A build never writes the live release. It runs in the stage; a release is the stage copied with `rsync --link-dest`
against the previous release (unchanged files are hard links, so a release costs only what changed). The update job
flips `/var/lib/trevor/current` with an atomic rename and writes `state/worker-restart`; the path unit restarts the
worker on the new files. The update job needs no root and never writes the code directory. `KEEP_RELEASES` (3)
releases are kept, the live one always.

A run that fails leaves the stage as it is and the next run continues from it, so finished summaries are not lost.
The stage is copied fresh from the live release when it does not exist, when the live release changed under it (a
rollback), or with `RESET_STAGE=1`.

## 3. Embeddings: BM25-only or fp32

The build machine's corpus vectors are INT8 (`model_int8.onnx`), and INT8 drifts to cosine 0.94 on x86. Two ways are safe:

- **BM25-only** (`TREVOR_RETRIEVAL=bm25`, `EMBED=0`). Nothing is embedded on the VPS; the worker never loads the
  dense index. Gold v2 BM25 recall@10 on P3b, the serving corpus: 0.636 (measured on 2026-10-16 by the gate, 131 items).
- **Hybrid** with fp32 vectors (`EMBED=1`, `EMBED_ONNX=model.onnx`, then `TREVOR_RETRIEVAL=hybrid`). The fp32 export
  gives the same vectors on every CPU. The daily job embeds only new or changed chunks (`embed-corpus` reuse, keyed on
  the text sha) and only in `EMBED_DIRS` (default `artifacts/p3b artifacts/p4x`, the corpora ask serves). It refuses
  to run when a directory it would embed into, or reuse from, holds vectors from another export
  (`vectors.meta.json` `modelFile`), so the runtimes never mix: the one-time full re-embed is
  `EMBED_FULL=1 FORCE=1 trevor-vps-update.sh daily` (deploy/README.md step 9, about 1 to 2 h at nice 19 by the plan's
  estimate, unmeasured on the VPS), or fp32 vectors copied from the build machine.
- `embed-corpus` since 2026-10-16 takes `--model-file` (alias of `--onnx-file`) and pauses itself under memory, CPU
  pressure or load (its guard is on by default; `--no-guard` turns it off). The script detects both from `--help`
  and warns when the binary predates them. `trevor-reembed.md` is the reference for the guard.

## 4. The daily update (`trevor-vps-update.sh daily`, 19:30 UTC)

1. **Detect.** Read the asset watcher's EN res_version from `assets/ArkAssets/en/.version`. The watcher
   (`assets/run.mjs`) writes it only after a successful download and extract, and `.last_extract` beside it. Compare
   it with `builtFrom` of the active version (`state/active.json`, written by publish.py after every activate). Equal
   means exit at once (most days). A `.version` younger than `SETTLE_MIN` (15 min) waits for the next run, so the
   backend has hot-reloaded the new data. `FORCE=1` runs anyway.
2. **Corpus.** `build-corpus --refresh --base http://127.0.0.1:3060` in the stage (refetch every story, replace the
   changed ones; `artifacts/changes.json` lists added, changed, removed).
3. **Deterministic rebuild**, `update.sh` steps 2 and 4 without a model: `build-prefixes` (P1a; not in update.sh,
   but p2-summaries.py reads each story's header from it, so a new story cannot be summarized without it),
   P0 and P3a indexes, `chrono.py derive/flashbacks/build/history`, `reading_guide.py dates`, `entities.py build`,
   `deaths.py build`, `attributes.py`, `sources.py`, the lores (`dossiers.py sets`, `build-profiles`, P3b, P4, P4x
   with `gametext.py`), `reading_guide.py build`, `source_dates.py`, `appearances.py`, `answer_bank.py seed` and
   `merge`. Every model output is read as it is (frozen): a new story gets no dossier, topic, chronology extraction
   or identity link until a manual rebuild on the build machine. Its bank questions and answers are the exception: just
   before the bank merge, `bank_regen.py detect --prev <live release>` lists the new and changed chunks for the
   overnight run, and the merge drops every generated answer whose evidence changed or vanished (section 5a).
4. **Panels** (`panels.py --built-from <res_version>`), which also writes `artifacts/chrono/gate_order.json` (section 6).
5. **Gates** (`gates.py`, section 7). A failed gate stops here.
6. **Publish** (`publish.py --no-activate`): the panels go up as an inactive version.
7. **Promote**: copy the stage into a new release, flip `current`, restart the worker, wait `WORKER_SETTLE` (30 s).
8. **Activate** (`publish.py --activate <version>`). If that call fails, `current` flips back and the worker restarts
   on the old release; the backend never activated the new version.
9. **Notify** (optional, `DISCORD_ALERT_WEBHOOK`): a "published <version>" notice.
10. **Warm-up** (contract v1.1 section 3c): `bank_regen.py warm --version <v> --limit $TREVOR_WARM_TOP` gets the
    most-asked questions that were not carried over and sends them back as deferred internal jobs. A failure here
    leaves the version live and sends an alert. `TREVOR_WARM_TOP=0` skips this step.

Any failed step sends ONE `update_failed` alert per run (contract v1.1 section 2.2). It is a Discord embed in the
backend's `core::alerts` style that gives the step, the exit code and the last `ALERT_LOG_LINES` (20) lines of the
run log. The script posts it, and step 9's notice, to `DISCORD_ALERT_WEBHOOK`. The backend reads only that variable
(its `core::alerts`, see [backend/README.md, Trevor](../../backend/README.md#trevor)); `TREVOR_DISCORD_WEBHOOK` (the
name before 2026-10-17) is read only by `trevor-vps-update.sh`, as its fallback when `DISCORD_ALERT_WEBHOOK` is unset
or empty. A marker file keyed on the run's pid keeps a subshell's error trap and the parent's from posting twice.

Measured on the development machine (Apple M5 Pro, 24 GB) against a scratch copy of the artifacts and a stand-in
backend, `SKIP_FETCH=1`, vectors reused (section 14 has the numbers).

`PLAN=1 trevor-vps-update.sh daily` prints every step it would run; detection and the embedding guard are real, nothing is
written.

## 5. Summaries (`trevor-vps-update.sh summaries`, 23:00 UTC)

Decision (2026-10-16): summaries are the only model stage of the update. `summaries_cf.py` writes p2-summaries.py's rows
(same prompts, inputs, max tokens and incremental keys) for every story and group that has NO summary row
(`--scope stale` would also redo changed ones).

1. **Cloudflare** inside `TREVOR_CF_WINDOW` (23:00 to 23:55 UTC), the last hour before the 00:00 UTC Neuron reset,
   so it spends what the day's readers left. Each response's usage is priced with the contract's rule
   (`ceil(in x 9091 / 1e6 + out x 27273 / 1e6)`): a 12,000-token story with a 300-token summary is 118 Neurons. A 429
   or a daily-allocation error stops Cloudflare for the night; the free plan hard-stops and never bills. A retired
   model id fails over to the next in `TREVOR_CF_MODELS`. The run's own spend is capped at
   `TREVOR_CF_SUMMARY_MAX_NEURONS` (9000).
2. **Local E2B** for what is left: a second llama-server on :8082 with a 16,384-token context (the serving server's
   4,096 cannot hold a story; the longest P2 prompt is 12,863 tokens), 2 threads on vCPUs 0 and 1, started only with
   `MemAvailable` above 2,500 MB and under the model lock, stopped by PID when done or at 07:00 UTC. Both servers map
   the same GGUF, so the second adds its KV cache and buffers, not another 3 GB of weights; that is the expectation,
   not a measurement.
3. When any summary or bank answer was written: the rebuild (without build-corpus), panels, gates, publish,
   promote and activate, as in section 4. A story still without a summary ships its panel without one
   (`recap.thisStory` null) and gets it the next night.

## 5a. Pre-generated answers that follow updates (`bank_regen.py`, contract v1.1 section 3a)

Requirement (2026-10-17): when there is a new update, generate new answers for it. Section 5 summarizes first, then the same run
does this with the same Cloudflare client and window, and then on the same E2B server.

- **What counts as changed.** `detect` compares the sha16 of every chunk's text in the stage's P3b (over P4x) with
  the live release's.
  - Pending units (`artifacts/bank/regen/pending.json`) are every question-eligible chunk that is new or changed
    (over 400 characters, not a profile or summary unit, not a gold-set story), plus every existing question whose
    own chunk is unchanged but whose answer cited a changed or vanished chunk. The latter are re-answered under
    their old qid.
  - Units carry over across nights until they are done.
  - The first `detect` also writes the evidence ledger (`artifacts/bank/evidence.jsonl`, the shas of each answer's
    question chunk and cited chunks) for the 900 answers made before it, from the live release's corpus.
- **Run.** Units go in slices of 10, at most `REGEN_MAX_UNITS` (150) a night. Re-answers go first, then changed
  chunks that had a question, then new chunks.
  - **Questions** come from `answer_bank.stage_questions(only=...)`, with the same prompt and incremental key.
  - **Answers** come from `ask --batch` with the `mid` profile, or with the eval's equivalent flags while `ask` has
    no `--profile mid`: `--router keywords --k 8 --max-context-tokens 3600 --n-predict 250` and the four
    no-retry switches.
  - **Cloudflare** is used inside `TREVOR_CF_WINDOW`, up to `TREVOR_CF_BANK_MAX_NEURONS` (9000) minus what the
    summaries spent. An answer's Neurons are estimated from its prompt tokens plus the full 250-token output.
  - **Local E2B** takes the rest until `SUMMARY_UNTIL`.
- **Merge** (`answer_bank.py merge`) keeps an old entry only while every chunk in its evidence row has the same
  sha.
  - A new answer is kept only when it does not decline, cites something, and has at least one citation at or before
    its question's own story by gatePos.
  - `REGEN_ANSWERS=0` turns all of this off. With it, bank.jsonl is byte-identical to the merge before
    (sha256 fb157643..., verified 2026-10-17).
- `PLAN=1 trevor-vps-update.sh summaries` and `PLAN=1 bank_regen.py run` print the pending units, their story groups
  and the model calls they cost.

**Measured on the development machine (Apple M5 Pro, 24 GB), 2026-10-17**, on a simulated update: a scratch tree whose "previous" corpus had 20 chunks
re-worded. These were 10 question chunks, 5 chunks cited by other answers, and 5 chunks with neither.
- `detect` takes 0.9 s and 446 MB. It found 20 changed chunks and made 25 units: 10 changed chunks that had a
  question, 10 that had none, and 5 re-answers. It backfilled 900 ledger rows.
- The merge then drops 15 of 875 entries (evidence changed).
- The run on the E2B (Metal, `-np 1`, 8,192-token slot) took 78.7 s for 25 units: 20 question calls and 25 answer
  calls. That is 3.1 s and 1.8 model calls a unit.
  - Answers take a median of 2.2 s, with a mean prompt of 3,628 tokens (max 4,273).
  - Of 25 answers, 1 declined and 2 cited only later stories and were dropped. One was a prologue question answered
    from act33side.
- Result: 860 entries kept byte for byte, 10 dropped (their chunk changed and the question was redone), 5
  re-answered in place, 17 added: 882 ask entries against 875.
- `panels.py` takes 10.0 s, and 84 of 1,862 panels differ from the build without the change. `gates.py
  --no-canary` passes (schema, spoiler 0 violations, recall 0.636132).

**A typical event update** is the newest side story, act49side: 196 chunks, 191 of them question-eligible. Add
about 5 archive chunks for each new operator (median 5 over 417 operators), so about 200 units and 400 model calls.
- On the build machine that is about 10 minutes.
- On Cloudflare a unit costs about 47 Neurons (7 for a question over a 583-token chunk, 40 for a 3,628-token mid
  answer), so 200 units cost about 9,400 Neurons. That is about one whole day's allocation, so leftover quota covers
  only part of it.
- On the VPS's one E2B core, mid answers run at roughly 1,000 to 1,900 a day by the contract's estimate, so 45 to
  86 s each. With a shorter question call, that is about 1 to 2 minutes a unit, or 3.5 to 7 hours for 200 units.
  This is not measured on the VPS. The 150-unit cap spreads one event over about two nights.

## 6. One spoiler scale: `artifacts/chrono/gate_order.json`

Contract section 1: every story has one gatePos, its 1-based rank in the EN release order. The order is the panel
prototype's rule: reading-guide position, then storySort, then the line in `spoiler.jsonl`, which breaks the storySort
ties (main_6 sort 20, act33side sorts 7 and 17) the old horizon let see each other. Unplaced stories and sources dated
by `source_dates.json` sit just before the first guide group whose latest date admits them; undated stories rank last
and are never visible.

The file's schema is `GateOrderFile` in `src/search/horizon/mod.rs`, written identically by `panels.py` and by `ask --gate-order`:

```
{"schema": 1,
 "inputs": {"readingGuide": sha16, "spoiler": sha16, "sourceDates": sha16 | null},
 "storyOrder": [storyId, ...],          gatePos = index + 1 (1,862 stories today)
 "never": [storyId, ...],               no position and no date: hidden at every horizon (0 today)
 "dated": [storyId, ...],               placed by their source date: the 367 operator records
 "sourcePos": {sourceId: int},          non-story sources (operator files, voice lines, modules, outfits, game text):
                                        visible iff sourcePos <= horizonPos (2,202 today)
 "eras": [{"id", "name", "position", "groupPosition"}]}   22 eras; position = gatePos of the group's last story
```

`panels.py --gate-order auto` reads a file whose inputs match and checks its storyOrder, never, dated, sourcePos and
era positions against its own ranking (exit 3 on any difference, no panels built); a missing or stale file is written.
The horizon (`src/search/horizon/`) rebuilds the order in memory, with a warning, when the file is missing or stale.

The **eras** (decision D7, provisional): every main episode, plus an event whenever 6 or more guide groups pass
without a boundary. Today: main_0 to main_9 at gatePos 14, 26, 46, 62, 82, 99, 202, 271, 362, 565; act18d3 488;
act16side 687; main_10 722; act12mini 853; main_11 to main_16 at 888, 1026, 1190, 1374, 1608, 1745; act33side 1329;
act38side 1503. Each panel's `era` and the manifest's `eras` carry `{id, name, position}` with position on the gatePos
scale (the prototype wrote the reading-guide position there; `panels.py --era-guide-position` restores it).

**Spoilers come from the user's profile, through the backend** (decision, 2026-10-16; contract section 1): the backend
takes the maximum gatePos over the stories the reader has read, lowers it to the story being read when the request
names one, buckets it to the era boundary at or before it, and sends the worker `horizonPos`. Nothing on the VPS side
decides a reader's horizon.

## 7. The gates (`gates.py`)

| gate | passes when | today |
|---|---|---|
| schema | the manifest and all 1,862 panels match schema 1; storyId = file name; position = gatePos; storyOrder and eras equal gate_order.json's; panelsSha recomputes | pass, 0 errors |
| spoiler | 0 facts past a panel's horizon: previous, soFar, firstSeen, firstNamed strictly earlier; every suggestion's gatePos and the era's position at or before the story | pass, 0 of 1,862 |
| recall | BM25 recall@10 on gold v2 (P3b) >= the active version's value - 0.02 (no active version: this run is the baseline) | 0.636132, baseline |
| canary | 10 fixed questions (`deploy/canary.jsonl`, 2 with a horizon before their evidence) through `ask --serve --profile mid` (the cpu rung's profile; `GATES_ASK_FLAGS` overrides) against the serving llama-server, under the model lock: every answer non-empty, every citation at or before the question's horizonPos, the served corpusVersion = the manifest's | not run today (no model) |

The report (`artifacts/panels/gate_report.json`) is the version's `gateReport`; publish.py refuses a report that did
not pass or that names another build. `CANARY=0` skips the canary (a skipped canary does not fail the report). The
canary sends contract v1's `horizonPos` and falls back to the story id when the `ask` binary predates it.

Checked on the real build: an injected `recap.previous` 819 stories ahead and a suggestion 1 past the story give
2 spoiler violations and exit 1; a previous recall of 0.70 fails the recall gate (0.636 < 0.68), 0.65 passes; with a
stand-in `ask`, a citation past the horizon and a wrong corpusVersion each fail the canary.

## 8. Publish protocol (`publish.py`)

- Chunks of `{version, server, builtAt, manifest, gateReport, panels, activate: false}`, gzip, each at most 4 MiB on
  the wire and 8 MiB decompressed; then `{..., panels: {}, activate: true}`, which resends the manifest and the gate
  report because every call overwrites them. Header `x-trevor-worker-key`. Retries after 1, 4 and 16 s on a network
  error, 408, 429 or 5xx; any other 4xx fails at once.
- Today's build: 1,862 panels in 3 chunks (780, 692, 390 panels; 0.73, 0.93, 0.65 MB on the wire; 8.38, 8.38, 5.17 MB
  decompressed) plus the activate call (8 KB wire, 62 KB decompressed); 2.31 MB on the wire in all.
- The manifest: `schema, version, server, corpusVersion, builtFrom` (EN res_version), `builtAt, storyOrder, eras,
  panels, panelsSha, gateOrderSha, answersInline, stats`.
- **Version names**: `<corpusVersion>-p<first 8 hex of the panels sha>`, e.g. `en-e79e364cfbdeeffa-pef0c5bba`.
  `corpusVersion` is the worker's rule (`corpus_version` in `src/bin/ask/serve/mod.rs`) (`en-` + 16 hex over the main corpus's and P4x's chunk shas), so the manifest
  names the corpus the worker reports. The panels suffix gives a summaries-only rebuild its own version, so its
  activate is atomic too; rebuilding identical content gives the identical name, and publishing it again is skipped.
- `--dry-run DIR` writes every request (exact wire bytes plus `requests.json`, key redacted) and sends nothing.

## 9. Rollback

`trevor-vps-update.sh rollback [VERSION]` (default: the version active before the current one, from `state/history.jsonl`)
points `current` back at that version's release (when it is still kept), restarts the worker and resends that
version's stored activate body (`state/versions/<v>/activate.json`). The backend keeps every version's panels, so a
rollback whose release was pruned still restores the panels; the worker then keeps the newer corpus, which the
worker's own reports will show. By hand: `publish.py --activate <version>` (with `TREVOR_WORKER_KEY` and
`TREVOR_STATE_DIR=/var/lib/trevor/state`).

## 10. Monitoring

- `curl -s http://127.0.0.1:3060/api/trevor/status`: `{online, rungs {cf, cpu}, activeVersion, queueDepth, cfBudget}`
  (contract section 4). `activeVersion` should equal `state/active.json`'s version.
- `trevor-vps-update.sh status`: the active version, the live release, the EN res_version, the backend's status.
- `journalctl -u trevor-update -u trevor-summaries -u trevor-worker -u trevor-llama --since today`
- `systemctl list-timers 'trevor-*'`; the Discord webhook posts every publish and one `update_failed` alert per
  failed run.
- Retention (2026-10-19, `scripts/retention.py`, policy in its docstring and `trevor-self-contained-deploy.md` step 9):
  every `daily` run logs the free space of TREVOR_HOME, the code and the assets and posts "Trevor: disk space is low"
  below the smaller of 10 GB and 10%; after each daily publish the run log lists the logs, eval runs, spool files,
  releases and asset manifests over the policy, deleted only with `PRUNE_APPLY=1` (manifests also `PRUNE_APPLY_ASSETS=1`);
  `trevor-vps-update.sh prune` shows the same list by hand; `PRUNE=0`, `DISK_CHECK=0` turn them off.
- `artifacts/bank/regen/history.jsonl`: every `detect` and `run`, with counts. `trevor-vps-update.sh summaries`
  logs `bank-cf.json` and `bank-local.json` in the run's log directory.

## 11. Costs

$0 a month. Cloudflare Workers AI's free plan has a 10,000 Neuron daily allocation that hard-stops (no card exists, so
no bill is possible); the VPS is already paid for; the models, llama.cpp and everything else are free. The worst case
of every failure below is a slower or passage-only answer, never a charge.

## 12. When something fails

| event | what happens | needs the maintainer? |
|---|---|---|
| a step of the update fails (build, gate, upload) | the run stops, the active version and the live release stay, the stage keeps the work, one `update_failed` alert names the step, the exit code and the last log lines; the next 19:30 run retries | only if it repeats |
| the activate call fails after the switch | `current` flips back, the worker restarts on the old release | no |
| Cloudflare retires a model | the canary (`ask --canary`, run by hand: no script or systemd unit schedules it; the worker's cf rung needs a pass recorded in `TREVOR_CANARY_FILE` within `TREVOR_CANARY_MAX_AGE_DAYS`, 8) and the summaries move to the next id in `TREVOR_CF_MODELS`; with none left, the cf rung is off and summaries run on E2B | add a model id when convenient |
| Cloudflare quota spent | the backend stops the cf rung at its ledger cap (`TREVOR_CF_DAILY_NEURONS` minus `TREVOR_CF_SUMMARY_RESERVE`, see [backend/README.md, Trevor](../../backend/README.md#trevor)); summaries stop at the first 429 and the rest go to E2B overnight | no |
| Cloudflare ends the free plan | the cf rung and Cloudflare summaries stop; CPU, passages and cache answer; summaries take longer on E2B | no |
| VPS reboot | the units start at boot (`WantedBy`), `/run/trevor` is recreated by tmpfiles, the timers catch up a missed run (`Persistent=true`). Needs the host's "Start at boot" | host setting once |
| the game changes its data format | build-corpus or a builder fails, the update is held, the site keeps the last version; new stories have no panel ("not covered yet") | to fix the parser |
| the backend is down at 19:30 | build-corpus fails, held, retried the next day | no |
| a new story has no summary yet | its panel ships without one; the 23:00 run adds it | no |
| an update changes many chunks (a new event) | the merge drops the stale answers at once; the questions and answers come back over the next nights, `REGEN_MAX_UNITS` (150) a night | no |
| the warm-up fails | the version stays live; one alert | only if it repeats |
| embedding would mix runtimes | `trevor-vps-update.sh` refuses before embedding (held) | run the one-time fp32 re-embed or set EMBED=0 |
| the E2B GGUF is missing or memory is short | local summaries are skipped; they wait for Cloudflare the next night | no |
| the 19:30 run still holds the update lock at 23:00 | the summaries run exits at once (flock) and the summaries wait a day | no |
| disk fills | rsync or a build fails, held; `KEEP_RELEASES` bounds the releases, logs go after 30 days; the daily disk check posts "Trevor: disk space is low" first, and the prune step's list in the run log says what `PRUNE_APPLY=1` would free | yes |

## 13. Decisions recorded here

- **Summaries are the only model stage** (decision, 2026-10-16); all other model outputs are frozen at the build machine's last build.
- **22 eras** by the provisional rule (D7), on the gatePos scale.
- **Spoilers from the reader's profile via the backend**; the VPS jobs never compute a reader's horizon.
- **Bank entries are gated by their `sources` too** (new in panels.py, default; `--no-gate-sources` is the prototype):
  24 bank entries (18 `who`, 6 `ask`) cite an operator file released after their gate stories (Ines's file is dated at
  gatePos 1609, her gate stories end at 1334). With the default, 43 identity lines and 95 suggestions that the
  prototype showed before those files were released are left out; 47 other suggestions take their places, and 15
  suggestions' gatePos rise. 1,849 panels change in all, 1,849 of them only by `era.position`.
- **Version naming** `<corpusVersion>-p<panels sha8>` (section 8).

## 14. Measured on the development machine (2026-10-16)

Measured on the development machine (Apple M5 Pro, 24 GB) unless a row says otherwise.

| what | value |
|---|---|
| panels.py, 1,862 panels | 7.9 to 11.8 s wall, 160 to 175 MB max RSS (the prototype, measured on a separate VM: 8.8 s, 191 MB) |
| `ask --worker` while loading, build machine (2026-10-17) | peak RSS 152 MB with `--retrieval bm25`, 356 MB hybrid; 12 and 21 MB once settled, since macOS stops counting the memory-mapped indexes. Linux counts touched mapped pages, so expect up to the peaks plus page cache; both fit `MemoryMax=1536M` |
| kill switches `--no-gate-sources --era-guide-position --gate-order off` | 1,862 of 1,862 panels byte-identical to the prototype's `out/` |
| panel sizes (default build) | 21,747,890 B raw, 2,270,197 B gzip -6 (ratio 0.104); per story median 12,134 B, max 19,547 B |
| gates.py without the canary | 0.88 s, 164 MB |
| publish.py --dry-run | 0.26 s, 154 MB; 3 chunks + activate, 2,312,290 B on the wire |
| eval BM25 gold v2 | P3a 0.650, P3b 0.636 recall@10, 0.3 s |

## 15. Open and unverified

- Nothing has run on the VPS: E2B speed (`llama-bench`), the fp32 re-embed time, the second llama-server's memory
  next to the serving one, and AllowedCPUs on this host are expectations.
- The backend side of the contract (gzip request bodies, the 4 / 8 MiB limits, `/api/trevor/status`) was built
  separately; publish.py was tested against a stand-in that enforces the same limits.
- Cloudflare's request shape for Gemma 4 thinking (`chat_template_kwargs`) is the worker's guess, not read from
  Cloudflare's page (no network here); `TREVOR_CF_THINKING` switches it.
- The canary has run only against a stand-in `ask`; its first real run needs a model.
- The warm-up routes (`GET /api/trevor/worker/popular`, `POST /api/trevor/worker/warm`) are specified in contract
  v1.1 section 3c but not built in the backend yet; `bank_regen.py warm` was tested against a stand-in only.
- Database retention is a backend follow-up, not built: nothing deletes a Trevor row today. The policy (cache rows
  forever, ask history 90 days, finished jobs 30 days), as SQL, is in `trevor-self-contained-deploy.md` "Step 9 built":
  `DELETE FROM trevor_asks WHERE created_at < now() - interval '90 days'`, then finished `trevor_jobs` older than 30
  days that no kept ask names, then `cached = false` answers older than 30 days that no job, ask or feedback names.
- Bank regeneration's speed on the VPS's one core is the contract's estimate, not a measurement; the build machine's E2B on
  Metal does a unit in 3.1 s.
