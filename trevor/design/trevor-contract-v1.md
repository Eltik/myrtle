# Trevor contract v1: backend, worker, publish (2026-10-16)

This is the binding interface for the build in `trevor-integration-plan-v3.md`. The backend (`backend/src/app/services/trevor/`), the worker (`trevor/src/bin/ask/serve.rs`, `ask --worker`) and the publish client (`trevor/scripts/`) all implement exactly this. Change it here first. JSON is camelCase. It supersedes `trevor-integration.md` section 8 where they differ.

## 1. One spoiler scale: `gatePos`

- Every story has one integer rank, its **gatePos**, which is its 1-based position in Trevor's EN release order.
  - The order is the panel builder's: reading-guide position, then storySort, then story-table file order, which breaks every tie.
  - Trevor's `horizon.rs`, the panels, `trevor_panels.gate_pos` and `trevor_answers.gate_pos` all use this one scale. No component computes its own order.
- **Eras** are the panel builder's 22 boundaries. An era is `{id, name, position}`, where `position` is a gatePos. Each version manifest carries `storyOrder` (story ids in gatePos order) and `eras`.
- **Horizon from the user's profile (Ian, 2026-10-16).** The backend owns it.
  - `profilePos` = the maximum gatePos over the stories the signed-in user has read (`story_progress`), using the active version's `trevor_panels.gate_pos`. A reader with no progress gets gatePos 0, meaning nothing has been released yet.
  - A request may name a `horizonStory` (the story being read now). Then `pos = min(gatePos(horizonStory), profilePos)`. A request can lower the horizon below the profile, never raise it.
  - The job's horizon is the era boundary at or before `pos`: `eraPos = max(era.position <= pos)`, or 0. Answers are cached by (version, server, qkey, eraPos). The response says `spoilerSafeUpTo: {eraId, name, position}`, or null for eraPos 0.
- **The worker receives `horizonPos` (an integer), not a story id.** It keeps only chunks whose gatePos (or dated-source equivalent, as today) is ≤ `horizonPos`. `horizonPos` null means no limit, which happens only for an internal job.

## 2. Rungs

`rung` takes one of these values:

| rung | what | when the backend picks it |
|---|---|---|
| `cache` | Postgres `trevor_answers` hit | always tried first |
| `cf` | Cloudflare Workers AI, full profile | the worker's last heartbeat reports `cf: true`, and today's Neuron ledger is below `TREVOR_CF_DAILY_NEURONS` (default 9000) minus `TREVOR_CF_SUMMARY_RESERVE` (default 1000) |
| `cpu` | local llama-server (E2B on the VPS, any GGUF in dev), slim profile | the heartbeat reports `cpu: true`, and fewer than `TREVOR_CPU_QUEUE_MAX` (default 6) jobs are queued or running on this rung |
| `passages` | no model: top cited passages | otherwise, and on any model failure |

`profile` is `full` or `slim`. `cf` uses `full`, `cpu` uses `slim`, and `passages` ignores it.
- The slim profile: keyword router, k 5, 2,000 context tokens, 250 output tokens, no typed fallback, no answer extension, no second-look retries.
- The full profile is today's default pipeline.

**Neurons.** For a `cf` result the backend adds `ceil(tokensIn × 9091 / 1e6 + tokensOut × 27273 / 1e6)` to the ledger row for the current UTC day.

## 3. Worker API

The worker authenticates with header `x-trevor-worker-key: <TREVOR_WORKER_KEY>`, compared in constant time. An empty configured key disables the worker routes with a 503. The global service key is NOT accepted on these routes, and the worker key is accepted nowhere else.

**`POST /api/trevor/worker/next`** takes `{workerId}`.
- It long-polls up to 20 s, woken by a `tokio::sync::Notify` when a job is enqueued.
- 204 means no job.
- 200 returns:
  ```
  {jobId, question, server: "en", horizonPos: int|null, rung: "cf"|"cpu"|"passages", profile: "full"|"slim", flags: [string]}
  ```

