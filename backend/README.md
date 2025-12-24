# Myrtle Backend

A high-performance Rust backend for the Myrtle.moe Arknights companion application. This server provides user authentication via Yostar OAuth, player data synchronization, static game data APIs, and asset CDN functionality.

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
| Multi-Server Support | EN, JP, KR, CN, Bilibili, TW game servers |
| Yostar OAuth | Email-based authentication flow |
| Player Sync | Fetch and store player game data |
| Static Data API | Operators, skills, modules, materials, skins, and more |
| Asset CDN | Serve avatars, portraits, and game assets |
| Redis Caching | Response caching with ETag support |
| Rate Limiting | Per-IP, per-endpoint rate limiting |
| Auto-Reload | Hourly configuration refresh from game servers |

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

## Installation

### Prerequisites

- **Rust** 1.85+ (Edition 2024)
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
DATA_DIR=./data
ASSETS_DIR=./assets
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

### 4. Start the Server

```bash
# Development
cargo run

# Production
cargo run --release

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
cargo run --release

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
│   │       └── yostar/         # Authentication
│   ├── core/
│   │   ├── authentication/     # Yostar OAuth, headers, sessions
│   │   ├── user/               # User data fetching & formatting
│   │   ├── local/              # Game data loading & types
│   │   ├── cron/               # Background jobs
│   │   └── dps_calculator/     # DPS calculations (WIP)
│   ├── database/
│   │   ├── pool.rs             # PostgreSQL connection
│   │   └── models/user.rs      # User model
│   └── events/
│       ├── mod.rs              # Event emitter
│       └── setup_event_listeners.rs
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
      └──▶ /get-user ──▶ PostgreSQL
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
- Avatar locations across `ui_char_avatar_0` through `ui_char_avatar_13`
- Portrait locations across `pack0` through `pack12`
- Skill icons, module images, item icons

## Database

### Schema

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uid VARCHAR(255) NOT NULL UNIQUE,
    server VARCHAR(50) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
RUST_LOG=debug cargo run

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
