# Integrating Trevor into myrtle.moe (design, 2026-10-07)

Status: proposal, nothing built. Ian asked (2026-10-06): integrate Trevor into the existing stories page, as scalable as possible, with automated "training" (rebuilds). This document says where each part should run, what the backend and frontend gain, how updates are automated, and in what order to build it.

## 1. The constraint that decides the shape

The production VPS cannot host the answer model. Ian's Proxmox view (2026-10-06) shows VM 103 at 88.5% of 3 CPUs and 89.7% of 11 GiB memory (9.87 GiB used), and pm2 shows the backend at 845 to 897 MB, the frontend at 289 to 416 MB, four asset watchers at 63 to 78 MB each and the Discord bot at 43 MB. Trevor's answer path needs Gemma 4 12B (7.0 GB of weights, about 8.6 GB resident with the vision projector); on the M5 Pro's GPU one answer takes about 22 to 28 s with up to three model calls (router, question form, answer). On 3 shared vCPUs the same model would generate a few tokens per second, so one answer would take minutes and would starve the backend, which already queues CPU work behind `CPU_TASK_PERMITS=2`. Even the retrieval side (the gte embedder, the reranker, BM25 and about 72 MB of vectors per corpus; `trevor/artifacts` is 1.2 GB on disk) would claim several hundred MB on a host with about 1.1 GiB free.

