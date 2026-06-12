#!/bin/sh
# Git-based update / restore for the VPS deployment.
# Run ON the VPS from the repo root (/var/www/myrtle.moe).
#
#   bash vps-update.sh                     # git pull + rebuild + pm2 restart
#   bash vps-update.sh --assets            # also download + extract game assets
#                                          # (manifest-diffed: only missing/changed
#                                          # bundles are downloaded)
#   bash vps-update.sh --salvage <olddir>  # bootstrap a fresh clone from a previous
#                                          # install: copies .env files and moves
#                                          # ArkAssets/output over, implies --assets
set -eu

ROOT="$(cd "$(dirname "$0")" && pwd)"
PM2_APPS="${PM2_APPS:-myrtle-backend myrtle-frontend myrtle-ws-en myrtle-ws-cn myrtle-discord}"
BACKEND_PORT="${BACKEND_PORT:-3060}"
JOBS="$(nproc 2>/dev/null || echo 2)"

ASSETS=0
SALVAGE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --assets) ASSETS=1 ;;
        --salvage)
            SALVAGE="${2:?--salvage needs a directory}"
            ASSETS=1
            shift
            ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

cd "$ROOT"

# --- salvage from a previous install ---------------------------------------
if [ -n "$SALVAGE" ]; then
    echo "==> Salvaging from $SALVAGE"
    for f in backend/.env frontend/.env frontend/.env.local discord/.env assets/.env .env; do
        if [ -f "$SALVAGE/$f" ] && [ ! -f "$ROOT/$f" ]; then
            cp -p "$SALVAGE/$f" "$ROOT/$f" && echo "    copied $f"
        fi
    done
    # Big data dirs move instantly on the same filesystem.
    for d in assets/ArkAssets assets/output; do
        if [ -d "$SALVAGE/$d" ] && [ ! -d "$ROOT/$d" ]; then
            mv "$SALVAGE/$d" "$ROOT/$d" && echo "    moved $d"
        fi
    done
fi

# --- code --------------------------------------------------------------------
if [ -d "$ROOT/.git" ]; then
    echo "==> git pull"
    git pull --ff-only
    git submodule update --init assets/OpenArknightsFBS 2>/dev/null || true
else
    echo "warning: $ROOT is not a git checkout - skipping pull" >&2
fi

# --- asset tooling binaries ----------------------------------------------------
echo "==> Building downloader + unpacker"
(cd "$ROOT/assets/downloader" && cargo build --release)
(cd "$ROOT/assets/unpacker" && cargo build --release)
mkdir -p "$ROOT/assets/binaries"
cp "$ROOT/assets/downloader/target/release/downloader" "$ROOT/assets/binaries/"
cp "$ROOT/assets/unpacker/target/release/unpacker" "$ROOT/assets/binaries/"

# --- game assets ----------------------------------------------------------------
if [ "$ASSETS" -eq 1 ]; then
    echo "==> Downloading EN assets (full profile)"
    "$ROOT/assets/binaries/downloader" --server en -d "$ROOT/assets/ArkAssets/en" \
        -t "$JOBS" download --all
    echo "==> Downloading CN assets (operators profile)"
    "$ROOT/assets/binaries/downloader" --server cn -d "$ROOT/assets/ArkAssets/cn" \
        -t "$JOBS" download --all --profile operators
    echo "==> Extracting EN assets"
    "$ROOT/assets/binaries/unpacker" extract \
        -i "$ROOT/assets/ArkAssets/en" -o "$ROOT/assets/output/en" -j "$JOBS"
    echo "==> Extracting CN assets"
    "$ROOT/assets/binaries/unpacker" extract \
        -i "$ROOT/assets/ArkAssets/cn" -o "$ROOT/assets/output/cn" -j "$JOBS"
    ENEMY_DIRS="$(find "$ROOT/assets/output/en/spine/Enemy" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)"
    echo "    enemy spine dirs: $ENEMY_DIRS"
fi

# --- apps -------------------------------------------------------------------------
echo "==> Building backend"
(cd "$ROOT/backend" && cargo build --release --bin backend)

echo "==> Building frontend"
(cd "$ROOT/frontend" && bun install --frozen-lockfile && bun run build)

echo "==> Installing asset watcher deps"
(cd "$ROOT/assets" && bun install --frozen-lockfile)

# --- restart -----------------------------------------------------------------------
echo "==> Restarting pm2 apps"
for app in $PM2_APPS; do
    pm2 restart "$app" 2>/dev/null || echo "    (no pm2 app named $app - skipped)"
done

# --- smoke test ----------------------------------------------------------------------
echo "==> Smoke testing (waiting for game data load)"
code=""
i=0
while [ $i -lt 24 ]; do
    sleep 5
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 \
        "http://localhost:$BACKEND_PORT/api/static/chibis" || true)"
    [ "$code" = "200" ] && break
    i=$((i + 1))
done
echo "chibis: HTTP $code"
curl -s -o /dev/null -w "enemy-chibis: HTTP %{http_code}\n" --max-time 30 \
    "http://localhost:$BACKEND_PORT/api/static/enemy-chibis"
curl -s -o /dev/null -w "enemy spine asset: HTTP %{http_code}\n" --max-time 10 \
    "http://localhost:$BACKEND_PORT/api/assets/spine/Enemy/enemy_1000_gopro/enemy_1000_gopro_2.skel"

echo "==> Done"
