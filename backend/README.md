# Myrtle Backend

The API server for [myrtle.moe](https://myrtle.moe). It loads the game data the [asset pipeline](../assets/README.md) extracts, serves it to the [frontend](../frontend/README.md) and the [Discord bot](../discord/README.md) over REST, signs players in against the game's own servers to sync their accounts, and runs the DPS, base, grading, planner, and story engines. A Cargo workspace of 25 packages (22 crates in strict layers, two tools, and an xtask), built into an API binary, a job worker, an operator CLI, and an object-storage sync tool.

[![Rust](https://img.shields.io/badge/Rust-edition_2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-sqlx_0.9-336791?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Redis](https://img.shields.io/badge/Redis-optional-dc382d?logo=redis&logoColor=white)](https://redis.io/)
[![Backend CI](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

## Features

- **Game data API** - operators, skins, voices, enemies, stages, chibis, and static tables for every configured server, compiled into an in-memory model at boot and swapped without a restart when a new extract lands.
- **Player sync** - Yostar email-code login (EN, JP, KR), Hypergryph passport, and Bilibili, then roster, inventory, base, and story progress pulled straight from the game servers. Game credentials are sealed at rest.
- **DPS & HPS** - per-skill damage and healing, with formulas transpiled from the upstream Python reference.
- **Base (RIIC)** - layout optimizer, shift rotations, and a base grade.
- **Account grading & leaderboards** - a composite account grade, score and per-item leaderboards, and daily snapshots.
- **Planner & squads** - upgrade plans with material and crafting requirements, and squad analysis.
- **Gacha** - synced pull history, pity, and community pull statistics.
- **Story, archives, and music** - parsed story scripts with a library and galleries, personnel files, and the Monster Siren catalogue.
- **Release planner** - CN to EN alignment, lag estimates, and outfit sale histories.
- **Tier lists** - official and community lists with per-list permissions.
- **Sign in with myrtle.moe** - an OAuth 2.0 provider for third-party sites. See [OAUTH.md](OAUTH.md).
- **Asset delivery** - images, audio, and Spine files served from the local extract, or redirected or proxied from Cloudflare R2.
- **Background jobs** - a Postgres-backed queue with schedules, run by a separate worker process.
- **Generated TypeScript types** - every wire type is exported with `ts-rs` into the frontend.

## Tech Stack

| Component     | Technology                                                        |
|---------------|-------------------------------------------------------------------|
| Language      | Rust (Edition 2024, `rust-version` 1.94)                          |
| Framework     | Axum 0.8 on Tower, with gzip and brotli compression               |
| Runtime       | Tokio (multi-threaded), jemalloc allocator                        |
| Database      | PostgreSQL via `sqlx` 0.9 (compile-time checked queries, embedded migrations) |
| Cache         | Redis (optional; an in-memory cache stands in without it)         |
| Job queue     | Postgres `SKIP LOCKED`, no broker                                 |
| HTTP client   | `reqwest` 0.13 over `rustls`                                      |
| Auth          | `jsonwebtoken` session tokens, XChaCha20-Poly1305 sealed credentials |
| Storage       | Cloudflare R2 through `rusty-s3`                                  |
| Types         | `ts-rs` 11 bindings for the frontend                              |
| CLI           | `clap` 4                                                          |
| Logging       | `tracing` + `tracing-subscriber`                                  |

## Prerequisites

| Tool        | Version | Notes                                                                 |
|-------------|---------|-----------------------------------------------------------------------|
| Rust        | 1.94+   | CI builds with 1.98.0. There is no `rust-toolchain` file              |
| PostgreSQL  | -       | Needed to **compile**, not only to run (see [Quick Start](#1-load-the-schema)). CI uses 18 |
| `psql`      | -       | Loads the schema the build checks queries against                     |
| Game assets | -       | An extract under `../assets/output/<server>/`, from the [asset pipeline](../assets/README.md) |
| Redis       | -       | Optional                                                              |
| Bun         | 1.4.2   | Only to regenerate the frontend types                                 |

## Quick Start

Every command runs from `backend/`.

### 1. Load the schema

The `sqlx` query macros in `crates/db` check each query against a live database at compile time, and the `.sqlx` offline cache is not committed. So the build needs a database that already has the migrations applied. Keep a scratch one for that, as CI and the production build do:

```bash
export PGPASSWORD=password   # Or psql asks once per file
psql -h localhost -U postgres -v ON_ERROR_STOP=1 -qc 'CREATE DATABASE myrtle_sqlx'
for f in crates/db/migrations/*.sql; do
  psql -h localhost -U postgres -d myrtle_sqlx -v ON_ERROR_STOP=1 -q -1 -f "$f"
done
```

Drop and rebuild it when a migration is added.

### 2. Build

```bash
DATABASE_URL=postgres://postgres:password@localhost:5432/myrtle_sqlx cargo build
```

Every cargo command that compiles `crates/db` (`check`, `clippy`, `test`, `run`) needs `DATABASE_URL` pointing at a migrated schema the same way. Do not `export` it in the shell you start the server from: the binaries read the process environment before `.env`, and would start on the scratch database.

### 3. Configure

```bash
cp config/myrtle.example.toml config/myrtle.toml
```

Then create `backend/.env` with the secrets the config refers to. There is no `.env.example`; these three are the required ones:

```bash
DATABASE_URL=postgres://postgres:password@localhost:5432/myrtle
JWT_SECRET=<openssl rand -hex 32>
GAME_CREDENTIAL_KEY=<openssl rand -hex 32>
```

`DATABASE_URL` here is the database the server runs on, which is not the scratch one from step 1. Create it empty (`createdb -h localhost -U postgres myrtle`); the API migrates it on first boot.

### 4. Add game data

Neither binary starts without the game data of the first server in `server.servers` (`en` in the example). Put an extract at `../assets/output/en/gamedata/excel/`, from the [asset pipeline](../assets/README.md). `character_table.json` is the one table the load cannot do without. Other servers that fail to load are logged and skipped.

### 5. Run

```bash
cargo run -p myrtle-api -- config/myrtle.toml      # HTTP API on 127.0.0.1:3060
cargo run -p myrtle-worker -- config/myrtle.toml   # Background jobs. Run exactly one
```

Both take the config path as their first argument and default to `config/myrtle.toml`. Start the API first: the worker does not migrate, and fails on a database the API has not set up yet. `GET /api/health` reports the state of the database, the cache, and each server's game data.

The API alone is enough to browse game data. Without the worker nothing regrades, refreshes gacha or event shops, polls maintenance, or reloads game data after an extract.

## Configuration

Configuration is one TOML file, `config/myrtle.toml` (gitignored). Parsing is strict: every section must be present and unknown keys are rejected, so start from the example and edit values.

[`config/myrtle.example.toml`](config/myrtle.example.toml) documents every key next to its default. It is generated from `FIELDS` in `crates/app/src/config.rs`; do not edit it by hand:

```bash
MYRTLE_BLESS=1 cargo test -p myrtle-app example_is_current
```

| Section        | Holds                                                                    |
|----------------|--------------------------------------------------------------------------|
| `[server]`     | Bind host and port, the `/metrics` listener, trusted proxy hops, the servers served (the first is the default), the public API URL |
| `[database]`   | Postgres URL, pool size, acquire timeout                                 |
| `[redis]`      | Redis URL                                                                |
| `[secrets]`    | Session signing key, credential sealing key, service key, GitHub token, Discord alert webhook |
| `[rate_limit]` | Requests per minute and exempt CIDRs                                     |
| `[cpu]`        | Permits and queueing for CPU-bound requests                              |
| `[assets]`     | Extract root, asset-pipeline WebSocket URLs, delivery mode               |
| `[storage]`    | Cloudflare R2 bucket and credentials                                     |
| `[hypergryph]` | Game-server client settings and device ids                               |
| `[jobs]`       | Worker lanes and one sub-table per scheduled job                         |
| `[dps]`        | DPS engine settings                                                      |
| `[release]`    | Release planner settings                                                 |
| `[story]`      | Story reader settings                                                    |
| `[cli]`        | Defaults for the `myrtle` CLI                                            |
| `[tools]`      | Settings for the tools under `tools/`                                    |
| `[debug]`      | Debug aids, such as dumping a sync to disk                               |
| `[switches.*]` | 19 kill switches, each with `enabled`, an `owner`, and a `remove_after` date |

A kill switch past its `remove_after` date is logged as a warning at boot and listed under `expiredSwitches` in `/api/health`.

### Secrets

A secret in the TOML is a reference, never a value: `"env:NAME"` or `"file:/path"`. An `env:` reference reads the process environment first, then the first `.env` found walking up from the working directory. From `backend/` that is `backend/.env`; the repo-root `.env` is read only when `backend/.env` is missing, and the two are never merged.

The names below are the example's. A deployment can rename them in the TOML.

| Variable                | Required | Description                                                          |
|-------------------------|----------|----------------------------------------------------------------------|
| `DATABASE_URL`          | Yes      | Postgres URL                                                         |
| `JWT_SECRET`            | Yes      | Session signing key. A warning is logged under 32 bytes              |
| `GAME_CREDENTIAL_KEY`   | Yes      | Seals stored game credentials. 32 bytes as hex or base64             |
| `REDIS_URL`             | No       | Shared cache. Unset, the cache is in-process                         |
| `SERVICE_KEY`           | No       | Sent as `x-service-key` by trusted tooling, such as the frontend's `i18n:sync` script. Acts as the system and skips the rate limit |
| `GITHUB_TOKEN`          | No       | Raises the GitHub API limit for the DPS upstream watch               |
| `DISCORD_ALERT_WEBHOOK` | No       | Where service-account and degraded-data alerts are posted            |
| `S3_ACCESS_KEY_ID`      | With R2  | Bucket credential, when `storage.type = "cloudflare"`                |
| `S3_SECRET_ACCESS_KEY`  | With R2  | Bucket credential, when `storage.type = "cloudflare"`                |

Apart from these references, the binaries read two variables straight from the environment: `RUST_LOG` (default `info`; targets are crate names such as `myrtle_api` and `myrtle_app`) and `ASSET_DELIVERY_ORACLE`. `PORT`, `HOST`, `SERVERS`, and the rest of the old backend's variables are ignored.

## Binaries

| Binary          | Package          | Role                                                              |
|-----------------|------------------|-------------------------------------------------------------------|
| `myrtle-api`    | `myrtle-api`     | The HTTP API. Runs the database migrations at boot                |
| `myrtle-worker` | `myrtle-worker`  | Job queue, schedules, and the asset-pipeline listener. Does not migrate |
| `myrtle`        | `myrtle-cli`     | Operator CLI                                                      |
| `asset-sync`    | `myrtle-storage` | Mirrors an asset extract into Cloudflare R2                       |

**Run exactly one worker.** Each worker loads the game service-account sessions into memory and refreshes them on its own, so a second one re-mints the same accounts and signs the first out.

### `myrtle`

```bash
cargo run -p myrtle-cli -- <command>
```

`--config <path>` (default `config/myrtle.toml`) and `-q` / `--quiet` apply to every command.

| Command | Description |
|---------|-------------|
| `migrate` | Apply pending migrations |
| `gamedata compile` · `publish` | Check that a server's tables compile and list the warnings, or publish a new data generation so the API reloads |
| `db export` · `import` · `translate-legacy` | Whole-database export and import, and translation of a legacy export |
| `user export` · `import` | One account and every table it uses, as JSON |
| `regrade` | Regrade accounts: all of them, one `--uid`, a `--dry-run`, or `--enqueue` as jobs |
| `grants users` · `role` · `list` · `add` · `remove` | Site roles and per-tier-list permissions |
| `tier-list list` · `visibility` · `set-type` | Tier-list administration |
| `gacha export` · `resync` | One user's pull records as JSON, or rarities rewritten to match the game data |
| `stats` | Recompute community statistics now: `operator-ownership`, `medal-ownership`, `trending`, `leaderboard-snapshot`, and with `--include-heavy` also `gacha-details`, `event-shops`, `regrade` |
| `session send-code` · `login` · `bili` · `sms` · `import` · `import-dir` · `status` | Sign the game service accounts in, import their sessions, and check them |
| `jobs list` · `enqueue` | Job counts and recent jobs, or queue one job by kind |
| `hypergryph device-ids` | Print fresh device ids as TOML for `[hypergryph]`. Needs no config |

### `asset-sync`

```bash
cargo run -p myrtle-storage --bin asset-sync -- <command>
```

| Command   | Description                                                     |
|-----------|-----------------------------------------------------------------|
| `sync`    | Upload new and changed files and delete orphans                 |
| `verify`  | Dry-run diff. Exits 2 when the bucket differs                   |
| `evict`   | Truncate local files the bucket already holds to 0-byte stubs   |
| `restore` | Download evicted stubs back                                     |
| `ls`      | List remote keys                                                |
| `cors`    | Set the bucket's CORS rule                                      |

`asset-sync` is the one binary configured by environment variables and not by `myrtle.toml`: `STORAGE_TYPE`, `CLOUDFLARE_BUCKET`, `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_API_ACCESS_KEY`, `CLOUDFLARE_API_SECRET_KEY`, and the optional `CLOUDFLARE_*` and `STORAGE_*` settings. It reads `.env` and `../.env`, or the files named by `--env-file`. Run any command with `--help` for its flags.

## API

Everything is served under `/api`. A server prefix is accepted on any path: `/api/jp/operators` is `/api/operators` answered from the JP data. Without one, the first entry of `server.servers` answers.

About 224 routes in 39 modules under `crates/api/src/routes/`:

| Area | Paths |
|------|-------|
| Login and sessions | `/login*`, `/auth/*`, `/refresh` |
| OAuth provider | `/oauth/*` |
| Accounts and profiles | `/user/*`, `/get-user*`, `/roster`, `/inventory`, `/friends`, `/players` |
| Game data | `/operators`, `/skins`, `/voices`, `/enemies`, `/stages`, `/static/*`, `/chibis`, `/search` |
| Calculators | `/dps`, `/hps`, `/base`, `/plan`, `/plans`, `/squads` |
| Community | `/tier-lists`, `/leaderboard`, `/gacha`, `/grids`, `/lookback`, `/stats` |
| Content | `/story`, `/archives`, `/music`, `/release`, `/i18n` |
| Assets | `/assets/*`, `/avatar`, `/portrait`, `/skill-icon`, `/item-icon`, and other image endpoints |
| Operations | `/health`, `/maintenance`, `/announcements`, `/admin/*` |

There is no OpenAPI document and no `/docs` page. The route modules and the generated TypeScript types are the reference.

### Authentication

Signed-in requests send `Authorization: Bearer <token>`, a JWT valid for 7 days. A token that claims a role above plain user has that role checked against the stored one on each request, and the lower of the two applies (`switches.role_recheck`). Auth uses no cookies. An OAuth access token is a different credential and is accepted only by `/api/oauth/userinfo`.

### Errors

```json
{ "error": { "code": "RATE_LIMITED", "message": "rate limited" } }
```

| Status | `code`                | Notes                                          |
|--------|-----------------------|------------------------------------------------|
| 400    | `BAD_REQUEST`         |                                                |
| 400    | `GAME_LOGIN_REQUIRED` |                                                |
| 401    | `UNAUTHORIZED`        |                                                |
| 403    | `FORBIDDEN`           |                                                |
| 404    | `NOT_FOUND`           |                                                |
| 409    | `CONFLICT`            |                                                |
| 422    | `VALIDATION_FAILED`   | With a `details` array of `{ field, message }` |
| 429    | `RATE_LIMITED`        | With `Retry-After`                             |
| 500    | `INTERNAL_ERROR`      |                                                |
| 503    | `SERVICE_UNAVAILABLE` |                                                |
| 503    | `GAME_MAINTENANCE`    | Carries `ends_at`                              |
| 504    | `TIMEOUT`             |                                                |

The three OAuth protocol endpoints use the RFC 6749 format, described in [OAUTH.md](OAUTH.md).

### Rate limits

Limits are per IP address, in one-minute windows, held in the API process:

| Bucket    | Limit per minute       | Covers                                                             |
|-----------|------------------------|--------------------------------------------------------------------|
| Auth      | 10                     | `/api/login*`, `/api/auth/*`                                       |
| Expensive | 20                     | `/dps` and `/hps` `calculate` and `batch`, `/dps/rank`, `/dps/supporters`, `/base/optimize`, `/base/rotation`, `/base/evaluate`, `/user/max-level-cost`, `/user/improvements`, `/squads/analyze` |
| Upstream  | 6                      | `/gacha/fetch`, `/refresh`, `/friends`, `/players/search`, which call the game servers. Signed-in requests only; anonymous ones count against Default |
| Default   | `rate_limit.rpm` (100) | Everything else                                                    |

A request over the limit answers `429` with `Retry-After: 60`. The service key and the CIDRs in `rate_limit.exempt_ips` are exempt. Loopback is **not** exempt by default. Behind a reverse proxy, set `server.trusted_proxy_hops` so the client address is read from `X-Forwarded-For`; at `0` every request appears to come from the proxy.

### Metrics

`/metrics` is not on the public router. It has its own listener at `server.metrics_bind` (`127.0.0.1:9464` in the example); an empty value turns it off.

## Game Data and Assets

`assets.dir` (`../assets/output` in the example, relative to the working directory) is the extract root. Each server reads `{dir}/{server}/gamedata/excel/*.json`; Bilibili shares the `cn` tree.

The API compiles every configured server at boot. After that, a reload is driven by a row in the `data_generations` table, which the API polls every 15 seconds:

1. The asset pipeline finishes an extract and announces `update_complete` on its WebSocket.
2. The worker, listening on `assets.ws_urls`, publishes a new generation.
3. The API sees it and swaps the model in.

By hand: `myrtle gamedata publish --server en`. A table that fails to load does not stop the reload: it is listed under `gameData` in `/api/health`, and reported to the Discord webhook when one is set.

Both processes write into the extract, under `{server}/derived/`: the API the files it derives while compiling, the worker gacha pool details, event shops, and mirrored release art.

### Delivery

`/api/assets/...` and the image endpoints serve files in one of three modes:

| Mode       | Behaviour                                            | Chosen when                              |
|------------|------------------------------------------------------|------------------------------------------|
| `local`    | Streamed from the extract on disk                    | Storage is off                           |
| `redirect` | Redirected to the bucket's public URL                | Storage is on and `public_base_url` is set |
| `proxy`    | Fetched from the bucket and passed through           | Storage is on with no public URL         |

`assets.delivery` overrides the choice while storage is on; with storage off the mode is always `local`. Storage means Cloudflare R2 (`storage.type = "cloudflare"`), or a local directory standing in for a bucket (`"local"`).

With `asset-sync evict`, the local copies of mirrored files become 0-byte stubs and the bucket holds the bytes. `evict` refuses to run when delivery resolves to `local`, but it judges that from `ASSET_DELIVERY` in its own environment, not from `myrtle.toml`. Keep the two in step, or the API serves empty files.

## Background Jobs

The queue is a Postgres table read with `SKIP LOCKED`, and schedules are rows in `job_schedules`. The worker polls every 2 seconds across separate lanes for short, long, backfill, and regrade work.

| Schedule                           | Interval                       | Set by                         |
|------------------------------------|--------------------------------|--------------------------------|
| Maintenance poll                   | 5 minutes                      | `jobs.maintenance.poll_secs`   |
| Trending recompute                 | 15 minutes                     | `jobs.trending.every_secs`     |
| Gacha detail refresh               | 6 hours                        | `jobs.gacha_detail.every_secs` |
| Event shop refresh                 | 6 hours                        | `jobs.event_shop.every_secs`   |
| Monster Siren sync                 | 6 hours                        | `jobs.siren.every_secs`        |
| Regrade all                        | 24 hours                       | `jobs.regrade.every_secs`      |
| DPS upstream watch                 | Off                            | `jobs.dps_watch.every_secs`    |
| Music stat dedupe sweep            | 1 hour                         | Fixed                          |
| Story community aggregate          | 6 hours, one per loaded server | Fixed                          |
| Leaderboard snapshot               | 24 hours                       | Fixed                          |
| Operator and medal ownership stats | 24 hours                       | Fixed                          |

The intervals shown are the example config's. Setting a configurable one to `0` turns that schedule off, which is how the DPS upstream watch ships.

`switches.background_jobs` stops the worker from queueing scheduled jobs and from listening to the asset pipeline, so game data no longer reloads on its own. A job runs at most 5 times before it is given up on; trending recompute and the maintenance poll run once. Finished jobs are pruned after 7 days.

Inspect or queue by hand with `myrtle jobs list` and `myrtle jobs enqueue <kind>`.

## Database

Seven migrations in [`crates/db/migrations/`](crates/db/migrations/) are embedded at compile time. `myrtle-api` applies them at boot, and `myrtle migrate` applies them on demand.

`0001_baseline.sql` is the whole schema of the previous backend in one file. A database that backend created is **adopted**, not migrated: the baseline is recorded as applied without being run, and only the later files execute. For that reason:

- A previous-backend database must be at its last migration, `v046_oauth`, to be adopted. Behind that, `myrtle-api` refuses to boot.
- Never run `sqlx migrate run` against an existing database. It skips adoption and tries to execute the baseline.
- Add schema changes as a new numbered file. Never edit an applied one.
- A new or changed query needs a rebuilt scratch schema (see [Quick Start](#1-load-the-schema)) before it compiles.

Redis is optional. It holds response caches and shared state between the API and the worker; without it each process keeps its own, and the worker's cache invalidations do not reach the API.

## Generated Types

Every type that crosses the API derives `ts_rs::TS`. The bindings land in `../frontend/src/types/generated/` (745 files) and are committed, so the frontend builds without a Rust toolchain.

```bash
cargo run -p xtask -- gen-ts           # Regenerate
cargo run -p xtask -- gen-ts --check   # Fail if the committed bindings are stale
```

`gen-ts` exports from every crate that uses `ts-rs`, rejects any `bigint` or `any` in the output (JSON carries plain numbers), copies the files into the frontend, deletes stale ones, and formats them with Biome. `bun run gen:types` in `frontend/` runs the same command.

## Project Structure

```
backend/
├── crates/
│   ├── types/            L0  Ids, Server, shared wire enums, Page<T>
│   ├── gamedata-raw/     L1  Excel table structs and the loader
│   ├── storage/          L1  Cloudflare R2 object storage, and the asset-sync binary
│   ├── gamedata/         L2  The compiled game model: normalised, enriched, indexed
│   ├── dps-formulas/     L3  Transpiled DPS/HPS formulas and their tables
│   ├── engine-dps/       L3  DPS/HPS engine: buff zones, targets, cycles, supporters
│   ├── engine-base/      L3  Base (RIIC) optimizer, shift rotation, base grade
│   ├── engine-grade/     L3  Account grade
│   ├── engine-planner/   L3  Upgrade planner: materials, crafting, plan validation
│   ├── engine-squad/     L3  Squad analysis
│   ├── engine-story/     L3  Story scripts, speaker attribution, library, galleries
│   ├── engine-archives/  L3  Personnel files, the network chart, roster counts
│   ├── engine-music/     L3  Music catalogue
│   ├── engine-release/   L3  Release planner: CN to EN alignment, lag estimates
│   ├── hypergryph/       L3  Game-server clients: Yostar, passport, Bilibili, Monster Siren
│   ├── db/               L4  Postgres pool, migrations, repositories
│   ├── jobs/             L4  Job queue, leases, schedules
│   ├── app/              L5  Services, config, policy, caches: everything the binaries share
│   ├── api/              L6  The HTTP API binary
│   ├── worker/           L6  The job worker binary
│   ├── cli/              L6  The myrtle CLI binary
│   └── testkit/          Test fixtures and fakes. A dev-dependency only
├── tools/
│   ├── dps-transpile/    ArknightsDpsCompare (Python) -> crates/dps-formulas
│   └── parity/           HTTP and engine parity harness against the previous backend
├── xtask/                check-layers and gen-ts
├── config/               myrtle.example.toml
├── ecosystem.config.cjs  PM2 apps for the API and the worker
└── deny.toml             cargo-deny: advisories, licenses, sources, and the dependency bans below
```

### Layers

A crate may depend only on its own layer or a lower one. The table lives in `[workspace.metadata.layers]` in `Cargo.toml` and is enforced by:

```bash
cargo run -p xtask -- check-layers
```

The engines are pure: no database, no network, no async runtime. They take game data and a request and return a result, which is why each has its own tests and none needs Postgres. `deny.toml` holds the same line from the other side: only `db`, `jobs`, and `testkit` may depend on `sqlx`, only `api` on `axum`, and `app` is reachable only from the three binaries, the test kit, and the parity tool.

SQL lives in `crates/db/src/repos/`, orchestration in `crates/app/src/services/`, and HTTP handlers in `crates/api/src/routes/`. A handler extracts, calls a service, and maps the error.

## Testing

Tests live in each crate's `tests/` directory, not inline in `src/`.

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/myrtle_sqlx

cargo test -p myrtle-engine-base                             # One crate
cargo test --workspace --exclude myrtle-dps-transpile        # Everything CI runs
```

| Needs | Why |
|-------|-----|
| Postgres at `DATABASE_URL` | To compile, and for every database test. Point it at the migrated scratch database, as CI does; each test creates and drops its own database on that server |
| An extract at `MYRTLE_TEST_ASSETS` | Game-data tests. Defaults to `../assets/output`. A test whose data is missing skips; it does not fail |
| `external/ArknightsDpsCompare` | Only the `myrtle-dps-transpile` tests, which is why CI excludes them |

`crates/testkit` supplies the throwaway databases, the game-data fixtures, and a fake session store.

Tests run against whatever extract is on disk, and the game data changes with every update. Assert on bounds and invariants, not on exact counts from one version of the data.

`tools/parity` compares this backend's answers with the previous one's. See its [README](tools/parity/README.md).

## Development

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/myrtle_sqlx

cargo fmt                                  # Format
cargo clippy --workspace -- -D warnings    # Lint
cargo check --workspace --all-targets      # Type-check
cargo run -p xtask -- check-layers         # Layer rules
cargo run -p xtask -- gen-ts --check       # Bindings are current
```

Lint levels come from `[workspace.lints]` in `Cargo.toml`: `clippy::all`, `pedantic`, and `nursery` as warnings, `unsafe_code` denied, and `unwrap_used` and `dbg_macro` flagged outside tests.

CI ([`backend-ci.yml`](../.github/workflows/backend-ci.yml)) runs rustfmt, Clippy, `cargo check`, the workspace tests against the asset pipeline's `game-data` artifact, a release build of all four binaries, and a non-blocking `cargo audit`, with `RUSTFLAGS="-D warnings"` throughout. It does not run `check-layers`, `gen-ts --check`, or `cargo deny`, so run the first two yourself.

## Building for Production

```bash
cargo build --release -p myrtle-api -p myrtle-worker -p myrtle-cli -p myrtle-storage
```

This produces `target/release/myrtle-api`, `myrtle-worker`, `myrtle`, and `asset-sync`. The release profile uses thin LTO, one codegen unit, and stripped symbols. CI uploads the same four as the `backend-release-binaries` artifact on `main` and `v3`.

[`ecosystem.config.cjs`](ecosystem.config.cjs) runs the API and the worker under PM2 from `backend/`, each with `config/myrtle.toml` as its argument, a 3 GB memory restart, and a single worker instance. The API binds `127.0.0.1` in the example config, so put a reverse proxy in front and set `trusted_proxy_hops` to match.

## Contributing

1. Branch from `main`.
2. Put code in the lowest layer that can hold it, and run `cargo run -p xtask -- check-layers`.
3. Add a config key to `FIELDS` in `crates/app/src/config.rs` and regenerate the example. A new environment variable is not the way in.
4. Add schema changes as a new migration, and rebuild the scratch schema.
5. If a wire type changed, run `cargo run -p xtask -- gen-ts` and commit the bindings.
6. Run `cargo fmt`, `cargo clippy --workspace -- -D warnings`, and the tests of the crates you touched.

See the repo-level [CONTRIBUTING.md](../CONTRIBUTING.md) for the full workflow.

## License

License TBD - see the [project root](../README.md).

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
