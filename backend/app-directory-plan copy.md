# `/app` Directory Plan — myrtle.moe v3 Backend

## Context

**Current state:** The v3 backend has strong foundations — game data loading, DPS engine, database schema with migrations, auth/JWT, and Hypergryph integration — but **no HTTP layer**. `main.rs` is a standalone game data loader. There is no Axum, no Redis, no route handlers.

**Old backend:** A working Axum-based `/app` with ~40 endpoints across 12 route modules, Redis caching, rate limiting, and a full tier list system. However, it has structural issues worth addressing:

| Issue | Old Backend | Proposed Fix |
|-------|-------------|--------------|
| Monolithic server.rs (14.5KB) | All route mounting + state setup in one file | Split into focused modules |
| Oversized route handlers | gacha.rs (20.2KB), placements.rs (14.7KB) | Extract shared logic into service layer |
| Business logic in handlers | DB queries, caching, validation mixed into handlers | Clean handler → service → repository layering |
| Inconsistent module structure | Some routes have types.rs, some don't | Standardized module template |
| Hard-coded operator mappings | 281 operators in DPS utils.rs | Derive from game data at startup |
| No request validation layer | Validation scattered per-handler | Extractor-based validation middleware |
| Minimal error taxonomy | Generic ApiError enum | Typed domain errors with consistent JSON shape |
| Cache keys are ad-hoc strings | Concatenated strings like `leaderboard:{sort}:{order}:...` | Typed cache key builder |

---

## Architecture: Three-Layer Design

```
┌─────────────────────────────────────────────────┐
│  app/routes/        (HTTP layer)                │
│  - Extract & validate request                    │
│  - Call service                                  │
│  - Map service result → HTTP response            │
│  - No business logic, no direct DB access        │
├─────────────────────────────────────────────────┤
│  app/services/      (Business logic layer)       │
│  - Orchestrates operations                       │
│  - Caching decisions (read-through, invalidate)  │
│  - Authorization checks                          │
│  - Calls repository for data access              │
├─────────────────────────────────────────────────┤
│  database/queries/  (Repository layer - exists)  │
│  - Pure data access, no business logic           │
│  - Already built in v3                           │
└─────────────────────────────────────────────────┘
```

**Why a service layer?** The old backend mixed caching, auth checks, and DB queries directly in handlers. This made handlers bloated and hard to test. A service layer:
- Keeps handlers thin (extract request → call service → return response)
- Makes caching strategy testable independent of HTTP
- Allows services to compose (e.g., `refresh` calls `user_service` + `gacha_service`)
- Enables future gRPC/WebSocket interfaces without duplicating logic

---

## Proposed Directory Structure

```
src/app/
├── mod.rs                      # pub mod declarations
├── server.rs                   # Axum router assembly + listener startup (~50 lines)
├── state.rs                    # AppState: db pool, redis, game data, config
├── error.rs                    # ApiError enum → (StatusCode, JSON) with From impls
│
├── extractors/                 # Custom Axum extractors
│   ├── mod.rs
│   ├── auth.rs                 # AuthUser extractor (JWT → user claims)
│   ├── pagination.rs           # Pagination extractor with defaults/limits
│   └── validated.rs            # Validated<T> wrapper using validator crate
│
├── middleware/
│   ├── mod.rs
│   ├── rate_limit.rs           # Token-bucket per IP (tower layer)
│   ├── request_id.rs           # X-Request-Id propagation for tracing
│   └── service_key.rs          # Internal service key bypass (for frontend SSR)
│
├── services/                   # Business logic, one file per domain
│   ├── mod.rs
│   ├── user.rs                 # get_user, search_users
│   ├── auth.rs                 # login, verify, refresh session
│   ├── gacha.rs                # fetch_records, store_records, stats
│   ├── roster.rs               # sync_roster, get_operators
│   ├── leaderboard.rs          # ranked_users, score_breakdown
│   ├── tier_list.rs            # CRUD, versioning, permissions, moderation
│   ├── dps.rs                  # calculate, list_operators (thin wrapper over dps::engine)
│   ├── static_data.rs          # serve game data with field selection + pagination
│   ├── stats.rs                # global + admin stats aggregation
│   └── operator_notes.rs       # notes CRUD + audit
│
├── cache/                      # Typed Redis caching layer
│   ├── mod.rs
│   ├── keys.rs                 # CacheKey enum → deterministic string keys
│   └── store.rs                # get/set/invalidate with TTL, serialization
│
└── routes/                     # Thin HTTP handlers, grouped by domain
    ├── mod.rs                  # Mounts all sub-routers
    ├── user.rs                 # GET /get-user
    ├── auth.rs                 # POST /login, /login/send-code, /auth/verify, /auth/update-settings
    ├── gacha.rs                # POST /gacha/all, GET /gacha/stats, /gacha/history
    ├── search.rs               # GET /search
    ├── leaderboard.rs          # GET /leaderboard
    ├── tier_lists/             # Sub-router (largest domain, warrants its own dir)
    │   ├── mod.rs              # Mounts all tier list routes
    │   ├── crud.rs             # create, get, list, update, mine
    │   ├── placements.rs       # add, move, remove, reorder operators
    │   ├── tiers.rs            # create, update, delete tier rows
    │   ├── versions.rs         # publish, changelog, history
    │   ├── permissions.rs      # grant, revoke, list permissions
    │   └── moderation.rs       # report, review reports
    ├── dps.rs                  # GET /dps/operators, POST /dps/calculate
    ├── static_data.rs          # GET /static/{resource} (single handler, resource as path param)
    ├── stats.rs                # GET /stats, GET /admin/stats
    ├── operator_notes.rs       # GET/PUT /operator-notes/{id}
    └── assets.rs               # GET /avatar/{id}, /portrait/{id}
```

