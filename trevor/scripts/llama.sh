# Helpers for the shell scripts that run a local llama-server (update.sh, p2-run.sh, goldset-run.sh); source it, do
# not run it.

# stop_pid PID: stop a background job this shell started (SIGTERM) and reap it; nothing when PID is empty or the job
# has already exited.
stop_pid() {
  [[ -n "$1" ]] && { kill "$1" 2>/dev/null || true; wait "$1" 2>/dev/null || true; }
  return 0
}

# await_health PID ON_TIMEOUT: wait up to 180 s for the llama-server on $PORT to answer /health; exits the script at
# once when the server process PID has died. After 180 s without an answer, ON_TIMEOUT "continue" returns and "exit"
# exits 1. Call it as a plain command (not after || or &&), so set -e stays in force inside it.
await_health() {
  for _ in $(seq 1 180); do
    curl -sf "http://127.0.0.1:$PORT/health" >/dev/null && return 0
    kill -0 "$1" || exit 1
    sleep 1
  done
  [[ "$2" == "continue" ]] || exit 1
}

# start_llama MODEL LOG SLOTS KV [ARG...]: start llama-server on $PORT in the background with the flags every script
# uses (SLOTS parallel slots of KV tokens each, flash attention, all layers on the GPU) plus ARGs, its output appended
# to LOG; the PID goes to SERVER_PID.
start_llama() {
  llama-server -m "$1" --host 127.0.0.1 --port "$PORT" -np "$3" --kv-unified-per-slot "$4" \
    -fa on -cram 512 --no-webui -ngl all "${@:5}" >> "$2" 2>&1 &
  SERVER_PID=$!
}

# await_no_llama: wait while another llama-server runs (one model loaded at a time).
await_no_llama() {
  while pgrep -x llama-server >/dev/null; do echo "waiting for another llama-server to exit"; sleep 30; done
}
