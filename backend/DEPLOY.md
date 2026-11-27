# Backend deploy and first cutover

The backend is a cargo workspace with three binaries:

| binary | package | role |
|---|---|---|
| `myrtle-api` | `myrtle-api` (`crates/api`) | HTTP on `server.port` (3060). Runs the database migrations at boot. |
| `myrtle-worker` | `myrtle-worker` (`crates/worker`) | Job queue, schedules, asset-watcher listener. Run exactly one (see below). |
| `myrtle` | `myrtle-cli` (`crates/cli`) | Operator CLI. |

pm2 runs the first two from `backend/ecosystem.config.cjs` as `myrtle-api` and `myrtle-worker`. Both take the config path as their first argument (`config/myrtle.toml`, relative to the working directory, so `cwd` is `backend/`). Secrets in the config are `env:NAME` references, read from the process environment first and then from the first `.env` found walking up from the working directory (`dotenvy`, `crates/app/src/config.rs`). With `cwd` at `backend/` that is `backend/.env`; only if `backend/.env` is missing does the repo-root `.env` get read instead. The two files are never merged.

**Run exactly one worker.** Each worker process loads the game service-account sessions into memory and refreshes or re-logs them on its own, so a second worker (a second pm2 instance, a stray `myrtle-worker` from a shell, or the old `myrtle-backend`) re-mints the same accounts and signs the other out. It would also run every schedule twice.

After the cutover, a normal deploy is `git pull && bash vps-update.sh` as before. This file covers the one-time switch from the old `myrtle-backend` app.

## Before you start

- All commands run as root on the VPS. `B=/var/www/myrtle.moe/backend`.
- `vps-update.sh` is gitignored, so `git pull` does not update it. Copy the repo's current `vps-update.sh`, `vps-start.sh` and `vps-info.sh` to `/var/www/myrtle.moe/` by hand (scp, never rsync `--delete`) before step 2. The old copy builds `--bin backend`, which no longer exists.
- Main must still end at migration `v046_oauth` on the old side. Adoption checks only for that row (see "Rollback").
- The old and new backends must never both run their jobs. Old `myrtle-backend` and new `myrtle-worker` are never online at the same time.

## 0. Back up (old backend still serving)

```sh
cd /var/www/myrtle.moe
mkdir -p /root/myrtle-backend-rollback
cp -p backend/target/release/backend /root/myrtle-backend-rollback/backend
cp -p backend/ecosystem.config.cjs /root/myrtle-backend-rollback/ecosystem.config.cjs
sed -i 's#script: "target/release/backend"#script: "/root/myrtle-backend-rollback/backend"#' \
    /root/myrtle-backend-rollback/ecosystem.config.cjs
cp -a backend/game_sessions backend/device_ids.json /root/myrtle-backend-rollback/ 2>/dev/null || true
```

The pull in step 2 replaces `backend/ecosystem.config.cjs` with the new two-app file, so the old one has to be copied first.

Dump the database. Postgres runs in Docker; find the container, then dump from inside it (UNVERIFIED: container name and database user, take them from `docker ps` and `DATABASE_URL`):

```sh
docker ps --format '{{.Names}}  {{.Image}}'
docker exec <pg-container> pg_dump -U <user> -Fc <db> > /root/myrtle-backend-rollback/pre-rewrite.dump
docker exec <pg-container> psql -U <user> -d <db> -Atc "SELECT name FROM _migrations WHERE name = 'v046_oauth'"
```

The last command must print `v046_oauth`. If it prints nothing, stop: the API refuses to boot with `LegacyBehind`.

## 1. Environment additions (old backend still serving)

List the variable names the old backend reads today, without printing values:

```sh
grep -hoE '^[[:space:]]*(export[[:space:]]+)?[A-Z0-9_]+=' backend/.env .env 2>/dev/null | sed -E 's/^[[:space:]]*(export[[:space:]]+)?//; s/=$//' | sort -u
```

