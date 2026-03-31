# myrtle.moe Backend

REST API for Arknights game data, user profiles, DPS calculations, gacha tracking, and community tier lists. Built in Rust with Axum, PostgreSQL, and Redis.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (Edition 2024) |
| Runtime | Tokio (async, multi-threaded) |
| HTTP | Axum 0.8 + Tower middleware |
| Database | PostgreSQL (sqlx, compile-time query validation) |
| Cache | Redis (async connection manager) |
| Auth | JWT (HS256, 7-day expiry) |
| Game Data | In-memory from extracted Arknights assets |

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.85.0+ (Edition 2024) |
| PostgreSQL | 14+ |
| Redis | 6+ |
| Game Assets | Extracted via the [asset pipeline](../assets/README.md) |

## Quick Start

### 1. Environment

Create a `.env` file:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/myrtle
REDIS_URL=redis://127.0.0.1:6379
GAME_DATA_DIR=../assets/output/gamedata/excel
ASSETS_DIR=../assets/output
JWT_SECRET=your-secret-here
SERVICE_KEY=your-service-key-here
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
# Production
docker build -t myrtle-backend .
docker run -p 3060:3060 --env-file .env myrtle-backend

# Development
docker build -f Dockerfile.dev -t myrtle-backend-dev .
docker run -p 3060:3060 -v $(pwd):/src --env-file .env myrtle-backend-dev
```

## Architecture

```
HTTP Request
  |
  v
routes/          Thin handlers: extract params, call service, return JSON
  |
  v
services/        Business logic: caching, auth checks, orchestration
  |
  v
database/        Data access: sqlx queries, stored procedures
  |
  v
PostgreSQL       Normalized tables + JSONB game state blobs
```

Game data is loaded once at startup into an `Arc<GameData>` (lock-free, immutable). Redis caches are typed via a `CacheKey` enum that co-locates key generation and TTL.

## API Endpoints

42 endpoints across 12 domains. All routes are under `/api`.

### Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Service health with Redis/DB latency |

### Users

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/get-user?uid={uid}` | No | User profile (cached 10min) |
| GET | `/search?q={query}` | No | Search users by nickname |
| GET | `/leaderboard?sort={col}&server={srv}` | No | Paginated rankings |

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/login/send-code` | No | Send Yostar verification email |
| POST | `/login` | No | Exchange code for JWT |
| GET | `/auth/verify` | Yes | Validate token |
| POST | `/auth/update-settings` | Yes | Update privacy settings |
| POST | `/refresh` | Yes | Sync game data from Arknights server |

### Roster

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/roster` | Yes | All operators for authenticated user |
| GET | `/roster/{operator_id}` | Yes | Single operator details |

### Gacha

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/gacha/fetch` | Yes | Fetch and store records from Yostar |
| GET | `/gacha/history?rarity={n}` | Yes | Pull history (paginated) |
| GET | `/gacha/stats` | Yes | Per-user pull statistics |
| GET | `/gacha/global-stats` | No | Community-wide pull rates |

### Static Game Data

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/static/{resource}` | No | Game data (cached 30min) |

`resource` is one of: `operators`, `skills`, `modules`, `skins`, `materials`, `stages`, `zones`, `enemies`, `gacha`, `voices`, `handbook`, `chibis`, `trust`, `ranges`

### Assets

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/avatar/{id}` | No | Operator avatar PNG |
| GET | `/portrait/{id}` | No | Operator portrait PNG |

### DPS Calculator

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/dps/operators` | No | Supported operators with skills/modules/conditionals |
| POST | `/dps/calculate` | No | Calculate DPS for operator + enemy config |

