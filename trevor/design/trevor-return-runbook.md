# Trevor after a long absence: the return runbook (2026-10-19)

The one page to read on returning to a self-contained Trevor instance (`trevor-self-contained-deploy.md`) after months
or years without attention. It orders the checks and says what each answer means; the operations detail lives in
`trevor-vps-runbook.md` (failure table, gates, publish, rollback) and the install detail in `../deploy/README.md`.
Paths are the bootstrap's (clone at `/var/www/myrtle.moe`, state in `/var/lib/trevor`; section 8). Written from the
files on 2026-10-19; nothing here has run on a real box, and each claim the files do not settle says "unverified".

## 1. The first ten minutes: is it serving?

Six read-only commands, in order:

```bash
curl -s http://127.0.0.1:3060/api/trevor/status; echo
sudo -u trevor env TREVOR_HOME=/var/lib/trevor /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh status
systemctl is-active trevor-llama trevor-worker trevor-update.timer; systemctl list-timers 'trevor-*' --no-pager
systemctl show trevor-update.service -p Result -p ExecMainExitTimestamp
curl -s http://127.0.0.1:3060/api/health | python3 -c 'import json,sys; g=json.load(sys.stdin)["gameData"]; print("stale:", g["stale"], "degraded:", g["degraded"])'
df -h /var/lib/trevor /var/www/myrtle.moe /var/www/myrtle.moe/assets
```

| answer | healthy | otherwise |
|---|---|---|
| status `online` | `true`: a worker heartbeat in the last 75 s (`ONLINE_SECS`, `backend/src/app/services/trevor/mod.rs`) | `journalctl -u trevor-worker -u trevor-llama -n 100 --no-pager`; `curl -s http://127.0.0.1:8081/health` must say `ok` |
| status `rungs` | `cpu: true`, `cf: false` | `cf: true` means a `TREVOR_CF_*` key is set in `/etc/trevor/trevor.env`; comment it out (goal 2 of the design) |
| status `guard` | not `tripped` | section 2, `guard_tripped` |
| `active:` line (`state/active.json`) | `builtFrom` equals the `assets:` EN res_version; `version` equals status `activeVersion` | unequal for more than 1 day: the daily update fails or is held; `ls -t /var/lib/trevor/logs \| head -3`, then read that `run.log` |
| `activatedAt` in the `active:` line | the last publish (or rollback); EN updates come roughly every 2 weeks (design section 4) | older than about 30 days with an unchanged res_version: the asset watcher stopped too (section 3) |
| `is-active` | `active` 3 times | `sudo systemctl reset-failed trevor-worker; sudo systemctl restart trevor-llama trevor-worker` (the worker stops retrying after 5 starts in 600 s, `StartLimitBurst`) |
| timer and `Result` | `trevor-update.timer` LAST under 24 h ago, `Result=success` | `journalctl -u trevor-update -n 100 --no-pager` |
| `stale` | `{}` | section 3 |
| `df` | free above the smaller of 10 GB and 10 percent on each (`DISK_MIN_FREE_GB`, `DISK_MIN_FREE_PCT`) | section 2, `disk_low` |
| the stage runner | `systemctl is-active trevor-stages`; `cat /var/lib/trevor/state/stages/status.json` | section 4; unverified: no `trevor-stages` unit exists in `../deploy/systemd/` on 2026-10-19 |

## 2. Read the alerts

Every alert goes to the Discord webhook `DISCORD_ALERT_WEBHOOK` (in `/etc/trevor/trevor.env` and `backend/.env`).
Two senders: `scripts/trevor-vps-update.sh` posts one `update_failed` per failed run (the step, the exit code, the last
20 log lines, the log directory in the footer), a `disk_low` embed, and the "published" and "rolled back" notices;
the backend posts the 5 kinds of `backend/src/app/services/trevor/alerts.rs`, each at most once per 1,800 s
(`TREVOR_ALERT_COOLDOWN_SECS`) with one "recovered" message when it clears, plus degraded game tables
(`backend/src/core/alerts.rs`). In the order they are likely over a long absence (design section 7 expects the first
within months):

