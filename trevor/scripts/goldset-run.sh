#!/usr/bin/env bash
# Build eval/goldset_v1.jsonl end to end with local models, no manual review.
#
# Needs: the backend running (story names), `WITH_LLM=1 ./scripts/fetch-model.sh`,
# `brew install llama.cpp`, and `cargo build --release --features embed --bins`.
# One model is loaded at a time, and the server is always stopped on exit,
# including on Ctrl-C or an error. Every stage resumes where it stopped.
set -euo pipefail
cd "$(dirname "$0")/.."

PORT="${PORT:-8081}"
SERVER="http://127.0.0.1:$PORT"
GEN=models/llm/gemma-4-12b-it-qat-q4_0.gguf
JUDGE=models/llm/Qwen3.5-9B-Q4_K_M.gguf
# Overrides for a new dataset version; the defaults reproduce gold set v1.
WORK="${WORK:-artifacts/goldgen}"
OUT="${OUT:-eval/goldset_v1}"
SCALE="${SCALE:-1.0}"
# Extra flags for `goldset generate`, e.g. "--evidence-max 200 --evidence-one-line".
GEN_ARGS="${GEN_ARGS:-}"
# Extra flags for `goldset filter`, e.g. "--evidence-min-prefix 40".
FILTER_ARGS="${FILTER_ARGS:-}"
# Corpus directory the stages read (chunks, BM25, vectors); the default is the P0 corpus.
CORPUS="${CORPUS:-artifacts}"
B=target/release
mkdir -p "$WORK"

SERVER_PID=""
stop_server() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
    SERVER_PID=""
  fi
}
trap stop_server EXIT INT TERM

start_server() {
  echo "== loading $(basename "$1")"
  llama-server -m "$1" --host 127.0.0.1 --port "$PORT" -np 2 --kv-unified-per-slot 4096 \
    -fa on -cram 512 --no-webui --reasoning off -ngl all >>"$WORK/server.log" 2>&1 &
  SERVER_PID=$!
  for _ in $(seq 1 180); do
    if curl -sf "$SERVER/health" >/dev/null; then return 0; fi
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then echo "llama-server exited; see $WORK/server.log" >&2; exit 1; fi
    sleep 1
  done
  echo "llama-server did not become healthy in 180 s" >&2
  exit 1
}

[[ -s "$WORK/sample.jsonl" ]] || "$B/goldset" --work "$WORK" sample --scale "$SCALE"

start_server "$GEN"
"$B/goldset" --work "$WORK" probe --server "$SERVER" --corpus "$CORPUS"
# shellcheck disable=SC2086
"$B/goldset" --work "$WORK" generate --server "$SERVER" --corpus "$CORPUS" $GEN_ARGS
stop_server

# shellcheck disable=SC2086
"$B/goldset" --work "$WORK" filter --corpus "$CORPUS" $FILTER_ARGS

start_server "$GEN"
"$B/goldset" --work "$WORK" closed-book --server "$SERVER"
stop_server

start_server "$JUDGE"
"$B/goldset" --work "$WORK" probe --server "$SERVER" --corpus "$CORPUS"
"$B/goldset" --work "$WORK" calibrate --server "$SERVER" --corpus "$CORPUS"
"$B/goldset" --work "$WORK" judge --server "$SERVER" --corpus "$CORPUS"
"$B/goldset" --work "$WORK" finish --server "$SERVER" --corpus "$CORPUS" --out "$OUT"
stop_server

"$B/eval" validate --goldset "$OUT.jsonl" --corpus "$CORPUS"
echo "== done: $OUT.jsonl, .meta.json, .spotcheck.md"