**`POST /api/trevor/worker/heartbeat`** takes:
```
{workerId, rungs: {cf: bool, cpu: bool}, corpusVersion, busyJobId: string|null}
```
- The worker sends it every 30 s from its own task, including mid-job.
- The backend keeps `trevor_workers(worker_id PK, last_seen, rungs jsonb, corpus_version, busy_job_id)`.
- A worker counts as online when `last_seen` is within 75 s.

**`POST /api/trevor/worker/jobs/{jobId}/delta`** takes `{workerId, seq, text}`.
- It carries streamed answer text, appended in `seq` order.
- The backend fans each delta out to SSE subscribers through an in-process `tokio::sync::broadcast` per job, and does not persist deltas.
- 404 when the job is unknown or not running.

**`POST /api/trevor/worker/result`** takes:
```
{jobId, workerId, ok, error?, result?}
```
`result` is:
```
{answer, rung, profile, source: "live"|"passages", citations: [{n, storyId, chunkId, lineStart, lineEnd, text}], route, form, corpusVersion, tokensIn, tokensOut, timings: {routeMs, answerMs, totalMs}}
```
- `citations` holds only the CITED passages; `text` is the cited passage trimmed to at most 600 chars at a line break.
- The full `passages` list is gone.
- The backend rejects a result over 64 KB with a 413.
- The worker retries a failed POST 3 times with backoff (1, 4, 16 s), then spools it to `TREVOR_SPOOL_DIR` (default `artifacts/spool/`) and re-sends on start.

**`POST /api/trevor/versions`** (publish): same auth.
- Bodies may be gzip (`content-encoding: gzip`), with a decompressed limit of 8 MiB and a compressed limit of 4 MiB per request.
- Chunked with `activate: false`, then one `activate: true` call that resends `manifest` and `gateReport`.
- The manifest gains `builtFrom` (EN res_version), `storyOrder` and `eras`.

## 4. Public API

**`POST /api/trevor/ask`**: signed-in users only.
- Request: `{question, horizonStory?: storyId, turnstileToken?}`.
- Turnstile is verified when `TURNSTILE_SECRET` is set. Unset means skipped, which is the dev default. When it is set and siteverify is unreachable, the backend fails CLOSED with a 503 "Trevor is busy".
- Quota: 30 an hour per user (as built).
- Response: `{jobId, status, rung, spoilerSafeUpTo, answer?: TrevorAnswer, etaSecs: int|null, streamUrl}`.
- `etaSecs` comes from the rung: the median `finished_at - started_at` of the last 50 jobs on that rung, or a fallback (cf 15, cpu 120, passages 1) when there are fewer than 5.

**`GET /api/trevor/jobs/{id}/stream`** is SSE, owner only. Events:
- `delta` `{seq, text}`
- `done` (the job view)
- `error` `{message}`
- a `: keepalive` comment every 15 s

It closes after `done` or `error`. When the job already finished, it sends `done` at once.

**`GET /api/trevor/jobs/{id}`**: polling fallback, as built.

**`GET /api/trevor/status`**: public, `Cache-Control: public, max-age=15`. Returns:
```
{online, rungs: {cf, cpu}, activeVersion, queueDepth: {cf, cpu}, cfBudget: "ok"|"low"|"spent"}
```
`cfBudget` is "low" under 20% left. It is a bucket, never a number.

**Panels: `GET /api/trevor/panels/{storyId}?v=`.** When `v` matches the active version, the response carries `Cache-Control: public, max-age=86400, immutable`; otherwise `max-age=300`.

## 5. Storage

Migration v039, still unapplied, is amended in place:
- `trevor_answers` gains `rung`, `source`, `era_pos`, `gate_pos`, and keys its cache on era_pos.
- `trevor_jobs` gains `rung`, `profile` and `horizon_pos`.
- New table `trevor_workers`.
- New table `trevor_neurons(day date PK, neurons int)`.
- Answers live in Postgres only: no Redis copies of answers or panels. Redis keeps the quota counters.
- Queue: a queued job expires after `TREVOR_QUEUE_TTL_SECS` (default 600). A job stale while running is requeued once, then failed. The sweep runs on a 60 s background tick, not on every poll.