---

## Detailed Component Specs

### 1. `server.rs` — Router Assembly

Keep this file small. Its only job is to build the Axum `Router` and start listening.

```
pub async fn run(state: AppState) -> Result<()>
  1. Build Router::new()
  2. Nest route groups: /api/v1/... (version prefix for future compat)
  3. Apply middleware layers (rate limit, request ID, CORS, compression)
  4. Bind to 0.0.0.0:3060
  5. axum::serve(...).await
```

**Improvement over old:** Old `server.rs` was 14.5KB because it mounted every route inline and initialized all state. New version delegates route mounting to `routes/mod.rs` and state creation to a builder.

### 2. `state.rs` — Application State

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,          // deadpool-redis or bb8-redis
    pub game_data: Arc<GameData>,  // Immutable after init, shared cheaply
    pub config: Arc<AppConfig>,    // JWT secret, rate limits, feature flags
}
```

**Key decision:** `GameData` is loaded once at startup and wrapped in `Arc`. No mutex needed since it's immutable. If hot-reloading game data is desired later, swap to `Arc<ArcSwap<GameData>>`.

### 3. `error.rs` — Error Taxonomy

```rust
pub enum ApiError {
    // Client errors
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    Conflict(String),           // e.g., duplicate placement
    ValidationFailed(Vec<FieldError>),

    // Server errors
    Internal(anyhow::Error),    // Logs full error, returns generic 500
    ServiceUnavailable,         // Redis/DB down
}
```

All variants serialize to:
```json
{ "error": { "code": "VALIDATION_FAILED", "message": "...", "details": [...] } }
```

**Improvement over old:** Old backend had a minimal enum. New version gives the frontend structured errors it can act on programmatically.

### 4. `extractors/` — Request Validation

**`auth.rs` — `AuthUser` extractor:**
- Reads `Authorization: Bearer <token>` header
- Falls back to internal service key header (for SSR frontend)
- Rejects with 401 if neither present/valid
- Provides `AuthUser { user_id, uid, server, role }` to handlers

**`pagination.rs` — `Pagination` extractor:**
- Query params: `?limit=20&offset=0`
- Enforces max limit (e.g., 100)
- Defaults: limit=20, offset=0

**`validated.rs` — `Validated<T>` wrapper:**
- Deserializes JSON body, then runs `validator` crate checks
- Returns 422 with field-level errors on failure
- Eliminates per-handler validation boilerplate

### 5. `services/` — Business Logic

Each service receives `&AppState` (or specific pools) and returns `Result<T, ApiError>`.

**Example — `user.rs`:**
```
pub async fn get_user(state: &AppState, uid: &str) -> Result<UserProfile, ApiError>
  1. Check Redis cache (key: user:{uid}, TTL: 10min)
  2. Cache miss → query v_user_profile view
  3. Store in Redis, return