### Tier Lists

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/tier-lists` | Yes | Create tier list (10 max per user) |
| GET | `/tier-lists` | No | List active tier lists |
| GET | `/tier-lists/mine` | Yes | User's own tier lists |
| GET | `/tier-lists/{slug}` | No | Get tier list with tiers + placements |
| PUT | `/tier-lists/{slug}` | Yes | Update metadata |
| POST | `/tier-lists/{slug}/tiers` | Yes | Create tier row |
| PUT | `/tier-lists/{slug}/tiers/{id}` | Yes | Update tier row |
| DELETE | `/tier-lists/{slug}/tiers/{id}` | Yes | Delete tier row |
| POST | `/tier-lists/{slug}/placements` | Yes | Place operator in tier |
| DELETE | `/tier-lists/{slug}/placements/{op}` | Yes | Remove operator |
| POST | `/tier-lists/{slug}/placements/{op}/move` | Yes | Move operator between tiers |
| GET | `/tier-lists/{slug}/versions` | No | Version history |
| POST | `/tier-lists/{slug}/publish` | Yes | Publish snapshot |
| GET | `/tier-lists/{slug}/permissions` | Yes | List permissions (admin) |
| POST | `/tier-lists/{slug}/permissions` | Yes | Grant permission (admin) |
| DELETE | `/tier-lists/{slug}/permissions/{uid}` | Yes | Revoke permission (admin) |

### Operator Notes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/operator-notes` | No | All operator notes |
| GET | `/operator-notes/{id}` | No | Single operator note |
| PUT | `/operator-notes/{id}` | Admin | Update note (with audit log) |
| GET | `/operator-notes/{id}/audit` | No | Change history |

