# Myrtle Backend

A high-performance Rust backend for the Myrtle.moe Arknights companion application. This server provides user authentication via Yostar OAuth, player data synchronization, static game data APIs, asset CDN functionality, and community tier list management with versioning and permission controls.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Starting the Server](#starting-the-server)
  - [API Overview](#api-overview)
- [Architecture](#architecture)
- [API Reference](#api-reference)
  - [Health & Status](#health--status)
  - [Authentication](#authentication)
  - [User Data](#user-data)
  - [Static Data](#static-data)
  - [CDN Assets](#cdn-assets)
  - [Tier Lists](#tier-lists)
  - [Admin](#admin)
- [Game Data](#game-data)
- [Database](#database)
- [Caching](#caching)
- [Rate Limiting](#rate-limiting)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Features

### Core Features

| Feature | Description |
|---------|-------------|
| Yostar OAuth | Email-based authentication flow |
| JWT Authentication | Token-based session management with verification |
| Player Sync | Fetch and store player game data |
| User Settings | Configurable user preferences and settings |
| Permission System | Role-based access control (admin/editor/viewer) |
| Static Data API | Operators, skills, modules, materials, skins, and more |
| Tier List Management | Create, edit, and version community tier lists |
| Asset CDN | Serve avatars, portraits, and game assets |
| Redis Caching | Response caching with ETag support |
| Rate Limiting | Per-IP, per-endpoint rate limiting |
| Auto-Reload | Hourly configuration refresh from game servers |
| Admin Statistics | System monitoring and usage statistics |

### Static Data Endpoints

| Endpoint | Description |
|----------|-------------|
| `/static/operators` | All operator data with stats, skills, talents |
| `/static/skills` | Skill definitions and level data |
| `/static/modules` | Module/equipment data with unlock costs |
| `/static/materials` | Items, crafting materials, currencies |
| `/static/skins` | Operator skins with asset paths |
| `/static/handbook` | Operator lore and profiles |
| `/static/ranges` | Attack range grids |
| `/static/trust` | Trust/favor calculation tables |
| `/static/voices` | Voice line data by operator |
| `/static/gacha` | Gacha pools and recruitment data |
| `/static/chibis` | Chibi/spine animation metadata |

### Asset Serving

| Asset Type | Endpoint | Description |
|------------|----------|-------------|
| Avatars | `/cdn/avatar/{id}` | Operator avatar icons |
| Portraits | `/cdn/portrait/{id}` | Full character portraits |
| General | `/cdn/{path}` | Any asset from assets directory |

### Tier List Management

| Endpoint | Description |
|----------|-------------|
| `/tier-lists` | List and create tier lists |
| `/tier-lists/{slug}` | Get, update, or delete a tier list |
| `/tier-lists/{slug}/tiers` | Manage tiers within a list |
| `/tier-lists/{slug}/placements` | Manage operator placements |
| `/tier-lists/{slug}/versions` | Version history and snapshots |
| `/tier-lists/{slug}/permissions` | User permission management |

## Installation

### Prerequisites

- **Rust** 1.85+
- **PostgreSQL** 14+
- **Redis** 6+
- **Game Data** JSON files (from asset extraction)
- **Assets** Extracted game assets (images, sprites)

### Install Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure you have the latest stable
rustup update stable
```

### Install Dependencies

```bash
# macOS
brew install postgresql redis

# Ubuntu/Debian
sudo apt install postgresql redis-server

# Start services
brew services start postgresql redis  # macOS
sudo systemctl start postgresql redis  # Linux
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Eltik/myrtle.moe.git
cd myrtle.moe/backend

# Build release binary
cargo build --release

# Binary will be at: ./target/release/backend
```

## Quick Start

### 1. Set Up Database

```bash
# Create PostgreSQL database
createdb myrtle

# The server will auto-create tables on startup
```

### 2. Configure Environment

```bash
# Create .env file
cat > .env << 'EOF'
DATABASE_URL=postgres://localhost/myrtle
REDIS_URL=redis://127.0.0.1:6379
DATA_DIR=../assets/Unpacked/gamedata/excel
ASSETS_DIR=../assets/Unpacked
EOF
```

### 3. Prepare Game Data

Place the extracted game data JSON files in your `DATA_DIR`:

```
data/
├── character_table.json
├── skill_table.json
├── uniequip_table.json
├── battle_equip_table.json
├── skin_table.json
├── item_table.json
├── range_table.json
├── favor_table.json
├── charword_table.json
├── gacha_table.json
└── handbook_info_table.json
```
You don't have to worry about this if you have already unpacked assets using the unpacker.

### 4. Start the Server

```bash
# Development
cargo run --bin backend

# Production
cargo run --bin backend --release

# Server starts on http://0.0.0.0:3060
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `REDIS_URL` | Yes | - | Redis connection string |
| `DATA_DIR` | No | `./data` | Directory containing game data JSON files (see `/assets` directory) |
| `ASSETS_DIR` | No | `./assets` | Directory containing extracted game assets (see `/assets` directory) |

### Example .env

```bash
# Database
DATABASE_URL=postgres://user:password@localhost:5432/myrtle

# Redis
REDIS_URL=redis://:password@localhost:6379

# Data directories
DATA_DIR=/opt/myrtle/assets/Unpacked/decoded/gamedata/excel
ASSETS_DIR=/opt/myrtle/assets/Unpacked
```

## Usage

### Starting the Server

```bash
# Load environment and start
source .env
cargo run --bin backend --release

# Output:
# Loading configuration...
# Configuration loaded
# Loading game data from: "./data"
# Loading assets from: "./assets"
# Game data loaded: 350 operators, 800 skills, 200 modules, 500 skins, 1000 items, 350 handbook entries, 350 chibis
# Server running on http://0.0.0.0:3060
```

### API Overview

```bash
# Health check
curl http://localhost:3060/health
# {"status":"ok"}

# Get all operators (paginated)
curl "http://localhost:3060/static/operators?limit=10"

# Get specific operator
curl http://localhost:3060/static/operators/char_002_amiya

# Refresh user data using authentication
curl -X POST "http://localhost:3060/refresh?uid=123&secret=abc&seqnum=1&server=en"
```

## Architecture

### Project Structure

```
backend/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Module exports
│   ├── app/
│   │   ├── mod.rs
│   │   ├── server.rs           # Axum server setup
│   │   ├── state.rs            # Application state
│   │   ├── error.rs            # Error types
│   │   ├── middleware/
│   │   │   ├── rate_limit.rs   # Rate limiting
│   │   │   └── static_assets.rs # Asset serving
│   │   └── routes/
│   │       ├── get_user.rs     # User lookup
│   │       ├── avatar.rs       # Avatar CDN
│   │       ├── portrait.rs     # Portrait CDN
│   │       ├── static_data/    # Static data API
│   │       ├── yostar/         # Yostar OAuth login
│   │       ├── auth/           # Token verification & settings
│   │       ├── admin/          # Admin statistics
│   │       └── tier_lists/     # Tier list management
│   ├── core/
│   │   ├── authentication/     # Yostar OAuth, JWT, headers, sessions
│   │   ├── user/               # User data fetching & formatting
│   │   ├── local/              # Game data loading & types
│   │   ├── cron/               # Background jobs
│   │   └── dps_calculator/     # DPS calculations (WIP)
│   ├── database/
│   │   ├── pool.rs             # PostgreSQL connection & table init
│   │   └── models/
│   │       ├── user.rs         # User model
│   │       └── tier_lists.rs   # Tier list models (6 tables)
│   └── events/
│       ├── mod.rs              # Event emitter
│       └── setup_event_listeners.rs
├── bin/
│   └── manage_permissions.rs   # CLI tool for permission management
├── Cargo.toml
└── README.md
```

### Key Components

#### Application State

```rust
pub struct AppState {
    pub db: PgPool,                           // PostgreSQL connection pool
    pub config: Arc<RwLock<GlobalConfig>>,    // Server configs (domains, versions)
    pub events: Arc<EventEmitter>,            // Event system
    pub client: Client,                       // HTTP client for game API
    pub game_data: Arc<GameData>,             // Static game data
    pub redis: MultiplexedConnection,         // Redis for caching
}
```

#### Game Data

```rust
pub struct GameData {
    pub operators: HashMap<String, Operator>,
    pub skills: HashMap<String, Skill>,
    pub materials: Materials,
    pub modules: Modules,
    pub skins: SkinData,
    pub handbook: Handbook,
    pub ranges: Ranges,
    pub favor: Favor,
    pub voices: Voices,
    pub gacha: GachaData,
    pub chibis: ChibiData,
    pub asset_mappings: AssetMappings,
}
```

### Request Flow

```
Client Request
      │
      ▼
┌─────────────┐
│ Rate Limit  │ ──▶ 429 Too Many Requests
└─────────────┘
      │
      ▼
┌─────────────┐
│   Router    │
└─────────────┘
      │
      ├──▶ /health ──▶ Health Check
      ├──▶ /static/* ──▶ Redis Cache ──▶ Game Data
      ├──▶ /cdn/* ──▶ File System ──▶ Asset Streaming
      ├──▶ /send-code ──▶ Yostar API
      ├──▶ /login ──▶ Yostar OAuth ──▶ Game Server Auth
      ├──▶ /refresh ──▶ Game Server ──▶ PostgreSQL
      ├──▶ /get-user ──▶ PostgreSQL
      ├──▶ /auth/* ──▶ JWT Verify ──▶ User Settings
      ├──▶ /tier-lists/* ──▶ Permission Check ──▶ PostgreSQL
      └──▶ /admin/* ──▶ Admin Check ──▶ Statistics
```

## API Reference

### Health & Status

#### GET /

Returns API identifier.

```bash
curl http://localhost:3060/
# "Myrtle API"
```

#### GET /health

Health check endpoint.

```bash
curl http://localhost:3060/health
# {"status":"ok"}
```

### Authentication

#### POST /send-code

Request a verification code via email.

**Query Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `email` | string | Yes | - | Email address |
| `server` | string | No | `en` | Server: `en`, `jp`, `kr` |

**Path Variants:**
- `POST /send-code?email={email}&server={server}`
- `POST /send-code/{email}`
- `POST /send-code/{email}/{server}`

```bash
curl -X POST "http://localhost:3060/send-code?email=user@example.com&server=en"
```

#### POST /login

Authenticate with email and verification code.

**Query Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `email` | string | Yes | - | Email address |
| `code` | string | Yes | - | Verification code |
| `server` | string | No | `en` | Server: `en`, `jp`, `kr` |

**Path Variants:**
- `POST /login?email={email}&code={code}&server={server}`
- `POST /login/{email}/{code}`
- `POST /login/{email}/{code}/{server}`

**Response:**
```json
{
  "uid": "12345678",
  "secret": "abc123...",
  "seqnum": 1
}
```

#### POST /auth/verify

Verify a JWT token and return user information.

**Headers:**
| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token |

**Response:**
```json
{
  "valid": true,
  "user": {
    "uid": "12345678",
    "server": "en",
    "role": "user"
  }
}
```

#### POST /auth/update-settings

Update user settings and preferences.

**Headers:**
| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token |

**Body:**
```json
{
  "settings": {
    "publicProfile": true
  }
}
```

### User Data

#### POST /refresh

Fetch latest player data from game server and store in database.

**Query Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `uid` | string | Yes | - | Player UID |
| `secret` | string | Yes | - | Session secret |
| `seqnum` | integer | Yes | - | Sequence number |
| `server` | string | No | `en` | Server |

**Path Variants:**
- `POST /refresh?uid={uid}&secret={secret}&seqnum={seqnum}&server={server}`
- `POST /refresh/{uid}/{secret}/{seqnum}`
- `POST /refresh/{uid}/{secret}/{seqnum}/{server}`

**Response:** Full player data with enriched static information.

#### GET /get-user

Retrieve stored user data from database.

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `uid` | string | Yes | Player UID |

**Path Variants:**
- `GET /get-user?uid={uid}`
- `GET /get-user/{uid}`

### Static Data

All static endpoints support:
- **Pagination:** `?limit=15&cursor={base64_cursor}`
- **Field filtering:** `?fields=id,name,rarity`
- **Caching:** Redis + ETag support

#### GET /static/operators

List all operators with pagination.

```bash
# First page
curl "http://localhost:3060/static/operators?limit=10"

# With field filtering
curl "http://localhost:3060/static/operators?limit=10&fields=name,rarity,profession"

# Response
{
  "operators": [...],
  "next_cursor": "MTU=",
  "has_more": true,
  "total": 350
}
```

#### GET /static/operators/{id}

Get single operator by ID.

```bash
curl http://localhost:3060/static/operators/char_002_amiya
```

#### GET /static/skills

List all skills.

```bash
curl "http://localhost:3060/static/skills?limit=20"
```

#### GET /static/skills/{id}

Get single skill.

```bash
curl http://localhost:3060/static/skills/skchr_amiya_1
```

#### GET /static/modules

List all modules.

```bash
curl "http://localhost:3060/static/modules?limit=20"
```

#### GET /static/modules/{id}

Get module by ID.

#### GET /static/modules/details/{id}

Get module with battle equip details.

#### GET /static/materials

List all materials/items.

#### GET /static/materials/{id}

Get single material.

#### GET /static/ranges

List all attack ranges.

#### GET /static/ranges/{id}

Get single range grid.

#### GET /static/skins

List all skins.

#### GET /static/skins/{id}

Get single skin.

#### GET /static/skins/char/{char_id}

Get all skins for an operator.

#### GET /static/handbook

List all handbook entries.

#### GET /static/handbook/{id}

Get single handbook entry.

#### GET /static/trust

Get trust/favor table.

#### GET /static/trust/calculate?trust={points}

Calculate trust level from favor points.

#### GET /static/voices

List all voice data.

#### GET /static/voices/char/{char_id}

Get voice lines for an operator.

#### GET /static/gacha

Get all gacha data.

#### GET /static/gacha/pools

Get gacha pool definitions.

#### GET /static/gacha/tags

Get recruitment tags.

#### GET /static/gacha/recruitment

Get recruitment pool data.

#### GET /static/gacha/calculate?recruitment={tags}

Calculate recruitment outcomes for given tags.

#### GET /static/chibis

List all chibi/spine data.

#### GET /static/chibis/operators

List operators with chibi data.

#### GET /static/chibis/{operator_code}

Get chibi data for specific operator.

### CDN Assets

#### GET /cdn/avatar/{avatar_id}

Serve avatar image with ETag caching.

```bash
curl http://localhost:3060/cdn/avatar/char_002_amiya
# Returns: image/png
```

#### GET /cdn/portrait/{char_id}

Serve character portrait with ETag caching.

```bash
curl http://localhost:3060/cdn/portrait/char_002_amiya_1
# Returns: image/png
```

#### GET /cdn/{*asset_path}

Serve any asset from the assets directory.

```bash
curl http://localhost:3060/cdn/upk/arts/chararts/char_002_amiya.png
```

### Tier Lists

The tier list system allows creating, managing, and versioning community tier lists with permission-based access control.

#### GET /tier-lists

List all tier lists.

```bash
curl http://localhost:3060/tier-lists
```

#### POST /tier-lists

Create a new tier list (requires authentication).

**Headers:**
| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token |

**Body:**
```json
{
  "name": "Meta Tier List",
  "slug": "meta-tier-list",
  "description": "Community meta rankings"
}
```

#### GET /tier-lists/{slug}

Get a specific tier list with all tiers and placements.

```bash
curl http://localhost:3060/tier-lists/meta-tier-list
```

#### PUT /tier-lists/{slug}

Update a tier list (requires edit permission).

#### DELETE /tier-lists/{slug}

Delete a tier list (requires owner permission).

#### GET /tier-lists/{slug}/tiers

Get all tiers in a tier list.

#### POST /tier-lists/{slug}/tiers

Create a new tier (requires edit permission).

**Body:**
```json
{
  "name": "S+",
  "order": 0,
  "color": "#FF0000"
}
```

#### GET /tier-lists/{slug}/placements

Get all operator placements in a tier list.

#### POST /tier-lists/{slug}/placements

Add an operator placement (requires edit permission).

**Body:**
```json
{
  "tier_id": "uuid",
  "operator_id": "char_002_amiya",
  "order": 0,
  "notes": "Optional notes"
}
```

#### GET /tier-lists/{slug}/versions

Get version history for a tier list.

#### POST /tier-lists/{slug}/versions

Create a new version snapshot (requires edit permission).

**Body:**
```json
{
  "version": "1.0.0",
  "changelog": "Initial release"
}
```

#### GET /tier-lists/{slug}/permissions

Get user permissions for a tier list.

#### POST /tier-lists/{slug}/permissions

Grant user permission (requires owner permission).

**Body:**
```json
{
  "user_id": "uuid",
  "permission": "editor"
}
```

### Admin

#### GET /admin/stats

Get system statistics (requires admin role).

**Headers:**
| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token (admin) |

**Response:**
```json
{
  "users": {
    "total": 1000,
    "by_server": {
      "en": 500,
      "jp": 300,
      "kr": 200
    }
  },
  "tier_lists": {
    "total": 50,
    "active": 45
  }
}
```

## Game Data

### Required Files

Place these JSON files in your `DATA_DIR` or extract them via the [assets unpacker](../assets/unpacker):

| File | Description |
|------|-------------|
| `character_table.json` | Operator definitions |
| `skill_table.json` | Skill data |
| `uniequip_table.json` | Module definitions |
| `battle_equip_table.json` | Module battle effects |
| `skin_table.json` | Skin data |
| `item_table.json` | Materials and items |
| `range_table.json` | Attack range grids |
| `favor_table.json` | Trust calculation |
| `charword_table.json` | Voice line data |
| `gacha_table.json` | Gacha pools |
| `handbook_info_table.json` | Operator lore |

### Data Enrichment

On startup, the server enriches raw game data:

1. **Operators** - Skills, modules, handbook, skins attached
2. **Skills** - Asset paths resolved
3. **Modules** - Item costs with material details
4. **Skins** - Asset paths from mappings
5. **Materials** - Icon paths resolved

### Asset Mappings

The server scans `ASSETS_DIR` to build mappings for:
- Avatar locations across `ui_char_avatar_0` through `ui_char_avatar_19` (20 directories)
- Portrait locations across `pack0` through `pack14` (15 directories)
- Skill icons, module images, item icons

## Database

### Schema

#### Users Table

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uid VARCHAR(255) NOT NULL UNIQUE,
    server VARCHAR(50) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    settings JSONB NOT NULL DEFAULT '{}',
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### Tier List Tables

```sql
-- Main tier list metadata
CREATE TABLE tier_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Individual tiers within a list
CREATE TABLE tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    "order" INTEGER NOT NULL,
    color VARCHAR(7),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Operator placements within tiers
CREATE TABLE tier_placements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_id UUID NOT NULL REFERENCES tiers(id) ON DELETE CASCADE,
    operator_id VARCHAR(255) NOT NULL,
    "order" INTEGER NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Version snapshots
CREATE TABLE tier_list_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    snapshot JSONB NOT NULL,
    changelog TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Change audit log
CREATE TABLE tier_change_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User permissions per tier list
CREATE TABLE tier_list_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tier_list_id, user_id)
);
```

### User Model Operations

```rust
// Create user
User::create(&pool, CreateUser { uid, server, data }).await?;

// Find by UID
User::find_by_uid(&pool, "12345678").await?;

// Update data
User::update_data(&pool, user_id, new_data).await?;

// Search by nickname
User::search_by_nickname(&pool, "Doctor").await?;

// List by server
User::find_by_server(&pool, "en").await?;

// Update settings
User::update_settings(&pool, user_id, settings).await?;
```

### Tier List Model Operations

```rust
// Create tier list
TierList::create(&pool, CreateTierList { name, slug, description, owner_id }).await?;

// Find by slug
TierList::find_by_slug(&pool, "meta-tier-list").await?;

// List all active tier lists
TierList::list_active(&pool).await?;

// Create tier
Tier::create(&pool, CreateTier { tier_list_id, name, order, color }).await?;

// Create placement
TierPlacement::create(&pool, CreatePlacement { tier_id, operator_id, order, notes }).await?;

// Create version snapshot
TierListVersion::create(&pool, tier_list_id, version, snapshot, changelog, user_id).await?;

// Grant permission
TierListPermission::grant(&pool, tier_list_id, user_id, permission).await?;
```

## Caching

### Redis Caching

Static data endpoints use Redis caching with configurable TTL:

```rust
cached_handler(
    &mut redis,
    "static:operators:all",
    3600,  // 1 hour TTL
    &headers,
    || async { /* fetch data */ }
).await
```

### ETag Support

Asset endpoints support HTTP caching:

```bash
# First request
curl -I http://localhost:3060/cdn/avatar/char_002_amiya
# ETag: "12345-1703548800"
# Cache-Control: public, max-age=86400

# Conditional request
curl -H "If-None-Match: \"12345-1703548800\"" http://localhost:3060/cdn/avatar/char_002_amiya
# 304 Not Modified
```

## Rate Limiting

The server implements per-IP, per-endpoint rate limiting:

- **Window:** 60 seconds
- **Limit:** 100 requests per endpoint per IP
- **Response:** `429 Too Many Requests`

```rust
// Rate limit key format: "{ip}:{path}"
// Example: "192.168.1.1:/static/operators"
```

## Development

### Building

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Running in Development

```bash
# With auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# With debug logging
RUST_LOG=debug cargo run
```

### CLI Tools

The project includes CLI tools for administrative tasks:

#### Permission Manager

Manage user permissions from the command line:

```bash
# Build the CLI tool
cargo build --release --bin manage_permissions

# Run the permission manager
./target/release/manage_permissions

# Grant admin role to a user
./target/release/manage_permissions --uid 12345678 --role admin

# List all admins
./target/release/manage_permissions --list-admins
```

## Troubleshooting

### Common Issues

#### "DATABASE_URL must be set"

**Cause:** Missing environment variable.

**Solution:**
```bash
export DATABASE_URL=postgres://localhost/myrtle
# Or use .env file
```

#### "REDIS_URL must be set"

**Cause:** Missing Redis configuration.

**Solution:**
```bash
export REDIS_URL=redis://127.0.0.1:6379
```

#### "Warning: Running with empty game data"

**Cause:** Game data JSON files not found in `DATA_DIR`.

**Solution:**
1. Ensure `DATA_DIR` points to correct directory
2. Verify JSON files exist and are valid
3. Check file permissions

#### "Failed to parse character_table"

**Cause:** Malformed or incompatible JSON.

**Solution:**
1. Verify JSON is valid: `jq . character_table.json`
2. Ensure data matches expected schema
3. Check for encoding issues (must be UTF-8)

#### Connection refused on port 3060

**Cause:** Server not running or port in use.

**Solution:**
```bash
# Check if port is in use
lsof -i :3060

# Kill existing process
kill -9 $(lsof -t -i :3060)
```

#### Rate limit errors (429)

**Cause:** Too many requests from same IP.

**Solution:**
- Wait 60 seconds for window to reset
- Reduce request frequency
- Consider caching on client side

### Debug Logging

Enable detailed logging:

```bash
# All debug output
RUST_LOG=debug cargo run --bin backend

# Specific module
RUST_LOG=backend::core::authentication=debug cargo run

# Multiple modules
RUST_LOG=backend::app=debug,backend::core=info cargo run
```

### Parse Error Debugging

When player data parsing fails, debug logs are saved to:

```
debug_logs/
└── parse_error_2024-01-15_10-30-00/
    ├── sync_data.json      # Raw response
    └── error_report.txt    # Error details with context
```

If this issue occurs, please open an issue. This feature might be removed in the future as it is just used for debugging purposes.

## License

This project is for educational purposes. All game assets and data remain the property of Hypergryph/Yostar.

## Related Projects

- [Arknights Asset Downloader](../assets/downloader) - Download game assets
- [Assets Unpacker](../assets/unpacker) - Extract Unity asset bundles
- [unity-rs](../assets/unity-rs) - Rust Unity asset parser
