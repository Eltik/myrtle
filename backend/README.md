# myrtle.moe Backend

High-performance REST API powering [myrtle.moe](https://myrtle.moe) - an Arknights companion platform. Serves game data, player-profile syncing, DPS/HPS calculations, gacha tracking, a scoring leaderboard, and community tier lists. Built in Rust on Axum with PostgreSQL and Redis.

[![Rust](https://img.shields.io/badge/Rust-edition_2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-336791?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

## Features

- **Game Data API** - operators, skills, modules, skins, materials, stages, enemies, gacha pools, voices, handbook, and more, served in-memory from extracted Arknights assets.
- **Player Profile Sync** - log in with a Yostar email code and pull your roster, inventory, and game state directly from Hypergryph servers.
- **DPS & HPS Calculators** - skill/total/average damage and healing for 200+ operators, transpiled from the upstream Python reference and validated against it in CI.
- **Gacha Tracking** - fetch, dedupe, and store pull history; per-user and community-wide statistics.
- **Scoring & Leaderboard** - composite account scores with grades, rankings, movers, and distribution; recomputed on a schedule.
- **Community Tier Lists** - full CRUD with tiers, placements, versioning/publishing, favorites, flairs, and a 4-level permission system.
- **Operator Notes** - community guides with full audit logging.
- **Image Asset Serving** - avatars, portraits, char art, and icons resolved from the asset pipeline.
- **Live Hot-Reload** - game data and DPS formulas update without downtime via background watchers.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (Edition 2024, 1.85+) |
| Runtime | Tokio (async, multi-threaded) |
| HTTP | Axum 0.8 + Tower / tower-http middleware |
| Database | PostgreSQL via `sqlx` (compile-time checked queries) |
| Cache | Redis (async connection manager) with in-memory fallback |
| Auth | JWT (HS256, 7-day expiry) + internal service key |
| Game Data | Loaded once at startup into an immutable `Arc<GameData>` |

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

Minimum required (`DATABASE_URL`, `JWT_SECRET`, and `SERVICE_KEY` panic on startup if unset):

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/myrtle
JWT_SECRET=your-secret-here
SERVICE_KEY=your-service-key-here
GAME_DATA_DIR=../assets/output/gamedata/excel
ASSETS_DIR=../assets/output
REDIS_URL=redis://127.0.0.1:6379
```

### 2. Run

```bash
cargo run --bin backend
```

The server starts on `http://localhost:3060`. All routes are prefixed with `/api`.

```bash
curl http://localhost:3060/api/health
```

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

- **Game data** is loaded once at startup into an `Arc<GameData>` (lock-free, immutable) and hot-reloaded by the asset watcher.
- **Redis caches** are typed via a `CacheKey` enum that co-locates key generation and TTL. Cache ops are fire-and-forget: a Redis failure degrades to no-cache, never a 500.
- **`core/`** holds HTTP-independent domain logic (auth, game data, Hypergryph integration, scoring, background jobs).

## API Endpoints

~80 endpoints across 14 domains, all under `/api`. Auth is either a JWT Bearer token (`Yes`), optional (`Opt` - richer response when authenticated), or none (`No`).

### Health & Stats

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Service health with Redis/DB latency |
| GET | `/stats` | No | Public stats (cached) |
| GET | `/admin/stats` | Admin | Detailed admin dashboard |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/login/send-code` | No | Send Yostar verification email |
| POST | `/login` | No | Exchange code for JWT |
| GET | `/auth/verify` | Yes | Validate token |
| POST | `/auth/update-settings` | Yes | Update privacy settings |
| POST | `/refresh` | No | Sync game data from Arknights server |

### Users, Search & Social

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/get-user` | No | User profile (cached) |
| GET | `/get-user-score` | No | Composite score + grade |
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

### Roster & User Data

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/roster` | Opt | Operators for a user |
| GET | `/roster/{operator_id}` | Opt | Single operator details |
| GET | `/get-user-supports` | No | Support units |
| GET | `/stage-clears` | No | Stage clear data |
| GET | `/inventory` | Opt | Item inventory |
| GET | `/user-skins` | Opt | Owned skins |
| GET | `/skins/popularity` | No | Skin popularity stats |
| GET | `/user/improvements` | Opt | Suggested account improvements |

### Gacha

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/gacha/fetch` | Yes | Fetch & store records from Yostar |
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
| GET | `/upcoming` | No | Operators on CN not yet on the default (EN) server |

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

The Static Game Data, operator index, image asset, and `/upcoming` routes above
also accept an optional `{server}` path prefix to select a server's data, e.g.
`/cn/static/operators`, `/cn/portrait/{id}`, `/cn/operators/index`, `/cn/upcoming`.
The bare (unprefixed) routes use the default server.

| Token | Server |
|-------|--------|
| `en` | Global / EN (default) |
| `cn`, `bili` | CN (Bilibili shares CN data) |
| `jp`, `kr`, `tw` | Yostar / Gryphline |

Servers are loaded per the `SERVERS` env var (e.g. `SERVERS=en,cn`). For **data**
routes (static game data, operator index, `/upcoming`), a valid but unconfigured
server (e.g. `/jp/...` when JP is not loaded) returns `404`, and an unknown token
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
| GET | `/tier-lists/mine` | Yes | User's own tier lists |
| GET | `/tier-lists/favorites` | Yes | User's favorited lists |
| GET | `/tier-lists/{slug}` | No | Get list with tiers + placements |
| PUT · DELETE | `/tier-lists/{slug}` | Yes | Update / delete metadata |
| POST | `/tier-lists/{slug}/tiers` | Yes | Create tier row |
| PUT · DELETE | `/tier-lists/{slug}/tiers/{id}` | Yes | Update / delete tier row |
| POST | `/tier-lists/{slug}/placements` | Yes | Place operator |
| DELETE | `/tier-lists/{slug}/placements/{op}` | Yes | Remove operator |
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

Five tasks are spawned at startup (`src/main.rs`) and run for the lifetime of the process:

| Job | Module | Cadence / Trigger | Config |
|-----|--------|-------------------|--------|
| Asset hot-reload | `core/asset_watcher.rs` | On WebSocket `update_complete` | `ASSET_WS_URL` |
| DPS formula watcher | `core/dps_watcher.rs` | Polls upstream GitHub | `DPS_POLL_INTERVAL` (unset = disabled), `DPS_*` |
| Trending recompute | `core/trending_job.rs` | Every `TRENDING_RECOMPUTE_SECS` (900s) | `TRENDING_RECOMPUTE_SECS`, `TRENDING_TOP_N` |
| Leaderboard snapshot | `core/leaderboard_snapshot_job.rs` | Daily | - |
| Regrade | `core/regrade_job.rs` | Every `REGRADE_INTERVAL_SECS` (24h) | `REGRADE_INTERVAL_SECS`, `REGRADE_CONCURRENCY`, `REGRADE_PAGE_SIZE`, `REGRADE_STATE_FILE` |

The asset watcher reloads game data, invalidates the static cache, and reconciles `gacha_records.rarity` against the new data. The DPS watcher fetches upstream changes, regenerates formulas, and optionally rebuilds/restarts the server.

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

`scripts/run-dps-tools.sh` installs the Python deps, runs the generator, then clippy-fixes and formats the output. See [`DPS_CALCULATOR.md`](DPS_CALCULATOR.md) for details.

## CLI Tools

Eight binaries beyond the main server (`cargo run --bin <name>`):

| Binary | Purpose |
|--------|---------|
| `generate-dps` | Generate/transpile DPS & HPS formulas and test fixtures |
| `export-database` | Export the full database to versioned JSONL + manifest |
| `import-database` | Import a JSONL export (single txn; `--truncate` to replace) |
| `translate-backup` | Convert a legacy myrtle backup into a v3 import bundle |
| `regrade-users` | Recompute scores for all/one/by-server users (`--dry-run`) |
| `manage-permissions` | Interactive CLI for roles and per-tier-list permissions |
| `resync-gacha` | Reconcile `gacha_records.rarity` against game data (`--dry-run`) |
| `export-gacha` | Export a single user's gacha history to JSON |

## Project Structure

```
src/
├── main.rs                  Entry: load game data, init DB/Redis, spawn jobs, run server
├── lib.rs                   Module exports
├── bin/                     8 CLI tools (see above)
│
├── app/                     HTTP application layer
│   ├── server.rs            Axum router (binds :3060, nests /api) + graceful shutdown
│   ├── state.rs             AppState / AppConfig
│   ├── error.rs             ApiError → HTTP status + JSON
│   ├── cache/               Typed CacheKey enum + fire-and-forget store
│   ├── extractors/          AuthUser / MaybeAuthUser, Pagination, Validated<T>
│   ├── services/            Business logic, one file per domain
│   └── routes/              Thin handlers (+ tier_lists/ sub-router)
│
├── core/                    HTTP-independent domain logic
│   ├── auth/                JWT + permission/role enums
│   ├── gamedata/            init_game_data(), AssetIndex, types, enrich/
│   ├── hypergryph/          Yostar → U8 → game login chain, signing, loaders
│   ├── grade/               Account scoring (base / stages / sandbox)
│   ├── asset_watcher.rs     Hot-reload WebSocket listener
│   ├── dps_watcher.rs       Upstream DPS repo poller
│   ├── gacha_resync.rs      Rarity reconciliation
│   ├── trending_job.rs      Tier-list trending recompute
│   ├── leaderboard_snapshot_job.rs
│   └── regrade_job.rs       Periodic score recompute
│
├── database/                PostgreSQL layer
│   ├── pool.rs              Connection pool
│   ├── migrations/          16 SQL migrations (v001–v016, auto-applied)
│   ├── models/              Row types (FromRow + Serialize)
│   └── queries/             Data-access functions
│
├── dps/                     DPS & HPS calculation engine
│   ├── engine.rs            calculate_dps() / calculate_hps()
│   ├── operator_unit.rs     Shared params, buffs, enemy stats, shred
│   ├── formulas.rs          Shred / damage math
│   ├── config/              operator_formulas.json · heal_formulas.json (generated)
│   └── custom/              dispatch + generated(.rs) / generated_hps.rs (transpiled)
│
└── utils/                   Helpers (e.g. platform random bytes)
```

## Database

16 migrations apply automatically on startup. Schema uses normalized tables for queryable data and JSONB for read-only game-state blobs.

### Key tables

| Table | Purpose |
|-------|---------|
| `users` / `user_settings` / `user_status` | Account, privacy flags, currencies |
| `user_operators` | Roster (elite, level, potential, skills) |
| `gacha_records` | Pull history (deduped) |
| `user_scores` | Composite scores + grade |
| `leaderboard_snapshots` | Historical rankings |
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

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `JWT_SECRET` | Yes | - | Secret for JWT signing |
| `SERVICE_KEY` | Yes | - | Internal service key for SSR bypass |
| `GAME_DATA_DIR` | No | `../assets/output/gamedata/excel` | Extracted game-data JSON |
| `ASSETS_DIR` | No | `../assets/output` | Extracted assets root |
| `REDIS_URL` | No | _(in-memory fallback)_ | Redis connection string |
| `ASSET_WS_URL` | No | - | Asset hot-reload WebSocket (`disabled`/empty = off) |
| `RATE_LIMIT_RPM` | No | `100` | Requests per minute per IP |
| `RUST_LOG` | No | `backend=info,tower_http=info` | Tracing filter |
| `TRENDING_RECOMPUTE_SECS` | No | `900` | Trending job interval |
| `TRENDING_TOP_N` | No | `10` | Entries surfaced as trending |
| `REGRADE_INTERVAL_SECS` | No | `86400` | Regrade job interval |
| `REGRADE_CONCURRENCY` | No | `4` | Parallel regrade workers |
| `REGRADE_PAGE_SIZE` | No | `500` | Users per keyset page |
| `REGRADE_STATE_FILE` | No | `regrade_state.json` | Last-run timestamp store |
| `DPS_POLL_INTERVAL` | No | _(unset = disabled)_ | DPS watcher poll interval (s) |
| `DPS_UPSTREAM_REPO` | No | `WhoAteMyCQQkie/ArknightsDpsCompare` | Repo to watch |
| `DPS_UPSTREAM_BRANCH` | No | `main` | Branch to track |
| `DPS_LOCAL_REPO_PATH` | No | `external/ArknightsDpsCompare` | Local checkout path |
| `DPS_AUTO_BUILD` | No | `true` | Rebuild after regeneration |
| `DPS_AUTO_RESTART` | No | `false` | Exit after rebuild (supervisor restarts) |
| `GITHUB_TOKEN` | No | - | Raises GitHub API rate limit |

## Testing

```bash
cargo test          # DPS + HPS engine parity tests
cargo check         # Type-check without building
cargo clippy        # Lints
```

The engine tests validate Rust DPS/HPS output against the Python reference for every operator × skill × module combination, within a tolerance of 0.15% (or ±1.0 damage / ±2.0 healing). Fixtures live in `tests/fixtures/`.

## Contributing

1. Branch from `main` (or the active development branch).
2. Keep handlers thin - business logic belongs in `services/`, data access in `database/queries/`.
3. Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before opening a PR.
4. Do not hand-edit generated files (`src/dps/custom/generated*.rs`, `config/*_formulas.json`) - regenerate them with `generate-dps`.

See the repo-level [CONTRIBUTING.md](../CONTRIBUTING.md) for the full workflow.

## Roadmap

- [ ] Expand the scoring model with additional game-mode coverage.
- [ ] Surface DPS-watcher status via the admin dashboard.

## License

License TBD - see the [project root](../README.md).

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
