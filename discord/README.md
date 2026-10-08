# myrtle.moe Discord Bot

The Discord companion for [myrtle.moe](https://myrtle.moe). It moderates guilds, announces Arknights asset updates the moment the [asset pipeline](../assets/README.md) finishes an extraction, and looks up platform stats and player profiles from the [Myrtle backend](../backend/README.md). Built in Rust on Poise and Serenity with an embedded SQLite database.

[![Rust](https://img.shields.io/badge/Rust-edition_2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Poise](https://img.shields.io/badge/Poise-0.6-5865f2?logo=discord&logoColor=white)](https://github.com/serenity-rs/poise)
[![Serenity](https://img.shields.io/badge/Serenity-0.12-5865f2)](https://github.com/serenity-rs/serenity)
[![SQLite](https://img.shields.io/badge/SQLite-embedded-003b57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Discord CI](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

> **Status: work in progress.** Moderation, warnings, asset announcements, operator birthday announcements, text-to-speech in voice channels, the two `/api` lookups, and the `/collection` game-data lookups below are implemented. The rest of the "game data in Discord" surface - DPS/HPS, recruitment, tier lists, leaderboards - is planned but not built. See [`TODO.md`](TODO.md) for the backlog.

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
- **Text-to-speech** - a member in a voice channel who types in that channel's chat is read out there, by an offline Piper voice. On in every server until a moderator turns it off. See [Text-to-speech](#text-to-speech).
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
| Voice | `songbird` 0.6 (send only, DAVE end-to-end encryption), libopus built from vendored source |
| Speech | `sherpa-onnx` 1.13 running a Piper VITS voice, linked statically with onnxruntime and espeak-ng |
| Logging | `tracing` + `tracing-subscriber` (env filter, JSON) |

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | 1.85.0+ | Edition 2024 |
| Discord bot token | - | From the [Discord Developer Portal](https://discord.com/developers/applications) |
| Backend | - | Optional. Only the `/api` commands need one |
| Asset pipeline | - | Optional. Only the `/assets` commands need one |
| CMake | 3.x | Builds the vendored libopus for voice. A C compiler too, which `cargo` already needs |
| TTS voice | - | Optional. Without it TTS is off and everything else runs. See [Installing the voice](#installing-the-voice) |

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
  -v "$(pwd)/tts:/app/tts:ro" \
  -v myrtle_discorddata:/data \
  myrtle-discord
```

The image is a two-stage build (`rust:1-bookworm` builder, `debian:bookworm-slim` runtime) carrying both binaries. `config.json`, the token, the TTS voice, and the SQLite file are mounted at runtime and never baked into the image. The bot exposes no ports; it dials out to Discord and to the asset pipeline.

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
| `/collection operator <query> [page]` | - | One operator, a page at a time, switched by a select under the reply. A second select picks within the page; a third appears only when a page runs past one message. Pages are listed only when the operator has them; `page` opens on one (default Overview). See [Operator pages](#operator-pages) |
| `/collection enemy <query>` | - | Class, attack and damage type, level-0 HP/ATK/DEF/RES, abilities |
| `/collection stage <query>` | - | Code and name, zone, sanity, danger level, the Challenge Mode / Extreme / Adverse condition, enemies, drops. Matches codes like `1-7` or `CE-6` |
| `/collection story <query>` | - | Story group, category, story and word counts, banner art, link to the reader |
| `/birthday today` | - | Today's birthday operators (the game day turns at 04:00 UTC-7) |
| `/birthday upcoming [days]` | - | Birthdays over the next 1 to 31 days, 7 by default |

### Voice

| Command | Permission | Description |
|---------|------------|-------------|
| `/tts enable` · `disable` · `status` | Manage Server | Turn text-to-speech on or off in this server, or see whether it is on and where the bot is speaking. Disabling makes the bot leave voice |
| `/tts skip` | In the bot's channel, or Move Members | Skip the message being read out |
| `/tts leave` | In the bot's channel, or Move Members | Make the bot leave its voice channel |

### Operator pages

| Page | Shows | Its select |
|------|-------|------------|
| Overview | Rarity, class, branch, position, trait, stats with the max trust bonus, attack range, talents as they stand at that promotion, potentials, faction, tags, art | Elite 0 to 2 |
| Skills | Every skill: SP recovery and trigger, initial SP, cost, duration, description, and the skill's own range when it has one | Level 1 to 7, Mastery 1 to 3 |
| Summons | The skill that brings the summon, the summon's trait, stats, range and talents, and its skill | Each summon, with each skill it comes with |
| Modules | Per module: unlock, stat bonus, trait change, talent upgrade, range, summon stat bonus | Stage 1 to 3 |
| Base skills | Name, room, unlock, description | - |
| Upgrade costs | Promotions (LMD included, plus levelling LMD and EXP), skill levels, masteries, module stages | Cost type |
| Outfits | Group and name, description, artists, designers, how it is obtained, release date, full art | Each outfit; past 25, "Earlier" and "More" options page through them |
| Lore | Recruitment blurb, then each archive file with its unlock condition | Each file |
| Voice lines | Voice actors, then the lines of one group, as the site groups them | Greetings, Conversations, Promotions, Battle, Dorm, Special, Other |
| Paradox Simulation | Stage name, unlock, description, enemies by class with spawn counts, reward, map preview | - |

Every select carries the whole view in its `custom_id` (`op:<owner>:<operator>:<page>:<choice>:<part>:<control>`), so a control never expires and nothing is stored. The controls belong to whoever ran the command: anyone else who uses one gets the view they picked as a private copy whose controls are theirs, and the public reply stays as it was. A failure is always answered privately with the reason.

Embeds rather than Components V2: Serenity 0.12.5, the version in `Cargo.lock`, has no Components V2 builders.

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

| `tts.enabled` | `false` turns TTS off without the missing-voice warning. Defaults to `true` |
| `tts.model_dir` | A `vits-piper-*` directory as the release archive unpacks. Defaults to `tts/vits-piper-en_US-ljspeech-medium`, relative to the working directory |
| `tts.model` · `tokens` · `data_dir` | Override one file: the `.onnx` model (default: the only one in `model_dir`), `tokens.txt`, `espeak-ng-data/` |
| `tts.max_chars` | Characters spoken per message after cleanup, cut at a word. Defaults to `300` |
| `tts.idle_secs` | Seconds without speech before the bot leaves. Defaults to `300` |
| `tts.queue_max` | Messages waiting per server; more are dropped silently. Defaults to `10` |
| `tts.user_cooldown_ms` | Least time between two spoken messages from one member. Defaults to `2000` |
| `tts.settle_ms` | Wait after joining before the first playback (see [DAVE](#dave)). Defaults to `1500` |

Omit `assets` entirely and the watcher subsystem stays off. Omit `tts` and TTS looks for the voice at the default path; when it isn't there TTS is off, with one warning at startup. The legacy singular `assets.ws_url` is still honoured and is treated as one server labelled `EN`.

When the bot runs in Docker against a pipeline on the host, point `ws_url` at `ws://host.docker.internal:<port>`.

## Text-to-speech

A member who is in voice channel X and types in X's built-in text chat is read out in X. The bot joins X if it is in no other voice channel of that server; while it is in another one, messages elsewhere are ignored without a reply. Stage channels are not read.

A message is spoken only when its author is in that very channel (from the cache's voice states), and never when it comes from a bot, a webhook, or the system, starts with the `-` prefix, carries a sticker, or has nothing left after cleanup. Cleanup reads mentions as names, custom emoji as their names (a run of one emoji once), URLs as "link", masked links as their label, and fenced code as "code block"; it drops markdown symbols and timestamps, cuts any character repeated more than three times to three, and caps the result at `max_chars` at a word boundary. Attachments are never read, so an attachment with no text says nothing.

Each server has one FIFO queue of `queue_max` messages; more are dropped silently. One member is spoken at most once per `user_cooldown_ms`. The bot leaves 5 seconds after the last human leaves its channel, after `idle_secs` without speech, when it is moved or disconnected, and on `/tts leave` or `/tts disable`. A server can join or leave voice at most once per 5 seconds, which holds each server to 24 gateway voice updates a minute against the 120-per-minute limit on the connection.

Synthesis runs one job at a time across all servers, on one thread, on Tokio's blocking pool. The voice loads on the first message that needs it and stays loaded. If its files are missing at startup, or it fails to load, TTS is off with one warning and the rest of the bot is unaffected.

### The voice

Piper `en_US-ljspeech-medium`, as packaged by sherpa-onnx: one US English female speaker, 22,050 Hz, trained from scratch on [LJ Speech](https://keithito.com/LJ-Speech-Dataset/), which is in the public domain ([model card](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/ljspeech/medium/MODEL_CARD)). Most other English Piper voices are fine-tuned from `lessac`, whose dataset licence is the Blizzard 2013 one, or carry a non-commercial dataset licence (`ryan` is CC BY-NC-SA 4.0), so they were passed over.

The binary links espeak-ng statically (it turns text into phonemes), and espeak-ng is GPL-3.0. Running the bot as a service distributes nothing; handing out the binary, as the CI artifact does, is distribution of a work that includes GPL code.

Measured on macOS arm64 (Apple Silicon), one thread, release build. These are not VPS numbers:

| Text | Synthesis | Audio | Real-time factor |
|------|-----------|-------|------------------|
| 20 chars, 3 words | 0.097 to 0.121 s | 1.149 to 1.277 s | 0.076 to 0.105 |
| 103 chars, 21 words | 0.473 to 0.484 s | 6.437 to 6.612 s | 0.071 to 0.075 |
| 289 chars, 55 words | 1.130 to 1.185 s | 16.006 to 16.451 s | 0.071 to 0.072 |

Loading takes 386 to 415 ms. Peak resident memory for loading the voice and speaking all three is 388.5 to 396.4 MB, so expect the bot's RSS to rise by roughly that much the first time it speaks. Output length varies by up to 0.4 s between runs because VITS samples noise.

### Installing the voice

From `discord/` (the bot's working directory under pm2):

```bash
mkdir -p tts
curl -fL https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-en_US-ljspeech-medium.tar.bz2 \
  | tar xj -C tts
ls tts/vits-piper-en_US-ljspeech-medium
# en_US-ljspeech-medium.onnx  en_US-ljspeech-medium.onnx.json  espeak-ng-data/  MODEL_CARD  tokens.txt
```

The archive is 67.2 MB and unpacks to 82.7 MB on disk (a 63.5 MB model and 18.1 MB of espeak-ng data). `tts/` is git-ignored. Restart the bot afterwards; with no `tts` section in `config.json` it finds the voice at this path.

### Building

| Piece | What the build needs | What the binary needs at runtime |
|-------|----------------------|----------------------------------|
| `sherpa-onnx` (`static` feature) | Network access the first time: its build script downloads `sherpa-onnx-v1.13.8-<os>-<arch>-static-lib.tar.bz2` (23.0 MB for linux-x64, 21.0 MB for osx-arm64) from the sherpa-onnx GitHub releases into `target/sherpa-onnx-prebuilt/`. No CMake, no onnxruntime install | Nothing: onnxruntime, espeak-ng and piper-phonemize are linked in. It links the system `libstdc++`, `libm`, `libpthread` and `libdl` (`libc++` and Foundation on macOS), which every Ubuntu and macOS install has |
| libopus (songbird, via `opus2` `bundled`) | CMake and a C compiler, to build the vendored libopus | Nothing: linked statically |

For an offline build, put the archive in a directory and set `SHERPA_ONNX_ARCHIVE_DIR` to it. The Linux archives are built with GCC 11 (manylinux), which links cleanly against Ubuntu 24.04's newer `libstdc++`.

On the Ubuntu 24.04 VPS, once, before the next `vps-update.sh`:

```bash
sudo apt-get update && sudo apt-get install -y cmake build-essential pkg-config
```

`vps-update.sh` then builds as before (`cargo build --release` in `discord/`), and the pm2 process needs nothing new. The CI runner (`ubuntu-latest`) already has CMake.

### DAVE

Discord voice now requires DAVE end-to-end encryption. Songbird 0.6.0 sends audio without the DAVE layer until the DAVE session reports ready, and clients discard those packets ([songbird#310](https://github.com/serenity-rs/songbird/issues/310)). Songbird exposes no "DAVE ready" event: its last connection event is `DriverConnect`, which `join` already waits for. So the first playback after a join waits a further `settle_ms` (1500 ms). Whether that is always long enough is not measured; if the first message after a join comes out silent, raise `settle_ms`. Later messages in the same session are not affected.

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
| Operator pages | `GET {public_backend}/api/operators/{id}` per view, plus `/api/static/ranges` and `/api/static/handbook` (Paradox Simulations; both cached, loaded at startup). Per page: `/api/static/materials` (cached; costs and rewards), `/api/static/skills` (cached; summon skills), `/api/skins/{id}` (outfits), `/api/voices/{id}` (voice lines, and once per operator to learn whether it has any), `/api/stages/{id}/detail` (Paradox) |
| Stories | `GET {public_backend}/api/story/index` |
| Images | `/api/avatar/{id}`, `/api/charart/{id}`, `/api/skill-icon/{id}`, `/api/module-icon/{id}`, `/api/enemy-icon/{id}`, `/api/assets/{path}` |

## Database

Seven migrations in [`migrations/`](migrations/) are embedded at compile time and applied automatically on startup.

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
| `guild_tts_disabled` | Servers where TTS is off, and since when. No row means on, so a new server needs no setup |

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
│   ├── operator/     /collection operator: page builders, layout and limits, state in custom_id
│   ├── tts.rs        /tts enable, disable, status, skip, leave
│   └── warn.rs       /warn add, list, remove, clear, policy
├── api/              Backend HTTP clients (status, stats, cached game data)
├── birthday.rs       Game-day dates, birthday parsing, the daily announcer
├── tts/              Text-to-speech: gate.rs (speak or not), text.rs (cleanup),
│                     engine.rs (Piper via sherpa-onnx), session.rs (join, queue, leave)
├── search.rs         Accent- and punctuation-insensitive name matching
├── gametext.rs       Game markup stripping and `{key:0%}` template interpolation
├── checks.rs         elevated(): owner OR mod role OR native permission
├── handler.rs        Gateway event handling, including the operator view's selects
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

# Time the real voice and write three WAVs (needs the voice on disk)
TTS_MODEL_DIR=tts/vits-piper-en_US-ljspeech-medium TTS_OUT_DIR=/tmp \
  cargo test --release -- --ignored --nocapture synthesize_three_sentences
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
