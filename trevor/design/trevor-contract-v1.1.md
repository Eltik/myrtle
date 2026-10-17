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

## 3. Pre-generated answers that follow game updates

**a. Panel suggestions and the bank.** The daily VPS update regenerates only what changed:
- bank questions for new or changed chunks (sources added by the update: a new story, new or revised operator records, modules, outfits);
- their answers;
- the panels.

Answers are made in the Cloudflare window (leftover quota), else by E2B overnight, using the `mid` profile. An answer is kept only when it does not decline and cites something within its gate.

**b. Cache carry-over on publish.** When a new version is activated, a cached answer of the previous version is copied to the new version when its question's evidence did not change. Every cited chunk id must exist in the new corpus with the same text sha (`trevor_answers.cited_shas`, stored at answer time), and the era position must still exist. No model call is involved. Dropped answers are re-made lazily on the next ask.

**c. Warm-up.** After a publish, the update script asks the backend for the top `TREVOR_WARM_TOP` (default 100) questions by hits in the last 30 days that were NOT carried over (`GET /api/trevor/worker/popular?version=&limit=`, worker key). It re-answers them as `deferred` internal jobs (no user) through the normal queue, so they are cached before readers ask.

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

## 6. The stories page "Ask Trevor" tab

- **Where:** a new library tab `ask` (`/stories?tab=ask`) beside Browse.
- **What it shows:** the same chat as the reader (AskChat), with no `horizonStory`, so the horizon is the reader's profile only. The spoiler-safe label names their era. Questions can be about anything in the lore.
- **Empty state:** popular questions in their era (section 5) as chips, then "My questions" history.
- **Signed out:** the tab explains that answers follow your reading progress and asks to sign in. Prepared answers need no account.
- **Flag:** behind `VITE_TREVOR=1` like the reader panel.

## 7. The dev preview uses the real Trevor

`bun run dev:trevor` runs the real `ask --worker` against the preview's mock backend, which implements the worker protocol (contract v1 section 3). The local Gemma 12B (or E2B) answers through the real pipeline. The canned answers remain only as a labelled fallback when the binary or the model is missing.