### Statistics

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/stats` | No | Public stats (cached 5min) |
| GET | `/admin/stats` | Admin | Detailed admin dashboard |

## Project Structure

```
src/
├── main.rs                         Entry point: init DB, Redis, game data, start server
├── lib.rs                          Module exports
├── bin/
│   └── generate_dps.rs             DPS formula generator (Python → JSON → Rust)
│
├── app/                            HTTP application layer
│   ├── server.rs                   Axum router + graceful shutdown
│   ├── state.rs                    AppState (DB, Redis, GameData, config)
│   ├── error.rs                    ApiError → HTTP status + JSON body
│   ├── cache/
│   │   ├── keys.rs                 CacheKey enum (typed keys + TTLs)
│   │   └── store.rs                get/set/invalidate (fire-and-forget)
│   ├── extractors/
│   │   ├── auth.rs                 AuthUser (JWT + service key bypass)
│   │   ├── pagination.rs           Pagination (limit/offset with defaults)
│   │   └── validated.rs            Validated<T> (JSON + validator crate)
│   ├── services/                   Business logic (one file per domain)
│   │   ├── auth.rs                 Login, send-code, session caching
│   │   ├── user.rs                 User profile lookup + cache
│   │   ├── roster.rs               Game data sync (Hypergryph API → DB)
│   │   ├── gacha.rs                Fetch/store gacha, stats
│   │   ├── leaderboard.rs          Rankings with parallel queries
│   │   ├── search.rs               User search with cache
│   │   ├── static_data.rs          Game data serving (in-memory)
│   │   ├── stats.rs                Public + admin statistics
│   │   ├── dps.rs                  DPS calculator wrapper
│   │   ├── tier_list.rs            CRUD + permission checks
│   │   └── operator_notes.rs       Notes CRUD + audit diffing
│   └── routes/                     Thin HTTP handlers
│       ├── mod.rs                  Route mounting (42 endpoints)
│       ├── health.rs               Health check (DB + Redis ping)
│       ├── auth.rs                 Auth + refresh handlers
│       ├── user.rs, search.rs      User endpoints
│       ├── roster.rs               Roster endpoints
│       ├── gacha.rs                Gacha endpoints
│       ├── leaderboard.rs          Leaderboard endpoint
│       ├── static_data.rs          Static game data endpoint
│       ├── assets.rs               Avatar/portrait image serving
│       ├── dps.rs                  DPS calculator endpoints
│       ├── stats.rs                Statistics endpoints
│       ├── operator_notes.rs       Operator notes endpoints
│       └── tier_lists/             Tier list sub-router
│           ├── crud.rs             Create, get, list, update, mine
│           ├── placements.rs       Add, remove, move operators
│           ├── tiers.rs            Tier row CRUD
│           ├── versions.rs         Publish + version history
│           └── permissions.rs      Grant, revoke, list
│
├── core/                           Domain logic (no HTTP dependency)
│   ├── auth/
│   │   ├── jwt.rs                  Token creation + verification
│   │   └── permissions.rs          Permission/GlobalRole enums
│   ├── gamedata/
│   │   ├── mod.rs                  init_game_data() — loads 15 tables
│   │   ├── assets.rs               AssetIndex (directory scanner)
│   │   ├── types/                  Game data structs (18 files)
│   │   │   ├── mod.rs              GameData struct
│   │   │   ├── operator.rs         Operator, Phase, Skill, Talent
│   │   │   ├── skill.rs            Skill levels, blackboard
│   │   │   ├── module.rs           Modules, sub-professions
│   │   │   ├── material.rs         Items, recipes
│   │   │   ├── enemy.rs            Enemy stats
│   │   │   ├── stage.rs            Stages, zones
│   │   │   └── ...                 skin, gacha, medal, voice, etc.
│   │   └── enrich/                 Asset path resolution + data enrichment
│   └── hypergryph/                 Game server integration
│       ├── constants.rs            Server enum, domains, AuthSession
│       ├── session.rs              Full login chain (Yostar → U8 → game)
│       ├── yostar.rs               Yostar API (send-code, auth, portal)
│       ├── fetch.rs                Authenticated HTTP requests
│       ├── crypto.rs               MD5/HMAC signing for game API
│       ├── config.rs               Global config (device IDs, versions)
│       └── loaders/                Version, device ID, network config
│
├── database/                       PostgreSQL layer
│   ├── pool.rs                     Connection pool (20 max, 2 min)
│   ├── migrations/                 6 SQL migrations (auto-applied)
│   │   ├── v001_initial.sql        Core tables (users, roster, gacha, tiers)
│   │   ├── v002_views.sql          Views (profile, roster, leaderboard, stats)
│   │   ├── v003_triggers.sql       Audit triggers
│   │   ├── v004_procedures.sql     sp_sync_user_data, sp_insert_gacha_batch
│   │   ├── v005_indexes.sql        Performance indexes
│   │   └── v006_operator_notes.sql Operator notes + audit log
│   ├── models/                     Row types (FromRow + Serialize)
│   │   ├── user.rs                 UserProfile, User, UserSettings
│   │   ├── gacha.rs                GachaRecord, GachaStats
│   │   ├── roster.rs               RosterEntry (with masteries/modules)
│   │   ├── score.rs                LeaderboardEntry, UserScore
│   │   ├── tier_list.rs            TierList, Tier, Placement, Version, Permission
│   │   ├── operator_notes.rs       OperatorNote, AuditEntry
│   │   └── audit.rs                AuditLogEntry
│   └── queries/                    Data access functions
│       ├── users.rs                CRUD + search
│       ├── gacha.rs                Batch insert + history + stats
│       ├── roster.rs               Roster + full sync (stored procedure)
│       ├── score.rs                Leaderboard + score upsert
│       ├── tier_lists.rs           Full tier list CRUD + permissions
│       ├── operator_notes.rs       Notes CRUD + audit log
│       └── items.rs                Inventory queries
│
├── dps/                            DPS calculation engine
│   ├── engine.rs                   calculate_dps() + formula loading
│   ├── operator_unit.rs            OperatorUnit (resolved stats)
│   ├── operator_data.rs            OperatorData (extracted from game data)
│   ├── formulas.rs                 Shred application math
│   ├── config/
│   │   └── operator_formulas.json  200+ operator metadata (generated)
│   └── custom/
│       ├── mod.rs                  Dispatch by operator ID
│       └── generated.rs            200+ transpiled DPS functions
│
└── utils/
    └── random.rs                   Platform-specific random bytes
```

## Database

6 migrations applied automatically on startup. Schema uses normalized tables for queryable data and JSONB for read-only game state blobs.

### Key Tables

| Table | Purpose |
|-------|---------|
| `users` | Account (uid, server, nickname, level, role) |
| `user_settings` | Privacy (public_profile, store_gacha, share_stats) |
| `user_status` | Currencies, sanity, tickets |
| `user_operators` | Roster (elite, level, potential, skills) |
| `gacha_records` | Pull history (deduped on user+timestamp+char+pool) |
| `user_scores` | Composite scores + grade |
| `tier_lists` | Tier list metadata (slug, type, active flag) |
| `tiers` | Tier rows (S, A, B, C...) |
| `tier_placements` | Operator placements in tiers |
| `tier_list_versions` | Published snapshots |
| `tier_list_permissions` | Per-list access control |
| `operator_notes` | Community operator guides |

### Views

| View | Purpose |
|------|---------|
| `v_user_profile` | Users + settings + status + scores joined |
| `v_user_roster` | Operators + masteries + modules aggregated |
| `v_leaderboard` | Rankings with RANK() window functions |
| `v_gacha_stats` | Per-user pull aggregates |

### Stored Procedures

| Procedure | Purpose |
|-----------|---------|
| `sp_sync_user_data` | Upserts user + replaces all roster/inventory/game state (20 params) |
| `sp_insert_gacha_batch` | Batch inserts gacha records with dedup |

## DPS Calculator

The DPS engine calculates skill DPS, total damage, and cycle-averaged DPS for 200+ Arknights operators.

### Pipeline

```
external/ArknightsDpsCompare/ (Python reference)
    |
    v  generate-dps --formulas
