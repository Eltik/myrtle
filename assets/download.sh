#!/bin/sh
set -e

SERVER=${1:-en}
THREADS=${2:-4}
PROFILE=${3:-full}
PROFILE_FLAG=""
[ "$PROFILE" != "full" ] && PROFILE_FLAG="--profile $PROFILE"
# shellcheck disable=SC2086 # PROFILE_FLAG intentionally word-splits into two args
downloader --server "$SERVER" --savedir /data/ArkAssets/"$SERVER" --threads "$THREADS" download --all $PROFILE_FLAG
