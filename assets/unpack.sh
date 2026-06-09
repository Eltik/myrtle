#!/bin/sh
set -e

REGION=${1:-en}
THREADS=${1:-4}

echo "Extracting $REGION assets with $THREADS threads..."
unpacker extract \
    --input /data/ArkAssets/"$REGION" \
    --output /data/output/"$REGION" \
    -j "$THREADS"

echo "Unpacking complete!"
