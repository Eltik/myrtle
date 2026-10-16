# Trevor in myrtle.moe: the build plan (v3, 2026-10-16)

This plan is the build order for the $0 design. The design itself is in `~/overnight-2026-10-15/reports/trevor-zero-cost.md`: cache, then Cloudflare Workers AI Gemma 4 26B, then Gemma 4 E2B on one VPS core, then a passage-only answer, with panels as static rows. The original design is `trevor-integration.md`.

Every row below was checked against the code on 2026-10-16. Where a plan and the code disagree, the code wins, and the correction is noted.

## What exists today

- **Branches.** All Trevor code is on `trevor`; `main` has none. Main's migrations end at v038, so `v035_trevor.sql` becomes v039 in the merge of main into trevor, on branch `trevor-main-merge`.
- **Backend: in place.** Routes, the job queue, worker pull/result/heartbeat, chunked panel publish with an atomic activate, the per-user ask quota, and the answer cache.
- **Backend: missing.** Five of the first plan's fixes are missing: C1 (queue lifetime and ETA), C2 (worker liveness), C3 (slim results), C4 (wake the worker on enqueue instead of polling), C5 (scoped worker key). So are C11 (era buckets), the request-body limit and gzip on publish (the missing half of C12), `/trevor/status`, SSE streaming, Neuron accounting and Turnstile.
- **Worker: in place.** `ask --serve` and `ask --worker` run one job at a time. The spoiler horizon works, but has an ordering tie that affects 8 stories (`horizon.rs:152`).
- **Worker: missing.** The model client speaks llama.cpp only, and nothing streams. `--serve` always loads the dense index and embedder.
- **Worker bug.** It sends its heartbeat only between jobs (`serve.rs:440`), so any answer longer than 75 s (every CPU-rung answer) makes the worker look offline while it works.
- **Panels.** A model-free prototype (`prototypes/trevor-panels/build_panels.py`) builds all 1,862 stories in 8.8 s. It isn't in the repo yet.
- **Frontend.** Nothing exists, and the visual design is paused for the friend's mockups.

## Corrections to the earlier plans

1. **The worker calls the model, not the backend.** Retrieval and prompt building exist only in the Trevor crate. So the Cloudflare client goes in the worker. The backend owns the daily Neuron budget and picks the rung for each job (the `flags` field already exists on the job). The zero-cost plan said the backend calls Cloudflare.
2. **No embedding on the VPS, as specified.** The corpus vectors are Mac INT8, and INT8 on x86 drifts to cosine 0.94. Embedding only the new chunks on the VPS would mix two runtimes. The choice is BM25-only serving (recall@10 0.681), or one full fp32 re-embed on the VPS for hybrid (0.747).
3. **The daily update is bigger than "summaries".** `update.sh` runs about 20 model stages plus a Qwen judge, and needs the backend on :3060. The VPS needs its own update script: build-corpus, BM25, panels, and summaries, with the other stages frozen until a manual rebuild.
4. **The slim path needs no JSON mode.** Only 3 serving calls are grammar-bound (router, form, flags), and `--router keywords` skips all of them. So a Cloudflare client for the slim profile needs plain chat only. That sidesteps the undocumented json_schema support for Gemma 4.
5. **The FAQ bank isn't a typed-question rung.** It matches 0.5% of real questions. It earns its keep as clickable suggestions in the panels, which already carry them.
6. **Queue lifetime of about 10 minutes, not 7 days** (the worker is always on). With CPU answers at 1 to 3 minutes, cap the queue per rung and fall to the passage answer rather than letting jobs expire.

## Build order

Each phase is shippable on its own, behind a switch that turns it off.

**Phase A: land the backend on main (Mac only, about 2 days).**
1. Merge main into trevor (done, on `trevor-main-merge`, pending review). Migration v039 gains the columns the later phases need while it is still unapplied: `trevor_answers.rung/era/source`, `trevor_jobs.rung`, a `trevor_workers` row, and a daily Neuron ledger.
2. C5: a scoped `TREVOR_WORKER_KEY`, accepted only on `/trevor/worker/*` and `/trevor/versions`.
3. C4: a `tokio::sync::Notify` wakes the long-poll on enqueue, and the stale sweep moves to a 60 s background tick.
4. C1/C2: the queue lifetime becomes an environment variable, and the ETA is computed per rung. A persistent `last_seen`, plus which rungs are live, feeds `GET /trevor/status`.
5. C3: the worker sends only the cited passages, trimmed. The backend caps results at 64 KB and caches answers in Postgres only, not Redis.
6. C12: a 4 MiB body limit and gzip on `/trevor/versions`.
7. A database-backed test on a throwaway local database: SKIP LOCKED with two claimers, the stale requeue, publish and activate. This needs Ian's OK to create the database.