operator_formulas.json (operator metadata)
    |
    v  generate-dps --transpile
custom/generated.rs (200+ Rust functions)
    |
    v  cargo build
engine.rs (calculate_dps)
```

```bash
# Regenerate formulas from Python source
cargo run --bin generate-dps -- --formulas

# Transpile Python DPS functions to Rust
cargo run --bin generate-dps -- --transpile

# Generate expected values for testing
cargo run --bin generate-dps -- --expected
```

### Calculation Flow

1. Look up operator formula (skills, modules, conditionals)
2. Build `OperatorUnit` with resolved stats (ATK, interval, talents, buffs)
3. Apply enemy shreds (flat then percentage, DEF and RES)
4. Dispatch to operator-specific function
5. Apply external fragile buff
6. Calculate total damage (skill DPS x duration)
7. Calculate average DPS (skill + off-skill over SP cycle)

## Caching Strategy

| Key | TTL | Description |
|-----|-----|-------------|
| `user:{uid}` | 10min | User profile |
| `stats:global` | 5min | Public statistics |
| `static:{resource}:...` | 30min | Game data JSON |
| `leaderboard:{sort}:{srv}:{page}` | 5min | Leaderboard page |
| `search:{hash}` | 2min | Search results |
| `tierlist:{slug}` | 10min | Tier list data |
| `game_session:{uid}` | 1hr | Arknights game session |
| `portal_session:{uid}` | 1hr | Yostar portal cookies |
| `gacha:global_stats` | 5min | Community gacha rates |

Cache operations are fire-and-forget. A Redis failure degrades to no-cache, never to a 500.

## Authentication

Two auth mechanisms:

1. **JWT Bearer token** — For user-facing requests. Created during `/login`, 7-day expiry. Claims: user_id, uid, server, role.

2. **Service key** — For internal SSR (Next.js frontend calling backend). Passed via `X-Service-Key` header. Grants SuperAdmin access.

### Permission Hierarchy

Tier list operations use a 4-level permission system:

```
View → Edit → Publish → Admin
```

Checked in order: global role (SuperAdmin/TierListAdmin bypass) → ownership → per-list permission grant.

### Global Roles

| Role | Access |
|------|--------|
| `user` | Default, no special permissions |
| `tier_list_editor` | Needs explicit per-list grants |
| `tier_list_admin` | Admin on all tier lists |
| `super_admin` | Full system access |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `REDIS_URL` | No | `redis://127.0.0.1:6379` | Redis connection string |
| `GAME_DATA_DIR` | No | `../assets/output/gamedata/excel` | Path to extracted game data JSON |
| `ASSETS_DIR` | No | `../assets/output` | Path to extracted assets (textures, portraits) |
| `JWT_SECRET` | Yes | - | Secret for JWT signing |
| `SERVICE_KEY` | Yes | - | Internal service key for SSR bypass |
| `RATE_LIMIT_RPM` | No | `100` | Requests per minute per IP |
| `RUST_LOG` | No | `backend=info,tower_http=info` | Tracing log filter |

## Testing

```bash
# DPS calculator integration tests
cargo test

# Check compilation
cargo check
```

The DPS test suite validates Rust calculations against the Python reference implementation with a tolerance of 0.15% or 1.0 damage.


## TODO
- Listen to asset websocket and reload game data
- Small script or function that autochecks when the DPS calculator Python repository updates, and then pull and re-generate everything.
