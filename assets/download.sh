#!/bin/sh
set -e

SERVER=${1:-en}
THREADS=${2:-4}

downloader --server "$SERVER" --savedir /data/ArkAssets --threads "$THREADS" download --all