| alert | means | first action |
|---|---|---|
| none, and no "published" notice for weeks | a stale game table. **Nothing posts a Discord message for it**: `.skipped.json` reaches only `/api/health` `gameData.stale`, a backend warn line and one line in the update's `run.log` (`common.py`). Silence is the symptom | section 3 |
| `update_failed`, step `gates` | a gate refused the build (schema, spoiler, recall more than 0.02 under the active version's, canary; runbook section 7); nothing was published, the last version serves, the next 19:30 UTC run retries | read the `run.log` the footer names; no rollback is needed, nothing changed |
| `update_failed`, another step | build-corpus, a builder or the upload failed; the stage keeps the work. A failed activate flips back by itself (runbook section 4, step 8) | the same `run.log`; the same step on 3 or more consecutive days is section 3 |
| `update_failed`, "only the warm-up failed" or "only the prune failed" | the new version is live | none unless it repeats |
| no worker online (`worker_offline`) | no heartbeat for 180 s with jobs queued, or 900 s in any case; answers fall to passages | the `is-active` row of section 1 |
| disk space is low (`disk_low`) | a filesystem of `TREVOR_HOME`, the code or the assets is under the threshold of section 1 | the prune below: a dry run first, then `PRUNE_APPLY=1` |
| power guard tripped (`guard_tripped`) | the worker stopped claiming model jobs and answers from passages; the embed gives each worker's reason (thresholds: contract v1.1 section 2.1) | find the load (`top -o %CPU`); the trip clears itself |
| questions are waiting (`backlog`), answers are failing (`failure_rate`) | the oldest queued job is over 300 s old; over 25 percent of the last 40 finished jobs failed (judged from 10) | usually behind one of the two rows above; `journalctl -u trevor-llama -n 100` (model file, `MemoryMax=4G`) |
| degraded game-data tables | a table failed to deserialize and serves empty, site-wide | the repair path of section 3 |
| `cf_budget` | cannot fire without Cloudflare keys (Neurons accrue only on cf results; design step 10) | a `TREVOR_CF_*` key is set |

```bash
sudo systemd-run --uid=trevor --gid=trevor -p EnvironmentFile=/etc/trevor/trevor.env --wait --pipe --collect \
  /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh prune   # a dry run; --setenv=PRUNE_APPLY=1 deletes
```

The policy (logs 30, eval runs 50, spool 14 days, releases 5, asset manifests 2) is the docstring of
`scripts/retention.py`. Nothing deletes `trevor_asks` rows yet: the SQL is in the design doc's "Step 9 built", a
backend follow-up; unverified how fast the table grows.

## 3. The most likely reason ingestion stopped: an upstream format change

Twice in October 2026 the EN data changed shape (design section 7). Confirm with three reads:

```bash
cat /var/www/myrtle.moe/assets/output/en/gamedata/.skipped.json       # each table kept from an older extract, "since"
pm2 logs myrtle-ws --lines 5000 --nostream | grep 'verifies under no schema'
ls -l --time-style=long-iso /var/www/myrtle.moe/assets/binaries/unpacker /var/www/myrtle.moe/assets/ArkAssets/en/.version
```

A table in `.skipped.json` with an unpacker binary older than its `since` is the expected case: the schemas compiled
into the unpacker predate the patch (`assets/unpacker/src/export/gamedata.rs` skips the table and keeps the old
`.json`). Unverified: the user `pm2` runs the watcher under; run the second line as that user.

**This is the step that needs a person**, about an hour each time (design section 7). The box never repairs itself
and never compiles. The repair path as it exists today:

1. Check whether the repository already fixed it. `.github/workflows/schema-watch.yml` runs every 6 hours, regenerates
   the schemas and merges the fix to main when Assets CI passes; the Release Bundle workflow then builds an unpacker
   for that commit. `bash scripts/fetch-release.sh --check <sha>` (from the clone root) exits 0 when that commit has
   a bundle.
2. If schema-watch failed (`schema-gen` exit 3: the client changed shape; `schema-sync` exit 2: a table lost every
   candidate), run the local path of `assets/schemas/README.md`, "Automation", on a development machine: `schema-gen`
   if the client moved, `schema-sync`, `generate-fbs`, `cargo fmt`, a release build to check, then a commit to main so
   CI builds the bundle. Read that README first: hand `.fbs` patches are refuted there ("Hand patches: none").
3. On the box, install only the new unpacker and downloader, by sha:
   `cd /var/www/myrtle.moe && sudo bash scripts/fetch-release.sh --components assets <sha>`. The watcher re-extracts
   when the binary is newer than `.last_extract` (`assets/ws/region.mjs`, line 290), on its next 30-minute tick.
   Done when `.skipped.json` lists no table and `.version` is newer than `activatedAt`.

## 4. What has been serving meanwhile

- **The cache** answered every question asked before: the bank plus every answered ask, carried across publishes
  while the cited chunks' text shas are unchanged (contract v1.1 section 3b), with no model call.
- **The `cpu` rung** (E2B, `mid`) answered novel questions, about 93 an hour on the 8-thread proxy of design section 5;
  passages when no worker heartbeated.
- **New stories** arrived with every nightly publish until the ingest stopped: passages, BM25, the appearance index,
  dates (`state/history.jsonl` lists every publish). A run with one stale table can still publish: the table's readers
  keep their last output (`trevor/CLAUDE.md`, "Missing game tables"), so the new stories serve without that table's
  content.
- **The model stages** lag by whatever the paced runner (`scripts/stages.py`, design step 5) has not done. Its
  `status.json` (`/var/lib/trevor/state/stages/` in `--publish` mode, else `trevor/artifacts/stages/`) gives the
  state, the current stage and every stage's units left; `python3 scripts/stages.py --plan` recounts them without a
  model. Without the runner (it was being built on 2026-10-19, and the bootstrap leaves `trevor-summaries.timer`
  off), the model outputs are frozen at the snapshot and every later story serves without a summary
  (`recap.thisStory` null), dossier or topic. Pending bank units:
  `cd /var/www/myrtle.moe/trevor && sudo -u trevor python3 scripts/bank_regen.py count | tail -1`.

## 5. Bringing it current

1. **Fix the ingest** (section 3).
2. **Run `daily`**: `sudo systemctl start trevor-update.service; journalctl -fu trevor-update`. Minutes to an hour
   (design section 4). One run absorbs every missed update, because `build-corpus --refresh` refetches every story
   from the current game data. It refuses a corpus with more than 20 percent fewer stories (`TREVOR_INGEST_MAX_DROP`);
   read the log before any `TREVOR_INGEST_ALLOW_SHRINK=1`.
3. **Let the model stages catch up.** One game update is about 400 model calls, 1 to 3 days on the proxy CPU and 2 to
   4 times slower generation on a 3 vCPU slice (design section 5, an upper bound measured on an Apple M5 Pro). At one
   EN update every 2 weeks, 6 months away is about 13 updates: 13 to 39 days on the proxy, longer on the box. The bank
   regenerates `REGEN_MAX_UNITS` (150) units a night and an event is about 200 (runbook section 5a): about 18 nights.
   Serving waits on none of it. The runner pauses on `touch /var/lib/trevor/state/stages/pause` and stops at duty 0
   (`echo 0 > /var/lib/trevor/state/stages/duty`; `scripts/stages.py` docstring). Without the runner,
   `trevor-vps-update.sh summaries` by hand runs summaries and bank answers on a second local E2B server; unverified
   on a box without Cloudflare keys (the script logs a failed Cloudflare call and goes on to the local server).
4. **Check the gates**: every publish passes them first; the report is
   `/var/www/myrtle.moe/trevor/artifacts/panels/gate_report.json` (runbook section 7).
5. **Publish**: `daily` publishes itself after the gates and posts "published <version>"; rerun section 1.

## 6. Rolling back

```bash
sudo systemd-run --uid=trevor --gid=trevor -p EnvironmentFile=/etc/trevor/trevor.env --wait --pipe --collect \
  /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh rollback          # or: ... rollback <version>
sudo systemctl stop trevor-update.timer                                      # until the cause is fixed
```

It re-activates the version before the current one (`state/history.jsonl`) or the one named, points `current` back
at its release while that release is kept (`KEEP_RELEASES`, 3), restarts the worker and resends the stored activate
body; a pruned release restores the panels only (runbook section 9). It touches no binary, model, llama.cpp or asset
file. Use it when a publish made answers or panels worse; never for a stale table. Stop the timer: the rollback writes
the older `builtFrom` into `active.json` (`scripts/publish.py`, `record`), so the next 19:30 UTC run would rebuild
and publish the same content again. `sudo systemctl start trevor-update.timer` once fixed.

## 7. What to update deliberately, and what not to touch

- **Release binaries, by sha only**: `sudo trevor/deploy/bootstrap.sh --only release --sha <sha>` installs the Trevor
  and asset binaries of that main commit. Bundles are kept for the newest 20 commits (`../deploy/README.md`), so after
  a long absence the installed sha (`trevor/target/release/.release-installed`) has likely lost its bundle: record it
  and expect no binary rollback by download.
- **llama.cpp**: pinned at `b11146` in `../deploy/pins.env` (tag, asset and sha256 change together), then
  `sudo trevor/deploy/bootstrap.sh --only llama` and `sudo systemctl restart trevor-llama`; a new build voids the
  section 5 measurements, so rerun `llama-bench` (`../deploy/README.md`, step 4).
- **Models**: `scripts/fetch-model.sh` pins each by repository revision and sha256. A new embedder voids every vector
  (`ask` refuses a query model other than the vectors'); a new GGUF changes every answer. Leave them.
- **Never** `cargo build` on the box (`../deploy/README.md` step 5), never an unpinned upgrade (llama.cpp, models, an
  OS release past Ubuntu 24.04, whose glibc and libstdc++ the binaries need), never `trevor-summaries.timer` with
  Cloudflare keys, never `INCR_ALLOW_SHRINK=1`, `KEEP_LAST=0` or `TREVOR_INGEST_ALLOW_SHRINK=1` as a standing setting.

## 8. Where everything is

| what | where |
|---|---|
| units | `/etc/systemd/system/trevor-{llama,worker,update}.service`, `trevor-update.timer`, `trevor-worker-restart.path`; sources in `../deploy/systemd/` |
| secrets | `/etc/trevor/trevor.env` (mode 600); the same `TREVOR_WORKER_KEY` in `backend/.env` |
| update logs | `/var/lib/trevor/logs/<stamp>-<cmd>/run.log` (30 days); `journalctl -u 'trevor-*'` |
| releases, live link | `/var/lib/trevor/releases/<stamp>-<version>/`; `/var/lib/trevor/current`; `trevor/artifacts` links to it |
| state | `/var/lib/trevor/state/` (`active.json`, `history.jsonl`, `releases.jsonl`, `versions/<v>/activate.json`) |
| stage, spool, snapshot | `/var/lib/trevor/stage/trevor/`, `/var/lib/trevor/spool/`, `/var/lib/trevor/snapshot/` |
| model lock, llama.cpp | `/run/trevor/llm.lock`; `/opt/llama.cpp/b11146`, `/usr/local/bin/llama-server` |
| game data | `assets/ArkAssets/en/.version`, `assets/output/en/gamedata/.skipped.json`, `assets/binaries/unpacker` |
| docs | `trevor-vps-runbook.md`, `trevor-self-contained-deploy.md`, `../deploy/README.md`, `assets/schemas/README.md` |

## The five numbers of a healthy instance

1. `/api/trevor/status`: `online` true, `rungs.cpu` true, `rungs.cf` false.
2. 0 versions behind: `builtFrom` in `active.json` equals the EN res_version (`trevor-vps-update.sh status`).
3. 0 stale tables: `/api/health` `gameData.stale` is `{}`.
4. Free space on all 3 filesystems above the smaller of 10 GB and 10 percent.
5. The last `trevor-update` run under 24 hours old, with `Result=success`.
