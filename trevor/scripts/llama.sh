# Helpers for the shell scripts that run a local llama-server (update.sh, p2-run.sh); source it, do not run it.

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