```

**Example — `tier_list.rs`:**
```
pub async fn create(state, auth, input) -> Result<TierList, ApiError>
  1. Check auth.role ≥ User
  2. Count user's existing lists (limit: 10 for community)
  3. Generate slug from name
  4. INSERT into tier_lists
  5. Return created list

pub async fn add_placement(state, auth, slug, input) -> Result<Placement, ApiError>
  1. Load tier list by slug
  2. Check permission ≥ Edit (via tier_list_permissions or global role)
  3. Check operator not already placed
  4. INSERT into tier_placements
  5. Invalidate tier list cache
  6. Return placement
```

**Key rule:** Handlers never touch `sqlx` or `redis` directly. They call services, which call the repository layer (`database::queries::*`) for DB access and `cache::store` for Redis.

### 6. `cache/` — Typed Caching

**`keys.rs`:**
```rust
pub enum CacheKey<'a> {
    User { uid: &'a str },
    Leaderboard { sort: &'a str, server: Option<&'a str>, page: u32 },
    Search { query_hash: u64 },
    Stats,
    StaticData { resource: &'a str, fields_hash: u64, page: u32 },
    TierList { slug: &'a str },
}

impl CacheKey<'_> {
    pub fn to_string(&self) -> String { ... }
    pub fn ttl(&self) -> Duration { ... }  // TTL co-located with key definition
}
```

**Improvement over old:** Old backend scattered cache key construction across handlers. New version makes keys typed, self-documenting, and impossible to construct incorrectly.

**`store.rs`:**
```
pub async fn get<T: DeserializeOwned>(redis, key) -> Option<T>
pub async fn set<T: Serialize>(redis, key, value) -> Result<()>  // uses key.ttl()
pub async fn invalidate(redis, key) -> Result<()>
pub async fn invalidate_prefix(redis, prefix) -> Result<()>      // e.g., all tier_list:*
```

### 7. `routes/` — Thin Handlers

**Pattern for every handler:**
```rust
pub async fn get_user(
    State(state): State<AppState>,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let user = services::user::get_user(&state, &params.uid).await?;
    Ok(Json(user))
}
```

That's it. No caching logic, no DB queries, no validation beyond what extractors provide.

### 8. `routes/static_data.rs` — Unified Static Endpoint

**Old approach:** 16 separate endpoint files (operators.rs, skills.rs, modules.rs, ...), each with similar boilerplate for caching, pagination, field selection, and compression.

**New approach:** Single handler with resource as a path parameter:

```
GET /static/{resource}?fields=name,rarity&limit=50&offset=0

resource ∈ {operators, skills, modules, skins, chibis, gacha, stages,
            enemies, zones, voices, handbook, materials, ranges, trust}
```

A `StaticResource` enum maps each resource to its data accessor on `GameData` and its serializable type. Field selection and pagination are handled generically.

**Why:** Eliminates ~2KB × 16 = ~32KB of near-identical boilerplate. Adding a new static resource becomes a 3-line enum variant instead of a new file.

### 9. `routes/tier_lists/` — Sub-Router

The tier list system is the largest domain (~13 files in old backend). Keep it as a sub-directory but reorganize by operation type rather than one-file-per-action:

| File | Endpoints | Old equivalent |
|------|-----------|----------------|
| `crud.rs` | create, get, list, update, delete, mine | create.rs + get.rs + list.rs + update.rs + mine.rs |
| `placements.rs` | add, move, remove, reorder | placements.rs (same but thinner — logic in service) |
| `tiers.rs` | create, update, delete | tiers.rs |
| `versions.rs` | publish, changelog, history | versions.rs |
| `permissions.rs` | grant, revoke, list | permissions.rs + middleware.rs |
| `moderation.rs` | report, review | report.rs + moderate.rs |

**Net reduction:** 13 files → 6 files, with business logic moved to `services/tier_list.rs`.

---

## New Dependencies Required

```toml
[dependencies]
# HTTP framework
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace", "request-id"] }

# Redis
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }

# Validation
validator = { version = "0.19", features = ["derive"] }

# Error handling
anyhow = "1"
thiserror = "2"

