# Trevor contract v1.1: scale, fail-safes, pre-generated answers, the stories tab (2026-10-17)

This adds to `trevor-contract-v1.md` and binds the same way: backend, worker, scripts and frontend implement exactly this. JSON is camelCase.

## 0. Ian's goals (2026-10-17)

1. Every question gets a real Trevor answer.
2. Pre-generated answers regenerate when a game update lands.
3. Questions are answered concurrently at the scale of 10,000+ users.
4. When the model uses too much power, Trevor fails safe quietly and pings Ian on Discord.
5. The stories page gets an "Ask Trevor" tab for any lore question.

## 1. Capacity and the scaling model

Model throughput at $0 is fixed:
- **Cloudflare (`cf`):** about 77 full answers a day.
- **`cpu`, one VPS core with E2B at the `mid` profile (section 4):** roughly 1,000 to 1,900 a day.
- **Cache and passages:** unlimited.

Concurrency is therefore made of four parts.

**a. Answer once.** The cache key is (version, server, normQ, eraPos).
- `normQ` = lowercase, Unicode NFKC, punctuation folded, whitespace collapsed, a trailing "?" dropped. It replaces the exact-text qkey.
- Twins (same normQ, same era) join one open job, as built.
- 77% of local users share one era bucket (measured 2026-10-17), so a popular question costs one model call per era per version.

**b. Admission control** (backend, per rung):
- `TREVOR_CPU_QUEUE_MAX` (default 6) jobs queued or running per online cpu worker.
- `TREVOR_CF_QUEUE_MAX` (default 4).
- `TREVOR_GLOBAL_QUEUE_MAX` (default 200) jobs open in all.

When no model rung can take a new job, the ask gets an immediate `passages` answer, labelled "Quick look". If the global cap is not reached, the ask is ALSO queued as a `deferred` job (`trevor_jobs.deferred = true`): workers take deferred jobs only when no normal job is waiting. The full answer then shows in the reader's "My questions" (section 5) when ready.

**Backend clarification (2026-10-17): how the instant passages answer is made.** The backend cannot retrieve, so an overflowed ask gets a quick `passages` job (normal, not deferred, `rung: passages`). Every worker claims quick passages jobs before any model job (they cost no model and were promised at once), and a guard-tripped worker still serves them. The ask's response carries the quick job (`jobId`, `streamUrl`, `rung: passages`), the deferred full job as `deferredJobId`, and, when an earlier era cached the same normQ, that answer as `quickLook` (spoiler-safe because its era is lower), so a reader has something even when no worker is online. Admission reads the rung depths and inserts under one Postgres advisory lock: without it, a 2,000-ask burst over 50 keys put all 50 jobs on a cpu rung capped at 24 (measured). A user holds at most 3 open deferred jobs; past that the overflowed ask gets the passages answer only. The per-user cap of 3 open counts normal jobs only. Several workers: the rungs offered are the OR over online workers (the latest heartbeat's while none is online), and a worker claims only jobs it can serve at their rung unless no other online worker can. Deferred jobs expire after `TREVOR_DEFERRED_TTL_SECS` (default 21,600). **This (`deferredJobId` + optional `quickLook`) is the one deferred response shape** (decided 2026-10-17); the dev preview mock's older inline answer with a `deferred` flag is read by the frontend for compatibility only and should be moved to this shape.

**c. Fair claiming.** `worker/next` claims the queued job whose user has the fewest jobs running, then the oldest (ORDER BY running-per-user, queued_at), so one heavy user cannot starve others. The per-user caps stay: 3 open, 30 an hour.

**d. Scale-out.** Any number of worker processes may pull from the queue (SKIP LOCKED, as built), each with its own `workerId`. Adding a VPS core or another machine adds throughput with no code change. `/trevor/status` reports workers online per rung.

Streaming at scale: SSE stays (15 s keepalive). Polling is the fallback. The frontend opens at most one SSE connection per turn, at most 3 per user.

## 2. Fail-safes

### 2.1 Worker power guard (`ask --worker`, every rung that runs a local model)

The guard samples every 5 s:

| platform | signals |
|---|---|
| Linux | PSI cpu and memory `some avg10` (/proc/pressure), MemAvailable, the llama-server's RSS, load average per CPU, CPU steal |
| macOS (dev) | `kern.memorystatus_vm_pressure_level`, free memory, `pmset -g therm` warnings |

Trip thresholds (env, these defaults), any one held for 3 consecutive samples:

