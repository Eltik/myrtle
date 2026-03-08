#!/bin/sh
set -e

THREADS=${1:-4}

echo "Step 1/4: Extracting Unity Studio Assets with $THREADS threads..."
assets-unpacker extract --input /data/ArkAssets --output /data/Unpacked -j "$THREADS"

echo "Step 2/4: Moving extracted assets..."
cp -rl /data/Unpacked/upk/ArkAssets/. /data/Unpacked/upk/

echo "Step 3/4: Combining RGB + alpha textures into RGBA..."
assets-unpacker combine --input /data/Unpacked/upk --output /data/Unpacked/cmb

echo "Step 4/4: Decoding Definitions..."
assets-unpacker decode --input /data/Unpacked/upk --output /data/Unpacked/decoded --manifest /data/ArkAssets/*.idx

echo "Unpacking complete! 🎉"
