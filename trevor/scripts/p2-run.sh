#!/usr/bin/env bash
# P2 summaries end to end on the generator model: story stage, then group stage.
# One model loaded; the server and the thermal logger always stop on exit.
# Every stage resumes where it stopped. Output: artifacts/p2/{stories,groups}.jsonl.
set -euo pipefail
cd "$(dirname "$0")/.."
PORT="${PORT:-8081}"
GEN=models/llm/gemma-4-12b-it-qat-q4_0.gguf
mkdir -p artifacts/p2
SERVER_PID=""; THERM_PID=""
cleanup() {
  [[ -n "$SERVER_PID" ]] && { kill "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true; }
  [[ -n "$THERM_PID" ]] && { kill "$THERM_PID" 2>/dev/null || true; wait "$THERM_PID" 2>/dev/null || true; }
  SERVER_PID=""; THERM_PID=""
}
trap cleanup EXIT INT TERM
( while true; do
    echo "$(date +%H:%M:%S) thermal: $(pmset -g therm | grep -i 'level' | grep -v 'No ' | tr '\n' ' ' || true)pressure $(sysctl -n kern.memorystatus_vm_pressure_level) $(pmset -g batt | head -1 | grep -o "'.*'")"
    sleep 600
  done ) &
THERM_PID=$!
llama-server -m "$GEN" --host 127.0.0.1 --port "$PORT" -np 2 --kv-unified-per-slot 16384 \
  -fa on -cram 512 --no-webui --reasoning off -ngl all >> artifacts/p2/server.log 2>&1 &
SERVER_PID=$!
for _ in $(seq 1 180); do curl -sf "http://127.0.0.1:$PORT/health" >/dev/null && break; kill -0 "$SERVER_PID" || exit 1; sleep 1; done
SERVER="http://127.0.0.1:$PORT" python3 scripts/p2-summaries.py story
SERVER="http://127.0.0.1:$PORT" python3 scripts/p2-summaries.py group
echo "== p2 done"