| env | default | condition |
|---|---|---|
| `TREVOR_GUARD_PSI_CPU` | 60 | PSI cpu above it |
| `TREVOR_GUARD_PSI_MEM` | 20 | PSI memory above it |
| `TREVOR_GUARD_MEM_MB` | 800 | MemAvailable below it |
| `TREVOR_GUARD_RSS_MB` | 4500 | llama-server RSS above it |
| `TREVOR_GUARD_LOAD` | 1.5 | load per CPU above it |

On macOS, pressure level 4 or a thermal warning also trips it.

**On a trip:**
- The worker stops claiming model jobs and reports `rungs.cpu = false` and `guard: {state: "tripped", reason, since}` in its heartbeat.
- The job in flight is answered with the `passages` rung (never an error page).
- If memory tripped, the worker stops its llama-server.

**Recovery:** after `TREVOR_GUARD_COOLDOWN_SECS` (default 300) with all signals below 80% of their thresholds, the guard resumes (`state: "ok"`).

`TREVOR_GUARD=0` turns it off. Heartbeat gains `guard: {state: "ok"|"tripped", reason: string|null, since: unixSecs|null}`.

### 2.2 Backend alerts (Discord, the existing `DISCORD_ALERT_WEBHOOK` of `core::alerts`)

Each alert kind fires at most once per `TREVOR_ALERT_COOLDOWN_SECS` (default 1800) and sends one "recovered" message when it clears. Kinds:

| kind | fires when |
|---|---|
| `guard_tripped` | a worker's heartbeat reports `guard.state = tripped` (include reason) |
| `worker_offline` | no worker online for 3 minutes while jobs are queued, or 15 minutes in any case |
| `backlog` | the oldest queued non-deferred job is older than 5 minutes |
| `failure_rate` | over 25% of the last 40 finished jobs failed or fell back to passages because a model failed |
| `cf_budget` | the day's Neurons are spent before 18:00 UTC |
| `update_failed` | posted by the VPS update script itself, with the same webhook and message style |

Alerts never block a request; a failed webhook is logged and swallowed (as `core::alerts` does).

### 2.3 Master switch

`TREVOR_MODE` = `on` (default) | `passages` | `off`.
- `passages`: every ask is answered at once from passages, and nothing is queued.
- `off`: ask returns 503 "Trevor is resting", while panels and cached answers still serve.

Readers never see a stack trace or a raw backend message.

**Backend clarification (2026-10-17).** `passages` still needs a worker to retrieve: each new question gets a `passages` job (claimed first), never a model or deferred job, and a model job queued before the switch runs as `passages` when claimed. `off` serves a cached answer from ask (200) and otherwise answers 503 "Trevor is resting"; workers claim nothing while it is off.

## 3. Pre-generated answers that follow game updates

**a. Panel suggestions and the bank.** The daily VPS update regenerates only what changed:
- bank questions for new or changed chunks (sources added by the update: a new story, new or revised operator records, modules, outfits);
- their answers;
- the panels.

Answers are made in the Cloudflare window (leftover quota), else by E2B overnight, using the `mid` profile. An answer is kept only when it does not decline and cites something within its gate.

**b. Cache carry-over on publish.** When a new version is activated, a cached answer of the previous version is copied to the new version when its question's evidence did not change. Every cited chunk id must exist in the new corpus with the same text sha (`trevor_answers.cited_shas`, stored at answer time), and the era position must still exist. No model call is involved. Dropped answers are re-made lazily on the next ask.