Add to `backend/.env`, keeping every old line (the old backend still reads them, and rollback needs them):

- **asset-sync, renamed with no aliases.** `vps-update.sh` installs the new `assets/binaries/asset-sync` on the first backend build, and it reads only these names: `STORAGE_TYPE` (value of `S3_TYPE`), `CLOUDFLARE_BUCKET` (`S3_BUCKET`), `CLOUDFLARE_API_ACCESS_KEY` (`S3_ACCESS_KEY_ID`), `CLOUDFLARE_API_SECRET_KEY` (`S3_SECRET_ACCESS_KEY`), `CLOUDFLARE_ACCOUNT_ID` (`S3_ACCOUNT_ID`), `CLOUDFLARE_S3_API` (`S3_ENDPOINT`), `CLOUDFLARE_PREFIX` (`S3_PREFIX`), `CLOUDFLARE_PUBLIC_BASE_URL` (`S3_PUBLIC_BASE_URL`), and `STORAGE_ROOTS`, `STORAGE_LOCAL_DIR`, `STORAGE_CACHE_CONTROL`, `STORAGE_SYNC_CONCURRENCY`, `STORAGE_SYNC_HASH_THREADS`, `STORAGE_MULTIPART_THRESHOLD_BYTES`, `STORAGE_EVICT_LOCAL`, `STORAGE_EVICT_KEEP` for the matching `S3_*` names. Keep `S3_TYPE`: `assets/run.mjs` still decides whether to run the sync stage by `S3_TYPE`. Without the new names asset-sync reports storage disabled, the watcher treats that as skipped, and new art never reaches R2.
- **Secrets the config references**, if they live only in the repo-root `.env`: `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET`, `GAME_CREDENTIAL_KEY`, `SERVICE_KEY`, `GITHUB_TOKEN`, `DISCORD_ALERT_WEBHOOK`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`. While `backend/.env` exists the new binaries read it and not the root one. `JWT_SECRET` and `GAME_CREDENTIAL_KEY` must keep their current values: logins and sealed game credentials carry over in both directions only if they match.

## 2. Pull and build (old backend still serving)

The sqlx query macros check every query against a live schema at compile time, and `.sqlx` is not in git. `vps-update.sh` therefore drops and rebuilds a scratch database `myrtle_sqlx` from `crates/db/migrations` on the server named in `backend/.env` and builds against it, never against production (prod lacks the new tables until the API boots). It needs a `psql` on the host, or:

```sh
export SQLX_PSQL="docker exec -i <pg-container> psql -U <user>"
```

The database user needs `CREATEDB` (UNVERIFIED for the prod user).

```sh
cd /var/www/myrtle.moe
git pull --ff-only     # vps-update.sh pulls too; harmless to do it first
bash vps-update.sh --no-frontend
```

While `myrtle-backend` is in pm2 the script builds `myrtle-api`, `myrtle-worker` and `myrtle`, installs the new asset-sync, and prints that it is NOT starting the new apps. That is expected. Check:

```sh
ls -la backend/target/release/myrtle-api backend/target/release/myrtle-worker backend/target/release/myrtle
```

If the box cannot finish the build (OOM), use the CI build instead: workflow "Backend CI", job "Release Build", artifact `backend-release-binaries` (uploaded for pushes to `main`, kept 7 days), holding `myrtle-api`, `myrtle-worker`, `myrtle` and `asset-sync`. UNVERIFIED that `gh` is installed and logged in on the VPS:

```sh
gh run download <run-id> -R Eltik/myrtle -n backend-release-binaries -D /tmp/backend-bins
for b in myrtle-api myrtle-worker myrtle; do install -m 755 "$(find /tmp/backend-bins -type f -name "$b")" "backend/target/release/$b"; done
```

## 3. Write `config/myrtle.toml`

```sh
cd /var/www/myrtle.moe/backend
cp config/myrtle.example.toml config/myrtle.toml
```

Parsing is strict: every section must be present, unknown keys are errors, so edit the copy rather than writing a file from scratch. Each key in the example names the env var it replaces (`was:`). Keys that MUST be changed for production:

| key | value |
|---|---|
| `server.bind_host` | `"127.0.0.1"` (the old default was `0.0.0.0`). Keep it only if every caller reaches the API on loopback: nginx (`grep -rn 3060 /etc/nginx`) and the frontend's SSR calls, which go to `BACKEND_URL` in the frontend's environment (`frontend/.env` or its pm2 env; `frontend/src/lib/fetch.ts`). Both must point at `127.0.0.1` or `localhost`, not the public IP or a Docker network name. |
| `server.trusted_proxy_hops` | value of `TRUSTED_PROXY_HOPS`; behind nginx this is `1`. With `0` every visitor shares nginx's IP for rate limiting. |
| `server.servers` | value of `SERVERS` as a list, e.g. `["en", "cn", "jp", "kr"]`. The first is the default. |
| `server.public_api_url` | value of `PUBLIC_API_URL`, e.g. `"https://api.myrtle.moe"`. |
| `server.metrics_bind` | leave `"127.0.0.1:9464"`. |
| `database.max_connections` | value of `DATABASE_MAX_CONNECTIONS` if set. |
| `rate_limit.rpm` | value of `RATE_LIMIT_RPM` if set. |
| `rate_limit.exempt_ips` | `["127.0.0.1/32", "::1/128"]` plus every entry of `RATE_LIMIT_EXEMPT_IPS`. The old backend exempted loopback implicitly; the new one does not, and frontend SSR calls from loopback. |
| `cpu.task_permits` | `2` (the old pm2 env set `CPU_TASK_PERMITS=2`). |
| `cpu.task_wait_ms` | `2500` (old pm2 env `CPU_TASK_WAIT_MS`). |
| `assets.dir` | `"../assets/output"` (unchanged default; matches the old `ASSETS_DIR`). |
| `assets.ws_urls` | one entry per server in `server.servers`: `{ en = "ws://127.0.0.1:9160", cn = "ws://127.0.0.1:9161", jp = "ws://127.0.0.1:9162", kr = "ws://127.0.0.1:9163" }` (ports from `assets/ecosystem.config.cjs`). The worker listens on these and publishes a new data generation after each extract; the API picks it up within 15 s. |
| `assets.delivery` | value of `ASSET_DELIVERY` if set. |
| `[storage]` | `type = "cloudflare"` (from `S3_TYPE`), `bucket`, `account_id`, `endpoint`, `prefix`, `public_base_url`, `roots`, `cache_control`, `evict_local`, `evict_keep` from the matching `S3_*` values. Keep `access_key_id = "env:S3_ACCESS_KEY_ID"` and `secret_access_key = "env:S3_SECRET_ACCESS_KEY"`. |
| `hypergryph.device_id`, `device_id2`, `device_id3` | the ids the old backend uses: `DEVICE_ID`/`DEVICE_ID2`/`DEVICE_ID3` from `backend/.env` if set (they took priority), else `backend/device_ids.json`. Left empty, the new code generates fresh ids per run and stored service-account sessions stop re-minting. |
| `hypergryph.network_version_override`, `cn_app_code` | values of `AK_NETWORK_VERSION`, `HYPERGRYPH_CN_APP_CODE` if set. |
| `jobs.*` | any interval the old env overrode (`TRENDING_RECOMPUTE_SECS`, `REGRADE_INTERVAL_SECS`, `GACHA_DETAIL_REFRESH_SECS`, `EVENT_SHOP_REFRESH_SECS`, `MAINTENANCE_POLL_SECS`, `SIREN_REFRESH_SECS`, ...). `jobs.dps_watch.every_secs` = `DPS_POLL_INTERVAL`, `0` = off. |
| `release.art_mirrors` | `RELEASE_ART_MIRRORS` if set, now a list of `"server=url"` strings. |
| `cli.admin_uid`, `cli.default_server` | values of `ADMIN_UID`, `BIN_SERVER`. |
| `[switches.*]` | flip `enabled` for every old flag env var that is set (`DISABLE_BACKGROUND_JOBS=1` is `background_jobs.enabled = false`, and so on; each switch names its old variable). |

Device ids as TOML lines from the file (prints ids, not secrets):

```sh
python3 -c 'import json; d = json.load(open("device_ids.json")); [print(f"{k} = \"{d[k]}\"") for k in ("device_id", "device_id2", "device_id3")]'
```

Check the file loads (it resolves `database.url`, `jwt_secret` and `game_credential_key` at load time). This prints the job counts, and fails until step 4 has created the `jobs` table, but a config or secret error shows first:

```sh
./target/release/myrtle jobs list --limit 1
```

## 4. Migrate (old backend still serving)

The API would run this at boot. Run it by hand instead so a slow index build is not cut short by a pm2 restart:

```sh
cd /var/www/myrtle.moe/backend
./target/release/myrtle migrate
```

It adopts the old schema as baseline 0001 without executing it, then applies every later file in `crates/db/migrations` that is not yet recorded. The set grows (another migration may have landed by deploy day), so do not count on a fixed number: `ls crates/db/migrations` lists what this build carries. At the time of writing: 0002 builds about 20 indexes on existing tables (`CREATE INDEX`, not concurrent) and drops one, 0003 to 0005 create the new tables `jobs`, `job_schedules`, `service_account_sessions`, `data_generations`, and the later files add nullable or defaulted columns to those new tables. Reads keep working; writes to `users`, `audit_log`, `user_scores` and the tier-list tables wait while 0002 builds its indexes. Run it at low traffic. It prints `migrations applied`.

It is not one transaction (`crates/db/src/migrate.rs`). The adoption row for 0001 commits first, then each file runs in its own transaction. A failure rolls back only the file that failed: the ones before it stay applied, and re-running `myrtle migrate` continues from the failed file. Check what is recorded:

```sh
docker exec <pg-container> psql -U <user> -d <db> -Atc "SELECT version, success FROM _sqlx_migrations ORDER BY version"
```

Every file in `crates/db/migrations` must appear once, with `success` = `t`.

## 5. Dry-run the session import

```sh
./target/release/myrtle session import-dir game_sessions
```

This is a check that every `<server>.json` parses and lands, not the real import. It prints one line per file: `imported`, or `kept the stored session`. The old backend keeps re-minting these files until it stops, so the copy stored now goes stale; step 7 imports again with `--force` right after the old backend stops. Do not run `myrtle session status` yet: it refreshes the session against the game server, which can sign out the copy the old backend is still using.

## 6. Smoke test beside the old backend

Run a second API on port 3061 with no metrics listener and an in-memory cache (the two builds share Redis key names, so a shared Redis would let each serve the other's cached shapes):

```sh
cd /var/www/myrtle.moe/backend
sed -e 's/^port = 3060$/port = 3061/' -e 's/^metrics_bind = .*/metrics_bind = ""/' \
    -e '/^\[redis\]/,/^\[/{/^url = /d}' config/myrtle.toml > config/myrtle.staging.toml
