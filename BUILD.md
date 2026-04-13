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
2a. Optionally, install Redis. Windows hates local Redis installations, so I recommend not trying it, but for MacOS and Linux, it's recommended.
3. Setup the `.env` variables:
```
DATABASE_URL="postgresql://postgres:password@localhost:5432" # Required
GAME_DATA_DIR="../assets/output/gamedata/excel" # Optional
ASSETS_DIR="../assets/output" # Optional
REDIS_URL=redis://127.0.0.1:6379 # Optional
JWT_SECRET="jwt-secret" # Required
SERVICE_KEY="service-key" # Required
ASSET_WS_URL="ws://localhost:9160" # Optional
RATE_LIMIT_RPM=100 # Optional
DPS_POLL_INTERVAL=1800 # Optional
DPS_AUTO_RESTART=true # Optional
GITHUB_TOKEN="github-token" # Optional
```
- The `GAME_DATA_DIR` variable should be either a relative or absolute path to the `/excel` directory extracted via the asset unpacker. **Default**: `../assets/output/gamedata/excel`
- The `ASSETS_DIR` variable should be either a relative or absolute path to the main directory you extracted assets via the asset unpacker. **Default**: `../assets/output`
- The `JWT_SECRET` variable is used by the backend for signing and verifying user tokens.
- The `SERVICE_KEY` variable is used by the frontend to bypass rate limits and caching.
- The `ASSET_WS_URL` variable is the websocket URL hosted by the asset builder for polling and checking for asset updates. If not provided, will ignore.
- The `RATE_LIMIT_RPM` variable is how many rate limit requests per minute. **Default**: `100`
- The `DPS_POLL_INTERVAL` variable is how long to poll the [DPS Repository](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare) in seconds.
- The `DPS_AUTO_RESTART` variable is whether to auto-restart whenever the DPS repository updates. If set to `true`, the backend will auto-build and regenerate the DPS configs and files before shutting down. It is recommended to use [pm2](https://pm2.keymetrics.io/) or [systemctl](https://www.freedesktop.org/software/systemd/man/latest/systemctl.html) to auto-restart the backend if you have enabled this.
- The `GITHUB_TOKEN` variable is used for authentication to utilize GitHub's API to poll the DPS repository in case of rate limits. By default, the backend attempts to avoid rate limits, but if you have set the poll interval to be very frequent, it is recommended you provide a token.
3a. If there are updates to the DPS repository that aren't pushed, regenerate the DPS functions and formulas:
```bash
$ cargo run --bin generate-dps
$ cargo clippy --fix --allow-dirty --all-targets --all-features -- -D warnings
$ cargo fmt --all
```
4. Run the server:
```bash
$ cargo run --bin backend --release
```

## Frontend
1. Install [NodeJS](https://nodejs.org/en/download). Anything above `v18` will work.
2. Install [Bun](https://bun.sh/).
3. Add the following to your `.env` file:
```
BACKEND_URL="http://localhost:3060"
INTERNAL_SERVICE_KEY="service-key"
```
4. Run the following commands:
```bash
$ cd frontend
$ bun i
$ bun run build
$ bun start
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
4. Clone OpenArknightsFBS into the assets directory
```bash
cd assets
git clone https://github.com/MooncellWiki/OpenArknightsFBS.git
```
5. Build the downloader and unpacker:
```bash
# Build downloader
$ cd downloader
$ cargo build --release

# Build unpacker
$ cd unpacker
$ cargo build --release
```
6. Download assets
```bash
# Default save directory: ./ArkAssets
# Can override with -d /path/to/dir
downloader --server en download --all
```
7. Extract assets
```bash
unpacker extract -i ./ArkAssets -o ./output
```