**Backend clarification (2026-10-17): where the chunk shas come from.** The publish sends `chunkShas: {chunkId: sha}` for EVERY chunk of the new version, in any of its calls (they accumulate, like `panels`); the backend stores them in `trevor_chunk_shas`. The worker result carries `citations[].sha` (the cited chunk's text sha; the same string the publish sends). An answer is stored with `cited_shas = {chunkId: sha}` only when every citation carries a non-empty sha; otherwise (or with no citations) it is never carried. On the activating call, before the active flag moves, each cached answer of the previous active version is copied when every `cited_shas` entry is in the new map with the same sha and its `era_pos` is 0, the internal no-limit value, or a `position` of the new manifest's `eras` (an era that moved is not carried). The copy keeps its hits and records `carried_from`; the publish response reports `carried` (and `chunkShas`, the count stored by the call). A version published without `chunkShas` carries nothing over. Chosen over sending a changed-chunk list because the full map makes the comparison against the new corpus itself, whatever version the publisher diffed against; its cost is one id and sha per chunk per publish. `TREVOR_CARRY_OVER=0` turns it off.

**c. Warm-up.** After a publish, the update script asks the backend for the top `TREVOR_WARM_TOP` (default 100) questions by hits in the last 30 days that were NOT carried over (`GET /api/trevor/worker/popular?version=&limit=`, worker key). It re-answers them as `deferred` internal jobs (no user) through the normal queue, so they are cached before readers ask.

**Warm-up shapes (2026-10-17, for the backend to implement; the script side is `scripts/bank_regen.py warm`).** Both routes take the worker key (contract v1 section 3), and both are JSON in camelCase.
- `GET /api/trevor/worker/popular?version=V&limit=N`, with N from 1 to 500 and a default of 100. It answers 200 `{version: V, items: [{question, normQ, eraPos, hits}]}`, most hits first.
  - The items are the (normQ, eraPos) pairs asked in the last 30 days that have no cached answer in version V, whether carried over or made since. `hits` counts asks, twins included; `question` is the latest asked text.
  - It answers 404 when V is unknown.
- `POST /api/trevor/worker/warm` takes `{version: V, items: [{question, eraPos}]}`, with at most 500 items; a body over 64 KB gets a 413.
  - For each item the backend enqueues one `deferred` internal job: user null, `horizonPos` = `eraPos`, and the rung picked by the normal admission rules when a worker claims it.
  - It enqueues nothing for an item whose (normQ, eraPos) already has a cached answer in V, and an item with an open job joins it.
  - It answers 200 `{queued, cached, joined}` (counts), 409 when V is no longer the active version, and 503 when `TREVOR_MODE` is not `on`.
  - Internal jobs count against no user quota and do not raise the `backlog` alert.

The script treats a 409 as done, and any other failure as an `update_failed` alert that leaves the published version live. `TREVOR_WARM_TOP=0` skips the warm-up.

## 4. Profiles

- **`mid`** = the eval winner on E2B: keyword router, k 8, a 3,600-token passage budget, 250 output tokens, one model call.
  - Measured: gold correct 57 of 131 against 42 for slim; fits a 4,096 slot.
  - `cpu` uses `mid` by default.
  - `slim` stays as built; `full` is for `cf`.
- **`mid6k`** (6,000-token budget, needs a 12,288-token slot, gold 73) is an opt-in profile for a cpu worker started with that slot size.

## 5. New public routes

- `GET /api/trevor/me/jobs?limit=20&before=`: signed-in. The reader's recent questions: jobId, question, status, rung, source, createdAt, finishedAt, the answer when done. Feeds "My questions".
- `GET /api/trevor/popular?limit=8`: signed-in or anonymous. The most-asked cached questions with answers inside the asker's era (anonymous: era 0, nothing released, so an empty list). Used as suggestions in the stories tab. Only questions asked by at least 2 distinct users are listed.
- `/api/trevor/status` adds `workers: {cf: n, cpu: n}`, `guard: "ok"|"tripped"`, `mode`.

**Backend clarification (2026-10-17): shapes.** Every ask that passes validation is recorded (`trevor_asks`), so a twin joiner and a cache hit appear in "My questions" too.
- `me/jobs` answers an array, newest first, of `{jobId: uuid|null, question, status, rung, source: "live"|"passages"|null, createdAt, finishedAt, answer: TrevorAnswer|null}`. `jobId` is the deferred job of an overflowed ask, else the ask's job, and null for a cache hit (`rung: "cache"`, `status: "done"`). `limit` is 1 to 50; `before` is an RFC 3339 instant (pass the last item's `createdAt`).
- `popular` answers `{spoilerSafeUpTo, items}`. `spoilerSafeUpTo` is the era the list is drawn from, `{eraId, name, position}` as in the ask response, or null signed out and before the first era; the stories tab labels the list with it (corrected 2026-10-17: a bare array left the tab no era to show). `items` is an array of `{question, askers, answer: TrevorAnswer}`: per normQ, the cached answer of the latest era at or before the reader's, listed when at least 2 distinct accounts asked it (in any version, so a publish does not empty the list), most askers then hits first. `limit` is 1 to 50 (default 8); `Cache-Control: private, max-age=60`.

## 6. The stories page "Ask Trevor" tab

- **Where:** a new library tab `ask` (`/stories?tab=ask`) beside Browse.
- **What it shows:** the same chat as the reader (AskChat), with no `horizonStory`, so the horizon is the reader's profile only. The spoiler-safe label names their era. Questions can be about anything in the lore.
- **Empty state:** popular questions in their era (section 5) as chips, then "My questions" history.
- **Signed out:** the tab explains that answers follow your reading progress and asks to sign in. Prepared answers need no account.
- **Flag:** behind `VITE_TREVOR=1` like the reader panel.

## 7. The dev preview uses the real Trevor

`bun run dev:trevor` runs the real `ask --worker` against the preview's mock backend, which implements the worker protocol (contract v1 section 3). The local Gemma 12B (or E2B) answers through the real pipeline. The canned answers remain only as a labelled fallback when the binary or the model is missing.
