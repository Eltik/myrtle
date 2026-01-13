# Myrtle Backend

A high-performance Rust backend for the Myrtle.moe Arknights companion application. This server provides user authentication via Yostar OAuth, player data synchronization, static game data APIs, asset CDN functionality, community tier list management with versioning and permission controls, and a comprehensive DPS calculator with 281 operator implementations.

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
  - [DPS Calculator](#dps-calculator-api)
  - [Tier Lists](#tier-lists)
  - [Admin](#admin)
- [DPS Calculator](#dps-calculator)
- [User Scoring](#user-scoring)
- [Testing](#testing)
  - [Running Tests](#running-tests)
  - [DPS Comparison Tests](#dps-comparison-tests)
  - [Score Calculation Tests](#score-calculation-tests)
  - [Generating Test Files](#generating-test-files)
- [Game Data](#game-data)
- [External Dependencies](#external-dependencies)
- [Database](#database)
- [Caching](#caching)
- [Rate Limiting](#rate-limiting)
- [Development](#development)
  - [CLI Tools](#cli-tools)
  - [Scripts](#scripts)
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
| DPS Calculator | 281 operators with 100% accuracy vs Python reference |
| DPS Calculator API | REST API for damage calculations with range support |
| User Scoring | Account valuation across operators, stages, roguelike, sandbox, medals, base efficiency |

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

### DPS Calculator API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/dps-calculator` | POST | Calculate DPS with full operator configuration |
| `/dps-calculator/operators` | GET | List all supported operators with metadata |

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
| `JWT_SECRET` | Yes | - | Secret key for JWT token signing and verification |
| `INTERNAL_SERVICE_KEY` | No | - | Secret key to bypass rate limiting (for internal services like frontend API routes) |

### Example .env

```bash
# Database
DATABASE_URL=postgres://user:password@localhost:5432/myrtle

# Redis
REDIS_URL=redis://:password@localhost:6379

# Data directories
DATA_DIR=/opt/myrtle/assets/Unpacked/decoded/gamedata/excel
ASSETS_DIR=/opt/myrtle/assets/Unpacked

# Authentication
JWT_SECRET=your-secret-key-at-least-32-characters-long

# Optional: Internal service key for rate limit bypass (min 32 chars)
# Generate with: openssl rand -base64 32
INTERNAL_SERVICE_KEY=your-internal-service-key-here
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

# Calculate DPS for an operator
curl -X POST http://localhost:3060/dps-calculator \
  -H "Content-Type: application/json" \
  -d '{"operatorId": "char_017_huang", "params": {"skillIndex": 2}, "enemy": {"defense": 500, "res": 0}}'

# List DPS calculator operators
curl http://localhost:3060/dps-calculator/operators

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
│   ├── bin/
│   │   ├── generate_dps_tests.rs   # Generate DPS comparison test files
│   │   ├── manage_permissions.rs   # CLI tool for permission management
│   │   └── translate_operators.rs  # Python-to-Rust operator translator
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
│   │       ├── dps_calculator/ # DPS calculation API
│   │       │   ├── calculate.rs    # POST /dps-calculator
│   │       │   └── list.rs         # GET /dps-calculator/operators
│   │       ├── yostar/         # Yostar OAuth login
│   │       ├── auth/           # Token verification & settings
│   │       ├── admin/          # Admin statistics
│   │       └── tier_lists/     # Tier list management
│   ├── core/
│   │   ├── authentication/     # Yostar OAuth, JWT, headers, sessions
│   │   ├── user/               # User data fetching, formatting & scoring
│   │   │   └── score/          # Account scoring (operators, stages, roguelike, sandbox, medals, base)
│   │   ├── local/              # Game data loading & types
│   │   ├── cron/               # Background jobs
│   │   └── dps_calculator/     # DPS calculations (281 operators)
│   │       ├── mod.rs
│   │       ├── operator_data.rs    # Operator stats and data loading
│   │       ├── operator_unit.rs    # DpsCalculator trait and OperatorUnit
│   │       └── operators/          # Individual operator implementations
│   │           ├── mod.rs          # Operator registry and factory
│   │           ├── a/              # Aak, Absinthe, Aciddrop, etc.
│   │           ├── b/              # Bagpipe, Blaze, etc.
│   │           └── ...             # Organized alphabetically (a-z)
│   ├── database/
│   │   ├── pool.rs             # PostgreSQL connection & table init
│   │   └── models/
│   │       ├── user.rs         # User model
│   │       └── tier_lists.rs   # Tier list models (6 tables)
│   └── events/
│       ├── mod.rs              # Event emitter
│       └── setup_event_listeners.rs
├── scripts/
│   ├── python_dps_harness.py       # Python harness for DPS calculations
│   └── extract_operator_data.py    # Extract operator data from Python source
├── tests/
│   ├── dps_comparison/
│   │   ├── dps_comparison_test.rs  # Generated Rust integration tests
│   │   ├── run_comparison.py       # Python test runner
│   │   ├── test_config.json        # Test case configurations
│   │   ├── operators.json          # Extracted operator metadata
│   │   └── expected_dps.json       # Pre-computed Python DPS values
│   └── score_calculation/
│       ├── score_calculation_test.rs  # User scoring integration tests
│       └── output/                     # Generated score JSON files
├── external/
│   └── ArknightsDpsCompare/        # Git submodule - reference implementation
├── Cargo.toml
└── README.md
```

### Key Components

#### Application State

```rust
pub struct AppState {
    pub db: PgPool,                              // PostgreSQL connection pool
    pub config: Arc<RwLock<GlobalConfig>>,       // Server configs (domains, versions)
    pub events: Arc<EventEmitter>,               // Event system
    pub client: Client,                          // HTTP client for game API
    pub game_data: Arc<GameData>,                // Static game data
    pub redis: MultiplexedConnection,            // Redis for caching
    pub jwt_secret: String,                      // JWT secret for token validation
    pub internal_service_key: Option<String>,    // Optional key for rate limit bypass
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

#### DPS Calculator Trait

```rust
/// Trait for operator DPS calculations
pub trait DpsCalculator {
    /// Calculate DPS against given enemy stats
    fn skill_dps(&self, enemy: &EnemyStats) -> f64;

    /// Get reference to the operator unit
    fn unit(&self) -> &OperatorUnit;

    /// Get mutable reference to the operator unit
    fn unit_mut(&mut self) -> &mut OperatorUnit;
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
      ├──▶ /dps-calculator ──▶ DPS Calculator ──▶ Operator Factory
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

#### GET /leaderboard

Get paginated leaderboard with customizable sorting. Results are cached in Redis for 5 minutes.

**Query Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `sort_by` | string | No | `totalScore` | Sort field (see options below) |
| `order` | string | No | `desc` | Sort order: `asc` or `desc` |
| `server` | string | No | - | Filter by server: `en`, `jp`, `kr` |
| `limit` | integer | No | 50 | Results per page (max 100) |
| `offset` | integer | No | 0 | Pagination offset |

**Sort options:**
- `totalScore` - Overall account score
- `compositeScore` - Weighted composite score
- `operatorScore` - Operator investment score
- `stageScore` - Stage completion score
- `roguelikeScore` - Integrated Strategies progress
- `sandboxScore` - Reclamation Algorithm progress
- `medalScore` - Medal achievement score
- `baseScore` - RIIC efficiency score

**Note:** Only users with `publicProfile: true` in their settings appear in the leaderboard.

```bash
# Get top 10 by total score
curl "http://localhost:3060/leaderboard?limit=10"

# Get top 50 by operator score for EN server
curl "http://localhost:3060/leaderboard?sort_by=operatorScore&server=en&limit=50"
```

#### GET /search

Search users with advanced filtering. Results are cached in Redis for 2 minutes.

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `nickname` | string | No | Search by nickname (partial match) |
| `uid` | string | No | Search by exact UID |
| `server` | string | No | Filter by server |
| `level` | integer | No | Minimum doctor level |
| `limit` | integer | No | Results limit (default 20) |

**Note:** Only users with `publicProfile: true` in their settings appear in search results.

```bash
# Search by nickname
curl "http://localhost:3060/search?nickname=Doctor"

# Search by UID
curl "http://localhost:3060/search?uid=12345678"
```

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

### DPS Calculator API

The DPS Calculator API provides endpoints for calculating operator damage per second with full configuration support.

#### POST /dps-calculator

Calculate DPS for an operator with given configuration against enemy stats.

**Request Body:**
```json
{
  "operatorId": "char_017_huang",
  "params": {
    "potential": 6,
    "promotion": 2,
    "level": 90,
    "trust": 100,
    "skillIndex": 2,
    "masteryLevel": 3,
    "moduleIndex": 1,
    "moduleLevel": 3,
    "targets": 3,
    "buffs": {
      "atkBuff": 0.0,
      "atkBuffFlat": 0,
      "aspd": 0,
      "fragile": 0.0,
      "defShred": 0,
      "defShredPercent": 0.0,
      "resShred": 0,
      "resShredPercent": 0.0,
      "damageBonus": 0.0
    },
    "baseBuffs": {
      "atkBuff": 0.0,
      "atkBuffFlat": 0
    },
    "conditionals": {
      "traitDamage": true,
      "talentDamage": true,
      "talent2Damage": true,
      "skillDamage": true,
      "moduleDamage": true
    }
  },
  "enemy": {
    "defense": 500,
    "res": 20
  },
  "range": {
    "defenseMin": 0,
    "defenseMax": 2000,
    "defenseStep": 100,
    "resMin": 0,
    "resMax": 90,
    "resStep": 10
  }
}
```

**Request Parameters:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `operatorId` | string | Yes | - | Operator ID (e.g., "char_017_huang") |
| `params.potential` | int | No | 6 | Potential rank (1-6) |
| `params.promotion` | int | No | max | Elite level (0-2) |
| `params.level` | int | No | max | Operator level |
| `params.trust` | int | No | 100 | Trust level (0-100) |
| `params.skillIndex` | int | No | default | Skill index (0=basic, 1=S1, 2=S2, 3=S3) |
| `params.masteryLevel` | int | No | 3 | Mastery level (0-3) |
| `params.moduleIndex` | int | No | default | Module index (0=none, 1-3) |
| `params.moduleLevel` | int | No | 3 | Module level (1-3) |
| `params.targets` | int | No | 1 | Number of targets |
| `params.buffs` | object | No | {} | External buffs from allies |
| `params.baseBuffs` | object | No | {} | Base ATK modifiers |
| `params.conditionals` | object | No | all true | Conditional damage toggles |
| `enemy.defense` | float | No | 0 | Enemy defense |
| `enemy.res` | float | No | 0 | Enemy resistance (%) |
| `range` | object | No | null | Calculate across defense/res ranges |

**Response (Single):**
```json
{
  "dps": {
    "skillDps": 5234.56,
    "totalDamage": 78518.4,
    "averageDps": 3921.45
  },
  "operator": {
    "id": "char_017_huang",
    "name": "Blaze",
    "rarity": 6,
    "elite": 2,
    "level": 90,
    "potential": 6,
    "trust": 100,
    "skillIndex": 2,
    "skillLevel": 10,
    "moduleIndex": 1,
    "moduleLevel": 3,
    "atk": 952.0,
    "attackInterval": 1.2,
    "attackSpeed": 100.0,
    "isPhysical": true,
    "skillDuration": 15.0,
    "skillCost": 40,
    "skillParameters": [0.8, 3],
    "talent1Parameters": [0.08],
    "talent2Parameters": []
  }
}
```

**Response (Range):**
```json
{
  "dps": {
    "byDefense": [
      {"value": 0, "dps": 8500.0},
      {"value": 100, "dps": 7850.0},
      {"value": 200, "dps": 7200.0}
    ],
    "byResistance": [
      {"value": 0, "dps": 8500.0},
      {"value": 10, "dps": 7650.0},
      {"value": 20, "dps": 6800.0}
    ]
  },
  "operator": { ... }
}
```

**Example Requests:**

```bash
# Basic DPS calculation
curl -X POST http://localhost:3060/dps-calculator \
  -H "Content-Type: application/json" \
  -d '{"operatorId": "char_017_huang", "enemy": {"defense": 500}}'

# With skill and module selection
curl -X POST http://localhost:3060/dps-calculator \
  -H "Content-Type: application/json" \
  -d '{
    "operatorId": "char_003_kalts",
    "params": {
      "skillIndex": 3,
      "moduleIndex": 1,
      "moduleLevel": 3
    },
    "enemy": {"defense": 800, "res": 20}
  }'

# DPS curve across defense values
curl -X POST http://localhost:3060/dps-calculator \
  -H "Content-Type: application/json" \
  -d '{
    "operatorId": "char_134_ifrit",
    "params": {"skillIndex": 2},
    "range": {
      "defenseMin": 0,
      "defenseMax": 1500,
      "defenseStep": 100
    }
  }'
```

#### GET /dps-calculator/operators

List all operators that have DPS calculator implementations.

**Response:**
```json
{
  "count": 281,
  "operators": [
    {
      "id": "char_225_haak",
      "name": "Aak",
      "calculatorName": "Aak",
      "rarity": 6,
      "profession": "Specialist",
      "availableSkills": [1, 2, 3],
      "availableModules": [1, 2],
      "defaultSkillIndex": 3,
      "defaultPotential": 1,
      "defaultModuleIndex": 1,
      "maxPromotion": 2
    },
    {
      "id": "char_002_amiya",
      "name": "Amiya",
      "calculatorName": "Amiya",
      "rarity": 5,
      "profession": "Caster",
      "availableSkills": [1, 2, 3],
      "availableModules": [1, 2],
      "defaultSkillIndex": 2,
      "defaultPotential": 1,
      "defaultModuleIndex": 1,
      "maxPromotion": 2
    }
  ]
}
```

**Example:**
```bash
curl http://localhost:3060/dps-calculator/operators
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

## DPS Calculator

The backend includes a comprehensive DPS (Damage Per Second) calculator that has been translated from the Python [ArknightsDpsCompare](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare) project.

### Overview

| Metric | Value |
|--------|-------|
| Operators Implemented | 281 |
| Test Cases | 5,100 |
| Pass Rate | 100% |
| Reference | ArknightsDpsCompare (Python) |

### How It Works

The DPS calculator computes damage output for operators based on:

- **Operator Stats**: ATK, attack interval, attack speed, skill multipliers
- **Enemy Stats**: Defense (DEF) and Resistance (RES)
- **Skill Selection**: Different skills have different damage formulas
- **Module Selection**: Modules can modify stats and behavior
- **Conditionals**: Trait, talent, and skill damage toggles
- **External Buffs**: ATK buffs, ASPD, fragile, DEF/RES shred

### Architecture

Each operator implements the `DpsCalculator` trait:

```rust
/// Core trait for operator DPS calculations
pub trait DpsCalculator {
    /// Calculate DPS while skill is active
    fn skill_dps(&self, enemy: &EnemyStats) -> f64;

    /// Access operator unit stats
    fn unit(&self) -> &OperatorUnit;
    fn unit_mut(&mut self) -> &mut OperatorUnit;
}

/// Operator unit containing all stats and parameters
pub struct OperatorUnit {
    pub atk: f64,
    pub attack_interval: f32,
    pub attack_speed: f64,
    pub skill_index: i32,
    pub module_index: i32,
    pub module_level: i32,
    pub skill_parameters: Vec<f64>,
    pub talent1_parameters: Vec<f64>,
    pub talent2_parameters: Vec<f64>,
    // ... additional fields
}
```

Operators are organized alphabetically in `src/core/dps_calculator/operators/`:

```
operators/
├── mod.rs              # Registry and factory function
├── a/
│   ├── mod.rs
│   ├── aak.rs
│   ├── absinthe.rs
│   ├── amiya.rs
│   ├── amiya_guard.rs
│   └── ...
├── b/
│   ├── mod.rs
│   ├── bagpipe.rs
│   ├── blaze.rs
│   ├── blaze_alter.rs
│   └── ...
└── ...                 # 26 folders (a-z)
```

### Translation System

Operators are automatically translated from Python to Rust using the `translate_operators` binary:

```bash
# Translate all operators from Python source
cargo run --bin translate-operators -- \
  external/ArknightsDpsCompare/damagecalc/damage_formulas.py \
  src/core/dps_calculator/operators
```

The translator handles:
- Python syntax conversion (self.x → self.unit.x)
- NumPy functions (np.fmax → f64::max)
- Operator-specific edge cases (Walter shadows, Muelsyse cloned_op, etc.)
- Init-time field modifications
- Module-dependent calculations

### Usage Example

```rust
use backend::core::dps_calculator::{
    operator_data::OperatorData,
    operator_unit::{OperatorParams, EnemyStats},
    operators::create_operator,
};

// Create operator with skill 3, module 1
let params = OperatorParams {
    skill_index: Some(3),
    module_index: Some(1),
    module_level: Some(3),
    ..Default::default()
};

// Create the operator calculator
let calculator = create_operator("SilverAsh", operator_data, params)
    .expect("Operator not found");

// Calculate DPS against 500 DEF, 0 RES
let enemy = EnemyStats { defense: 500.0, res: 0.0 };
let dps = calculator.skill_dps(&enemy);
println!("SilverAsh S3 DPS: {:.2}", dps);
```

### Reference Implementation

The Rust implementations are validated against the Python reference:
- **Source**: `external/ArknightsDpsCompare/damagecalc/damage_formulas.py`
- **Data**: `external/ArknightsDpsCompare/Database/JsonReader.py`

## User Scoring

The backend includes a comprehensive user account scoring system that evaluates player progress across multiple game modes.

### Overview

The scoring system calculates a total account score by aggregating:

| Category | Description |
|----------|-------------|
| Operator Score | Investment in operators (level, mastery, modules, skins) |
| Stage Score | Story and event completion progress |
| Roguelike Score | Integrated Strategies (IS) progress |
| Sandbox Score | Reclamation Algorithm (RA) progress |
| Medal Score | Medal achievements with rarity and category bonuses |
| Base Score | RIIC efficiency (trading posts, factories, power plants, etc.) |

### Operator Scoring

Each operator is scored based on investment level:

#### Base Score (by Rarity)

| Rarity | Points |
|--------|--------|
| 6-star | 500 |
| 5-star | 400 |
| 4-star | 150 |
| 3-star | 30 |
| 2-star | 10 |
| 1-star | 5 |

#### Level Score

| Elite | Formula |
|-------|---------|
| E0 | level × 0.5 |
| E1 | 50 + (level × 1.0) |
| E2 | 150 + (level × 2.0) |

Example: E2 Level 90 = 150 + 180 = **330 points**

#### Trust Score

| Trust % | Points |
|---------|--------|
| 0-100% | 0-30 pts (linear) |
| 100-200% | 30-50 pts (dedication bonus) |

#### Potential Score

| Potential | Points |
|-----------|--------|
| P1 | 0 |
| P2 | 10 |
| P3 | 25 |
| P4 | 45 |
| P5 | 70 |
| P6 | 100 |

#### Mastery Score (per skill)

| Mastery | Points |
|---------|--------|
| M0 | 0 |
| M1 | 30 |
| M2 | 70 |
| M3 | 150 |

M9 operator (3 skills at M3) = 450 mastery points

#### Module Score (per module)

| Level | Points |
|-------|--------|
| Mod 1 | 50 |
| Mod 2 | 100 |
| Mod 3 | 150 |
| Mod 4 | 160 |
| Mod 5 | 170 |
| Mod 6 | 180 |

#### Skin Score

| Skin Type | Points |
|-----------|--------|
| Store skin | 15 |
| L2D skin | 21 |
| Event skin | 8 |

**Collection Bonuses:**
- 100% collection: +30 pts
- 75%+ collection: +15 pts
- 50%+ collection: +8 pts

#### Completion Status

Operators are categorized by investment level:

| Status | Criteria |
|--------|----------|
| Not Started | E0, no mastery, no modules |
| In Progress | Some investment but no M3 |
| Partially Completed | At least one M3 skill |
| Highly Invested | M6 or multiple max modules |
| Absolutely Completed | M9 (all skills at M3) |

### Stage Scoring

Stage completion tracks progress across game modes:

| Mode | Description |
|------|-------------|
| Mainline | Main story chapters |
| Sidestory | Side story events (zones with "side" in ID) |
| Activity | Limited-time events |

**Metrics tracked:**
- Completion percentage per mode
- Total stages completed
- Perfect (3-star) clears

### Roguelike Scoring (Integrated Strategies)

Progress across all IS themes (Phantom, Mizuki, Sami, Sarkaz, Babel):

| Achievement | Points |
|-------------|--------|
| Theme played | 50 |
| Ending unlocked | 25 |
| BP level | 5 |
| Buff unlocked | 10 |
| Band unlocked | 3 |
| Relic unlocked | 2 |
| Capsule unlocked | 2 |
| Challenge grade cleared | 15 |
| Grade 2 (max difficulty) clear | 25 |
| Theme at max difficulty | 100 |

**Breakdown includes:**
- Themes played
- Total endings unlocked
- Total BP levels
- Total buffs
- Total collectibles (bands + relics + capsules)
- Total runs
- Grade 2 challenges cleared
- Themes at max difficulty

### Sandbox Scoring (Reclamation Algorithm)

Progress in RA mode:

| Achievement | Points |
|-------------|--------|
| Place discovered | 5 |
| Place completed | 15 |
| Battle node | 3 |
| Choice node | 2 |
| Event node | 2 |
| Ending node | 25 |
| Tech node | 5 |
| Treasure node | 3 |
| Landmark node | 8 |
| Special node | 5 |
| Tech tree completed | 50 |
| Story unlocked | 10 |
| Event completed | 1 |
| Log entry collected | 2 |
| Chapter with logs | 15 |

**Breakdown includes:**
- Places completed/discovered/total
- Completion percentage
- Nodes completed by type
- Tech trees completed
- Stories unlocked
- Events triggered
- Log entries collected

### Medal Scoring

Progress in earning game medals with rarity-based point values:

#### Base Points (by Rarity)

| Rarity | Points | Description |
|--------|--------|-------------|
| T1 (Common) | 5 | Basic achievement medals |
| T2 (Uncommon) | 15 | Intermediate medals |
| T3 (Rare) | 50 | Difficult achievement medals |
| T2D5 (Special) | 75 | Special difficulty medals |

#### Category Multipliers

| Category | Multiplier | Description |
|----------|------------|-------------|
| Records Medal (playerMedal) | 1.0x | Player statistics |
| Episodes Medal (stageMedal) | 1.1x | Stage completion |
| Annihilation Medal (campMedal) | 1.2x | Annihilation clears |
| SSS Medal (towerMedal) | 1.3x | Stationary Security Service |
| Progress Medal (growthMedal) | 1.0x | Account progression |
| Chronicles Medal (storyMedal) | 1.0x | Story completion |
| Base Medal (buildMedal) | 1.0x | RIIC achievements |
| Event Medal (activityMedal) | 1.0x | Limited-time events |
| Traveler Medal (rogueMedal) | 1.2x | Integrated Strategies |
| Secret Medal (hiddenMedal) | 1.5x | Hidden achievements |

#### Group Completion Bonuses

| Group Size | Bonus |
|------------|-------|
| Small (1-5 medals) | 25 pts |
| Medium (6-10 medals) | 50 pts |
| Large (11+ medals) | 100 pts |

**Breakdown includes:**
- Total medals earned/available
- Completion percentage by rarity
- Medals earned per category
- Groups fully completed

### Base (RIIC) Efficiency Scoring

Scores for your Rhodes Island Infrastructure Complex based on operational efficiency:

#### Trading Post (per room)

| Component | Formula |
|-----------|---------|
| Base | 50 pts |
| Level | 25 pts x level |
| Efficiency | (speed - 1.0) x 100 pts |
| Max efficiency (>=200%) | +200 pts |
| Gold strategy | +10 pts |
| Order capacity | (stockLimit - 6) x 5 pts |
| Presets | 25 pts per additional preset |
| Full rotation | +15 pts x presets if all full |

#### Factory (per room)

| Component | Formula |
|-----------|---------|
| Base | 50 pts |
| Level | 25 pts x level |
| Efficiency | (speed - 1.0) x 100 pts |
| Max efficiency (>=200%) | +200 pts |
| Capacity | (capacity - base) x 2 pts |
| Presets | 25 pts per additional preset |

#### Power Plant (per room)

| Component | Formula |
|-----------|---------|
| Base | 30 pts |
| Level | 20 pts x level |
| Electricity | output x 0.5 pts |
| Drone recovery | 10 pts x level |

Electricity output: 60/130/270 for levels 1/2/3

#### Dormitory (per room)

| Component | Formula |
|-----------|---------|
| Base | 20 pts |
| Level | 15 pts x level |
| Comfort | comfort x 0.02 pts |
| Max comfort (5000) | +50 pts |

#### Control Center

| Component | Formula |
|-----------|---------|
| Base | 100 pts |
| Level | 50 pts x level |
| Buff score | (trading + manufacture buff) x 200 pts |
| AP reduction | |apCost| x 2 pts |
| Presets | 25 pts per additional preset |

#### Reception Room & Office

| Component | Formula |
|-----------|---------|
| Base | 40 pts |
| Level | 20 pts x level |
| Operators | 10 pts per stationed operator |

#### Global Bonuses

| Bonus | Points | Condition |
|-------|--------|-----------|
| Electricity Balanced | 100 | Output >= consumption |
| High Efficiency | 300 | Average efficiency > 150% |
| Full Base | 500 | All production buildings at max level |

**Breakdown includes:**
- Building counts and levels
- Average trading/factory efficiency
- Total comfort across dormitories
- Electricity balance
- Labor metrics
- Operator distribution (production/support/rest)

### Architecture

The scoring system is modular with dedicated calculators:

```
src/core/user/score/
├── mod.rs                 # Module exports
├── types.rs               # UserScore, ScoreBreakdown
├── calculate.rs           # Main aggregation function
├── operators/
│   ├── mod.rs
│   ├── types.rs           # OperatorScore, CompletionStatus
│   ├── calculate.rs       # Per-operator scoring
│   └── helpers.rs         # Score calculation helpers
├── stages/
│   ├── mod.rs
│   ├── types.rs           # ZoneScore, StageBreakdown
│   └── calculate.rs       # Stage completion scoring
├── roguelike/
│   ├── mod.rs
│   ├── types.rs           # RoguelikeScore, ThemeScore
│   └── calculate.rs       # IS progress scoring
├── sandbox/
│   ├── mod.rs
│   ├── types.rs           # SandboxScore, AreaScore
│   └── calculate.rs       # RA progress scoring
├── medal/
│   ├── mod.rs
│   ├── types.rs           # MedalScore, MedalCategoryScore
│   └── calculate.rs       # Medal achievement scoring
└── base/
    ├── mod.rs
    ├── types.rs           # BaseScore, building types
    └── calculate.rs       # RIIC efficiency scoring
```

### Usage Example

```rust
use backend::core::user::score::calculate_user_score;

// Calculate score for a user
let score = calculate_user_score(&user, &game_data);

println!("Total Score: {:.2}", score.total_score);
println!("Operator Score: {:.2}", score.operator_score);
println!("Stage Score: {:.2}", score.stage_score);
println!("Roguelike Score: {:.2}", score.roguelike_score);
println!("Sandbox Score: {:.2}", score.sandbox_score);
println!("Medal Score: {:.2}", score.medal_score);
println!("Base Score: {:.2}", score.base_score);

// Access breakdown
println!("6-star operators: {}", score.breakdown.six_star_count);
println!("M9 operators: {}", score.breakdown.m9_count);
println!("Mainline completion: {:.1}%", score.breakdown.mainline_completion);
println!("Medals earned: {}/{}", score.breakdown.medal_total_earned, score.breakdown.medal_total_available);
println!("Base efficiency: {:.1}%", score.breakdown.base_avg_trading_efficiency);
```

### Response Structure

```json
{
  "totalScore": 242010.22,
  "operatorScore": 203431.72,
  "stageScore": 2780.00,
  "roguelikeScore": 9728.00,
  "sandboxScore": 2682.00,
  "medalScore": 11011.50,
  "baseScore": 12377.00,
  "operatorScores": [
    {
      "charId": "char_002_amiya",
      "name": "Amiya",
      "rarity": 5,
      "baseScore": 400,
      "levelScore": 330,
      "trustScore": 50,
      "potentialScore": 100,
      "masteryScore": 450,
      "moduleScore": 300,
      "skinScore": 45,
      "totalScore": 1675,
      "completionStatus": "absolutely_completed",
      "masteryDetails": { "m3Count": 3, "totalMasteryLevels": 9 },
      "moduleDetails": { "modulesUnlocked": 2, "modulesAtMax": 2, "highestLevel": 3 },
      "skinDetails": { "ownedCount": 3, "totalAvailable": 5, "completionPercentage": 60.0 }
    }
  ],
  "zoneScores": [...],
  "roguelikeThemeScores": [...],
  "sandboxAreaScores": [...],
  "medalCategoryScores": [
    {
      "category": "stageMedal",
      "categoryName": "Episodes Medal",
      "totalScore": 1250.50,
      "medalsEarned": 45,
      "medalsAvailable": 60,
      "completionPercentage": 75.0,
      "t1Earned": 20,
      "t2Earned": 15,
      "t3Earned": 8,
      "t2d5Earned": 2
    }
  ],
  "medalDetails": {
    "totalScore": 11011.50,
    "rarityScore": 9500.00,
    "categoryBonusScore": 1011.50,
    "groupBonusScore": 500.00,
    "breakdown": {
      "totalMedalsEarned": 450,
      "totalMedalsAvailable": 1304,
      "totalCompletionPercentage": 34.5,
      "groupsComplete": 12,
      "groupsTotal": 85
    }
  },
  "baseDetails": {
    "totalScore": 12377.00,
    "tradingScore": 505.00,
    "factoryScore": 728.00,
    "powerScore": 765.00,
    "dormitoryScore": 980.00,
    "controlCenterScore": 8569.00,
    "receptionScore": 120.00,
    "officeScore": 110.00,
    "globalBonusScore": 600.00,
    "tradingPosts": [...],
    "factories": [...],
    "powerPlants": [...],
    "dormitories": [...],
    "controlCenter": {
      "slotId": "slot_34",
      "level": 5,
      "globalBuffScore": 8364.00,
      "apCostScore": 150.00,
      "totalScore": 8569.00,
      "details": {
        "tradingBuff": 20.07,
        "manufactureBuff": 20.00,
        "apCostReduction": 75,
        "operatorsStationed": 5,
        "presetCount": 2,
        "operatorsPerPreset": [5, 5]
      }
    },
    "breakdown": {
      "tradingPostCount": 2,
      "factoryCount": 4,
      "powerPlantCount": 3,
      "dormitoryCount": 4,
      "maxLevelBuildings": 16,
      "avgTradingEfficiency": 90.0,
      "avgFactoryEfficiency": 94.25,
      "totalComfort": 20000,
      "electricityBalance": 70
    }
  },
  "breakdown": {
    "totalOperators": 280,
    "sixStarCount": 45,
    "fiveStarCount": 80,
    "m9Count": 15,
    "m3Count": 50,
    "e2Count": 120,
    "mainlineCompletion": 100.0,
    "sidestoryCompletion": 85.5,
    "activityCompletion": 72.3,
    "roguelikeThemesPlayed": 5,
    "roguelikeTotalEndings": 25,
    "sandboxPlacesCompleted": 45,
    "sandboxCompletionPercentage": 90.0,
    "medalTotalEarned": 450,
    "medalTotalAvailable": 1304,
    "medalCompletionPercentage": 34.5,
    "medalGroupsComplete": 12,
    "baseTradingPostCount": 2,
    "baseFactoryCount": 4,
    "baseAvgTradingEfficiency": 90.0,
    "baseMaxLevelBuildings": 16
  }
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run DPS comparison tests only
DATA_DIR=/path/to/gamedata cargo test --test dps_comparison_test
```

### DPS Comparison Tests

The DPS comparison test suite validates the Rust implementations against the Python reference. It includes **5,100 test cases** covering **281 operators** across multiple scenarios with **100% pass rate**.

#### Test Configuration

Each test case specifies:
- Operator name
- Skill index (1-based)
- Module index (0 for none, 1+ for modules)
- Defense value
- Resistance value

Test scenarios per operator:
| Defense | Resistance | Scenario |
|---------|------------|----------|
| 0 | 0 | Baseline (no mitigation) |
| 300 | 0 | Low defense |
| 0 | 20 | Low resistance |
| 500 | 30 | Medium defense + resistance |
| 1000 | 50 | High defense + resistance |

#### Running DPS Tests

```bash
# Set game data directory
export DATA_DIR=/path/to/gamedata/excel

# Run DPS comparison tests
cargo test --test dps_comparison_test -- --nocapture

# Output:
# === DPS Comparison Results ===
# Tested: 5100, Passed: 5100, Failed: 0, Skipped: 450
# Pass rate: 100.0%
```

#### Running Python Comparison

The Python runner executes the reference implementation and outputs DPS values:

```bash
# Run single operator calculation
python3 scripts/python_dps_harness.py SilverAsh 500 0 --skill 3 --module 1

# Output:
{
  "operator": "SilverAsh",
  "name": "SilverAsh P1 S3 ModX3 rangedAtk",
  "defense": 500.0,
  "res": 0.0,
  "skill": 3,
  "module": 1,
  "dps": 5432.10,
  "atk": 892.0,
  "attack_speed": 133.0,
  "attack_interval": 1.3
}
```

### Score Calculation Tests

The score calculation test suite validates the user scoring system with real user data.

#### Running Score Tests

```bash
# Set data directories
export DATA_DIR=/path/to/gamedata/excel
export ASSETS_DIR=/path/to/assets

# Run score calculation tests
cargo test --test score_calculation_test -- --nocapture

# Output:
# === User Score Calculation Results ===
# Total Score: 242,010.22
# ├── Operator Score: 203,431.72
# ├── Stage Score: 2,780.00
# ├── Roguelike Score: 9,728.00
# ├── Sandbox Score: 2,682.00
# ├── Medal Score: 11,011.50
# └── Base Score: 12,377.00
```

#### Score Output Files

The tests generate JSON files in `tests/score_calculation/output/`:

| File | Description |
|------|-------------|
| `user_score_full.json` | Complete scoring data |
| `operator_scores.json` | Individual operator scores |
| `zone_scores.json` | Stage completion by zone |
| `roguelike_scores.json` | IS theme progress |
| `sandbox_scores.json` | RA area progress |
| `medal_scores.json` | Medal category breakdown |
| `base_scores.json` | RIIC efficiency breakdown |
| `score_breakdown.json` | Summary statistics |

### Generating Test Files

The test generation system extracts operator data from the Python source and generates test files automatically.

#### Step 1: Extract Operator Data

```bash
# Generate operators.json from ArknightsDpsCompare source
python3 scripts/extract_operator_data.py > tests/dps_comparison/operators.json
```

This script parses:
- `external/ArknightsDpsCompare/Database/JsonReader.py` - Character IDs
- `external/ArknightsDpsCompare/damagecalc/damage_formulas.py` - Skills and modules

Output format (`operators.json`):
```json
[
  {
    "class_name": "Aak",
    "display_name": "Aak",
    "char_id": "char_225_haak",
    "rust_module": "aak",
    "skills": [1, 3],
    "modules": [1, 2],
    "default_module": 1
  }
]
```

#### Step 2: Generate Test Files

```bash
# Generate all test files from operators.json
cargo run --bin generate-dps-tests
```

This generates:
- `tests/dps_comparison/test_config.json` - Test case configurations
- `tests/dps_comparison/dps_comparison_test.rs` - Rust integration tests
- `tests/dps_comparison/expected_dps.json` - Pre-computed Python DPS values

#### Step 3: Run Tests

```bash
# Run Rust tests
DATA_DIR=/path/to/gamedata cargo test --test dps_comparison_test -- --nocapture
```

### Test Files Reference

| File | Description |
|------|-------------|
| `tests/dps_comparison/operators.json` | Extracted operator metadata (281 operators) |
| `tests/dps_comparison/test_config.json` | Test case configurations |
| `tests/dps_comparison/expected_dps.json` | Pre-computed Python DPS values (5,100+ cases) |
| `tests/dps_comparison/dps_comparison_test.rs` | Generated Rust integration tests |
| `tests/dps_comparison/run_comparison.py` | Python test runner script |

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

## External Dependencies

### ArknightsDpsCompare (Git Submodule)

The project includes [ArknightsDpsCompare](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare) as a git submodule for DPS calculation reference data.

#### Location

```
external/ArknightsDpsCompare/
├── Database/
│   ├── JsonReader.py      # Operator ID mappings (id_dict)
│   └── StageAnimator.py   # Animation data
└── damagecalc/
    ├── damage_formulas.py # Operator damage implementations
    ├── healing_formulas.py
    ├── utils.py
    └── commands.py
```

#### Setup

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/Eltik/myrtle.moe.git

# Or initialize after cloning
git submodule update --init --recursive

# Update submodule to latest
git submodule update --remote external/ArknightsDpsCompare
```

#### Usage

The submodule is used by:
- `src/bin/translate_operators.rs` - Translates Python operators to Rust
- `scripts/extract_operator_data.py` - Extracts operator metadata
- `scripts/python_dps_harness.py` - Runs Python DPS calculations
- `tests/dps_comparison/run_comparison.py` - Reference test runner

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
    score JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Settings fields:**
- `publicProfile` (boolean) - Whether profile appears in leaderboard/search results

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

### Internal Service Bypass

Internal services (like the frontend's API routes) can bypass rate limiting by including a secret key in the request headers:

```bash
# Header for rate limit bypass
X-Internal-Service-Key: <your-internal-service-key>
```

**Security features:**
- Uses constant-time comparison to prevent timing attacks
- Header name is intentionally non-obvious (`x-internal-service-key`)
- Key is never exposed to browser clients (server-side only)
- Missing or invalid key gracefully falls back to normal rate limiting

**Setup:**
1. Generate a secure key: `openssl rand -base64 32`
2. Set `INTERNAL_SERVICE_KEY` environment variable on both backend and frontend
3. Frontend API routes automatically include the header via `backendFetch()` utility

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

The project includes several CLI tools for various tasks:

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

#### DPS Test Generator

Generate DPS comparison test files from operator data:

```bash
# Build and run
cargo run --bin generate-dps-tests

# Or build release binary
cargo build --release --bin generate-dps-tests
./target/release/generate-dps-tests
```

Output:
```
Generating DPS comparison tests...
Output directory: tests/dps_comparison
Loaded 281 operators from operators.json
Generated 5100 test cases

Generated test files:
  - tests/dps_comparison/test_config.json
  - tests/dps_comparison/dps_comparison_test.rs
  - tests/dps_comparison/expected_dps.json
```

#### Operator Translator

Translate Python operator implementations to Rust:

```bash
# Translate all operators
cargo run --bin translate-operators -- \
  external/ArknightsDpsCompare/damagecalc/damage_formulas.py \
  src/core/dps_calculator/operators

# Output:
# Parsing Python file...
# Found 281 operator classes
# Generating Rust files...
#   a/  (21 operators)
#   b/  (11 operators)
#   ...
# Next steps:
#   1. Run 'cargo build' to check for compilation errors
#   2. Fix any translation issues manually
#   3. Run tests to verify DPS calculations
```

The translator handles:
- Python-to-Rust syntax conversion
- NumPy function translation
- Self-reference conversion (self.x → self.unit.x)
- Operator-specific edge cases
- Init-time modifications
- Module-dependent calculations

### Scripts

Python scripts for DPS calculation and data extraction:

#### python_dps_harness.py

Python harness for running DPS calculations against the reference implementation:

```bash
# Calculate DPS for an operator
python3 scripts/python_dps_harness.py SilverAsh 500 0 --skill 3 --module 1

# Output:
{
  "operator": "SilverAsh",
  "name": "SilverAsh P1 S3 ModX3 rangedAtk",
  "defense": 500.0,
  "res": 0.0,
  "skill": 3,
  "module": 1,
  "dps": 5432.10,
  "atk": 892.0,
  "attack_speed": 133.0,
  "attack_interval": 1.3
}
```

Functions available:
| Function | Description |
|----------|-------------|
| `calculate_dps(operator, defense, res, skill, module)` | Calculate DPS for an operator |
| `list_operators()` | List all available operator names |
| `get_operator_info(operator)` | Get operator metadata |

#### extract_operator_data.py

Extract operator metadata from ArknightsDpsCompare source files:

```bash
# Generate operators.json
python3 scripts/extract_operator_data.py > tests/dps_comparison/operators.json

# Output goes to stdout, logs to stderr
python3 scripts/extract_operator_data.py 2>/dev/null > operators.json
```

This script parses:
- `id_dict` from `JsonReader.py` (operator name to char_id mapping)
- Class definitions from `damage_formulas.py` (skills, modules)

Output fields:
| Field | Description |
|-------|-------------|
| `class_name` | Python class name (e.g., "SilverAsh") |
| `display_name` | Display name from init (e.g., "SilverAsh") |
| `char_id` | Game character ID (e.g., "char_003_kalts") |
| `rust_module` | Rust module name (e.g., "silver_ash") |
| `skills` | Available skill indices (e.g., [1, 2, 3]) |
| `modules` | Available module indices (e.g., [1, 2]) |
| `default_module` | Default module for testing |

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

#### DPS calculator operator not found

**Cause:** Operator not implemented or name mismatch.

**Solution:**
```bash
# List available operators
curl http://localhost:3060/dps-calculator/operators

# Use exact operatorId from game data
curl -X POST http://localhost:3060/dps-calculator \
  -H "Content-Type: application/json" \
  -d '{"operatorId": "char_017_huang"}'  # Use char_id, not name
```

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