grep -nE '^(port|metrics_bind) =' config/myrtle.staging.toml; sed -n '/^\[redis\]/,/^\[/p' config/myrtle.staging.toml
nohup ./target/release/myrtle-api config/myrtle.staging.toml > /tmp/myrtle-api-staging.log 2>&1 &
echo $! > /tmp/myrtle-api-staging.pid
```

It logs `listening` when ready (`tail -f /tmp/myrtle-api-staging.log`). Then:

```sh
curl -s http://127.0.0.1:3061/api/health; echo
canon() { python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin), sort_keys=True))' 2>/dev/null | md5sum; }
for p in /api/static/chibis /api/static/enemy-chibis /api/static/stage-index /api/operators/index \
         /api/skins/index /api/dps/operators /api/hps/operators /api/base/catalog /api/squads/rules; do
    ca=$(curl -s -o /tmp/old.json -w '%{http_code}' "http://127.0.0.1:3060$p")
    cb=$(curl -s -o /tmp/new.json -w '%{http_code}' "http://127.0.0.1:3061$p")
    [ "$(canon < /tmp/old.json)" = "$(canon < /tmp/new.json)" ] && r=same || r=DIFFERS
    echo "$r  old=$ca new=$cb  $p"
done
curl -sI http://127.0.0.1:3061/api/assets/spine/Enemy/enemy_1000_gopro/enemy_1000_gopro_2.skel | head -3
```

Health must start `{"status":"ok"` with `"degraded":false` under `gameData`. Every path in the loop is public, built only from the extracted game data (no database rows, no caller, no service account) and exists on both builds, so both sides should answer 200, and a difference is a real shape change rather than fresh numbers. A `DIFFERS` needs a look before the switch: fetch that one path from both ports again and diff the two bodies. Stop the staging API:

```sh
kill "$(cat /tmp/myrtle-api-staging.pid)"; rm -f config/myrtle.staging.toml /tmp/myrtle-api-staging.pid
```

## 7. Switch

Downtime runs from `pm2 stop myrtle-backend` until the new API passes `/api/health`: the session import, the Redis flush, and the new API's boot (game data load; migrations are already applied).

The Redis flush happens while neither build is running, so neither ever serves the other's cached shapes. It empties the one database `REDIS_URL` names (its `/<n>` suffix, `0` if none), which both builds share. What it costs:

- **Response caches** (`static:`, `operators:`, `operator_detail:`, `user:`, `tierlist:`, `gacha:`, `level:`, `i18n:`, `music:` and the rest): rebuilt on the next request, so the first minutes are slower.
- **Players' game sessions** (`game_session:<uid>`, 1 h TTL): the next sync rebuilds the session from the game credential sealed in Postgres. A user who never stored one gets "game login required" and logs in to the game again.
- **Gacha portal sessions** (`portal_session:<uid>`, 7 days): `/api/gacha/fetch` answers "no portal session - login again" until the user logs in again.
- **Pending Bilibili SMS codes** (`bilibili_sms:<phone>`): a code sent before the switch no longer verifies; request a new one.

Site logins survive. A site login is a JWT signed with `JWT_SECRET` and checked against the `users` row in Postgres (`crates/app/src/services/accounts/jwt.rs`, `account.rs` `verify`); nothing of it is in Redis, so logged-in users stay logged in as long as `JWT_SECRET` is unchanged (step 1).

The flush below uses `redis-cli -u "$REDIS_URL"`, which reaches the same host, port, password and database index the backend uses. With no `redis-cli` on the host, set `REDIS_CLI` to one inside the Redis container (UNVERIFIED: container name, take it from `docker ps`; the URL's port must then also be the container's own port).

```sh
cd /var/www/myrtle.moe/backend
REDIS_CLI="${REDIS_CLI:-redis-cli}"     # or: REDIS_CLI="docker exec -i <redis-container> redis-cli"
REDIS_URL="$(grep -m1 -E '^[[:space:]]*(export[[:space:]]+)?REDIS_URL=' .env | sed -E 's/^[^=]*=[[:space:]]*//; s/[[:space:]]+#.*$//' | tr -d "\"'")"
[ -n "$REDIS_URL" ] && $REDIS_CLI -u "$REDIS_URL" PING || echo "REDIS_URL not found in backend/.env"
```

That must print `PONG`. Stay in the same shell (the next blocks use `REDIS_CLI` and `REDIS_URL`) and switch:

```sh
pm2 stop myrtle-backend
./target/release/myrtle session import-dir game_sessions --force   # the sessions the old backend last wrote
$REDIS_CLI -u "$REDIS_URL" FLUSHDB                                  # prints OK
pm2 start ecosystem.config.cjs --only myrtle-api
deadline=$(( $(date +%s) + 300 )); healthy=0
while [ "$(date +%s)" -lt "$deadline" ]; do
    if curl -sf --max-time 5 http://127.0.0.1:3060/api/health > /dev/null; then healthy=1; break; fi
    sleep 2
