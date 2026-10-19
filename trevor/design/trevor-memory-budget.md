# Fitting the model stages into 11 GB: the memory budget and the plan (2026-10-19)

The one-box deployment (`trevor-self-contained-deploy.md`) runs on a VPS with 11 GB of RAM and 3 vCPU that hosts
only this project. The 12B model the stages use does not fit beside serving. This doc records what the box holds,
what the models cost resident, and the three layers that make the stages fit without removing a feature: a site
diet, llama.cpp hygiene, and a nightly time-sliced window for the 12B. Numbers are measured where marked; the rest
are estimates to be replaced by the VPS measurement in section 5. Research of 2026-10-19 (three read-only passes and
one measurement on the development machine); sources inline.

## 1. What the box holds today

| process | pinned | measured or documented | estimate at rest |
|---|---|---|---|
| backend (pm2) | `max_memory_restart 3G` (`backend/ecosystem.config.cjs:14`) | EN alone 211 MB, EN plus CN 600 MB steady; reload peaks EN 0.84 GiB, CN 1.76 GiB (memory notes 2026-09) | 0.6 to 1.4 GB |
| frontend SSR (bun, pm2) | cap 1500M, SSR cache 48 MB (`frontend/ecosystem.config.cjs:12-28`) | 212 to 230 MB after load, peaks 580 to 613 MB (macOS) | 0.25 to 0.6 GB |
| asset watchers | none; four pm2 processes by the legacy names (`vps-update.sh:50` (repo root, git-ignored)) | about 50 MB each idle (`assets/ecosystem.config.cjs:49`) | 0.2 to 0.4 GB |
| Discord bot | cap 512M | never measured | 0.1 GB |
| Postgres, Redis | nothing pinned in the repo | nothing | 0.4 to 0.8 GB |
| trevor-llama (E2B, `-c 4096 -np 1`) | `MemoryMax=4G` | **measured here: 3.66 GB after load, 1.93 GB steady** | 1.9 to 3.7 GB |
| trevor-worker | `MemoryMax=1536M`; BM25 by default | **measured here: 1.53 GB peak with the fp32 embedder and hybrid retrieval** (over its own cap), about 0.2 GB BM25-only | 0.2 GB |
| OS | | | 0.3 GB |

Serving today: about 5.4 GB in the low case and 8 GB in the high case, leaving 3 to 5.5 GB. The unit caps sum to
16.5 GB, so only the MemAvailable guards stand between a stage and the OOM killer; no unit sets `OOMScoreAdjust`.

## 2. What the models cost resident (measured on the development machine, CPU only, 3 threads)

RSS transfers to Linux for mapped weights; the speeds do not (unified memory and Accelerate make this machine 2 to 4
times faster per thread).

| model, context | after load | steady after a 2k-token prompt | 2k prompt plus 16 tokens |
|---|---|---|---|
| E2B Q4_K_M, 4k | 3.66 GB | 1.93 GB | 59 s |
| 12B QAT Q4_0, 8k | 9.60 GB | 7.34 GB | 147 s |
| 12B, 8k, `-ctk q8_0 -ctv q8_0` | 10.29 GB | 7.58 GB | 126 s |
| 12B, 16k | 10.35 GB | 8.19 GB | 142 s |

So the 12B needs 7.3 to 7.6 GB at 8k context; KV quantization saves nothing measurable here (the research puts its
saving at 0.28 GiB on the 12B); 16k costs about 0.85 GB more. Beside 5.4 to 8 GB of serving it cannot fit, which is
why section 4 time-slices it.

Two facts from llama.cpp's code matter for every server on the box (research pass, llama.cpp master and build 11146):
- On x86 with AVX2, repacking copies Q4_0 and Q4_K tensors into anonymous memory by default. Those pages cannot be
  shared between processes or evicted, only swapped. `--no-repack` keeps the weights page-cache backed, so two
  servers mapping one GGUF hold one copy, at an unmeasured cost to prompt speed. The stage launch in
  `trevor-vps-update.sh` already passes it; the serving unit does not. Correction (2026-10-19): it does, and has
  since f201aa08 (`trevor-llama.service` runs `-np 1 --no-repack`); what was missing is the measurement of its cost.