**Phase B: the worker for the VPS (Mac only, about 3 days).**
1. Move the heartbeat into its own task, and give the result POST a retry and a spool.
2. Horizon tie fix: one shared story rank, used by `horizon.rs` and by the backend's `furthest_read`.
3. A `--profile slim` setting: keyword router, k 4 to 5, about 2k context tokens, about 250 output tokens, no typed fallback. It can be picked per job.
4. A BM25-only serving mode (`Runtime::load` already supports it), unless Ian chooses the fp32 re-embed.
5. An OpenAI-compatible client behind an enum next to the llama.cpp one: a model-id list with failover, thinking off, and token counts returned for the Neuron ledger. Test it against Homebrew `llama-server`'s `/v1/chat/completions`.
6. Rung 4: when the model fails or the budget is spent, answer with the top cited passages, labelled as such.
7. The three-way eval from the zero-cost plan (gold v2 131 and the 120 real questions):
   - the 26B GGUF on the Mac as a proxy for Cloudflare, full pipeline;
   - the same model with the slim profile;
   - E2B with the slim profile.

   This needs an E2B download, about 3 GB. The result decides whether E2B answers are labelled "quick answer".

**Phase C: content and publish (about 2 days).**
1. Move `panels.py` into `trevor/scripts`, using the shared story rank.
2. A publish client: gzip chunks under 4 MiB, then an activate call that resends the manifest and the gate report.
3. The VPS update script and its gates: schema, 0 passages past the horizon, BM25 recall@10 at least the previous minus 0.02, and a 10-question canary. Summaries run in the hour before 00:00 UTC.

**Phase D: live (needs Ian's accounts and the VPS).**
1. Cloudflare token and Neuron accounting with the ~9,000 daily stop. Turnstile verification on the ask route.
2. SSE streaming: worker deltas, a fan-out per job in the backend, keep-alive comments every 30 s, and polling as the fallback.
3. systemd units:
   - `llama-server`: E2B, `-t 1`, `AllowedCPUs=2 CPUWeight=10 Nice=19 MemoryMax=4G`, no CPUQuota;
   - the worker;
   - the daily update timer.
4. An off-box `pg_dump` and a tested restore before the first deploy that carries v039.

**Phase E: the reader (after the mockups).**
- `lib/api/trevor.ts`;
- a TrevorPanel in the story reader toolbar;
- the summary and primer in ChapterModal;
- the ask box with Turnstile, "spoiler-safe up to <era>", streaming, and citations that jump to the line.

## Decisions only Ian can make

1. The Cloudflare Workers AI token (free plan, no card), and the Turnstile keys with a fail-open or fail-closed choice.
2. VPS retrieval: BM25-only, or a one-time fp32 re-embed for hybrid.
3. Which model stages the daily VPS update runs (recommended: summaries only).
4. The era rule (the prototype's 22 eras) and the "spoiler-safe up to" wording.
5. Whether E2B answers show as answers or as "quick answer" (after the eval).
6. When the worker process is down: panels and cache only, or BM25 inside the backend (about +15 to 30 MB).
7. The asks for the host owner (Start at boot, VirtIO SCSI, guest agent, +1 to 2 vCPU), and permission to run `llama-bench -t 1,2` on the box.
8. A throwaway local database for the Phase A test.

## Previewing the reader panel without a Trevor backend

To preview it, run `cd frontend && bun run dev:trevor`. The command starts `scripts/trevor-preview.mjs`, which runs a mock backend on 127.0.0.1:3999 and `vite dev` on 127.0.0.1:3005 with `VITE_TREVOR=1` pointed at that mock. Ctrl+C stops both.

Open http://127.0.0.1:3999/demo-login to sign in as the demo account. It sets the cookie `site_token=trevor-demo` and lands on Lingering Echoes LE-5 in the reader; `/demo-logout` clears it.

The mock serves panels from `trevor/artifacts/panels` and fakes `/trevor/status` and `/trevor/ask`. It streams a canned answer over SSE: 2.5 s in the queue, then about 60 deltas 120 ms apart. Every other read is proxied as a GET to the backend on :3060, and any other method is refused, so the preview writes nothing.

The answer kind can be switched while the preview runs:
- `/demo-mode/stream`, the default;
- `/demo-mode/passages`, a cached passages-only answer;
- `/demo-mode/busy`, a 503;
- `/demo-mode/offline`, a question that stays queued with no worker.

Both servers bind 127.0.0.1 rather than localhost, so the demo cookie never replaces a real session on the localhost dev server at :3000.