# Tracing (replaces println)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Optional: graceful shutdown
tokio-util = "0.7"
```

---

## Implementation Order

### Phase 1: Foundation (no routes yet)
- [ ] Add Axum + tower + tower-http + redis + tracing to Cargo.toml
- [ ] Create `src/app/mod.rs`, `state.rs`, `error.rs`
- [ ] Create `src/app/cache/` (keys.rs, store.rs)
- [ ] Create `src/app/extractors/` (auth.rs, pagination.rs, validated.rs)
- [ ] Create `src/app/middleware/` (rate_limit.rs, service_key.rs, request_id.rs)
- [ ] Wire up `server.rs` with empty router that boots and health-checks
- [ ] Update `main.rs` to init DB pool, Redis, game data, then call `server::run()`

### Phase 2: Core Read Endpoints
- [ ] `services/user.rs` + `routes/user.rs` — GET /get-user
- [ ] `services/static_data.rs` + `routes/static_data.rs` — GET /static/{resource}
- [ ] `services/leaderboard.rs` + `routes/leaderboard.rs` — GET /leaderboard
- [ ] `services/search.rs` + `routes/search.rs` — GET /search
- [ ] `services/stats.rs` + `routes/stats.rs` — GET /stats
- [ ] `routes/assets.rs` — GET /avatar/{id}, /portrait/{id}

### Phase 3: Auth + Write Endpoints
- [ ] `services/auth.rs` + `routes/auth.rs` — login, send-code, verify, refresh
- [ ] `services/gacha.rs` + `routes/gacha.rs` — fetch, store, stats, history
- [ ] `services/roster.rs` — sync_roster (called during refresh)

### Phase 4: Tier Lists
- [ ] `services/tier_list.rs` (full service with permission checks)
- [ ] `routes/tier_lists/crud.rs`
- [ ] `routes/tier_lists/placements.rs`
- [ ] `routes/tier_lists/tiers.rs`
- [ ] `routes/tier_lists/versions.rs`
- [ ] `routes/tier_lists/permissions.rs`
- [ ] `routes/tier_lists/moderation.rs`

### Phase 5: Remaining Features
- [ ] `services/dps.rs` + `routes/dps.rs` — DPS calculator endpoints
- [ ] `services/operator_notes.rs` + `routes/operator_notes.rs`
- [ ] Admin stats endpoint

### Phase 6: Production Hardening
- [ ] Graceful shutdown (tokio signal handler)
- [ ] Structured JSON logging with request IDs
- [ ] Health check endpoint (DB + Redis connectivity)
- [ ] OpenAPI spec generation (utoipa crate, optional)
- [ ] Integration tests for each route group

---

## Key Decisions & Trade-offs

### Why Axum again (not Actix)?
Axum integrates natively with Tower middleware ecosystem and tokio. The old backend already used it successfully. No reason to switch.

### Why not GraphQL?
The API is consumed by a single Next.js frontend. REST with field selection on static endpoints gives the frontend what it needs without the complexity of a GraphQL schema + resolver layer.

### Why a service layer instead of fat handlers?
The old backend proved that handler-level business logic doesn't scale. Gacha handler (20.2KB) and placements handler (14.7KB) became hard to maintain. Services are independently testable and composable.

### Why typed cache keys?
The old backend had cache bugs from typos in string keys. A `CacheKey` enum makes invalid keys unrepresentable and co-locates TTLs with their keys.

### Why a single static data handler?
16 near-identical files was the old backend's biggest source of boilerplate. A generic handler with a resource enum cuts ~32KB of duplicated code.

### Redis vs in-process cache (Moka)?
Use Redis as primary cache (shared across instances if scaled horizontally). Moka can optionally be used as an L1 cache for rate limiting (per-instance is fine) or extremely hot data.

---

## Migration Notes from Old Backend

### Port directly (logic is sound):
- Rate limiting strategy (per-IP, 100/60s)
- JWT auth flow (HS256, 7-day expiry)
- Yostar OAuth integration
- Leaderboard scoring/grading system
- Gacha dedup logic (user_id + timestamp + char_id)
- Tier list permission hierarchy

### Rewrite (structural issues):
- Search query builder → use typed query builder, not string concatenation
- Operator name normalization → derive from game data at startup instead of hard-coding 281 mappings
- Static data endpoints → unify into single generic handler
- Cache key construction → typed CacheKey enum

### Drop (already handled differently in v3):
- Asset mode abstraction (S3/local) → decide on one strategy
- Binary CLI tools (manage-permissions, etc.) → if needed, add as Axum admin routes or keep as separate bins
