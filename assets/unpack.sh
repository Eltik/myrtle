#!/bin/sh
set -e

THREADS=${1:-4}

echo "Extracting all assets with $THREADS threads..."
unpacker extract \
    --input /data/ArkAssets \
    --output /data/output \
    -j "$THREADS"

echo "Unpacking complete!"