done
if [ "$healthy" -eq 1 ]; then
    curl -s http://127.0.0.1:3060/api/health; echo
else
    pm2 logs myrtle-api --lines 100 --nostream
    echo "myrtle-api did not pass /api/health within 5 minutes: run Rollback below"
fi
```

Go on only if it printed a health body starting `{"status":"ok"`. On the timeout message, or on `"status":"degraded"`, run Rollback. Then:

```sh
pm2 start ecosystem.config.cjs --only myrtle-worker
pm2 delete myrtle-backend
pm2 save
```

Deleting `myrtle-backend` keeps a reboot from resurrecting it next to the new apps; the rollback starts it from `/root/myrtle-backend-rollback/`. Check:

```sh
pm2 logs myrtle-worker --lines 50 --nostream          # "worker started", then "claiming jobs"
./target/release/myrtle jobs list                      # counts by state, recent jobs
./target/release/myrtle session status --server en    # one per server with a session
curl -s http://127.0.0.1:9464/metrics | head
curl -s -o /dev/null -w '%{http_code}\n' https://api.myrtle.moe/api/health
```

### Catch up the deploy stamp

`vps-update.sh` decides what to rebuild from `.git/vps-deployed`, the last commit a run finished with nothing skipped. The step 2 run did not write it (the old backend was in pm2, so it skipped the restart), so the next run still sees every `backend/` change since the last pre-cutover deploy. Run it once at low traffic:

```sh
cd /var/www/myrtle.moe && bash vps-update.sh
```

Know what that run does:

- It re-runs the backend build (cargo finds the step 2 output current, but the scratch `myrtle_sqlx` rebuild and the link check still run) with `myrtle-api` and `myrtle-worker` both resident. If memory is tight, `pm2 stop myrtle-worker` first; the script starts it again once the API passes health, and leaves it stopped (with a warning) if the API does not.
- It restarts `myrtle-api`, a second outage of one API boot, then the worker.
- If the box had no stamp at all, the script diffs only what this pull brought in. With nothing new it restarts nothing and just writes the stamp.
- It builds and restarts the frontend if `frontend/` changed since that stamp. With `--no-frontend` and a `frontend/` change, the script records a skip and does not advance the stamp, so the catch-up is still pending afterwards. Run it without `--no-frontend` to finish it.

## Rollback

The migration is forward compatible (see the db audit): adoption writes only the new `_sqlx_migrations` table and never touches `_migrations`, so the old runner finds all 44 of its names and applies nothing. The later files add tables, columns and indexes, drop no column, and drop only `idx_tier_lists_slug`, which duplicates the `tier_lists_slug_key` constraint index. No database restore is needed.

The worker's two sidecar files are compatible too. The gacha pool details (`derived/gacha_pool_details.json`) and event shops (`derived/event-shops.json`) under `{assets.dir}/{server}/` keep the old path, the old file versions (1 and 2), and the same serde field names as the old reader on `main` (`backend/src/core/gamedata/types/gacha_detail.rs` and `event_shop.rs` there, `crates/gamedata-raw/src/tables/gacha_detail.rs`, `event_shop.rs` and `gacha.rs` here). The old backend reads what the worker wrote, so they need no rollback step. One exception: if the old backend ran with `GACHA_DETAIL_DIR` or `EVENT_SHOP_DIR` set, the new `jobs.gacha_detail.dir` and `jobs.event_shop.dir` must name the same directories (step 3), or each side refreshes its own copy.

Run Rollback as one block. The Redis flush sits between stopping the new apps and starting the old one, for the same reason as in step 7 and at the same cost; site logins survive it.

```sh
cd /var/www/myrtle.moe/backend
REDIS_CLI="${REDIS_CLI:-redis-cli}"     # or: REDIS_CLI="docker exec -i <redis-container> redis-cli"
REDIS_URL="$(grep -m1 -E '^[[:space:]]*(export[[:space:]]+)?REDIS_URL=' .env | sed -E 's/^[^=]*=[[:space:]]*//; s/[[:space:]]+#.*$//' | tr -d "\"'")"
pm2 stop myrtle-worker 2>/dev/null || true
pm2 stop myrtle-api 2>/dev/null || true
[ -n "$REDIS_URL" ] && $REDIS_CLI -u "$REDIS_URL" FLUSHDB || echo "FLUSH NOT DONE: flush Redis by hand before the old backend serves"
pm2 delete myrtle-backend 2>/dev/null || true                       # still in pm2 if step 7 stopped short of deleting it
pm2 start /root/myrtle-backend-rollback/ecosystem.config.cjs
pm2 delete myrtle-worker 2>/dev/null || true
pm2 delete myrtle-api 2>/dev/null || true
pm2 save
```

What rollback does not carry back by itself:

- **Service-account sessions.** The new build keeps them in Postgres. If it re-minted or re-logged an account, `backend/game_sessions/<server>.json` is stale: log in again with the old backend's `set-session` tool.
- **Grades.** `user_scores` holds the new scorer's values until the old backend's next regrade.
- **Leftover rows.** `jobs`, `job_schedules`, `service_account_sessions`, `data_generations` and `_sqlx_migrations` stay; the old backend ignores them, and a later cutover reuses them (adoption is then a no-op).
- **asset-sync.** `assets/binaries/asset-sync` is the new binary; it keeps working because step 1 added its names. The old binary is not kept.

Never run `sqlx migrate run` against production: it bypasses adoption and tries to execute the baseline.

## Behaviour changes an operator must know

- `/docs` and `/api/openapi.json` are gone.
- `/metrics` is no longer on the public router. It listens on `127.0.0.1:9464` (`server.metrics_bind`; empty disables it). Point any scraper there.
- Configuration is `config/myrtle.toml`, not env vars. `PORT`, `HOST`, `SERVERS`, `CPU_TASK_PERMITS` and the rest are ignored; only `env:` references and `RUST_LOG` are read from the environment.
- `RUST_LOG` targets are crate names now (`myrtle_api`, `myrtle_app`, ...). The old `backend=info,tower_http=info` mutes almost everything; the ecosystem sets `info`.
- Background jobs run only in `myrtle-worker`. With the worker stopped, nothing regrades, refreshes gacha or event shops, polls maintenance, or reloads game data after an extract.
- Game data reload: the worker hears the watchers' `update_complete` on `assets.ws_urls` and publishes a new generation; the API polls for it every 15 s. By hand: `./target/release/myrtle gamedata publish --server <en|cn|jp|kr>`.
- Loopback is not exempt from rate limiting unless it is in `rate_limit.exempt_ips`.
- The API binds `server.bind_host` (`127.0.0.1` in the example), not `0.0.0.0`.
- `/api/health` reports `"status":"degraded"` when Redis or Postgres is down, not only `gameData.degraded`.
- Service-account sessions live in Postgres (`service_account_sessions`); `game_sessions/`, `device_ids.json`, `regrade_state.json`, `.dps-updater-state.json` and `startup_timings.json` are no longer read. Keep them until rollback is off the table.
- Builds need a database: `vps-update.sh` builds against the scratch `myrtle_sqlx` database, CI against its own Postgres service.
