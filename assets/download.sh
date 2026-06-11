#!/bin/sh
set -e

SERVER=${1:-en}
THREADS=${2:-4}
PROFILE=${3:-full}
PROFILE_FLAG=""
[ "$PROFILE" != "full" ] && PROFILE_FLAG="--profile $PROFILE"
downloader --server "$SERVER" --savedir /data/ArkAssets/"$SERVER" --threads "$THREADS" download --all