- Two server defaults eat RAM: `--cache-ram` (8192 MiB default; serving passes 512, the stages 0) and
  `--ctx-checkpoints` (32 per slot, each about the SWA cache state, about 0.5 GiB on the 12B). Stages set both to 0.
  `--kv-unified` does nothing at `-np 1`; pass `-np 1` explicitly, the default is auto.

## 3. Layer one: the site diet (about 0.5 to 0.75 GB, low risk, about 1.5 days)

Ranked by MB freed per day of work; file:line is where each starts.

| # | change | MB freed | risk | days | starts at |
|---|---|---|---|---|---|
| 1 | one asset-watcher process instead of four (the one-process queue shipped in 71864a54; `PM2_WS` still defaults to the four legacy names) | about 150 | low; the legacy config stays as the kill switch | 0.25 | `vps-update.sh:50` (repo root, git-ignored), `vps-start.sh:37` |
| 2 | `SERVERS_LAZY=jp,kr` (built, off by default) | 219 (measured, af6a07c8) | the first JP or KR request after 30 min idle waits up to 20 s or gets 503 plus Retry-After | 0 | `backend/src/app/residency.rs:1-20`, `backend/README.md:294` |
| 3 | clear the per-server `ServerCache`s when a lazy server unloads (today only `GameData` is dropped; the 21 static-table slots and the story, music, archive, gallery and sprite caches keep their `Arc`s until the next request). **Done 2026-10-19** behind `LAZY_UNLOAD_CACHES` (`0`: as before); measured: the unload frees what the slots held, 127.1 MiB of heap (75.5 MiB RSS) for EN-sized tables with both forms, 7.3 MiB with #4, plus the story caches (not measured) | 85 to 100 per lazy server; this makes #2 real | low | 0.5 | `residency.rs:331-368`, `services/static_data.rs:57`, `services/story/cache.rs:50` |
| 4 | static tables: keep the brotli form only, or write the identity `SharedJson` to disk so the kernel can reclaim it (about 75 MB identity plus 8 MB brotli per server). **Done 2026-10-19**, brotli only, behind `STATIC_TABLES_BR_ONLY` (`0`: both forms); measured on EN (`memory_bench`, `BENCH_TABLES_BR_ONLY`): the 21 slots hold 127.1 MiB of heap with both forms (72.8 MiB identity, 7.3 MiB brotli, the rest spare capacity), 7.3 MiB brotli only; RSS with EN loaded goes from 270.9 to 346.4 MB with both forms and from 263.3 to 275.2 MB brotli only, so about 64 MB saved per server; an identity or gzip response decodes operators (2.5 MB brotli, 23.8 MB JSON) in 25.2 ms median (24.8 to 31.1, 7 runs, optimized build), and the SSR asks for identity (`frontend/src/lib/fetch.ts:14`) | 150 to 300 | low; gzip and identity clients pay 25 to 100 ms to decode | 0.5 to 1 | `app/cache/mod.rs:46-49` |
| 5 | frontend: stop caching the whole camelized operator table per server for 30 min (about 40 MB serialized); serve the export fields and the OG user card as projections from the backend | 100 to 300 of bun heap, most of its peak | medium; the export dialog and OG card need the same fields | 1 to 2 | `frontend/src/lib/api/operators.ts:64-86,116-118`, `lib/og/impl/registry.ts:258` |
| 6 | make CN lazy too | 150 to 250 | medium; the release planner and CN preview's first load, and the release and gacha jobs that read CN reload it | 1 to verify the jobs | `backend/src/app/state.rs:504-525` |
| 7 | gate backend reloads and lazy loads on MemAvailable and on the Trevor model lock, like the watchers' `WS_MIN_FREE_MB`. **Done 2026-10-19** behind `RELOAD_GUARD` (`0`: every build starts at once, as before): a hot reload or lazy load does not start while `MemAvailable` is below `RELOAD_MIN_FREE_MB` (EN 900, CN 1800, others 1200 MiB, the measured peaks rounded up; JP, KR and TW unmeasured) or while `TREVOR_LLM_LOCK` (`/run/trevor/llm.lock`) names a live pid; a refused reload keeps the old data and is retried every `RELOAD_RETRY_SECS` (300), a refused lazy load answers 503 plus `Retry-After`, one Discord embed past `RELOAD_MAX_WAIT_SECS` (6 h). Tests: 10 unit tests in `core/reload_guard.rs`, one through `perform_reload` and `ensure_loaded` with a held and a released lock (`residency.rs`). Unverified: the memory half on Linux (macOS has no `/proc/meminfo`, so it ran only from a fixture) and the whole path on the VPS | removes a transient 0.84 to 1.76 GiB second copy that can collide with a model | low; an update lands up to the window's end late | done | `core/reload_guard.rs`, `residency.rs` `start_load`, `asset_watcher.rs` `perform_reload` |
| 8 | jemalloc `narenas:4` (default 12 on 3 vCPU); check with `examples/memory_bench.rs`. **Measured 2026-10-19, not shipped**: `memory_bench`, `BENCH_LOAD_ONCE=1`, EN, debug build, one process per run, 3 runs per arm, `_RJEM_MALLOC_CONF` with the pm2 string plus `narenas:N` (read back as `narenas=N`; unset reads 0, which is 4 per CPU: 60 on this 15-core machine). With `RAYON_NUM_THREADS=15`, RSS after load median 255.8 (default) / 261.8 (12) / 262.6 (4) MiB, peak RSS 284.0 / 283.4 / 283.5 MiB, load 17.68 / 18.51 / 17.47 s; with `RAYON_NUM_THREADS=3` (the VPS's core count) 257.4 / 250.4 / 249.9 MiB after load, peak 278.2 / 278.3 / 272.2 MiB, load 17.52 / 17.72 / 21.47 s (runs 16.10 to 23.52 s). Peak live heap is 231.0 MiB in every arm, so the arena count moves only allocator slack, and the 7.5 MiB best case (3 threads, 4 against default) sits inside the run-to-run spread (three runs fell to 127 to 169 MiB after load when macOS purged). No change made: `narenas:4` saves nothing measurable here. Open: macOS jemalloc has no background thread, and a load-only bench has no tokio workers allocating per request, which is where extra arenas hold memory; a `pm2 ls` RSS week on the VPS with and without `narenas:4` would decide | 20 to 60 estimated; 0 measured locally | low | 0.25 | `backend/ecosystem.config.cjs:38` |
| 9 | Discord `max_messages` 1000 to about 200 per channel | 10 to 50 | audit logs lose older deleted messages' content | 0.1 | `discord/src/main.rs:214` |
| 10 | parse `roguelike_topic_table` into typed structs (a `serde_json::Value` at boot, then dropped). **Done 2026-10-19** as a pruned read behind `ROGUELIKE_VALUE_PARSE` (`1`: the whole `Value`, as before): a serde visitor keeps, per path, only the fields the three readers use (theme maxima, tier-list entries, the stage classifier's names) in the `Value` shape they already read, so the readers are unchanged; `Topics` plus `Details` go from 8.1 MB to 0.99 MB as JSON on EN (8.9 to 1.08 MB on CN). Identical themes, entries (1,681 EN, 1,936 CN), IS nodes (385, 453), node ids and KV banners on EN, CN, JP and KR (`tests/roguelike_prune_real_data_test.rs`). Measured (`memory_bench`, `BENCH_LOAD_ONCE=1`, debug build, one process per arm, 2 runs): EN peak live heap 231.0 -> 190.2 MiB, peak footprint 268.8/268.2 -> 224.9/225.3 MiB; CN peak live heap 291.1 -> 245.0 MiB, footprint 326.9/327.3 -> 278.3/278.5 MiB; live heap after load unchanged (141.4 EN, 149.3 CN MiB: the `Value` was transient). Peak RSS moves the same way but within macOS noise (EN 284.3/229.3 -> 187.2/226.9, CN 273.3/255.9 -> 233.7/228.8 MiB). Load time did not slow: EN 31.44/25.26 -> 27.35/22.96 s, CN 27.91/35.70 -> 27.48/28.28 s. The trade: a level id outside the kept paths is still found (the walk keeps any object carrying one), but a new field a reader starts reading must be added to the spec in `roguelike_prune.rs`, or the real-data test fails | 41 (EN) to 46 (CN) MiB of boot and reload peak heap, measured | low | done | `core/gamedata/types/roguelike_prune.rs`, `core/gamedata/mod.rs:331` |
| 11 | sandbox counting parse: `sandbox_perm_table` was a `serde_json::Value` held to the end of the load (169 MB of startup RSS over the four servers in the production `STEP_RSS` run) and parsed again by the stage classifier, for 11 counts and 106 level names. **Done 2026-10-19**: one streaming serde visitor pass feeds both, behind `SANDBOX_VALUE_PARSE` (`1`: both `Value` parses, as before); counts and names identical on EN, CN, JP and KR (`tests/sandbox_perm_scan_real_data_test.rs`). Measured (`memory_bench`, `BENCH_LOAD_ONCE=1`, debug build, one process per arm): EN peak live heap 242.0 -> 231.0 MiB, peak RSS 296.8 -> 283.8 MiB, RSS after load 276.8 -> 263.1 MiB; CN peak live heap 322.9 -> 291.1 MiB, peak RSS 363.6 -> 344.5 MiB, RSS after load 292.9 -> 280.6 MiB. So 11.0 MiB (EN) and 31.8 MiB (CN) of heap per server, 12 to 14 MiB of resident RSS after load on this machine. Alongside: `activity_skin_refs` is scanned on CN only (`ACTIVITY_SKIN_REFS_ALL=1`: every server, as before), and the activity table is dropped after its last derive, which moved nothing measurable on CN (peak live heap 291.1 -> 291.1 MiB, peak RSS 342.5 -> 344.5 MiB, noise). Open: production saw +42 MB on EN against 11.0 MiB of heap here; a `STEP_RSS` boot on the VPS after deploy decides whether the gap is the newer tree or the allocator | 13 to 32 per server measured locally | low | done | `core/gamedata/types/sandbox_perm_scan.rs`, `core/gamedata/mod.rs:443` |

Pins that belong on the box and are set nowhere in the repo: Postgres `shared_buffers=512MB` (not 25% of RAM; the page
cache is reclaimable, shared buffers are not), `work_mem=8MB`, `max_parallel_workers_per_gather=1`,
`maintenance_work_mem=128MB`, `max_connections=40` (the pool is 20), `effective_cache_size=2GB` (planner only); Redis
`maxmemory 256mb` (every key has a TTL, so any policy evicts sessions: read `INFO memory` first, it is probably small).
Not worth doing: static prerendering (saves CPU, not memory; the SSR cache already hits 88%), sharing
language-independent tables across servers (a large refactor for 30 to 50 MB per server), a systemd timer for the
watcher (it breaks the backend's WebSocket contract on ports 9160 to 9163 for about 50 MB).

With #1 to #4 done, the site without Trevor sits at about 2.5 to 3.5 GB.

- done (2026-10-19): #1. The repo-root `vps-update.sh` and `vps-start.sh` default `PM2_WS` to the one app `myrtle-ws`
  (`assets/ecosystem.config.cjs`); `PM2_WS_LEGACY=1` restores the four legacy apps and their config. Both layouts bind
  ports 9160 to 9163, so a deploy that finds the other layout's apps in pm2 deletes them before starting its own and
  warns if the new one did not come up; `vps-start.sh` only reports a mismatch. Both files are git-ignored: the change
  reaches the VPS only when they are copied there by hand. Unverified: no run on the VPS, `sh -n` only.
- done (2026-10-19): #2 as a flag. `bootstrap.sh --lazy-servers jp,kr` writes `SERVERS_LAZY=jp,kr` into
  `backend/.env`; without it the env step writes nothing and prints the recommendation. What to watch afterwards:
  `trevor-vps-runbook.md` section 10.
- done (2026-10-19): the pins. `bootstrap.sh`'s new `settings` step prints them as a block to copy, with the commands
  that read the current values first; it applies nothing. `deploy/README.md`, "Memory".

## 4. Layer two and three: llama.cpp hygiene, then the 12B in a nightly window

Hygiene, free: `--no-repack` on the serving server as well (measure its prompt-speed cost once), `--cache-ram 0` and
`--ctx-checkpoints 0` on every stage server, `-np 1` explicit, `OOMScoreAdjust=1000` on `trevor-llama` and
`trevor-stages` so the kernel kills a resumable model before Postgres, `MemoryMax=2G` on the worker before hybrid
retrieval is ever switched on there (its measured peak is 1.53 GB against a 1536M cap).

The window. The model lock already guarantees one llama-server at a time; the design let the E2B worker and the
stage runner run side by side. Instead the runner stops the serving server for a bounded nightly window, loads the
12B, runs stages, and restores E2B; serving falls to cache plus passages for the window, which contract v1.1 already
supports (the passages rung). Resident during the window: the 12B at 7.3 to 7.6 GB plus the trimmed site at 2.5 to
3.5 GB, which fits 11 GB with about 0.5 to 1 GB of margin; it does not fit beside today's untrimmed site in its high
case, so layer one comes first. The 3 vCPU make this the right shape regardless of memory: two models would halve
each other's speed. Throughput: on 3 threads a 7,000-token prompt plus 450 output tokens is about 11 minutes
(estimate from the measurement above, 3 times slower than this machine); one game update's roughly 400 calls is
about 75 window-hours, so a 6-hour window covers it in about 12 nights, inside the two-week EN cadence. Stage server
flags: `-np 1 -c 12288 --cache-ram 0 --ctx-checkpoints 0 -fa on`. The runner's lock, pause and `STAGES_DAILY_AT`
guard already exist; the window itself (stop and restore `trevor-llama`, a start and end hour, the passages notice)
is the one piece to build, about a day.

- done (2026-10-19): the hygiene. `trevor-llama.service` keeps `-np 1 --no-repack` (present since f201aa08) and gains
  `OOMScoreAdjust=1000`, with a comment naming the one open measurement (prompt tokens/s on the VPS with and without
  `--no-repack`); `trevor-stages.service` gains `OOMScoreAdjust=1000`; `trevor-worker.service` goes to
  `MemoryHigh=1792M MemoryMax=2G`. Every llama-server `stages.py` starts gets `-np 1 --cache-ram 0 --ctx-checkpoints 0
  -fa on` in the fixed part of its command (`server_cmd`), so `STAGES_SERVER_ARGS` (which keeps `--no-repack`) cannot
  drop them; `STAGES_SERVER_LEAN=0` restores update.sh's flags (the profile's slots, `-cram 512`), checked identical
  on 16 command lines. The spellings `--cache-ram` and `--ctx-checkpoints` are build 11146's (`llama-server --help`).
  Not set here: the `-c 12288` of the window, which is a separate change.
- window: built (2026-10-19), tested with stubs, unverified with a model and on a box. `stages.py` asks root through a
  marker, `$TREVOR_HOME/state/stages/llama-window` (`stop` or `start`, renamed into place); the root pair
  `deploy/systemd/trevor-llama-window.{path,service}` watches it and runs `systemctl stop|start trevor-llama`. Before
  the first unit that needs a model inside `STAGES_WINDOW_START` to `STAGES_WINDOW_END` (UTC, 18 to 0: 03:00 to 09:00
  JST) the runner writes `stop`, waits at most `STAGES_WINDOW_WAIT` (300) s for `STAGES_SERVING_CHECK` (`pgrep -x
  llama-server`) to show no server, takes the model lock, and runs GEN from `STAGES_WINDOW_GEN` (update.sh's 12B; JUDGE
  `STAGES_WINDOW_JUDGE`, extra flags `STAGES_WINDOW_SERVER_ARGS`, where the `-c 12288` goes once measured) at
  `STAGES_WINDOW_DUTY` (1). It writes `start` at the window's end (an interruptible stage is stopped first), when a pass
  has nothing left, before a `--publish` rebuild (the canary asks through serving), on every pause (the 19:30 UTC
  update's lock included, so the window pauses for it like any stage and reopens after), on duty 0, on every exit
  including SIGTERM, and at startup when it finds `stop`; a 12B that fails to start, a memory floor tripping in a window
  stage, or a serving server still up after the wait gives the night up (the runner then works as outside the window).
  Canon does not start within `STAGES_CANON_GUARD` (2 h) of the window's end. `status.json` carries `window: {state,
  since, model, end, reason, error}`; one Discord embed per window when serving is not back `STAGES_WINDOW_GRACE` (600)
  s after `start`. Kill switch `STAGES_WINDOW=0`: no marker, no model change, no status field (default 1 with
  `--publish`, 0 without). The order inside the window is the runner's: summaries, the re-answer, then the rest.
  Tests: `scripts/tests/test_stages.py` class `Window`, 7 runner tests (the unit's own ExecStart line run against a
  stub `systemctl` and a stub serving check) plus a bounds unit test; the suite is 31 tests in 294 s, most of it the
  real tree's plan. Open: `bootstrap.sh` must install and enable the `.path` unit (until then the runner waits 300 s
  each night, logs an error and gives the window up); `systemd-analyze verify` was not run (macOS); outside the window
  the runner still runs E2B by day, and `incr.py`'s keys hold no model name, so a unit E2B did by day is never redone
  by the 12B: `STAGES_WINDOW_ONLY=1` keeps the stages to the window and is the setting to use once a night is verified.
- daily waits for the backend's reload (2026-10-19): the 19:30 UTC update runs inside the window, where the backend
  defers its game-data reload (89f07593), so `trevor-vps-update.sh daily` waits up to `BACKEND_RELOAD_WAIT_MIN` (20)
  min, after its run lock pauses the runner and frees the model lock, until the guard's inputs have admitted EN for 420
  s (the 300 s retry plus 120 s for the build); else it skips the night and the next run retries. Tested with a stub
  lock holder; unverified on the box.
- daily waits for the backend's reload: exact since 2026-10-19, `/api/health` `gameData.loaded.en.version` must equal
  the `.version` res (the 420 s inference stays the fallback for a backend without the field or with a null version).
  Tested with a stub `/api/health`; unverified on the box.

Fallbacks, in order, each with its quality evidence:
- 12B at IQ3_M (bartowski, 5.56 GiB) or Q3_K_M (unsloth, 5.30 GiB): about 1.2 to 1.5 GB less resident; mean KL
  divergence against F16 on Gemma-3-12B is Q4_K_M 0.128, IQ3_M 0.190, Q3_K_M 0.274 (eaddario's measurements), so
  1.5 to 2 times the drift of today's Q4. Use only if the site cannot be trimmed enough.
- Gemma 4 E4B QAT (unsloth UD-Q4_K_XL, 3.93 GiB) for the low-stakes stages by day (dossiers, topic mining, cues),
  beside serving: MMLU-Pro E2B 60.0, E4B 69.4, 12B 77.2; GPQA 43.4, 58.6, 78.8 (Google's model card); QAT top-1
  agreement 98.5% (unsloth). The 12B window keeps summaries and the bank re-answer. Costs daytime CPU.
- Q2 quantizations of the 12B: no Gemma measurement found, labelled experimental; not credible.
- A bigger box replaces all of this: Hetzner CX53, 8 vCPU, 32 GB, about 29.49 EUR a month ex VAT (price list of
  2026-06-15); or a second box as the stage host, CX43 16 GB at 15.99. Against the owner's no-spend posture, the
  three layers above reach the same end on the current box.

Rejected: zram or swap for weights (quantized weights barely compress; mapped pages never swap; anonymous repacked
weights swap at disk speed); `--cpu-moe` (no GPU, and the 26B-A4B does not fit at 14.4 GB); the Qwen 9B judge beside
anything (5.3 GiB plus KV; it runs inside the window after the 12B, one model after the other).

## 5. Measure first, on the VPS, in one quiet hour

1. `ps -eo pid,rss,args --sort=-rss | head -20`, `free -m`, `smem -tk -c "name pss"` if installed: how many
   watchers run and what pm2 hides. `grep SERVERS /var/www/myrtle.moe/backend/.env`: which servers load.
2. Backend by component: restart once with `STEP_RSS=1` (`core/startup.rs:334-366` prints `step_rss` per load step),
   then curl each `/{server}/static/{resource}` and read `Rss` and `Anonymous` from `/proc/$PID/smaps_rollup` after
   each. `redis-cli INFO memory`; the Postgres database size.
3. Stop `trevor-llama`. Run `nice -n 19 llama-server -m gemma-4-12b-it-qat-q4_0.gguf -t 3 -np 1 -c 12288 --cache-ram 0
   --ctx-checkpoints 0 -fa on`; record the wall time from a cold page cache to `/health`, the log lines
   `CPU_REPACK` or `CPU_Mapped model buffer`, `llama_kv_cache size` and `compute buffer size`, `smaps_rollup`, and
   prompt and generation tokens per second on one real 7,000-token summary prompt. Repeat once with `--no-repack`.
4. The site's resident set plus that RSS decides between the Q4 12B and IQ3_M; the tokens per second decide whether
   E4B is needed by day.

## 6. Order of work

1. The VPS measurement (section 5), one hour, no code.
2. Site diet #1 to #4 and the pins (section 3), about 1.5 days, each behind its own switch.
3. The window in `stages.py` (section 4), about a day, then the first supervised night.
4. Decide the fallbacks from the first week's numbers.
