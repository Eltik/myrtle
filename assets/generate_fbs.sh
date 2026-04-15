#!/bin/sh
set -e

echo "=== Generating FlatBuffer schemas ==="

cd /src/unpacker

export CARGO_TARGET_DIR=/tmp/cargo_target
cargo run --bin generate-fbs

echo ""
echo "=== FBS generation complete ==="
echo "Generated files:"
echo "  src/generated_fbs/         (CN schemas)"
echo "  src/generated_fbs_yostar/  (Yostar schemas)"
echo "  src/fb_json_auto.rs"
echo "  src/fb_json_auto_yostar.rs"
echo "  src/flatbuffers_decode.rs"
