# myrtle.moe Discord Bot

The Discord companion for [myrtle.moe](https://myrtle.moe). It moderates guilds, announces Arknights asset updates the moment the [asset pipeline](../assets/README.md) finishes an extraction, and looks up platform stats and player profiles from the [Myrtle backend](../backend/README.md). Built in Rust on Poise and Serenity with an embedded SQLite database.

[![Rust](https://img.shields.io/badge/Rust-edition_2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Poise](https://img.shields.io/badge/Poise-0.6-5865f2?logo=discord&logoColor=white)](https://github.com/serenity-rs/poise)
[![Serenity](https://img.shields.io/badge/Serenity-0.12-5865f2)](https://github.com/serenity-rs/serenity)
[![SQLite](https://img.shields.io/badge/SQLite-embedded-003b57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Discord CI](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

> **Status: work in progress.** Moderation, warnings, asset announcements, operator birthday announcements, the two `/api` lookups, and the `/collection` game-data lookups below are implemented. The rest of the "game data in Discord" surface - DPS/HPS, recruitment, tier lists, leaderboards - is planned but not built. See [`TODO.md`](TODO.md) for the backlog.

## Features

- **Moderation** - ban, unban, kick, and bulk purge, each written to the Discord audit log with a reason.
- **Warnings** - warn members with a reason, DM them, review or remove warnings, and optionally escalate to a timeout, kick, or ban once a member reaches a threshold.
- **Mod role** - grant moderation access by role instead of by Discord permission. A command passes for bot owners, for the configured mod role, **or** for the native permission.
- **Auto role** - assign a role automatically when a member joins.
- **Reaction roles** - bind emoji reactions on a message to roles.
- **Anti-spam** - per-guild ping limits over a sliding window, with a configurable action (delete, warn, timeout, kick, or ban) and an exempt role.
- **Audit logging** - mirror Discord audit events into a channel, with per-category toggles.
- **Asset announcements** - one WebSocket connection per Arknights region (EN, CN, JP, KR) to the asset pipeline, posting to a bound channel when an extraction completes. Each guild can limit announcements to some regions.
- **Game-data lookups** - `/collection` shows operators, enemies, stages, and story groups from the public backend, with fuzzy, accent-insensitive autocomplete.
- **Operator birthdays** - a daily post at the EN reset (04:00 UTC-7) for every operator whose birthday it is, plus `today` and `upcoming` lookups.
- **Backend lookups** - endpoint reachability and latency, and platform statistics.
- **Embed builder** - create and edit rich embeds from fields or raw JSON, and read an existing embed back as JSON.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (Edition 2024) |
| Framework | Poise 0.6 over Serenity 0.12 |
| Runtime | Tokio (multi-threaded) |
| Database | SQLite via `sqlx` 0.9 (runtime queries, embedded migrations) |
| HTTP | `reqwest` 0.13 for backend API calls |
| WebSocket | `tokio-tungstenite` 0.29 for the asset-pipeline watcher |
| Logging | `tracing` + `tracing-subscriber` (env filter, JSON) |

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | 1.85.0+ | Edition 2024 |
| Discord bot token | - | From the [Discord Developer Portal](https://discord.com/developers/applications) |
| Backend | - | Optional. Only the `/api` commands need one |
| Asset pipeline | - | Optional. Only the `/assets` commands need one |

SQLite needs no separate install. The database file is created on first run and migrations are compiled into the binary, so there is no `sqlx database setup` step. Install `sqlx-cli` only if you want offline tooling for local schema work.

## Quick Start

### 1. Configure

```bash
cp .env.example .env
cp config.example.json config.json
```

Set `DISCORD_TOKEN` in `.env`. The bot panics on startup without it.

### 2. Run

```bash
cargo run --bin discord
```

### 3. Register slash commands

Commands are registered by a separate binary, so you can iterate on the bot without re-registering on every restart:

```bash
# Guild-scoped: appears immediately, best for development
cargo run --bin commands -- deploy guild <guild_id>

# Global: propagates across Discord in up to an hour
cargo run --bin commands -- deploy global

# Remove previously registered commands
cargo run --bin commands -- reset guild <guild_id>
```

### 4. Docker

```bash
docker build -t myrtle-discord .
docker run --env-file .env \
  -v "$(pwd)/config.json:/app/config.json:ro" \
  -v myrtle_discorddata:/data \
  myrtle-discord
```

The image is a two-stage build (`rust:1-bookworm` builder, `debian:bookworm-slim` runtime) carrying both binaries. `config.json`, the token, and the SQLite file are mounted at runtime and never baked into the image. The bot exposes no ports; it dials out to Discord and to the asset pipeline.

## Commands

Elevated commands pass for bot owners, for the guild's configured mod role, or for the native Discord permission listed.

### General

| Command | Permission | Description |
|---------|------------|-------------|
| `ping` | Owner | Health check. Also available as a prefix command |
| `say <message>` | Manage Messages | Speak as the bot |
| `/embed create` · `edit` · `source` | Manage Messages | Build, edit, or dump a rich embed as JSON |

### Moderation

| Command | Permission | Description |
|---------|------------|-------------|
| `/ban_user <user> [reason] [delete_days]` | Ban Members | Ban with an audit-log reason |
| `/unban_user <user> [reason]` | Ban Members | Unban. Autocompletes banned users, or takes a raw ID |
| `/kick_user <user> [reason]` | Kick Members | Kick with an audit-log reason |
| `/purge <count>` | Manage Messages | Bulk-delete recent messages |
| `/warn add <user> <reason>` | Timeout Members | Warn a member: DM them, mirror to the audit-log channel, run the escalation policy |
| `/warn list` · `remove` · `clear` | Timeout Members | Review a member's warnings, remove one by id, or remove all of them |
| `/warn policy set` · `show` · `clear` | Manage Guild | Threshold, optional day window, and action (timeout, kick, or ban). No policy means no escalation |
| `/modrole set` · `show` · `remove` | Manage Guild | Configure the role that substitutes for a permission |

### Automation

| Command | Permission | Description |
|---------|------------|-------------|
| `/autorole set` · `clear` · `show` | Manage Roles | Role granted automatically to new members |
| `/reactionrole add` · `remove` · `list` · `delete` | Manage Roles | Bind emoji reactions to roles |
| `/antispam set` · `clear` · `show` | Manage Guild | Ping limits, window, action, and exempt role |
| `/auditlog set` · `clear` · `show` · `enable` · `disable` | Manage Guild | Mirror audit events to a channel, per category |
| `/birthday channel set` · `clear` · `show` | Manage Guild | Bind the daily operator-birthday post. A channel bound after today's reset starts tomorrow |

### Platform

| Command | Permission | Description |
|---------|------------|-------------|
| `/api status` | - | Reachability and latency for all four configured endpoints |
| `/api stats` | - | Platform counts from `GET /api/stats` |
| `/assets channel set [servers]` · `clear` · `show` | Owner | Bind the asset-announcement channel. `servers` (e.g. `EN,JP`) limits it to some regions; omit for all |
| `/assets status` | - | Last-known state of each region's asset watcher |
| `/assets resources` | - | Live `list_resources` round-trip to the asset pipeline |

### Game data

| Command | Permission | Description |
|---------|------------|-------------|
| `/collection operator <query> [compact]` | - | Rarity, class and branch, faction, tags, birthday, artists, voice actors, art. `compact:false` adds attributes, trait, talents, potentials, skills at max level, modules, base skills, costs, and profile, over as many embeds as it takes |
| `/collection enemy <query>` | - | Class, attack and damage type, level-0 HP/ATK/DEF/RES, abilities |
| `/collection stage <query>` | - | Code and name, zone, sanity, danger level, the Challenge Mode / Extreme / Adverse condition, enemies, drops. Matches codes like `1-7` or `CE-6` |
| `/collection story <query>` | - | Story group, category, story and word counts, banner art, link to the reader |
| `/birthday today` | - | Today's birthday operators (the game day turns at 04:00 UTC-7) |
| `/birthday upcoming [days]` | - | Birthdays over the next 1 to 31 days, 7 by default |

## Configuration

### Environment

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | Yes | - | Bot token. Panics if unset |
| `DISCORD_ID` | No | - | Application ID |
| `DATABASE_URL` | No | `sqlite:database.sqlite` | SQLite path. Created if missing |
| `DISCORD_CONFIG_PATH` | No | `./config.json` | Config file location |
| `RUST_LOG` | No | `discord=info` | Tracing filter |

### `config.json`

Every section is optional, and unknown fields are rejected. Copy [`config.example.json`](config.example.json) as a starting point.

| Field | Description |
|-------|-------------|
| `endpoints.local_backend` | Backend URL used by `/api stats` |
| `endpoints.local_frontend` | Checked by `/api status` |
| `endpoints.public_backend` | Checked by `/api status`. Source of `/collection` and birthday data |
| `endpoints.public_frontend` | Checked by `/api status`. Base of the links in `/collection` and birthday embeds |
| `assets.servers[]` | One `{ label, ws_url }` per Arknights region: EN 9160, CN 9161, JP 9162, KR 9163 by default |
| `assets.reconnect_secs` | WebSocket reconnect backoff. Defaults to `5` |

Omit `assets` entirely and the watcher subsystem stays off. The legacy singular `assets.ws_url` is still honoured and is treated as one server labelled `EN`.

When the bot runs in Docker against a pipeline on the host, point `ws_url` at `ws://host.docker.internal:<port>`.

## Backend Integration

`/api` commands call the Myrtle backend over plain HTTP:

| Command | Request |
|---------|---------|
| `/api stats` | `GET {local_backend}/api/stats` |
| `/api status` | `GET` against each of the four configured endpoint URLs |

`stats` currently targets the local backend only.

`/collection` and the birthday announcer read the public backend. There is no name-search endpoint, so the bot fetches whole lists and matches locally; each list is cached in memory for 30 minutes and refreshed in the background, keeping the stale copy if a refresh fails:

| Data | Request |
|------|---------|
| Operators | `GET {public_backend}/api/operators/index` |
| Enemies | `GET {public_backend}/api/static/enemies` |
| Stages | `GET {public_backend}/api/static/stage-index`, plus `/api/stages/{id}/detail` per lookup |
| Full operator card | `GET {public_backend}/api/operators/{id}` per lookup, plus `/api/static/materials` (cached) for item names |
| Stories | `GET {public_backend}/api/story/index` |
| Images | `/api/avatar/{id}`, `/api/charart/{id}`, `/api/enemy-icon/{id}`, `/api/assets/{path}` |

## Database

Six migrations in [`migrations/`](migrations/) are embedded at compile time and applied automatically on startup.

| Table | Purpose |
|-------|---------|
| `guild_auto_role` | Role granted to joining members |
| `guild_reaction_roles` | Emoji-to-role bindings |
| `guild_asset_channel` | Asset-announcement channel, plus an optional server filter (NULL means every server) |
| `guild_max_ping` | Anti-spam policy |
| `guild_audit_log` | Audit-log channel plus a `disabled_events` bitmask |
| `guild_mod_role` | Mod role. A deleted row means no mod role |
| `guild_warnings` | One row per warning: member, moderator, reason, time |
| `guild_warn_policy` | Warning escalation policy. No row means no escalation |
| `guild_birthday_channel` | Birthday channel plus the last game day posted, so restarts never double-post |

`guild_audit_log` stores the **disabled** event set rather than the enabled one, so new audit categories default to on for guilds that configured logging before those categories existed.

All queries are runtime `sqlx::query`, not the compile-time macros, so `SQLX_OFFLINE=true` builds need neither a live database nor a `.sqlx` cache.

## Project Structure

```
src/
├── main.rs           Entry: load config, init pool, start the client and watchers
├── lib.rs            Module exports
├── bin/commands.rs   Standalone slash-command deployer
├── cmds/             Command implementations
│   ├── general.rs    ping, say, embed
│   ├── admin.rs      ban, kick, purge, modrole, autorole, antispam, reactionrole
│   ├── api.rs        /api status, stats
│   ├── assets.rs     /assets channel, status, resources
│   ├── auditlog.rs   /auditlog configuration
│   ├── birthday.rs   /birthday channel, today, upcoming
│   ├── collection.rs /collection operator, enemy, stage, story
│   ├── operator_card.rs  The full operator card (`compact:false`)
│   └── warn.rs       /warn add, list, remove, clear, policy
├── api/              Backend HTTP clients (status, stats, cached game data)
├── birthday.rs       Game-day dates, birthday parsing, the daily announcer
├── search.rs         Accent- and punctuation-insensitive name matching
├── gametext.rs       Game markup stripping and `{key:0%}` template interpolation
├── checks.rs         elevated(): owner OR mod role OR native permission
├── handler.rs        Gateway event handling
├── hooks.rs          Pre- and post-command hooks
├── audit.rs          Audit-event mirroring
├── config.rs         config.json parsing
└── db.rs             Pool init and embedded migrations
```

## Development

```bash
cargo fmt                                        # Format
cargo clippy --all-targets -- -D warnings        # Lint
cargo check --all-targets                        # Type-check
cargo build --release --bin discord --bin commands
```

CI ([`discord-ci.yml`](../.github/workflows/discord-ci.yml)) runs rustfmt, Clippy with `clippy::all`, `clippy::pedantic`, and `clippy::nursery` as denied warnings, `cargo check`, a release build of both binaries, and a non-blocking `cargo audit`.

## Contributing

1. Branch from `main`.
2. Add new commands in `src/cmds/` and register them in `src/cmds/mod.rs::all()`.
3. Gate elevated commands with `checks::elevated()` rather than Poise's `required_permissions`, so the mod-role path keeps working.
4. Add schema changes as a new timestamped migration. Never edit an applied one.
5. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo check --all-targets` before opening a pull request.

See the repo-level [CONTRIBUTING.md](../CONTRIBUTING.md) for the full workflow.

## License

License TBD - see the [project root](../README.md).

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
