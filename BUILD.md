# Building Myrtle
The v3 of Myrtle is sorted into three sections:
1. The backend
2. The frontend
3. The assets builder/extractor
The following will be instructions on how to build and run the website (I am handwriting this for future reference).

## Backend
1. Install [Rust](https://rustup.rs/). If you have issues with Rust on Windows, just Google the error you get or use Claude/ChatGPT. I hate using Rust on Windows, which is why I code on Mac or Linux the majority of the time. If I can find a reliable way to install Rust on Windows, I'll update this message.
2. Install [PostgreSQL](https://www.postgresql.org/). Use `v18` or higher for the version. On MacOS:
```bash
$ brew install postgresql@18
```
On Windows, download the EXE installer or `winget`:
```bash
$ winget install PostgreSQL.PostgreSQL.18
```
2a. Optionally, install Redis. Windows hates local Redis installations, so I recommend not trying it, but for MacOS and Linux, it's recommended. Without it the backend uses an in-memory cache.
3. Load the schema into a scratch database. The backend's SQL is checked against a live database at compile time, so it will not build without one:
```bash
$ cd backend
$ export PGPASSWORD=password
$ psql -h localhost -U postgres -v ON_ERROR_STOP=1 -qc 'CREATE DATABASE myrtle_sqlx'
$ for f in crates/db/migrations/*.sql; do psql -h localhost -U postgres -d myrtle_sqlx -v ON_ERROR_STOP=1 -q -1 -f "$f"; done
```
4. Build against it:
```bash
$ DATABASE_URL=postgres://postgres:password@localhost:5432/myrtle_sqlx cargo build --release
```
5. Copy the config and set the secrets it refers to:
```bash
$ cp config/myrtle.example.toml config/myrtle.toml
$ createdb -h localhost -U postgres myrtle
```
Then create `backend/.env`:
```
DATABASE_URL="postgres://postgres:password@localhost:5432/myrtle" # Required
JWT_SECRET="<openssl rand -hex 32>" # Required
GAME_CREDENTIAL_KEY="<openssl rand -hex 32>" # Required
REDIS_URL="redis://127.0.0.1:6379" # Optional
SERVICE_KEY="service-key" # Optional
GITHUB_TOKEN="github-token" # Optional
```
- `DATABASE_URL` here is the database the server runs on, not the scratch one from step 3. The API migrates it on first boot.
- The `JWT_SECRET` variable is used by the backend for signing and verifying user tokens.
- The `GAME_CREDENTIAL_KEY` variable seals the game credentials stored for synced accounts.
- The `SERVICE_KEY` variable lets trusted tooling bypass rate limits (sent as the `x-service-key` header).
- The `GITHUB_TOKEN` variable raises GitHub's API limit for the job that watches the [DPS Repository](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare).
- Everything else (port, servers, asset directory, rate limit, job intervals, asset watcher WebSocket URLs) is a key in `config/myrtle.toml`, not an environment variable. `PORT`, `ASSETS_DIR`, `GAME_DATA_DIR`, `ASSET_WS_URL`, `RATE_LIMIT_RPM`, `DPS_POLL_INTERVAL` and `DPS_AUTO_RESTART` are no longer read. Every key is documented in `config/myrtle.example.toml`.
- Object storage (optional): to serve `textures`, `spine`, `audio`, `video` and `portraits` from Cloudflare R2 instead of the local disk, fill in the `[storage]` section of `config/myrtle.toml` and set `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY` in `backend/.env`. With `storage.type` empty nothing changes. The bucket is mirrored by `asset-sync` (built by `cargo build --release -p myrtle-storage --bin asset-sync` in `backend/`), which reads its own `STORAGE_*` and `CLOUDFLARE_*` environment variables; see the "Object Storage Sync" section of `assets/README.md`. The frontend can then build asset URLs straight against the bucket by setting `VITE_ASSET_BASE_URL` (see `frontend/.env.example`); unset, it keeps using the backend's URLs.
6. Make sure the game data is extracted (see the asset section below). The backend will not start without `../assets/output/en/gamedata/excel/`.
7. Run the API, then the worker. Run exactly one worker:
```bash
$ ./target/release/myrtle-api config/myrtle.toml
$ ./target/release/myrtle-worker config/myrtle.toml
```
7a. If there are updates to the DPS repository that aren't pushed, regenerate the DPS and HPS formulas. This needs the `backend/external/ArknightsDpsCompare` submodule and Python 3.10+:
```bash
$ cargo run -p myrtle-dps-transpile
$ cargo run -p myrtle-dps-transpile -- --healing
$ cargo fmt --all
```

See [backend/README.md](backend/README.md) for the full reference.

## Frontend
1. Install [NodeJS](https://nodejs.org/en/download). Anything above `v18` will work.
2. Install [Bun](https://bun.sh/).
3. Add the following to your `.env` file:
```
BACKEND_URL="http://localhost:3060"
VITE_BACKEND_URL="http://localhost:3060"
VITE_SITE_URL="http://localhost:3000"
```
4. Run the following commands:
```bash
$ cd frontend
$ bun i
$ bun --bun run build
$ bun --bun run start
```

## Asset Builder/Extractor
1. Install Git
```bash
# macOS:
$ xcode-select --install
# or
$ brew install git

# Linux:
$ sudo apt install git

# Windows
# Download from https://git-scm.com
```
2. Install Rust
```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Install a C compiler
```bash
# macOS
$ xcode-select --install

# Linux
$ sudo apt install build-essential

# Windows
# Install Visual Studio Build Tools (C++ workload). See https://learn.microsoft.com/en-us/cpp/build/vscpp-step-0-installation?view=msvc-170
```
4. Build the downloader and unpacker, from `assets/`:
```bash
# Build downloader
$ cd downloader
$ cargo build --release

# Build unpacker
$ cd unpacker
$ cargo build --release
```
5. Download assets
```bash
# Default save directory: ./ArkAssets
# Can override with -d /path/to/dir
downloader --server en download --all
```
6. Extract assets
```bash
unpacker extract -i ./ArkAssets -o ./output
```