So the VPS keeps doing what it does well (HTTP, auth, Postgres, Redis, caching) and gains only light Trevor plumbing; all model work runs on a separate Trevor worker with a GPU (today Ian's Mac; later any always-on Apple Silicon or GPU box), which connects out to the backend. Trevor's "no external LLM APIs" rule is unchanged: the worker is Ian's own hardware.

## 2. Architecture

```
Browser (stories reader and library)
   |  ask / poll / panels            (existing frontend server functions -> backend)
   v
myrtle-backend (VPS, axum)   Postgres: trevor_versions, trevor_jobs, trevor_answers, trevor_feedback
   |                         Redis: answer cache, ask rate limits, worker heartbeat
   ^
   |  outbound long-poll (later WebSocket) from the worker, x-service-key
Trevor worker (GPU host: the Mac now, an always-on box later)
   trevor serve:  model loaded once, retrieval in memory, the ask pipeline as a library call
   trevor update: incremental rebuild, evaluation gate, publish
```

Three properties make this scale:

1. **The worker pulls; the backend never calls into it.** The worker long-polls `POST /api/trevor/worker/next` with the service key and takes at most `slots` jobs at a time. No inbound port on the worker, no VPN; a second worker (another machine or another slot) is just another poller. With no worker connected, jobs wait in the queue and the UI says so.
2. **Most answers never reach a model.** Answers are cached by `(corpus_version, normalized question, spoiler horizon, server)`. Deterministic tools (reading guide, reading_compare, appearances, canon, deaths, IS endings, topics, dossiers) are published as precomputed JSON and served from Postgres/Redis on the VPS with no worker at all. Popular questions are precomputed off-peak for each new corpus version (the answer bank exists: `scripts/answer_bank.py`).
3. **Every corpus change is a versioned, gated release.** A game update becomes a new `corpus_version` only after it passes the evaluation gate (section 6); the backend switches versions atomically and old cache entries simply stop matching.

## 3. Trevor changes (worker side)

- **`trevor serve`** (a new binary sharing the `ask` pipeline): loads the runtime (store, indexes, embedder, reranker, tools) once, keeps llama-server running, and executes jobs as library calls. Today every `ask` run reloads indexes and may start the server (about 15 s). One answer stays about 22 to 28 s, so about 130 uncached answers an hour per slot; `-np 2` may add a second slot at some latency cost (measure; memory caps it near 2 on a 24 GB Mac).
- **Spoiler horizon** (required for the stories page): an optional `horizon` (a story id or release date) limits retrieval, tool outputs, topics, dossiers and canon evidence to stories released at or before it. The release order exists (`artifacts/chrono`, `artifacts/spoiler.jsonl`, the answer bank's spoiler gates); it needs one filter in the store, and generated summaries built from later stories must be excluded or built per release era (a handful of eras, not one per story, to keep the count small).
- **Structured output**: answer text plus citations as story ids and line spans, the route taken, flags, `corpus_version` and timings, so the frontend can link citations into the reader.
- **Streaming (later)**: llama-server streams tokens; the worker can forward chunks and the backend relay them over SSE. Not needed for v1.

## 4. Backend changes (VPS, light)

- **Migration v029**: `trevor_versions(version, built_at, gate_report jsonb, active)`, `trevor_jobs(id, user_id, question, horizon, server, status queued|running|done|failed, worker_id, created_at, started_at, finished_at, answer_id)`, `trevor_answers(id, version, qkey, horizon, answer jsonb, created_at, hits)`, `trevor_feedback(answer_id, user_id, vote, note, created_at)`.
- **Routes** (`backend/src/app/routes/trevor.rs`, utoipa like the rest, services in `app/services/trevor/`):
  - `POST /api/trevor/ask {question, storyId?, server}`: a cached answer (200) or `{jobId, position, eta}` (202). The handler only normalizes, looks up the cache and enqueues, far under the 30 s `HANDLER_TIMEOUT`.
  - `GET /api/trevor/jobs/{id}`: polled every 2 s (v2: an SSE stream route exempt from the handler timeout).
  - `GET /api/trevor/story/{storyId}/panels`: the deterministic panels for one story (who speaks here and where else they appear, what to read before, what this event builds on, topics it names), from the published JSON, cached with `cached_json_detached`.
  - `POST /api/trevor/answers/{id}/feedback`.
  - Worker routes (service key): `worker/next`, `worker/result`, `worker/heartbeat`, `versions` (publish).
- **Limits**: a new `Bucket` for ask (for example 10 an hour anonymous, 30 signed in; kept in Redis so restarts do not reset it), a global queue cap with a clear "Trevor is busy" response, and a per-user cap on queued jobs.
- **Cost on the VPS**: a few tables, a Redis namespace and published JSON in the tens of MB (in Postgres or on disk, not in the backend heap). No new process.

## 5. Frontend: Trevor in the stories page

Built into `components/story/reader/` and `components/story/library/`, with the existing server-function client (`lib/api/`) and ICU messages (`*.messages.ts`).

- **Reader side panel ("Ask Trevor")** from the reader toolbar. It knows the current `storyId` and the reader position (`lib/story/progress.ts`, synced through `/user/story-progress`) and sends `horizon = current story` by default, with a visible "spoiler-safe up to this chapter" toggle.
- **Context chips that need no model**, filled from `/panels`: who is speaking here, where else a character appears, what to read before this, what happened so far (recap, horizon-limited). They answer instantly and keep working when no worker is online, so most reader help never touches the queue.
- **Free questions** go through ask: cached answers show at once; queued ones show position and ETA; citations jump to the cited line in the reader.
- **Library page**: an "Ask about the story" box with the same behavior, horizon defaulting to the user's furthest-read story.
- **Feedback**: thumbs up/down and "this is wrong" with an optional note on every answer.
- **Worker offline**: the panel says free questions are offline and offers the chips; never an endless spinner.

## 6. Automating "training": rebuild, evaluate, release

Trevor is not trained by gradient descent: fine-tuning the answer model was measured and rejected on 2026-09-29, and the embedder fine-tune gave no hybrid gain. "Training" means rebuilding its knowledge and checking it, which `scripts/update.sh` already does incrementally with hash-keyed stages. Automation wraps it:

1. **Trigger**: the backend's asset watcher already receives `update_complete` from `myrtle-ws-en`; it records the game-data version and enqueues a `trevor-update` job (same queue, job type `update`).
2. **Data to the worker**: the worker fetches changed gamedata and story text from the backend (a service-key endpoint that streams changes since a version, or rsync over SSH), so it never runs its own downloader.
3. **Incremental rebuild on the worker**: `scripts/update.sh` (corpus refresh with vector reuse, indexes, changed topics, dossiers, canon evidence, art captions, appearances, reading guide). A typical event update takes minutes; a full rebuild takes hours and runs overnight.
4. **Evaluation gate** (blocks publishing): route check (keywords 61/61, model at least 75/76), Ian's questions against their `verified` notes, and a fixed gold v2 and real-probe sample judged by Qwen, new version against the active one, with the 0-loss rule used for every default since 2026-09-29. A failing gate holds the release and sends the report to Ian through the existing Discord bot.
5. **Publish**: upload the panel JSON and version manifest (`POST /api/trevor/versions`), switch `active`, and precompute the top questions (by `trevor_answers.hits` and the answer bank) for the new version off-peak.
6. **Feedback loop**: weekly, "this is wrong" reports become candidate eval items; once Ian verifies them they join `eval/ian-questions.jsonl`, so the gate grows from real usage. Thumbs-down clusters show which question types to work on next.

## 7. Scalability and the numbers to watch

- **VPS load**: small and constant (enqueue, cache lookups, panel JSON); the handler timeout and CPU permits are never involved.
- **Answer capacity**: about 130 uncached answers an hour per slot today; cached answers and chips are effectively unlimited. Scale out by adding pollers. An always-on Apple Silicon box with 24 to 32 GB would remove the laptop outages and could also run the 26B model that the 24 GB laptop could not load cleanly.
- **Availability**: worker heartbeats in Redis drive the UI's online state; queued jobs older than a set age expire with a clear message.
- **Metrics**: queue length and wait, cache hit rate, answer latency, feedback ratio per route, gate results per version.

## 8. Build order

1. `trevor serve` with structured output and the spoiler horizon, measured locally on the existing eval sets with the 0-loss rule.
2. Backend v029 tables, routes, worker protocol and the ask rate-limit bucket; end-to-end test with the Mac as the worker.
3. Frontend reader panel: chips first (they ship value with no worker), then free questions with polling.
4. Automation: asset-watcher trigger, data sync, gate, publish, precompute.
5. Feedback loop and SSE streaming.

**Status (2026-10-07 morning).** Step 1 is built as `ask --serve` (one binary with the ask modules, not a second binary): the HTTP JSON API on 127.0.0.1, the structured result with `corpusVersion` and `server`, the spoiler horizon over retrieval, and the `--worker` pull loop as an unexercised skeleton. Without a horizon it is byte-identical to the one-shot `ask --no-cache-prompt` on 10 of 10 of Ian's questions; with one, 0 later passages reached the model on 3 of 3 examples (design/trevor-questions.md section 12, 2026-10-07). Left for step 1: horizon-aware table tools (today a table route under a horizon falls back to retrieval) and per-era generated summaries, dated operator files and P4 units (hidden under a horizon today), the full 0-loss run on the eval sets with and without a horizon, a measured answer-time comparison (the first showed no speedup), `-np 2` and concurrency, and an idle unload. Step 2 owns the backend side of the worker protocol.

**Resume note (2026-10-07 08:12, stopped for class; nothing running).** Measured 07:56 to 08:08 on ian1 to ian10, one client, `--no-cache-prompt`, memory sampled every 5 s: `ask --serve` took 307 s (6 s load, 301 s of jobs: router 55.5 s, answer 203.8 s) against 353 s for the 10 one-shot processes, so serve is 46 s (13 %) faster this time, and the answers are identical on 10 of 10 (text, and the same number of cited lines). Yesterday's 26 s gap did not reproduce; it was model-time variance under memory pressure (yesterday router 64.8 s and answer 235.1 s against 55.5 s and 203.8 s today for the same prompts), not the pipeline: both arms ran at the same pressure (free memory median 31 % serve, 32 % one-shot, minimum 20 % and 21 %, level "warning", swap 11.1 GB to 13.75 GB, most of it at llama-server's load, which is 7.8 to 9.0 GB resident in both arms). The serve process's own resident set is 64 MB median (661 MB peak) and under 50 MB in 31 of 63 samples, so its indexes are paged out while Gemma runs, not double-held. One cost found: ian7 (a declined answer that takes the typed fallback) spends 21.2 s outside the router and answer timings against 1 to 3.5 s for the other nine; reading `swap_p4` and the fallback, a corpus loaded on first use builds a second `Runtime` with its own query embedder (`Runtime::load(.., true, ..)`), so the first job needing P4x or the art corpus pays that load once and keeps a second embedder resident. Not fixed yet (sharing one `QueryEmbedder` across runtimes, or loading P4x at start, is the cheap fix; measure ian7 twice in one session to confirm the 21 s is the load). Horizon coverage: `scripts/source_dates.py` (no model, seconds) writes `artifacts/chrono/source_dates.json`, EN release dates (an upper bound each) for 2,569 unplaced sources from the EN game data (records: `HandbookAvgList.StoryGetTime`, 367 of 367; operator files and voice lines: the operator's earliest record date, else earliest non-initial module, 394 of 417 and 394 of 413; modules `UniEquipGetTime` 467 of 467; outfits `GetTime` else the operator's date, 474 of 474; game-text units naming a dated guide group, 473 of 1,415); `src/search/horizon.rs` admits such a chunk when its date is at or before the latest guide date at or before the horizon's position. Chunks with a position under a horizon: P3b 10,495 placed plus 5,202 newly dated of 16,050 (353 still hidden: 200 profiles, 88 summaries, 65 file chunks), P4x 10,495 plus 8,549 of 24,616 (5,572 hidden: 1,533 enemy, 1,359 item, 1,220 IS, 1,014 game text, 200 profiles, 88 summaries, 46 topics, 112 file and voice). The 3 horizon examples rerun after the change keep 12, 14 and 15 passages from the same groups as before (main_2/main_3; main_1/main_2/main_4; act3d0/act5d0/main_6), 0 after the horizon, and no newly dated source reached their top k (Ch'en and Mon3tr still decline, Talulah still answers from Chernobog). Kill switch `TREVOR_HORIZON_DATES=0` (or no file) leaves every unplaced chunk hidden, which is the path the unchanged horizon tests exercise; not yet run end to end. Tests 96 passed (1 new: dated sources), clippy clean. Next: a question that needs an operator file under a horizon (to see the dating used), the shared embedder fix and its ian7 timing, the remaining undated kinds (enemies and items via their event or stage, IS via `roguelike_topic_table`, game-text intros by story id), and the step 1 items listed above.

## 9. Reducing CPU and memory strain (2026-10-07, static reading; measure before changing)

The VPS is already near its limits before Trevor (88.5% of 3 CPUs, 89.7% of 11 GiB), so the integration must add almost nothing there, and some existing load should come off first. Findings from reading the code (no production measurement yet), in order of impact against effort:

1. **Measure first** (30 min, no risk): `pidstat -u -r 60 10`, `pm2 describe myrtle-frontend` for restart reasons, `pm2 logs --err`, and one backend boot with `STEP_RSS=1`. The steady 88% CPU is not explained by request work or by the background jobs, which are cheap (operator_ownership every minute at 62.5 ms of database time, trending every 15 min, regrade daily), so it needs attributing before anything is changed.
2. **Stop compiling on the VPS** (medium effort, high impact): `vps-update.sh` builds the backend, the downloader and the unpacker (about 2M lines of generated FlatBuffer code) with cargo, and runs `vite build` with a 6 GB heap, on the 3-vCPU host. `assets-ci` already builds binaries; deploy the built backend, unpacker, downloader and `frontend/.output` instead (built on the VPS's distro image to avoid a glibc mismatch). This removes deploy spikes of several GB of RAM and all three cores for minutes.
3. **Load fewer game servers** (low effort, about 200 MB per server): the backend's ~850 to 900 MB is mostly one resident GameData per server in `SERVERS` (about 211 MB measured for EN alone on 2026-09-04, about 600 MB for EN plus CN), and the four watchers suggest all four are loaded. Dropping JP and KR, or loading them on first request with an idle unload, saves about 400 MB, at the cost of a slow first request and care in jobs that walk every server.
4. **One asset watcher instead of four** (medium effort, about 200 MB plus safety): replace four node processes (63 to 78 MB each) with one scheduler that queues `{server, port}` updates and holds a real cross-server lock; the ecosystem file records DMA timeouts and RCU stalls when extracts overlapped, and a manual force_update can still overlap today.
5. **Move extraction off the VPS** (high effort, large CPU win): run the unpacker on the Mac or in CI and sync outputs (JP and KR are about 1.1 GB each; CN and EN are 33 to 39 GB, so heavier to sync but cheaper than decoding on 3 cores), gating the swap on the existing truncated-output check. This also fits the Trevor worker, which needs the same game data.
6. **Reload without a second full copy** (medium effort, about 200 MB of peak): the 6-hourly gacha and event-shop jobs can trigger a full server reload, which holds two GameData copies until the swap; patch only the pool tables instead.
7. **Postgres and Redis settings** (low effort, small): `shared_buffers=512MB`, `work_mem=8MB`, a pool of 20 instead of 40 on 3 cores, Redis `maxmemory 128mb` with `allkeys-lru`. The large tables (user_medals 274 MB, user_operator_skills 184 MB locally) are proportional to their rows, not drift.
8. **SSR micro-cache** (medium effort, CPU): cache anonymous HTML for 30 to 60 s on the busiest static routes, keyed on the absence of a session cookie so no personal markup leaks. The SSR memory leak itself is already fixed (server-side gcTime clamp), and the frontend's 30 restarts are most likely deploys, since RSS (289 to 416 MB) is far under its 1,500 MB restart limit.

Already in good shape: jemalloc, a thin-LTO release profile, Redis-backed caches, capped sprite rendering.

**The Trevor worker (the Mac, 24 GB)**: this week's 12 to 17 GB of swap came from the 12B model (7.0 GB) plus an f16 KV cache sized for two slots, the 8 GB default `-cram` where it was not set, the vision projector, the in-process embedder and reranker, and Qwen 9B (5.7 GB) loading while Gemma was still tearing down. For serving: `-np 1` (one slot is enough at 22 to 28 s per answer), a quantized KV cache (`-ctk q8_0 -ctv q8_0`, about half the KV memory; measure the answer effect with the usual 0-loss rule before adopting), `-cram 128`, the vision projector only for caption jobs, an idle unload after N minutes (a cold start costs a few seconds), and evaluation jobs (Qwen judge) scheduled separately from serving, never overlapping. Keep QAT Q4_0 and mmap (no `--mlock`). Replacing the Qwen judge with Gemma would save a model swap but bias the gate toward Gemma's own answers, so it is not recommended.

## 10. Decisions (Ian, 2026-10-07) and the plan for running on the VPS

- **Free questions:** signed-in users only (30 an hour); the no-model panels are open to everyone.
- **Spoiler horizon default:** the user's furthest-read story (from `/user/story-progress`), with a toggle to lift it; signed-out readers of panels get the current story.
- **Servers:** EN now, CN later. The corpus version, cache keys, panel JSON and worker jobs carry a `server` field from day one, so a CN corpus is a second version stream, not a redesign.
- **The model will eventually run on the VPS.** The design keeps that open by construction: `trevor serve` is location-agnostic, and a worker on the VPS is just another poller on localhost; nothing in the backend or frontend changes. What changes is capacity, and today's VPS cannot carry it (3 vCPUs at 88.5%, about 1.1 GiB free; a 12B model needs about 8 GB and on 3 shared cores would take minutes per answer, since an answer prompt is about 8,000 tokens and CPU prompt processing runs at tens of tokens per second). Planning for it, in order:
  1. Free headroom first with section 9 (stop compiling on the VPS, load fewer game servers, one watcher, no double reload copies): roughly 0.8 to 1 GB and the deploy-time CPU spikes back.
  2. A measured CPU profile for Trevor: a smaller instruct model for the router and question form, at most one generation call per question where the route allows, shorter contexts and a quantized KV cache, each change evaluated with the same 0-loss rule against the GPU profile so the quality cost is a number.
  3. Precompute as much as possible (panels, the answer bank per corpus version, cached answers) so live generation is the exception.
  4. Size the host to the measured CPU profile (cores and RAM for one resident model plus today's services), or keep generation on a GPU host if the CPU profile's losses are too large.

## 11. Former open decisions

- Where the worker lives long term: the laptop with an offline mode, or an always-on box.
- Whether anonymous users may ask free questions, and the limits.
- Spoiler horizon default: the current story (safest) or the furthest-read story (more helpful).
- Which server's text Trevor answers from first (EN only today; the site also serves CN, JP and KR).
