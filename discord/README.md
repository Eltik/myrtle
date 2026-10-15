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
- **Text-to-speech** - `/tts join` brings the bot into your voice channel, and members in that channel who type in its chat are read out as "name said: text", in a voice each member picks (25 languages and accents, spoken by Google Translate). See [Text-to-speech](#text-to-speech).
- **Operator voice lines** - `/voiceline` plays an operator's voice line, in any language it is recorded in, in your voice channel, or posts it as an audio file when you aren't in one. See [Voice lines](#voice-lines).
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
| Speech | Google Translate's `translate_tts` endpoint over `reqwest`; MP3 decoded by symphonia |
| Voice lines | Ogg Vorbis recordings from the public backend's `/api/assets/audio`, decoded by symphonia |
| Logging | `tracing` + `tracing-subscriber` (env filter, JSON) |

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | 1.85.0+ | Edition 2024 |
| Discord bot token | - | From the [Discord Developer Portal](https://discord.com/developers/applications) |
| Backend | - | Optional. Only the `/api` commands need one |
| Asset pipeline | - | Optional. Only the `/assets` commands need one |
| CMake | 3.x | Builds the vendored libopus for voice. A C compiler too, which `cargo` already needs |

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
| `/collection operator <query> [page]` | - | One operator, a page at a time, switched by a select under the reply. A second select picks within the page; a third appears only when a page runs past one message. Pages are listed only when the operator has them; `page` opens on one (default Overview). See [Operator pages](#operator-pages) |
| `/collection enemy <query>` | - | Class, attack and damage type, level-0 HP/ATK/DEF/RES, abilities |
| `/collection stage <query>` | - | Code and name, zone, sanity, danger level, the Challenge Mode / Extreme / Adverse condition, enemies, drops. Matches codes like `1-7` or `CE-6` |
| `/collection story <query>` | - | Story group, category, story and word counts, banner art, link to the reader |
| `/birthday today` | - | Today's birthday operators (the game day turns at 04:00 UTC-7) |
| `/birthday upcoming [days]` | - | Birthdays over the next 1 to 31 days, 7 by default |

### Voice

| Command | Permission | Description |
|---------|------------|-------------|
| `/tts join` | In a voice channel | Join your voice channel and read its chat aloud. Refused, with the reason, when you aren't in one, the voice isn't installed, or the bot is reading another channel of the server (it names which) |
| `/tts leave` | In the bot's channel, or Move Members | Make the bot leave its voice channel |
| `/tts skip` | In the bot's channel, or Move Members | Skip the rest of the message being read out |
| `/tts nickname set <name> [user]` · `clear [user]` · `show [user]` | Yourself; Manage Nicknames for someone else | How a name is read before its messages, 1 to 32 characters with no mentions or links |
| `/tts voice set <voice>` · `show` · `clear` | - | The voice your messages are read in, in every server. `voice` autocompletes from the 25 voices |
| `/voiceline <operator> [line] [language]` | - | Play an operator's voice line in your voice channel, after anything being read out there; outside voice, the reply carries it as an `.ogg` file. `line` autocompletes from the operator's lines (default a random one), `language` from the languages it has (default Japanese). See [Voice lines](#voice-lines) |

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

| `tts.enabled` | `false` turns TTS off: `/tts join` says so. Defaults to `true` |
| `tts.default_voice` | The voice for members who haven't picked one, as a `/tts voice` id. Defaults to `en-us`; an unknown id warns and falls back to it |
| `tts.max_chars` | Characters spoken per message after cleanup, cut at a word, not counting the "name said:" prefix. Defaults to `300` |
| `tts.idle_secs` | Seconds without speech before the bot leaves. Defaults to `300` |
| `tts.queue_max` | Messages waiting per server; more are dropped silently. Defaults to `10` |
| `tts.user_queue_max` | Messages one member may have waiting; more from them are dropped silently until some are read. Defaults to `3` |
| `tts.engine` · `model_dir` · `model` · `tokens` · `data_dir` · `voices` · `lexicon` · `speaker` · `user_cooldown_ms` | Deprecated and ignored, with one warning at startup naming the ones set, so an older config still parses. The first eight configured the local Piper and Kokoro voices; `user_cooldown_ms` dropped a member's second message inside its window |
| `tts.settle_ms` | Wait after joining before the first playback (see [DAVE](#dave)). Defaults to `1500` |

Omit `assets` entirely and the watcher subsystem stays off. Omit `tts` and TTS is on with every default. The legacy singular `assets.ws_url` is still honoured and is treated as one server labelled `EN`.

When the bot runs in Docker against a pipeline on the host, point `ws_url` at `ws://host.docker.internal:<port>`.

## Text-to-speech

The bot never joins voice on its own. `/tts join`, run by a member who is in a voice channel, brings it into that channel, and it posts one line in the channel's chat: "Reading this channel's chat aloud (voice by Google Translate). `/tts leave` to stop." From then on, a message typed in that channel's chat is read aloud when its author is in the channel (from the cache's voice states), as "name said: text", in the author's voice (`/tts voice`, else `tts.default_voice`). Messages in any other channel are ignored. Stage channels can't be joined.

**Every message read aloud is sent to Google**: the text, name included, goes to Google Translate's speech endpoint in the request URL.

The name is the member's TTS nickname (`/tts nickname`) if one is set, else their server nickname, else their global display name, else their username. It goes through the same cleanup as the message, capped at 32 characters, and a name with nothing speakable left falls back to the username. Every message carries it, read in the speaker's voice: a Japanese voice pronounces an English name the Japanese way, with no translation.

A message is never read when it comes from a bot, a webhook, or the system, starts with the `-` prefix, carries a sticker, or has nothing left after cleanup. Cleanup reads mentions as names, custom emoji as their names (a run of one emoji once), URLs as "link", masked links as their label, and fenced code as "code block"; it drops markdown symbols and timestamps, cuts any character repeated more than three times to three, and caps the text at `max_chars` at a word boundary. Attachments are never read.

Each server has one FIFO queue of `queue_max` messages, and each member may have `user_queue_max` of them waiting; anything past either cap is dropped silently and the older messages are kept. The bot leaves 5 seconds after the last human leaves its channel, after `idle_secs` without speech, when it is moved or disconnected, and on `/tts leave`. A server can join or leave voice at most once per 5 seconds, which holds each server to 24 gateway voice updates a minute against the 120-per-minute limit on the connection.

### Voice lines

`/voiceline` plays one recording from the operator's own voice set, the lines the `/collection` Voice page lists; outfit and dialect sets (Ling's `nian#12`, `CN_TOPOLECT`) aren't offered. The recording comes from the public backend (`/api/voices/<id>` names it, `/api/assets/audio/...` serves it) as Ogg Vorbis, 44.1 kHz mono, about 9 KB a second (Amiya's idle line: 43 to 59 KB across its four languages). Nothing goes to Google.

Where the line plays:

- **The bot is in your voice channel**: the line joins the same queue as chat being read out, and counts against your `user_queue_max`. `/tts skip` skips it.
- **The bot isn't in voice in this server**: it joins your channel to play the line but does **not** read the channel's chat. It leaves like any session (empty channel, `idle_secs` of quiet, `/tts leave`). A `/tts join` in that channel starts reading the chat without rejoining.
- **Anywhere else** (you aren't in voice, the bot is in another channel, a stage channel, a full queue, TTS switched off, the 5-second join cooldown): the reply carries the recording as an `.ogg` file Discord plays inline, with one line saying why when it isn't simply that you aren't in voice.

The reply is public: the operator, the line's title and text, the language and the voice actor.

### Speech from Google Translate

`GET https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=<voice>&q=<text>`, with a desktop browser's User-Agent, through the bot's shared HTTP client: unofficial, keyless, and liable to change or refuse without notice. At most 2 requests are in flight across all servers, each with a 10 s deadline. A 429, a 5xx or a network error is retried once after 750 ms; anything else, or a body that isn't MP3 (Google answers refusals with HTML), skips that chunk with a warning in the log and nothing in chat.

The endpoint takes at most **200 UTF-16 code units** per request, measured: 200 ASCII characters, 200 kana and 100 emoji (2 units each) pass; 201 of any of them, and 100 emoji plus one letter, answer HTTP 400 with an HTML page. So a message is split into chunks: each sentence is a chunk, a sentence over 200 units is split at spaces between whole words, and a run of unspaced text (Chinese, Japanese, Thai) over 200 units is split after its own punctuation (`。` `、` `，` ...), and only at a character boundary when it has none. Each chunk is fetched while the previous one plays, and `/tts skip` drops the rest of the message.

Measured from macOS (2026-10-08, a desktop connection, not the VPS): a 21-word, 103-character en-US sentence came back in 0.170, 0.171, 0.172, 0.245 and 0.313 s (p50 0.172 s, first byte 0.141 to 0.285 s), as 58,560 B of MP3 for 7.32 s of audio. Every voice returns 64 kbps CBR mono MP3 at 24 kHz, 8,000 B per second of audio. The time to first audio is the fetch of the first chunk, about 0.2 s for a sentence of that length, plus songbird's start of playback, which only Discord can show.

### The voices

25 voices, each a Google Translate language code; `/tts voice set` autocompletes them. Every one was requested live on 2026-10-08, twice, and answered HTTP 200 with `audio/mpeg`:

English (US) `en-US`, English (UK) `en-GB`, English (Australia) `en-AU`, English (India) `en-IN`, Japanese `ja`, Korean `ko`, Chinese (Mandarin, Simplified) `zh-CN`, Chinese (Taiwan) `zh-TW`, Cantonese `yue`, Spanish (Spain) `es-ES`, Spanish (Mexico) `es-MX`, French (France) `fr-FR`, French (Canada) `fr-CA`, Portuguese (Brazil) `pt-BR`, Portuguese (Portugal) `pt-PT`, German `de`, Italian `it`, Russian `ru`, Indonesian `id`, Vietnamese `vi`, Thai `th`, Filipino `fil`, Dutch `nl`, Polish `pl`, Turkish `tr`.

Accents come from the region in the language code, not from the host's TLD, which the folklore credits. `tl=en` on translate.google.co.uk, .com.au, .co.in and .ca returned the same two encodings that `.com` alternates between from one request to the next, while `tl=en-GB` returned its own bytes on both `.com` and `.co.uk`, and `en-AU` and `en-IN` likewise. So every request goes to translate.google.com. `es-US` was left out because it returned the same bytes as `es-MX`.

### Building

Speech needs nothing at build time or on disk. The one native piece is libopus, which songbird encodes voice with: `opus2`'s `bundled` feature builds the vendored source with CMake and a C compiler and links it statically, so the binary needs nothing at runtime. On the Ubuntu 24.04 VPS, once:

```bash
sudo apt-get update && sudo apt-get install -y cmake build-essential pkg-config
```

`vps-update.sh` then builds as before (`cargo build --release` in `discord/`). The CI runner (`ubuntu-latest`) already has CMake. A `tts/` directory left on the VPS by the sherpa-onnx releases is no longer read and can be deleted.

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

Ten migrations in [`migrations/`](migrations/) are embedded at compile time and applied automatically on startup.

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
| `guild_tts_nicknames` | How a member's name is read by TTS in a server, who set it, and when. No row means the display name |
| `user_tts_voice` | The voice each member picked, global across servers. No row, or a voice no longer listed, means the default voice |

`guild_tts_disabled`, the old per-server TTS off switch, is created by one migration and dropped by the next: TTS no longer joins on its own, so there is nothing to opt out of. Both files stay, because sqlx refuses to start when an applied migration is missing.

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
│   ├── tts.rs        /tts join, leave, skip, nickname
│   ├── voiceline.rs  /voiceline
│   └── warn.rs       /warn add, list, remove, clear, policy
├── api/              Backend HTTP clients (status, stats, cached game data)
├── birthday.rs       Game-day dates, birthday parsing, the daily announcer
├── tts/              Text-to-speech: gate.rs (speak or not, queue caps), text.rs (cleanup,
│                     names, sentences, chunks), engine.rs (Google Translate requests),
│                     voices.rs (the 25 voices),
│                     session.rs (join, queue, sentence pipeline, leave)
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
