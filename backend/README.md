# myrtle.moe Backend

High-performance REST API powering [myrtle.moe](https://myrtle.moe) - an Arknights companion platform. Serves game data, player-profile syncing, DPS/HPS calculations, RIIC base optimization, gacha tracking, a scoring leaderboard, and community tier lists. Built in Rust on Axum with PostgreSQL and Redis.

[![Rust](https://img.shields.io/badge/Rust-edition_2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-336791?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Backend CI](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

## Features

- **Game Data API** - operators, skills, modules, skins, materials, stages, enemies, gacha pools, voices, handbook, and more, served in-memory from extracted Arknights assets across six server regions.
- **Player Profile Sync** - log in with a Yostar email code (EN/JP/KR), a CN passport, or a Bilibili account, and pull your roster, inventory, and game state directly from Hypergryph servers.
- **DPS & HPS Calculators** - skill, total, and cycle-averaged damage and healing for 200+ operators, transpiled from the upstream Python reference and validated against it in CI.
- **RIIC Base Optimizer** - evaluates and optimizes base layouts from parsed `building_data.json`: shift rotations, dormitory morale, per-skill contribution ledgers, and trading-post order economics priced per post level.
- **Operator Planner** - per-operator promotion, level, skill, and module targets, organized into shareable plan groups.
- **Gacha Tracking** - fetch, dedupe, and store pull history; per-user and community-wide statistics, with full banner rate-ups read through service accounts.
- **Scoring & Leaderboard** - composite account scores with grades, rankings, movers, and distribution; recomputed on a schedule.
- **Community Tier Lists** - full CRUD with tiers, placements, versioning and publishing, favorites, flairs, and a 4-level permission system.
- **Operator Notes** - community guides with full audit logging.
- **Image Asset Serving** - avatars, portraits, char art, chibis, and icons resolved from the asset pipeline, with per-server fallback.
- **TypeScript Bindings** - every wire type exported to the frontend via `ts-rs`, guarded by a shape test.
- **Live Hot-Reload** - game data and DPS formulas update without downtime via background watchers.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (Edition 2024, 1.85+) |
| Runtime | Tokio (async, multi-threaded) |
| HTTP | Axum 0.8 + Tower / tower-http middleware |
| Database | PostgreSQL via `sqlx` 0.8 (runtime queries, so `SQLX_OFFLINE` builds need no `.sqlx` cache) |
| Cache | Redis (async connection manager) with in-memory fallback |
| Auth | JWT (HS256, 7-day expiry) + internal service key |
| Game Data | Loaded per server at startup into an `ArcSwap<GameData>` for lock-free hot-reload |
| Allocator | jemalloc (non-MSVC), to stop glibc arena fragmentation inflating RSS under load spikes |
| Bindings | `ts-rs` 11, exporting to `../frontend/src/types/generated` |

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | 1.85.0+ | Edition 2024 |
| PostgreSQL | 14+ | Migrations auto-apply on startup |
| Redis | 6+ | Optional - falls back to in-memory cache |
| Python 3 | 3.x | Only for (re)generating DPS/HPS formulas |
| Game Assets | - | Extracted via the [asset pipeline](../assets/README.md) |

## Quick Start

### 1. Configure environment

Copy the example and fill in secrets:

```bash
cp .env.example .env
```

Four variables panic on startup if unset:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/myrtle
JWT_SECRET=your-secret-here
SERVICE_KEY=your-service-key-here
GAME_CREDENTIAL_KEY=<32 bytes, hex or base64>

ASSETS_DIR=../assets/output
SERVERS=en
REDIS_URL=redis://127.0.0.1:6379
```

`GAME_CREDENTIAL_KEY` seals the durable Yostar credentials that let a gacha resync outlive its cache entry. Generate one with `openssl rand -hex 32`.

Game data is located as `{ASSETS_DIR}/{server}` for each entry in `SERVERS`, the first of which is the default server. There is no separate game-data path variable; `GAME_DATA_DIR` is read by the test suite only.

### 2. Run

```bash
cargo run --bin backend
```

The server starts on `http://localhost:3060` (override with `PORT`). All routes are prefixed with `/api`.

```bash
curl http://localhost:3060/api/health
```

Startup runs tens of seconds - game data per server, a 2.3 GB `activity_table`,
~2800 level files - so it draws a bar per phase (game data per server, database,
cache, hypergryph config, service accounts, jobs) under one bar for the whole
boot.

The estimate is measured rather than counted: each step's duration is written to
`startup_timings.json` and read back on the next boot, so the second boot on a
machine knows how long it has left. The first says `estimating`. Off a terminal
(pm2, Docker without `-t`) the bars become one log line per phase, with its
duration against what was expected. `NO_PROGRESS=1` takes that path on a terminal
too; `STARTUP_TIMINGS_FILE` moves the timing file; `STEP_RSS=1` adds a per-step
RSS trace on stderr.

If a non-default server fails to load, it falls back to serving the default server's data and is marked unloaded. If the *default* server fails, the process exits.

### 3. Docker

```bash
# Production (multi-stage, SQLX_OFFLINE build)
docker build -t myrtle-backend .
docker run -p 3060:3060 --env-file .env myrtle-backend

# Development (source mounted)
docker build -f Dockerfile.dev -t myrtle-backend-dev .
docker run -p 3060:3060 -v $(pwd):/src --env-file .env myrtle-backend-dev
```

## Architecture

```
HTTP Request
  │
  ▼
routes/        Thin handlers: extract params, call service, return JSON
  │
  ▼
services/      Business logic: caching, auth checks, orchestration
  │
  ▼
database/      Data access: sqlx queries, stored procedures
  │
  ▼
PostgreSQL     Normalized tables + JSONB game-state blobs
```

- **Game data** is loaded per server at startup into an `ArcSwap<GameData>` (lock-free reads, atomic swap) and hot-reloaded by the asset watcher. Bilibili shares CN's `Arc`.
- **Redis caches** are typed via a `CacheKey` enum that co-locates key generation and TTL. Cache ops are fire-and-forget: a Redis failure degrades to no-cache, never a 500.
- **`core/`** holds HTTP-independent domain logic (auth, game data, Hypergryph integration, scoring, background jobs).

## API Endpoints

Around 70 unique paths under `/api`, most of which also accept a `{server}` prefix. Auth is either a JWT Bearer token (`Yes`), optional (`Opt` - richer response when authenticated), or none (`No`).

### Health & Stats

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Service health with Redis/DB latency and per-server load status |
| GET | `/stats` | No | Public stats (cached) |
| GET | `/admin/stats` | Admin | Detailed admin dashboard |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/login/send-code` | No | Send Yostar verification email |
| POST | `/login` | No | Exchange code for JWT |
| POST | `/login/cn/send-code` · `/login/cn` | No | CN passport login |
| POST | `/login/bilibili` · `/login/bilibili/send-code` · `/login/bilibili/sms` | No | Bilibili login |
| GET | `/auth/verify` | Yes | Validate token |
| POST | `/auth/update-settings` | Yes | Update privacy settings |
| POST | `/auth/disconnect` | Yes | Disconnect the linked game account |
| POST | `/refresh` | No | Sync game data from the Arknights server |

### Users, Search & Social

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/get-user` | No | User profile (cached) |
| GET | `/get-user-score` | No | Composite score + grade |
| GET | `/get-user-checkin` | No | Check-in / daily supply state |
| GET | `/search` | No | Search users by nickname |
| GET | `/players/search` | Yes | In-game player search |
| GET | `/friends` | Yes | Authenticated user's friends |

### Leaderboard

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/leaderboard` | No | Paginated rankings |
| GET | `/leaderboard/movers` | No | Biggest rank changes |
| GET | `/leaderboard/distribution` | No | Score distribution |
| GET | `/leaderboard/standing` | No | A user's standing |
| GET | `/leaderboard/history` | No | A user's score history |

### Roster & User Data

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/roster` | Opt | Operators for a user |
| GET | `/roster/{operator_id}` | Opt | Single operator details |
| GET | `/get-user-supports` | No | Support units |
| GET | `/stage-clears` | No | Stage clear data |
| GET | `/encountered-enemies` | No | Enemies the user has met |
| GET | `/encountered-enemies/community-average` | No | Community baseline for the above |
| GET | `/inventory` | Opt | Item inventory |
| GET | `/user-skins` | Opt | Owned skins |
| GET | `/skins/popularity` | No | Skin popularity stats |
| GET | `/user/improvements` | Opt | Suggested account improvements |

### Planner

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/plans` | Yes | The user's operator plans |
| GET | `/plans/public` | No | Public plans |
| POST · DELETE | `/plan/{operator_id}` | Yes | Set or clear an operator's targets |
| POST | `/plan/group` | Yes | Create a plan group |
| PUT · DELETE | `/plan/group/{group_name}` | Yes | Rename or delete a plan group |

### Base / RIIC Optimizer

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/base/catalog` | No | Buff and facility catalog parsed from game data |
| GET | `/base/layout` | Opt | The user's current base layout |
| POST | `/base/evaluate` | Opt | Score a given layout |
| POST | `/base/optimize` | Opt | Solve for an improved layout |
| POST | `/base/rotation` | Opt | Shift rotation plan |
| PUT | `/base/facts` | Yes | Persist account facts the optimizer cannot infer |

### Gacha

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/gacha/fetch` | Yes | Fetch and store records from Yostar |
| GET | `/gacha/history` | Yes | Pull history (paginated) |
| GET | `/gacha/history/{char_id}` | Yes | History for one operator |
| GET | `/gacha/stored-records` | Yes | Raw stored records |
| GET | `/gacha/stats` | Yes | Per-user pull statistics |
| GET/POST | `/gacha/settings` | Yes | Read/update gacha settings |
| GET | `/gacha/global-stats` | No | Community-wide pull rates |
| GET | `/gacha/stats/enhanced` | No | Enhanced community stats |
| GET | `/gacha/stats/per-banner` | No | Per-banner community stats |

### Static Game Data

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/static/{resource}` | No | Game data JSON (cached) |
| GET | `/operators/index` | No | Lightweight operator index |
| GET | `/operators/{id}` | No | Single operator |
| GET | `/operators/ownership` | No | Community ownership rates |
| GET | `/upcoming` | No | Operators on CN not yet on the default (EN) server |
| GET | `/voices/{id}` | No | Voice lines for an operator |
| GET | `/chibis/{operator_id}` | No | Chibi Spine manifest |
| GET | `/skins/index` · `/skins/{id}` | No | Skin index and detail |
| GET | `/level/{stage_id}` | No | Level tile and route data |
| GET | `/stages/{stage_id}/detail` | No | Stage detail with waves and drops |
| GET | `/enemies/{id}` · `/enemies/{id}/stages` | No | Enemy detail and appearances |

`resource` is one of: `operators`, `skills`, `modules`, `skins`, `materials`, `stages`, `zones`, `enemies`, `gacha`, `voices`, `handbook`, `chibis`, `trust`, `ranges`.

### Image Assets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/avatar/{id}` | Operator avatar |
| GET | `/portrait/{id}` | Operator portrait |
| GET | `/charart/{id}` | Full character art |
| GET | `/skin-portrait/{id}` | Skin portrait |
| GET | `/skill-icon/{id}` | Skill icon |
| GET | `/module-icon/{id}` · `/module-big/{id}` | Module icons |
| GET | `/enemy-icon/{id}` · `/item-icon/{id}` · `/medal-icon/{id}` | Misc icons |
| GET | `/assets/{*path}` | Raw asset passthrough |

### Per-Server Data (multi-region)

The static game data, operator index, chibi, skin, voice, stage, enemy, image asset, and `/upcoming` routes all accept an optional `{server}` path prefix, for example `/cn/static/operators`, `/cn/portrait/{id}`, `/cn/operators/index`. The bare (unprefixed) routes use the default server.

| Token | Server |
|-------|--------|
| `en` | Global / EN (default) |
| `cn`, `bili` | CN (Bilibili shares CN data) |
| `jp`, `kr`, `tw` | Yostar / Gryphline |

Servers are loaded per the `SERVERS` env var (e.g. `SERVERS=en,cn`). For **data**
routes, a valid but unconfigured server (e.g. `/jp/...` when JP is not loaded) returns `404`, and an unknown token
returns `400`. **Asset** routes instead fall back to the default (EN) server when
the requested server has no entry for that asset. A server downloaded with the
operators-only profile (such as the CN preview) carries only operator-facing
assets, so anything it lacks - shared icons (elite/potential/item/camp/class), or
assets for an unconfigured server - is served from the default server. An asset
absent on both still returns `404`.

### DPS / HPS Calculator

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/dps/operators` | No | Supported operators (skills/modules/conditionals) |
| POST | `/dps/calculate` | No | Calculate DPS for operator + enemy config |
| GET | `/hps/operators` | No | Supported healers |
| POST | `/hps/calculate` | No | Calculate HPS for healer config |

### Tier Lists

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/tier-lists` | Yes | Create tier list |
| GET | `/tier-lists` | No | List active tier lists |
| GET | `/tier-lists/details` | No | Listing with tiers and placements inlined |
| GET | `/tier-lists/mine` | Yes | User's own tier lists |
| GET | `/tier-lists/favorites` | Yes | User's favorited lists |
| GET | `/tier-lists/{slug}` | No | Get list with tiers + placements |
| PUT · DELETE | `/tier-lists/{slug}` | Yes | Update / delete metadata |
| POST | `/tier-lists/{slug}/tiers` | Yes | Create tier row |
| PUT · DELETE | `/tier-lists/{slug}/tiers/{id}` | Yes | Update / delete tier row |
| POST | `/tier-lists/{slug}/placements` | Yes | Place operator |
| DELETE · PATCH | `/tier-lists/{slug}/placements/{op}` | Yes | Remove / update placement |
| POST | `/tier-lists/{slug}/placements/{op}/move` | Yes | Move operator between tiers |
| GET | `/tier-lists/{slug}/versions` | No | Version history |
| POST | `/tier-lists/{slug}/publish` | Yes | Publish snapshot |
| GET · POST | `/tier-lists/{slug}/permissions` | Yes | List / grant permissions |
| DELETE | `/tier-lists/{slug}/permissions/{uid}/{perm}` | Yes | Revoke permission |
| POST | `/tier-lists/{slug}/view` | No | Record a view |
| GET | `/tier-lists/{slug}/stats` | No | Engagement stats |
| GET · POST | `/tier-lists/{slug}/favorite` | Opt | Check / toggle favorite |
| PUT | `/tier-lists/{slug}/flair` | Yes | Set flair |
| PUT | `/tier-lists/{slug}/visibility` | Yes | Set visibility |
| GET · POST | `/tier-list-flairs` | Opt | List / create flairs |

### Operator Notes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/operator-notes` | No | All operator notes |
| GET | `/operator-notes/{id}` | No | Single note |
| PUT | `/operator-notes/{id}` | Admin | Update note (audit-logged) |
| GET | `/operator-notes/{id}/audit` | No | Change history |
| GET | `/admin/operator-notes/audit` | Admin | Global audit log |

## Background Jobs

Eight tasks are spawned at startup (`src/main.rs`) and run for the lifetime of the process. Set `DISABLE_BACKGROUND_JOBS=1` to skip all of them.

| Job | Module | Cadence / Trigger | Config |
|-----|--------|-------------------|--------|
| Asset hot-reload | `core/asset_watcher.rs` | On WebSocket `update_complete` | `ASSET_WS_URLS`, `ASSET_WS_URL` |
| DPS formula watcher | `core/dps_watcher.rs` | Polls upstream GitHub | `DPS_POLL_INTERVAL` (unset = disabled), `DPS_*` |
| Trending recompute | `core/trending_job.rs` | Every `TRENDING_RECOMPUTE_SECS` (900s) | `TRENDING_RECOMPUTE_SECS`, `TRENDING_TOP_N` |
| Leaderboard snapshot | `core/leaderboard_snapshot_job.rs` | Daily | - |
| Regrade | `core/regrade_job.rs` | Every `REGRADE_INTERVAL_SECS` (24h) | `REGRADE_INTERVAL_SECS`, `REGRADE_CONCURRENCY`, `REGRADE_PAGE_SIZE`, `REGRADE_STATE_FILE` |
| Operator ownership | `core/operator_ownership_job.rs` | Periodic | - |
| Medal ownership | `core/medal_ownership_job.rs` | Periodic | - |
| Gacha pool detail | `core/gacha_detail_job.rs` | Every `GACHA_DETAIL_REFRESH_SECS` (6h) | `GACHA_DETAIL_*` |

The asset watcher reloads game data, invalidates the static cache, and reconciles `gacha_records.rarity` against the new data. The DPS watcher fetches upstream changes, regenerates formulas, and optionally rebuilds/restarts the server. The gacha pool-detail job reads full banner rate-ups through the configured service accounts and checkpoints its progress.

If a game-data table fails to deserialize and silently falls back to its default, the boot reports it and, when `DISCORD_ALERT_WEBHOOK` is set, posts an alert.

## DPS & HPS Calculator

The engine calculates skill DPS/HPS, total output, and cycle-averaged values for 200+ operators. Rust functions are transpiled from the upstream Python reference ([ArknightsDpsCompare](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare)) and live in `src/dps/`.

### Pipeline

```
external/ArknightsDpsCompare/ (Python reference)
        │  generate-dps
        ▼
config/operator_formulas.json · config/heal_formulas.json   (metadata)
        │
        ▼
custom/generated.rs · custom/generated_hps.rs               (transpiled Rust)
        │  cargo build
        ▼
engine.rs  →  calculate_dps() / calculate_hps()
```

### Regenerating formulas

```bash
# DPS
cargo run --bin generate-dps -- --formulas          # regen operator_formulas.json
cargo run --bin generate-dps -- --transpile         # Python → Rust (~5 min)
cargo run --bin generate-dps -- --expected          # regen test fixtures

# HPS
cargo run --bin generate-dps -- --healing           # all HPS steps
cargo run --bin generate-dps -- --healing-formulas  # regen heal_formulas.json
cargo run --bin generate-dps -- --healing-transpile # Python → Rust

# Defaults to the full DPS pipeline when run with no flags.
# Use --repo <path> for a custom ArknightsDpsCompare checkout.
cargo run --bin generate-dps
```

`scripts/run-dps-tools.sh` installs the Python deps, runs the generator, then clippy-fixes and formats the output. See [`DPS_CALCULATOR.md`](DPS_CALCULATOR.md) for details, including the known Ebenholz parity failure (a bug in the Python reference, not in the Rust port).

## TypeScript Bindings

Every type crossing the wire is annotated with `#[ts(export)]` and exported by `ts-rs` into `../frontend/src/types/generated` (341 files today). The export directory is set in `.cargo/config.toml`, so generation is just a test run:

```bash
cargo test export_bindings            # write the bindings
cargo test --test bindings_shape_test # guard against wire-type drift
```

From the frontend, `bun run gen:types` does both. `bindings_shape_test` fails on two mistakes `ts-rs` makes silently: emitting `bigint` for `i64`/`u64` (JSON gives plain `number`) and emitting bare `any` where the recursive `JsonValue` type belongs.

Never hand-edit a generated file. Change the Rust struct and regenerate.

## CLI Tools

Ten binaries beyond the main server (`cargo run --bin <name>`):

| Binary | Purpose |
|--------|---------|
| `generate-dps` | Generate/transpile DPS & HPS formulas and test fixtures |
| `export-database` | Export the full database to versioned JSONL + manifest |
| `import-database` | Import a JSONL export (single txn; `--truncate` to replace) |
| `translate-backup` | Convert a legacy myrtle backup into a v3 import bundle |
| `regrade-users` | Recompute scores for all/one/by-server users (`--dry-run`) |
| `manage-permissions` | Interactive CLI for roles and per-tier-list permissions |
| `set-tier-list-official` | Mark a tier list as official |
| `resync-gacha` | Reconcile `gacha_records.rarity` against game data (`--dry-run`) |
| `export-gacha` | Export a single user's gacha history to JSON |
| `set-session` | Create or refresh a service-account game session |

Standalone binaries load the server named by `BIN_SERVER`, defaulting to the first entry in `SERVERS`.

## Project Structure

```
src/
├── main.rs                  Entry: boot phases, load game data, init DB/Redis, spawn jobs, run server
├── lib.rs                   Module exports
├── bin/                     10 CLI tools (see above)
├── db_export.rs             Shared export/import logic for the database binaries
│
├── app/                     HTTP application layer
│   ├── server.rs            Axum router (binds :3060, nests /api) + graceful shutdown
│   ├── state.rs             AppState / AppConfig
│   ├── error.rs             ApiError → HTTP status + JSON
│   ├── validation.rs        Request validation helpers
│   ├── cache/               Typed CacheKey enum + fire-and-forget store
│   ├── extractors/          AuthUser / MaybeAuthUser, Pagination, Validated<T>
│   ├── services/            Business logic, one file per domain
│   └── routes/              Thin handlers (+ tier_lists/ sub-router)
│
├── core/                    HTTP-independent domain logic
│   ├── auth/                JWT + permission/role enums
│   ├── gamedata/            init_game_data(), AssetIndex, types, enrich/
│   ├── hypergryph/          Yostar / CN passport / Bilibili login chains, signing, loaders
│   ├── grade/               Account scoring (operators / medals / stages / sandbox / roguelike / base)
│   ├── startup.rs           Boot phase bars, measured ETAs, optional RSS trace
│   ├── alerts.rs            Degraded-table reporting via Discord webhook
│   ├── service_account.rs   Backend-owned game accounts for authenticated reads
│   ├── asset_watcher.rs     Hot-reload WebSocket listener
│   ├── dps_watcher.rs       Upstream DPS repo poller
│   ├── gacha_resync.rs      Rarity reconciliation
│   ├── gacha_detail_job.rs  Periodic banner pool-detail refresh
│   ├── trending_job.rs      Tier-list trending recompute
│   ├── leaderboard_snapshot_job.rs
│   ├── operator_ownership_job.rs · medal_ownership_job.rs
│   └── regrade_job.rs       Periodic score recompute
│
├── database/                PostgreSQL layer
│   ├── pool.rs              Connection pool
│   ├── migrations/          13 SQL migrations (v001-v013, auto-applied)
│   ├── models/              Row types (FromRow + Serialize)
│   └── queries/             Data-access functions
│
├── dps/                     DPS & HPS calculation engine
│   ├── engine.rs            calculate_dps() / calculate_hps()
│   ├── operator_unit.rs     Shared params, buffs, enemy stats, shred
│   ├── formulas.rs          Shred / damage math
│   ├── config/              operator_formulas.json · heal_formulas.json (generated)
│   └── custom/              dispatch + generated.rs / generated_hps.rs (transpiled)
│
└── utils/                   Helpers (e.g. platform random bytes)
```

The base optimizer lives under `core/grade/base/`. `ledger.rs` is the scorer: it resolves per-room operator clauses into per-entity contributions, settling pools, then peer scaling, then suppression, in that order. Extend `ClauseKind` rather than special-casing a room.

## Database

13 migrations apply automatically on startup. Schema uses normalized tables for queryable data and JSONB for read-only game-state blobs.

`v001_initial` through `v005_indexes` are a **squashed baseline**, generated by `pg_dump` of the original v001-v022 migrations and split by object type. They deliberately reuse the original migration names, so a database that already ran the old migrations has those names recorded and skips the bodies. Add new work as a new migration after the baseline; never edit the baseline.

### Key tables

| Table | Purpose |
|-------|---------|
| `users` / `user_settings` / `user_status` | Account, privacy flags, currencies |
| `user_operators` | Roster (elite, level, potential, skills) |
| `user_game_credentials` | Sealed Yostar credentials for durable resync |
| `user_checkin` | Claim-order flag list, not a per-day calendar bitmap |
| `gacha_records` | Pull history (deduped) |
| `user_scores` | Composite scores + grade |
| `operator_plans` / `plan_groups` | Operator planner targets and groups |
| `leaderboard_snapshots` | Historical rankings |
| `operator_ownership_stats` / `medal_ownership_stats` | Community ownership rates |
| `tier_lists` / `tiers` / `tier_placements` | Tier list content |
| `tier_list_versions` / `tier_list_permissions` | Snapshots + access control |
| `operator_notes` | Community operator guides (audit-logged) |

### Views & procedures

| Object | Purpose |
|--------|---------|
| `v_user_profile` | Users + settings + status + scores joined |
| `v_user_roster` | Operators + masteries + modules aggregated |
| `v_leaderboard` | Rankings with `RANK()` window functions |
| `v_gacha_stats` | Per-user pull aggregates |
| `sp_sync_user_data` | Upsert user + replace roster/inventory/game state |
| `sp_insert_gacha_batch` | Batch insert gacha records with dedup |

## Authentication

Two mechanisms:

1. **JWT Bearer token** - user-facing requests. Created at `/login`, 7-day expiry. Claims: `user_id`, `uid`, `server`, `role`.
2. **Service key** - internal SSR (frontend → backend) via the `X-Service-Key` header. Grants SuperAdmin access.

### Permissions & roles

Tier-list operations use a 4-level hierarchy - **View → Edit → Publish → Admin** - checked in order: global role bypass → ownership → per-list grant.

| Role | Access |
|------|--------|
| `user` | Default |
| `tier_list_editor` | Needs explicit per-list grants |
| `tier_list_admin` | Admin on all tier lists |
| `super_admin` | Full system access |

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Secret for JWT signing |
| `SERVICE_KEY` | Internal service key for SSR bypass |
| `GAME_CREDENTIAL_KEY` | 32-byte key sealing durable game credentials |

### Core

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3060` | Bind port |
| `ASSETS_DIR` | `../assets/output` | Extracted assets root; game data is `{ASSETS_DIR}/{server}` |
| `SERVERS` | `en` | Comma-separated servers to load. The first is the default |
| `BIN_SERVER` | first of `SERVERS` | Server loaded by the standalone CLI binaries |
| `REDIS_URL` | _(in-memory fallback)_ | Redis connection string. The fallback is not multi-instance safe |
| `ASSET_WS_URLS` | - | Per-server hot-reload map, e.g. `en=ws://host:9160,cn=ws://host:9161` |
| `ASSET_WS_URL` | - | Single-server form (`disabled`/empty = off) |
| `RATE_LIMIT_RPM` | `100` | Requests per minute per IP |
| `RUST_LOG` | `backend=info,tower_http=info` | Tracing filter |
| `DISABLE_BACKGROUND_JOBS` | - | `1`/`true`/`yes` skips every watcher and cron job |
| `DISCORD_ALERT_WEBHOOK` | - | Posts an alert when a game-data table fails to deserialize |

### Game accounts and gacha

| Variable | Default | Description |
|----------|---------|-------------|
| `GAME_SESSION_DIR` | `game_sessions` | Per-server service-account session storage |
| `GAME_SESSION_MAX_AGE_SECS` | `1800` | Session reuse window |
| `GACHA_DETAIL_DIR` | derived under `ASSETS_DIR` | Pool-detail cache. Must be overridden when assets are mounted read-only |
| `GACHA_DETAIL_REFRESH_SECS` | `21600` | Pool-detail refresh interval |
| `GACHA_DETAIL_CALL_DELAY_MS` | `120` | Delay between pool-detail calls |
| `GACHA_DETAIL_MAX_FAILURES` | `5` | Failures before the job backs off |
| `GACHA_DETAIL_CHECKPOINT_EVERY` | `50` | Pools per checkpoint write |
| `DEVICE_ID` · `DEVICE_ID2` · `DEVICE_ID3` | - | Pin the Arknights device identity |
| `DEVICE_IDS_FILE` | `device_ids.json` | Device-identity persistence when not pinned |
| `HYPERGRYPH_CN_APP_CODE` | built-in | CN passport app code override |

### Jobs

| Variable | Default | Description |
|----------|---------|-------------|
| `TRENDING_RECOMPUTE_SECS` | `900` | Trending job interval |
| `TRENDING_TOP_N` | `10` | Entries surfaced as trending |
| `REGRADE_INTERVAL_SECS` | `86400` | Regrade job interval |
| `REGRADE_CONCURRENCY` | `4` | Parallel regrade workers |
| `REGRADE_PAGE_SIZE` | `500` | Users per keyset page |
| `REGRADE_STATE_FILE` | `regrade_state.json` | Last-run timestamp store |
| `DPS_POLL_INTERVAL` | _(unset = disabled)_ | DPS watcher poll interval (s) |
| `DPS_UPSTREAM_REPO` | `WhoAteMyCQQkie/ArknightsDpsCompare` | Repo to watch |
| `DPS_UPSTREAM_BRANCH` | `main` | Branch to track |
| `DPS_LOCAL_REPO_PATH` | `external/ArknightsDpsCompare` | Local checkout path |
| `DPS_AUTO_BUILD` | `true` | Rebuild after regeneration |
| `DPS_AUTO_RESTART` | `false` | Exit after rebuild (supervisor restarts) |
| `GITHUB_TOKEN` | - | Raises GitHub API rate limit |

### Diagnostics and tooling

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTUP_TIMINGS_FILE` | `startup_timings.json` | Where measured boot timings are stored |
| `STEP_RSS` | - | Per-boot-step RSS trace on stderr |
| `NO_PROGRESS` | - | Force log-line boot output on a terminal |
| `BASE_PIN_TRACE` | - | Trace base-optimizer pin/shift decisions |
| `PYTHON_BIN` | - | Python interpreter for `generate-dps` |
| `ADMIN_UID` | - | Attributes grants made by `manage-permissions` |
| `GAME_DATA_DIR` | `../assets/output/en/gamedata/excel` | **Tests only.** The server derives its path from `ASSETS_DIR` + `SERVERS` |

## Testing

```bash
cargo test          # Full suite
cargo check         # Type-check without building
cargo clippy        # Lints
```

Most integration tests need real extracted game data; point `GAME_DATA_DIR` and `ASSETS_DIR` at it. `enemy_chibi_smoke` skips itself when the Spine tree is absent.

| Test | Guards |
|------|--------|
| `dps_engine_test` · `hps_engine_test` | Parity against the Python reference for every operator × skill × module, within 0.15% (or ±1.0 damage / ±2.0 healing) |
| `api_shape_test` | A committed JSON-shape snapshot of every `/static/{resource}` payload, catching silent field renames |
| `bindings_shape_test` | `ts-rs` wire-type drift (`bigint`, bare `any`) |
| `gamedata_integrity_test` | Tables that silently deserialize to their default. A real incident: CN's `ClassifyType: "MEMENTO"` broke `item_table` for 1,556 items unnoticed |
| `base_optimizer_test` | Synergies, Control Center stacking, faction buffs, rotation, and grade against real game data |
| `grade_operators_test` | Operator score breakdown consistency |
| `module_order_test` | Deterministic module ordering (`HashMap` iteration order once varied per process) |
| `sandbox_universe_test` | Reclamation Algorithm node-count inflation |
| `enemy_chibi_smoke` | Enemy Spine export presence |

CI ([`backend-ci.yml`](../.github/workflows/backend-ci.yml)) runs rustfmt, Clippy with `pedantic` and `nursery`, `cargo check --all-targets`, the test suite against a game-data artifact from the assets pipeline, a release build, a Docker build, and a non-blocking `cargo audit`. When the game-data artifact is unavailable the DPS tests are skipped with a warning rather than failing.

## Contributing

1. Branch from `main` (or the active development branch).
2. Keep handlers thin - business logic belongs in `services/`, data access in `database/queries/`.
3. Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before opening a PR.
4. Do not hand-edit generated files (`src/dps/custom/generated*.rs`, `config/*_formulas.json`, `../frontend/src/types/generated/`) - regenerate them.
5. Add schema changes as a new migration. Never edit the squashed baseline.

See the repo-level [CONTRIBUTING.md](../CONTRIBUTING.md) for the full workflow.

## Roadmap

- [ ] Expand the scoring model with additional game-mode coverage.
- [ ] Surface DPS-watcher status via the admin dashboard.
- [ ] Run `gamedata_integrity_test` against a CN extract in CI, since CN receives new enum values first.

## License

License TBD - see the [project root](../README.md).

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
