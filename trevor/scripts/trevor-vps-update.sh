#!/usr/bin/env bash
# Trevor's content update on the VPS (design/trevor-vps-runbook.md). Ubuntu 24.04, bash, python3 (stdlib only), the
# release binaries in target/release, util-linux (flock, ionice), curl and rsync; nothing from the Mac.
#
# Decisions (2026-10-16): summaries are the ONLY model stage; everything else is deterministic; every other
# model output (dossiers, topics, primers, chronology extraction, identities, deaths, the bank's generated answers,
# canon, design) is frozen at what the Mac built until a manual rebuild.
#
#   trevor-vps-update.sh daily       19:30 UTC (trevor-update.timer): exit at once unless the asset watcher's EN res_version
#                             differs from the active version's builtFrom; else refresh the corpus from the local
#                             backend, rebuild every deterministic artifact, embed only new or changed chunks, build
#                             the panels, run the gates, publish, switch the worker to the new artifacts, notify.
#   trevor-vps-update.sh summaries   23:00 UTC (trevor-summaries.timer): story and group summaries for stories that have
#                             none, then the bank's pre-generated answers for chunks the update added or changed
#                             (scripts/bank_regen.py; contract v1.1 section 3a), each on Cloudflare inside 23:00-23:55
#                             UTC, then on the local E2B until 07:00 UTC; then the same rebuild, gates and publish when
#                             any summary or answer was written.
#   trevor-vps-update.sh rollback [VERSION]   re-activate VERSION (default: the version active before the current one) and
#                             point the worker back at that version's artifacts.
#   trevor-vps-update.sh status      the active version, the live release, the backend's /api/trevor/status.
#   trevor-vps-update.sh init        one time: move <trevor>/artifacts (copied from the Mac) into a release and symlink it.
#   trevor-vps-update.sh prune       scripts/retention.py by hand under the run lock (a dry run unless PRUNE_APPLY=1),
#                             then the disk check.
#
# Failure at any step keeps the last good version: the work happens in a staging copy ($TREVOR_HOME/stage), the
# backend's active version flips only after every gate passed and every chunk uploaded, and the worker reads
# <trevor>/artifacts, a symlink that moves to the new release only at the end (flipped back only when the backend
# refused the activate). A failed run leaves the stage as it is, so the next run continues from it (summaries are not
# lost; a stage copy that did not finish is redone); RESET_STAGE=1 copies the live release over it first. One run at a time (flock); nice 19, ionice idle.
#
# Environment (defaults in brackets; the systemd units set these from /etc/trevor/trevor.env):
#   TREVOR_DIR [this script's ../]   code: scripts, prompts, data, eval, deploy, target/release, models
#   TREVOR_HOME [/var/lib/trevor]    releases/, stage/, state/, logs/
#   ASSETS_DIR [$TREVOR_DIR/../assets]   the asset watcher's tree; RES_VERSION_FILE [$ASSETS_DIR/ArkAssets/en/.version]
#   TREVOR_BACKEND [http://127.0.0.1:3060]  TREVOR_WORKER_KEY (publish, warm-up)
#   DISCORD_ALERT_WEBHOOK (optional; the backend's core::alerts variable; TREVOR_DISCORD_WEBHOOK, the name before
#              2026-10-17, is read when it is unset): one update_failed alert per failed run (contract v1.1 section
#              2.2: the step, the exit code, the last ALERT_LOG_LINES [20] log lines) and a "published" notice
#   REGEN_ANSWERS [1]  0 = the bank's answers are never regenerated: no change detection, no answer model run, and
#              answer_bank.py merge as before 2026-10-17 (byte-identical bank.jsonl, verified)
#   REGEN_MAX_UNITS [150]  pending bank units (a changed chunk: 1 question + 1 mid answer; a re-answer: 1 answer)
#              one run regenerates at most; the rest stay pending for the next night
#   TREVOR_CF_BANK_MAX_NEURONS [9000]  the bank's Cloudflare cap, minus what the summaries spent the same night
#   TREVOR_WARM_TOP [100]  questions the warm-up re-asks after a publish (contract v1.1 section 3c); 0 = no warm-up
#   EMBED [1]  0 = no embedding at all (only for a worker serving BM25-only); EMBED_ONNX [model.onnx] the export the
#              vectors must have been made with (a mismatch refuses: x86 must not mix with the Mac's INT8);
#              EMBED_FULL [0] 1 = re-embed every chunk (the one-time fp32 re-embed, about 1 to 2 h);
#              EMBED_DIRS ["artifacts/p3b artifacts/p4x"] the corpora that get vectors (ask's main corpus and P4x);
#              the others keep BM25 only, and their old vectors are never reused
#   PANELS_ARGS  extra panels.py flags (e.g. --no-gate-sources, a kill switch)
#   CANARY [1]  0 = gates without the 10-question model canary;  TREVOR_LLM_URL [http://127.0.0.1:8081] the
#              serving llama-server the canary asks through;  TREVOR_LLM_LOCK [/run/trevor/llm.lock]
#   GATES_ASK_FLAGS ["--profile mid --retrieval $TREVOR_RETRIEVAL"] the canary's ask flags (mid is what the cpu
#              rung serves, contract v1.1 section 4, and fits the serving server's 4,096-token slot);  GATES_ARGS  extra gates.py flags (e.g. --no-recall
#              once, to accept a lower recall as the new baseline)
#   LOCAL_SUMMARIES [1]  E2B_GGUF [$TREVOR_DIR/models/llm/gemma-4-e2b-it-q4_k_m.gguf]  LLAMA_SERVER [llama-server]
#   SUMMARY_PORT [8082]  SUMMARY_CTX [16384]  SUMMARY_THREADS [2]  SUMMARY_UNTIL [07:00]  SUMMARY_MIN_AVAIL_MB [2500]
#   PLAN=1      print what would run (detection and the summary plan are real, nothing is written)
#   FORCE=1     run `daily` even when the res_version is unchanged;  SKIP_FETCH=1  no build-corpus (test)
#   NO_PUBLISH=1  publish.py --dry-run into the log dir; no upload, no switch;  KEEP_RELEASES [3] releases in all:
#              the live one, the previous live one (history.jsonl), then the newest others
#   WORKER_SETTLE [30]  seconds between the worker restart and the activate call;  SETTLE_MIN [15] minutes a new
#              .version must be old before `daily` uses it (the backend hot-reloads after the watcher's extract)
#   BACKEND_RELOAD_WAIT_MIN [20]  minutes `daily` waits, before any fetch, for the backend to reload the new EN data:
#              since backend 89f07593 a reload is deferred while the Trevor model lock names a live pid or MemAvailable
#              is under the EN floor, and retried every RELOAD_RETRY_SECS, and the 19:30 UTC run falls inside the 12B
#              window. The backend reports no loaded version, so the wait reads the guard's own inputs (the lock,
#              /proc/meminfo) and passes once they have admitted an EN reload for RELOAD_RETRY_SECS [300] plus
#              RELOAD_BUILD_SECS [120] without a break (RELOAD_RETRY_SECS and RELOAD_EN_MIN_FREE_MB [900] must match
#              backend/.env); polled every BACKEND_RELOAD_POLL_SECS [30], one log line a minute. Still refused at the
#              end: no build, exit 0, the next run retries; the second consecutive night posts one embed
#              ($STATE/reload-wait.json). 0 = no wait (the script before 2026-10-19, for a box without the window)
#   PRUNE [1]  0 = no prune step after a daily publish (the script before 2026-10-19). The step runs scripts/retention.py
#              (logs, eval runs, the spool, releases, asset manifests; policy in its docstring and RETAIN_* variables)
#              as a dry run that lists what it would delete; PRUNE_APPLY=1 deletes, PRUNE_APPLY_ASSETS=1 also the old
#              asset manifests
#   DISK_CHECK [1]  0 = no disk check; else every `daily` run (and `prune`) warns, and posts one disk_low embed, when a
#              filesystem of TREVOR_HOME, TREVOR_DIR or ASSETS_DIR has less free space than the smaller of
#              DISK_MIN_FREE_GB [10] GB and DISK_MIN_FREE_PCT [10] percent
#   WATCHER_MAX_AGE_DAYS [21]  every `daily` run checks the age of RES_VERSION_FILE; past this many days it posts one
#              embed per crossing ("the asset watcher has not updated in N days", keyed on the file's mtime in
#              $STATE/watcher-alerted.json) and goes on; 0 = no check
#   RUN_TAIL_DIR  a tmpfs directory for the in-memory copy of the last ALERT_LOG_LINES log lines the failure alert
#              quotes (default: the first writable of $XDG_RUNTIME_DIR, /run/trevor, /dev/shm; none: run.log is read)
#
# Once the backend has activated a version it is the source of truth: a failure after that (publish.py exit 5, the
# local record; or any later step) never flips the worker back, so the backend and the worker always serve the same
# version. Only an activate the backend refused flips back.
set -Eeuo pipefail

CMD="${1:-daily}"
ARG2="${2:-}"
TREVOR_DIR="${TREVOR_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TREVOR_HOME="${TREVOR_HOME:-/var/lib/trevor}"
ASSETS_DIR="${ASSETS_DIR:-$TREVOR_DIR/../assets}"
RES_VERSION_FILE="${RES_VERSION_FILE:-$ASSETS_DIR/ArkAssets/en/.version}"
export TREVOR_BACKEND="${TREVOR_BACKEND:-http://127.0.0.1:3060}"
EMBED="${EMBED:-1}"; EMBED_ONNX="${EMBED_ONNX:-model.onnx}"; EMBED_FULL="${EMBED_FULL:-0}"
EMBED_DIRS="${EMBED_DIRS:-artifacts/p3b artifacts/p4x}"; PANELS_ARGS="${PANELS_ARGS:-}"
CANARY="${CANARY:-1}"; GATES_ARGS="${GATES_ARGS:-}"
export GATES_ASK_FLAGS="${GATES_ASK_FLAGS:---profile mid --retrieval ${TREVOR_RETRIEVAL:-bm25}}"
export TREVOR_LLM_URL="${TREVOR_LLM_URL:-http://127.0.0.1:8081}"
export TREVOR_LLM_LOCK="${TREVOR_LLM_LOCK:-/run/trevor/llm.lock}"
LOCAL_SUMMARIES="${LOCAL_SUMMARIES:-1}"
E2B_GGUF="${E2B_GGUF:-$TREVOR_DIR/models/llm/gemma-4-e2b-it-q4_k_m.gguf}"
LLAMA_SERVER="${LLAMA_SERVER:-llama-server}"
SUMMARY_PORT="${SUMMARY_PORT:-8082}"; SUMMARY_CTX="${SUMMARY_CTX:-16384}"; SUMMARY_THREADS="${SUMMARY_THREADS:-2}"
SUMMARY_UNTIL="${SUMMARY_UNTIL:-07:00}"; SUMMARY_MIN_AVAIL_MB="${SUMMARY_MIN_AVAIL_MB:-2500}"
KEEP_RELEASES="${KEEP_RELEASES:-3}"; WORKER_SETTLE="${WORKER_SETTLE:-30}"; SETTLE_MIN="${SETTLE_MIN:-15}"
BACKEND_RELOAD_WAIT_MIN="${BACKEND_RELOAD_WAIT_MIN:-20}"; BACKEND_RELOAD_POLL_SECS="${BACKEND_RELOAD_POLL_SECS:-30}"
RELOAD_RETRY_SECS="${RELOAD_RETRY_SECS:-300}"; RELOAD_BUILD_SECS="${RELOAD_BUILD_SECS:-120}"
RELOAD_EN_MIN_FREE_MB="${RELOAD_EN_MIN_FREE_MB:-900}"
PLAN="${PLAN:-}"; FORCE="${FORCE:-0}"; SKIP_FETCH="${SKIP_FETCH:-}"; NO_PUBLISH="${NO_PUBLISH:-}"
ALERT_WEBHOOK="${DISCORD_ALERT_WEBHOOK:-${TREVOR_DISCORD_WEBHOOK:-}}"; ALERT_LOG_LINES="${ALERT_LOG_LINES:-20}"
export REGEN_ANSWERS="${REGEN_ANSWERS:-1}"; TREVOR_WARM_TOP="${TREVOR_WARM_TOP:-100}"

STATE="$TREVOR_HOME/state"
WORK=""   # the tree the build runs in: the stage, or (PLAN=1) the live tree, where only reads happen
STAGE="$TREVOR_HOME/stage"
RELEASES="$TREVOR_HOME/releases"
export TREVOR_STATE_DIR="$STATE"
LIVE="$TREVOR_DIR/artifacts"         # -> $CURRENT, made once by `init`
CURRENT="$TREVOR_HOME/current"        # -> releases/<stamp>-<version>/artifacts, flipped by this script
BIN="$TREVOR_DIR/target/release"
MODEL_DIR="models/gte-modernbert-base"
TOK="$MODEL_DIR/tokenizer.json"

# Lowest CPU and IO priority for everything below (systemd sets the same; this covers a run by hand).
if [[ -z "${TREVOR_NICED:-}" && -z "$PLAN" ]]; then
  export TREVOR_NICED=1
  if command -v ionice >/dev/null 2>&1; then exec nice -n 19 ionice -c3 "$0" "$@"; else exec nice -n 19 "$0" "$@"; fi
fi

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG=""
RUN_TAIL=""    # tmpfs copy of the last ALERT_LOG_LINES lines (open_log), read by alert_failed before run.log
ACTIVATED=""   # the version the backend activated in this run: no flip back after this
STEP="start"
LLM_PID=""
MODEL_LOCK_HELD=""

log() { echo "== $(date -u +%H:%M:%S) $*"; }
die() {
  echo "ERROR: $*" >&2
  [[ "$CMD" == "daily" || "$CMD" == "summaries" || "$CMD" == "rollback" ]] && alert_failed 1 "$*"
  exit 1
}
step() { STEP="$*"; log "$*"; }
# run CMD...: run it, or under PLAN=1 print it.
run() { if [[ -n "$PLAN" ]]; then echo "PLAN: $*"; else "$@"; fi; }
py() { python3 - "$@"; }
abspath() { python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$1"; }
mtime() { python3 -c 'import os,sys; print(int(os.path.getmtime(sys.argv[1])))' "$1"; }
json_get() {  # json_get FILE KEY[.KEY...]: a value of a JSON file, empty when missing
  python3 -c 'import json,sys
try:
    v = json.load(open(sys.argv[1]))
    for k in sys.argv[2].split("."):
        v = v[k]
    print("" if v is None else v)
except Exception:
    print("")' "$1" "$2"
}

# post_embed TITLE DESCRIPTION COLOR FOOTER: one Discord embed, core::alerts' shape ({"embeds": [{title, description,
# color, footer}]}); never fails the run (a failed webhook is logged and swallowed, as core::alerts does).
post_embed() {
  [[ -n "$ALERT_WEBHOOK" && -z "$PLAN" ]] || return 0
  python3 -c 'import json,sys
t, d, c, f = sys.argv[1:5]
print(json.dumps({"embeds": [{"title": t[:250], "description": d[:3900], "color": int(c), "footer": {"text": f[:200]}}]}))' \
    "$1" "$2" "$3" "$4" \
    | curl -sS --connect-timeout 10 -m 20 -H 'content-type: application/json' -d @- "$ALERT_WEBHOOK" >/dev/null 2>&1 \
    || echo "== discord alert failed to send (swallowed)" >&2
  return 0
}

notify() {  # informational only; never fails the run
  local msg="$1"
  log "notify: $msg"
  post_embed "Trevor update ($CMD)" "$msg" 3447003 "trevor-vps-update.sh $CMD"
}

# alert_failed RC [MESSAGE [OUTCOME]]: contract v1.1 section 2.2 `update_failed`, at most once per run (a marker file
# keyed on the run's pid, so a subshell's ERR trap and the parent's do not both post). OUTCOME replaces the default
# "last good version stays active" line (the warm-up fails after the version went live).
ALERT_MARK="${TMPDIR:-/tmp}/trevor-update-alerted.$$"
alert_failed() {
  local rc="$1" msg="${2:-}" tail_lines="" src
  local outcome="${3:-The last good version stays active; the stage keeps the work for the next run.}"
  if [[ -z "${3:-}" && -n "$ACTIVATED" ]]; then
    outcome="Version $ACTIVATED is live: the backend activated it and the worker serves it (no flip back); only a later step failed."
  fi
  [[ -e "$ALERT_MARK" ]] && return 0
  : > "$ALERT_MARK" 2>/dev/null || true
  log "update_failed: step '$STEP', exit $rc${msg:+: $msg}"
  # The tail comes from the tmpfs copy first: run.log is the file that fails first on a full disk.
  if [[ -n "$RUN_TAIL" || -n "$LOG" ]]; then
    sleep 0.5
    if [[ -n "$RUN_TAIL" && -s "$RUN_TAIL" ]]; then src="$RUN_TAIL"; else src="$LOG/run.log"; fi
    [[ -f "$src" ]] && tail_lines="$(tail -n "$ALERT_LOG_LINES" "$src" | sed 's/```/` ` `/g')"
  fi
  post_embed "Trevor update failed: $STEP" \
    "\`trevor-vps-update.sh $CMD\` failed at step **$STEP** with exit **$rc**.${msg:+ $msg}
$outcome
\`\`\`
${tail_lines:-(no log)}
\`\`\`" 13646394 "update_failed · $(hostname 2>/dev/null || echo vps) · log ${LOG:-none}"
}

on_error() {
  local rc=$?
  stop_local_llm
  release_model_lock
  alert_failed "$rc"
  exit "$rc"
}
trap on_error ERR
trap 'stop_local_llm; release_model_lock; rm -f "$ALERT_MARK"; [[ -z "$RUN_TAIL" ]] || rm -f "$RUN_TAIL"' EXIT
trap 'exit 143' TERM INT

# ---------------------------------------------------------------- the model lock (ask's convention: "<pid> <label>")
take_model_lock() {  # take_model_lock LABEL WAIT_S
  local label="$1" deadline=$((SECONDS + ${2:-3600})) pid
  mkdir -p "$(dirname "$TREVOR_LLM_LOCK")"
  while :; do
    if ( set -o noclobber; echo "$$ $label" > "$TREVOR_LLM_LOCK" ) 2>/dev/null; then MODEL_LOCK_HELD="$TREVOR_LLM_LOCK"; return 0; fi
    pid="$(awk '{print $1; exit}' "$TREVOR_LLM_LOCK" 2>/dev/null || true)"
    if [[ -z "$pid" ]] || ! kill -0 "$pid" 2>/dev/null; then rm -f "$TREVOR_LLM_LOCK"; continue; fi
    (( SECONDS >= deadline )) && return 1
    sleep 10
  done
}
release_model_lock() {
  if [[ -n "$MODEL_LOCK_HELD" && "$(awk '{print $1; exit}' "$MODEL_LOCK_HELD" 2>/dev/null || true)" == "$$" ]]; then
    rm -f "$MODEL_LOCK_HELD"
  fi
  MODEL_LOCK_HELD=""
}

# ---------------------------------------------------------------- setup
# The last ALERT_LOG_LINES lines, kept by a small reader of the log stream and rewritten into a tmpfs file on every
# line (a full disk fails run.log first, so the alert would quote nothing); the reader removes the file at EOF.
RING_PY='import collections, os, sys
path, n = sys.argv[1], int(sys.argv[2])
ring, tmp = collections.deque(maxlen=n), sys.argv[1] + ".tmp"
for line in sys.stdin.buffer:
    ring.append(line)
    try:
        with open(tmp, "wb") as f:
            f.writelines(ring)
        os.replace(tmp, path)
    except OSError:
        pass
for p in (path, tmp):
    try:
        os.remove(p)
    except OSError:
        pass'

run_tail_path() {  # the tmpfs file for the in-memory tail, or nothing
  local d
  for d in "${RUN_TAIL_DIR:-}" "${XDG_RUNTIME_DIR:-}" /run/trevor /dev/shm; do
    if [[ -n "$d" && -d "$d" && -w "$d" ]]; then echo "$d/trevor-update-tail.$$"; return 0; fi
  done
  return 0
}

open_log() {
  mkdir -p "$TREVOR_HOME/logs" "$STATE"
  LOG="$TREVOR_HOME/logs/$STAMP-$CMD"
  mkdir -p "$LOG"
  RUN_TAIL="$(run_tail_path)"
  if [[ -n "$RUN_TAIL" ]]; then
    exec > >(tee -a "$LOG/run.log" >(python3 -u -c "$RING_PY" "$RUN_TAIL" "$ALERT_LOG_LINES")) 2>&1
  else
    exec > >(tee -a "$LOG/run.log") 2>&1
  fi
  find "$TREVOR_HOME/logs" -mindepth 1 -maxdepth 1 -type d -mtime +30 -exec rm -rf {} + 2>/dev/null || true
}

# take_run_lock [BUSY_RC]: the flock on update.lock, or exit BUSY_RC (0: nothing to do; `rebuild` passes 4 so the stage
# runner keeps its publish owed instead of counting it done). Opened for append, so a second run that fails the flock
# does not truncate the holder's line; the holder then writes "<pid> <command>", which scripts/stages.py reads to see
# a running update without taking the flock itself (a probe's brief flock made a nightly run skip).
take_run_lock() {
  mkdir -p "$STATE"
  exec 9>>"$STATE/update.lock"
  if command -v flock >/dev/null 2>&1; then
    flock -n 9 || { log "another update holds $STATE/update.lock; nothing to do"; exit "${1:-0}"; }
  else
    log "WARNING: no flock(1) here; runs are not serialized (util-linux provides it on Ubuntu)"
  fi
  echo "$$ $CMD" > "$STATE/update.lock"
}

require_live_release() {
  [[ -L "$LIVE" && -L "$CURRENT" ]] || die "$LIVE -> $CURRENT -> a release is not set up; run \`$0 init\` once (design/trevor-vps-runbook.md)"
}

# The stage mirrors the trevor directory: code copied (python resolves its imports next to the real file, so the
# scripts must not be symlinks), binaries, models and eval linked, artifacts a real copy of the live release.
prepare_stage() {
  local live_target base
  live_target="$(abspath "$LIVE")"
  base="$(cat "$STATE/stage-base" 2>/dev/null || true)"
  mkdir -p "$STAGE/trevor"
  ln -sfn "$(abspath "$ASSETS_DIR")" "$STAGE/assets"
  for d in scripts prompts data deploy; do
    rsync -a --delete "$TREVOR_DIR/$d/" "$STAGE/trevor/$d/"
  done
  for d in target models eval; do ln -sfn "$(abspath "$TREVOR_DIR/$d")" "$STAGE/trevor/$d"; done
  # A stage is continued only when $STAGE/.complete says its copy finished: the marker goes before a copy starts and
  # is written last, so a copy cut short (ENOSPC midway) is never reused even when stage-base names the live release.
  local why=""
  if [[ ! -d "$STAGE/trevor/artifacts" ]]; then why="no stage yet"
  elif [[ "${RESET_STAGE:-0}" == "1" ]]; then why="RESET_STAGE=1"
  elif [[ "$base" != "$live_target" ]]; then why="the live release moved"
  elif [[ ! -f "$STAGE/.complete" ]]; then why="no completion marker: the last copy did not finish"
  fi
  if [[ -n "$why" ]]; then
    log "stage: copying the live release ($live_target) into the stage ($why)"
    rm -f "$STAGE/.complete"
    rsync -a --delete "$live_target/" "$STAGE/trevor/artifacts/"
    echo "$live_target" > "$STATE/stage-base"
    date -u +%FT%TZ > "$STAGE/.complete"
  else
    log "stage: continuing from the stage left by the last run (base $base)"
  fi
  export TREVOR_GAMEDATA="$STAGE/assets/output/en/gamedata"
  WORK="$STAGE/trevor"
}

# ---------------------------------------------------------------- the deterministic build (inside the stage)
model_file_of() { json_get "$1/vectors.meta.json" modelFile; }

in_list() { local x="$1" y; shift; for y in "$@"; do [[ "$x" == "$y" ]] && return 0; done; return 1; }

embed_dir() {  # embed_dir DIR [REUSE_FROM...]: only a DIR in EMBED_DIRS, reusing only from EMBED_DIRS
  local dir="$1"; shift
  if [[ "$EMBED" == "0" ]]; then log "EMBED=0: $dir gets no vectors (the worker must serve BM25-only)"; return 0; fi
  # shellcheck disable=SC2086
  if ! in_list "$dir" $EMBED_DIRS; then log "$dir is not in EMBED_DIRS: BM25 only, its old vectors stay unused"; return 0; fi
  local d mf reuse=()
  # shellcheck disable=SC2086
  for d in "$@"; do if in_list "$d" $EMBED_DIRS; then reuse+=("$d"); fi; done
  set -- ${reuse[@]+"${reuse[@]}"}
  for d in "$dir" "$@"; do
    mf="$(model_file_of "$d")"
    if [[ -n "$mf" && "$mf" != "$EMBED_ONNX" && "$EMBED_FULL" != "1" ]]; then
      die "$d holds vectors from $mf, not $EMBED_ONNX: reusing them would mix two runtimes (x86 INT8 drifts to cosine 0.94). Run the one-time full re-embed: EMBED_FULL=1 $0 ${CMD} (design/trevor-vps-runbook.md section 3)"
    fi
  done
  # embed-corpus since 2026-10-16: --model-file is an alias of --onnx-file, and the thermal/memory/load guard is on
  # by default (--no-guard turns it off). An older binary has neither; this run then goes unguarded.
  local args=(--model-dir "$MODEL_DIR" --corpus "$dir")
  if "$BIN/embed-corpus" --help 2>/dev/null | grep -q -- '--model-file'; then args+=(--model-file "$EMBED_ONNX"); else args+=(--onnx-file "$EMBED_ONNX"); fi
  "$BIN/embed-corpus" --help 2>/dev/null | grep -q -- '--no-guard' || log "WARNING: this embed-corpus has no guard (build from 2026-10-16 or later)"
  for d in "$@"; do args+=(--reuse-from "$d"); done
  [[ "$EMBED_FULL" == "1" ]] && args+=(--no-reuse)
  run "$BIN/embed-corpus" "${args[@]}"
}

build_lores() {  # update.sh's build_lores, verbatim in what it builds
  run python3 scripts/dossiers.py sets
  run "$BIN/build-profiles" --dossiers artifacts/dossiers/dossiers.v1.jsonl
  embed_dir artifacts/p3b artifacts/p3a; run "$BIN/build-index" --corpus artifacts/p3b
  run sh -c 'cat artifacts/sources/units.jsonl artifacts/topics/units.v1.jsonl > artifacts/sources/all_units.jsonl'
  run "$BIN/build-units" --units artifacts/sources/all_units.jsonl
  embed_dir artifacts/p4 artifacts/p3b; run "$BIN/build-index" --corpus artifacts/p4
  run python3 scripts/gametext.py
  run "$BIN/build-units" --corpus artifacts/p3b --units artifacts/gametext/p4x_units.jsonl --out artifacts/p4x
  embed_dir artifacts/p4x artifacts/p4; run "$BIN/build-index" --corpus artifacts/p4x
}

deterministic_build() {  # in the stage; FETCH=1 refreshes the corpus from the backend first
  local fetch="$1"
  cd "$WORK"
  if [[ "$fetch" == "1" && -z "$SKIP_FETCH" ]]; then
    step "build-corpus --refresh against $TREVOR_BACKEND"
    local code; code="$(curl -s --connect-timeout 10 -m 30 -o /dev/null -w '%{http_code}' "$TREVOR_BACKEND/" || true)"
    [[ "$code" != "000" ]] || die "the backend does not answer at $TREVOR_BACKEND"
    run "$BIN/build-corpus" --tokenizer "$TOK" --refresh --base "$TREVOR_BACKEND"
    [[ -n "$PLAN" ]] || python3 -c "import json; c=json.load(open('artifacts/changes.json')); print({k: len(v) if isinstance(v, list) else v for k, v in c.items()})" | tee "$LOG/changes.txt"
  fi
  # update.sh steps 2 and 4 without a model. build-prefixes is not in update.sh, but p2-summaries.py reads each
  # story's header and synopsis from artifacts/p1a, so a new story needs it before it can be summarized.
  step "P0 index, P1a prefixes, P3a archives"
  run "$BIN/build-prefixes"
  embed_dir artifacts; run "$BIN/build-index" --corpus artifacts
  run "$BIN/build-archives"
  embed_dir artifacts/p3a artifacts; run "$BIN/build-index" --corpus artifacts/p3a
  step "chronology, dates, identities, deaths, attributes (frozen model outputs, rebuilt joins)"
  run python3 scripts/chrono.py derive; run python3 scripts/chrono.py flashbacks
  run python3 scripts/chrono.py build; run python3 scripts/chrono.py history
  run python3 scripts/reading_guide.py dates
  run python3 scripts/entities.py build; run python3 scripts/deaths.py build
  run python3 scripts/attributes.py
  step "typed sources and lores (p3b, p4, p4x)"
  run python3 scripts/sources.py
  build_lores
  step "reading guide, source dates, appearances, answer bank"
  run python3 scripts/reading_guide.py build
  run python3 scripts/source_dates.py
  run python3 scripts/appearances.py
  # Contract v1.1 section 3a: which chunks the update added or changed against the live release (the version readers
  # have now); their bank units join the pending list the overnight run answers, and the merge drops every generated
  # answer whose evidence changed or vanished. REGEN_ANSWERS=0: neither (the merge before 2026-10-17).
  if [[ "$REGEN_ANSWERS" != "0" && -e "$LIVE" ]]; then
    run python3 scripts/bank_regen.py detect --prev "$(abspath "$LIVE")"
  fi
  run env PRIMERS=1 python3 scripts/answer_bank.py seed; run python3 scripts/answer_bank.py merge
}

panels_and_gates() {
  local res="$1"
  cd "$WORK"
  step "panels"
  # Boss enemy handbook lines for "Who's here" (panels.py skips the source when the file is missing).
  [[ "${ENEMY_INTROS:-1}" == "1" ]] && run python3 scripts/enemy_intros.py --gamedata "${TREVOR_GAMEDATA:-$ASSETS_DIR/output/en/gamedata}"
  # shellcheck disable=SC2086
  run python3 scripts/panels.py --built-from "$res" $PANELS_ARGS
  step "gates"
  local g=()
  [[ "$CANARY" == "1" ]] || g+=(--no-canary)
  # shellcheck disable=SC2086
  run python3 scripts/gates.py ${g[@]+"${g[@]}"} $GATES_ARGS
}

# ---------------------------------------------------------------- publish and switch
# flip_live RELEASE_ARTIFACTS_DIR: atomically point $CURRENT at it and request the worker restart, as one ordered step.
# The restart marker (trevor-worker-restart.path restarts trevor-worker on it) is written to a temp file BEFORE the
# flip, so a full disk fails before anything moved; after the flip it lands by a rename, and if even that fails the
# symlink goes back, so `current` never moves without the worker being told. The temp file is always a new file (any
# old one removed first, then created exclusively): an existing worker-restart.tmp stays writable in a read-only
# state/, and opening it let the flip go ahead and flip back (chaos run 2026-10-19, mismatch 5). A flip that raises
# removes the temp file it wrote.
flip_live() {
  python3 - "$1" "$CURRENT" "$STATE/worker-restart" <<'PY'
import os, sys, time
target, cur, marker = sys.argv[1:4]
tmp, nxt = marker + '.tmp', cur + '.next'
if os.path.lexists(tmp):
    os.remove(tmp)
fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
with os.fdopen(fd, 'w') as f:
    f.write(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()) + '\n')
    f.flush()
    os.fsync(f.fileno())
prev = os.readlink(cur) if os.path.islink(cur) else None
def point(to):
    if os.path.lexists(nxt):
        os.remove(nxt)
    os.symlink(to, nxt)
    os.replace(nxt, cur)
try:
    point(target)
except BaseException:
    try:
        os.remove(tmp)
    except OSError:
        pass
    raise
try:
    os.replace(tmp, marker)
except OSError as e:
    if prev is not None:
        point(prev)
    sys.exit(f'flip_live: the restart marker could not land ({e}); {cur} is back on {prev}')
PY
  log "live artifacts -> $1; worker restart requested"
}

# clean_partial_releases: a promote that did not finish leaves <release>.tmp (never in releases.jsonl); removed at the
# start of the next run so it holds no disk and no prune slot. A failed removal is logged, never fatal.
clean_partial_releases() {
  local d
  [[ -d "$RELEASES" ]] || return 0
  for d in "$RELEASES"/*.tmp; do
    [[ -d "$d" ]] || continue
    if [[ -n "$PLAN" ]]; then log "PLAN: would remove the partial release $d"; continue; fi
    log "removing the partial release $d (a promote that did not finish)"
    rm -rf -- "$d" || log "WARNING: could not remove $d"
  done
  return 0
}

# prune_releases: keep (a) the live release, (b) the previous live one (retention.py --previous-release: the release
# of the last version history.jsonl names before the current one, the one `rollback` switches the worker to), then
# (c) the newest others up to KEEP_RELEASES in all. A release that never went live (a failed activate or flip) only
# fills (c): it never displaces (a) or (b) (chaos run 2026-10-19, mismatch 6: three never-live releases pruned the
# last good one).
prune_releases() {
  local live_target prev keep=1 r
  live_target="$(abspath "$LIVE")"
  prev="$(retention --previous-release 2>/dev/null || true)"
  [[ -z "$prev" ]] || keep=2
  while IFS= read -r r; do
    [[ -n "$r" && "$r/artifacts" != "$live_target" ]] || continue
    case "$r" in *.tmp) continue ;; esac   # a partial promote: clean_partial_releases' job, not a slot
    if [[ -n "$prev" && "$(abspath "$r")" == "$prev" ]]; then log "keeping release $r (the previous live release)"; continue; fi
    if (( keep < KEEP_RELEASES )); then keep=$((keep + 1)); continue; fi
    case "$r" in "$RELEASES"/*) log "pruning release $r"; rm -rf -- "$r" ;; esac
  done < <(python3 -c 'import os,sys
d=sys.argv[1]
rs=[os.path.join(d,x) for x in os.listdir(d) if os.path.isdir(os.path.join(d,x))] if os.path.isdir(d) else []
print("\n".join(sorted(rs, key=os.path.getmtime, reverse=True)))' "$RELEASES")
}

# stamp_built_from RES: active.json's builtFrom becomes RES, its version untouched (a new res that built the same
# content as the active version). Temp file and rename; no history row (nothing was activated).
stamp_built_from() {
  local res="$1" cur
  cur="$(json_get "$STATE/active.json" builtFrom)"
  [[ -n "$res" && "$res" != "$cur" && -f "$STATE/active.json" ]] || return 0
  python3 - "$STATE/active.json" "$res" <<'PY'
import json, os, sys
p, res = sys.argv[1], sys.argv[2]
a = json.load(open(p))
a['builtFrom'] = res
with open(p + '.tmp', 'w') as f:
    json.dump(a, f, indent=1)
os.replace(p + '.tmp', p)
PY
  log "active.json: builtFrom ${cur:-none} -> $res (the version is unchanged: res $res built the same content)"
}

publish_stage() {
  cd "$WORK"
  local version active
  version="$(json_get artifacts/panels/manifest.json version)"
  active="$(json_get "$STATE/active.json" version)"
  if [[ -n "$PLAN" ]]; then echo "PLAN: publish, promote, activate"; return 0; fi
  # A flip that failed in an earlier run may have left the restart marker's temp file; flip_live never reuses it.
  if [[ -e "$STATE/worker-restart.tmp" ]]; then
    log "removing a stale $STATE/worker-restart.tmp (left by a failed flip)"
    rm -f -- "$STATE/worker-restart.tmp" 2>/dev/null || log "WARNING: could not remove $STATE/worker-restart.tmp; the flip will fail before it moves anything"
  fi
  [[ -n "$version" ]] || die "no version in the panels manifest"
  if [[ "$version" == "$active" ]]; then
    log "version $version is already active; nothing to publish"
    # The res this build came from built the same content: record it, or daily rebuilds every night until the
    # content changes (the version stays as it is).
    if [[ -z "$NO_PUBLISH" ]]; then stamp_built_from "$(json_get artifacts/panels/manifest.json builtFrom)"; fi
    return 0
  fi
  if [[ -n "$NO_PUBLISH" ]]; then
    step "publish (dry run)"
    python3 scripts/publish.py --dry-run "$LOG/publish-dry"
    log "NO_PUBLISH=1: requests in $LOG/publish-dry; the live release is unchanged"
    if [[ "$TREVOR_WARM_TOP" != "0" && -n "${TREVOR_WORKER_KEY:-}" ]]; then
      python3 scripts/bank_regen.py warm --version "$version" --limit "$TREVOR_WARM_TOP" --dry-run "$LOG/warm-dry" \
        || log "warm-up dry run failed (ignored under NO_PUBLISH)"
    fi
    return 0
  fi
  [[ -n "${TREVOR_WORKER_KEY:-}" ]] || die "TREVOR_WORKER_KEY is not set"
  step "upload $version (inactive)"
  python3 scripts/publish.py --no-activate
  step "promote the stage to a release"
  local prev_target rel
  prev_target="$(abspath "$LIVE")"
  rel="$RELEASES/$STAMP-$version"
  # Into <release>.tmp, renamed only once complete: a failed copy is never a release (clean_partial_releases).
  rm -rf -- "$rel.tmp"
  mkdir -p "$rel.tmp"
  rsync -a --delete --link-dest="$prev_target/" "$STAGE/trevor/artifacts/" "$rel.tmp/artifacts/"
  mv -- "$rel.tmp" "$rel"
  echo "{\"version\": \"$version\", \"release\": \"$rel\", \"at\": \"$STAMP\"}" >> "$STATE/releases.jsonl"
  flip_live "$rel/artifacts"
  echo "$rel/artifacts" > "$STATE/stage-base"
  sleep "$WORKER_SETTLE"
  step "activate $version"
  local rc=0
  python3 scripts/publish.py --activate "$version" --how "$CMD" || rc=$?
  if (( rc == 5 )); then
    # The backend activated it; only publish.py's local record failed. The backend is the source of truth: no flip back.
    ACTIVATED="$version"
    log "ERROR: the backend ACTIVATED $version but $STATE/history.jsonl or active.json could not be written; the worker stays on $rel/artifacts with the backend (no flip back)"
    alert_failed 5 "The backend activated $version, then the local record failed." \
      "Version $version IS live: the backend and the worker agree (no flip back). $STATE/active.json may still name the old version; the next run rebuilds and re-sends $version, which converges it. Free space or fix permissions under $STATE."
    exit 5
  elif (( rc != 0 )); then
    flip_live "$prev_target"
    echo "$prev_target" > "$STATE/stage-base"
    die "activate failed; the worker is back on $prev_target"
  fi
  ACTIVATED="$version"
  prune_releases
  local changes=""
  [[ -f "$LOG/changes.txt" ]] && changes=" changes: $(cat "$LOG/changes.txt")"
  notify "published $version (builtFrom $(json_get artifacts/panels/manifest.json builtFrom)).$changes"
  warm_up "$version"
  if [[ "$CMD" == "daily" ]]; then prune_step "$version"; fi
}

# Contract v1.1 section 3c: the backend's most-asked questions that were not carried over to $1 go back to it as
# deferred internal jobs. Never fails the run (the version is already live); a failure is an update_failed alert.
warm_up() {
  [[ "$TREVOR_WARM_TOP" != "0" ]] || { log "TREVOR_WARM_TOP=0: no warm-up"; return 0; }
  STEP="warm-up"
  local rc=0
  python3 scripts/bank_regen.py warm --version "$1" --limit "$TREVOR_WARM_TOP" || rc=$?
  (( rc == 0 )) || alert_failed "$rc" "" "Version $1 is live; only the warm-up failed (no answers were pre-asked)."
  return 0
}

# ---------------------------------------------------------------- retention and the disk check (scripts/retention.py)
retention() { python3 "$TREVOR_DIR/scripts/retention.py" --trevor-dir "$TREVOR_DIR" --home "$TREVOR_HOME" --assets-dir "$ASSETS_DIR" "$@"; }

# disk_check: one log line per filesystem; the "WARNING: disk low" lines also go to Discord as one embed. Never fails.
disk_check() {
  [[ "${DISK_CHECK:-1}" != "0" ]] || return 0
  local out low
  out="$(retention --only disk 2>&1)" || true
  printf '%s\n' "$out" | grep -v '^{' || true
  # `|| true` INSIDE the substitution: bash 3.2 runs the inherited ERR trap (set -E) in the subshell when grep finds
  # nothing, whatever the outer `|| true` says; the trap's alert and its log line then landed in $low (chaos run).
  low="$(printf '%s\n' "$out" | grep 'WARNING: disk low' | sed 's/^== [0-9:]* //' || true)"
  if [[ -n "$low" ]]; then
    post_embed "Trevor: disk space is low" "$low
Retention: \`trevor-vps-update.sh prune\` lists what the policy would delete; PRUNE_APPLY=1 deletes it." \
      13646394 "disk_low · $(hostname 2>/dev/null || echo vps) · trevor-vps-update.sh $CMD"
  fi
  return 0
}

# stale_tables_check: the unpacker keeps a game table it could not decode and lists it in gamedata/.skipped.json
# (backend core/gamedata/stale.rs); nothing alerted on it, so a format change showed only as weeks without a publish.
# One Discord embed per table the first time it is seen stale (keyed on server, table and `since` in
# $STATE/stale-alerted.json), a log line every run while it stays stale. STALE_CHECK=0 turns it off. Never fails.
stale_tables_check() {
  [[ "${STALE_CHECK:-1}" != "0" ]] || return 0
  local out new
  out="$(python3 - "$ASSETS_DIR" "$STATE/stale-alerted.json" <<'PY' 2>&1
import glob, json, os, sys
assets, seen_path = sys.argv[1], sys.argv[2]
# An unreadable file reads as empty (every stale table alerts once more), never as a reason to stop alerting.
try:
    seen = json.load(open(seen_path)) if os.path.exists(seen_path) else {}
    if not isinstance(seen, dict):
        raise ValueError('not an object')
except (OSError, ValueError) as e:
    print(f'stale: {seen_path} unreadable ({e}); starting from empty')
    seen = {}
stale, new = [], []
for f in sorted(glob.glob(os.path.join(assets, 'output', '*', 'gamedata', '.skipped.json'))):
    server = f.split(os.sep)[-3]
    try:
        tables = json.load(open(f)).get('tables', [])
    except (OSError, ValueError) as e:
        print(f'stale: cannot read {f}: {e}'); continue
    for t in tables:
        key = f"{server}/{t.get('table')}"
        line = f"{key} stale since {t.get('since', '?')} (res {t.get('resVersion', '?')}, previous output kept: {t.get('previousOutput')})"
        stale.append(line)
        if seen.get(key) != t.get('since'):
            new.append(line); seen[key] = t.get('since')
for l in stale:
    print('stale: ' + l)
if new:
    os.makedirs(os.path.dirname(seen_path), exist_ok=True)
    tmp = f'{seen_path}.tmp.{os.getpid()}'
    with open(tmp, 'w') as fh:
        json.dump(seen, fh, indent=1)
    os.replace(tmp, seen_path)
    print('NEW:' + '\n'.join(new))
PY
)" || true
  printf '%s\n' "$out" | grep -v '^NEW:' | grep -v '^$' | while IFS= read -r l; do log "$l"; done || true
  new="$(printf '%s\n' "$out" | sed -n '/^NEW:/,$p' | sed 's/^NEW://')"
  if [[ -n "$new" ]]; then
    post_embed "Trevor: a game table is stale" "$new
The unpacker kept the previous table because the new one verifies under no schema. New rows of that table are not served until the schema is fixed (assets/schemas/README.md: schema-sync, generate-fbs, a release build). The nightly update keeps publishing the rest." \
      15105570 "stale_table · $(hostname 2>/dev/null || echo vps) · trevor-vps-update.sh $CMD"
  fi
  return 0
}

# watcher_age_check: an asset watcher that stopped writing RES_VERSION_FILE looks exactly like "up to date" for ever.
# Past WATCHER_MAX_AGE_DAYS (21; 0 = off) days one embed per crossing, keyed on the file's mtime in
# $STATE/watcher-alerted.json (a newer .version that ages past the limit again alerts again); a log line every run.
# Never fails, never stops the run.
watcher_age_check() {
  local max="${WATCHER_MAX_AGE_DAYS:-21}" out days
  [[ "$max" != "0" && -f "$RES_VERSION_FILE" ]] || return 0
  out="$(python3 - "$RES_VERSION_FILE" "$STATE/watcher-alerted.json" "$max" "${PLAN:-}" <<'PY' 2>&1
import json, os, sys, time
f, seen_path, mx, plan = sys.argv[1], sys.argv[2], float(sys.argv[3]), sys.argv[4]
m = int(os.path.getmtime(f))
days = int((time.time() - m) // 86400)
if days < mx:
    sys.exit()
print(f'watcher: WARNING {f} has not changed in {days} days (WATCHER_MAX_AGE_DAYS {mx:g})')
try:
    seen = json.load(open(seen_path))
except (OSError, ValueError):
    seen = {}
if isinstance(seen, dict) and seen.get('mtime') == m:
    sys.exit()
if not plan:
    try:
        tmp = f'{seen_path}.tmp.{os.getpid()}'
        with open(tmp, 'w') as fh:
            json.dump({'mtime': m, 'days': days, 'at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}, fh)
        os.replace(tmp, seen_path)
    except OSError as e:
        print(f'watcher: cannot write {seen_path} ({e}); the alert repeats next run')
print(f'NEW:{days}')
PY
)" || true
  printf '%s\n' "$out" | grep -v '^NEW:' | grep -v '^$' | while IFS= read -r l; do log "$l"; done || true
  days="$(printf '%s\n' "$out" | sed -n 's/^NEW://p')"
  if [[ -n "$days" ]]; then
    post_embed "Trevor: the asset watcher has not updated in $days days" "$RES_VERSION_FILE was last written $days days ago (limit WATCHER_MAX_AGE_DAYS=$max). Until it changes, every nightly update says \"up to date\" and publishes nothing. Check the asset watcher's service and its log." \
      15105570 "watcher_stale · $(hostname 2>/dev/null || echo vps) · trevor-vps-update.sh $CMD"
  fi
  return 0
}

# prune_step VERSION: after a successful publish only (never before one): retention over logs, eval runs, the spool,
# releases and asset manifests; a dry run that lists what is over the policy unless PRUNE_APPLY=1. Exit 2 (something
# was over the policy) is normal; an error is one update_failed alert and never fails the run (VERSION is live).
prune_step() {
  [[ "${PRUNE:-1}" != "0" ]] || return 0   # PRUNE=0: silent, the script before 2026-10-19
  STEP="prune"
  local args=(--only logs,runs,spool,releases,assets) rc=0
  if [[ "${PRUNE_APPLY:-0}" == "1" ]]; then args+=(--apply); fi
  if [[ "${PRUNE_APPLY_ASSETS:-0}" == "1" ]]; then args+=(--apply-assets); fi
  log "prune: retention.py ${args[*]}"
  retention "${args[@]}" || rc=$?
  if (( rc == 1 )); then alert_failed 1 "" "Version ${1:-?} is live; only the prune failed (nothing else changed)."; fi
  return 0
}

# ---------------------------------------------------------------- the local summary model
start_local_llm() {
  if [[ ! -f "$E2B_GGUF" ]]; then log "no E2B model at $E2B_GGUF; local summaries skipped"; return 1; fi
  if [[ -r /proc/meminfo ]]; then
    local avail; avail="$(awk '/MemAvailable/ {print int($2 / 1024)}' /proc/meminfo)"
    if (( avail < SUMMARY_MIN_AVAIL_MB )); then log "MemAvailable ${avail} MB < ${SUMMARY_MIN_AVAIL_MB} MB; local summaries skipped"; return 1; fi
  fi
  take_model_lock summaries 3600 || { log "model lock $TREVOR_LLM_LOCK busy for an hour; local summaries skipped"; return 1; }
  "$LLAMA_SERVER" -m "$E2B_GGUF" --host 127.0.0.1 --port "$SUMMARY_PORT" -t "$SUMMARY_THREADS" -c "$SUMMARY_CTX" -np 1 \
    --no-repack --poll 0 --cache-ram 0 --no-webui >> "$LOG/summary-llm.log" 2>&1 &
  LLM_PID=$!
  local i
  for i in $(seq 1 300); do
    curl -sf --connect-timeout 2 -m 5 "http://127.0.0.1:$SUMMARY_PORT/health" >/dev/null 2>&1 && return 0
    kill -0 "$LLM_PID" 2>/dev/null || { log "llama-server exited while loading; see $LOG/summary-llm.log"; LLM_PID=""; release_model_lock; return 1; }
    sleep 1
  done
  log "llama-server not healthy in 300 s"; stop_local_llm; return 1
}
stop_local_llm() {
  if [[ -n "$LLM_PID" ]]; then
    kill "$LLM_PID" 2>/dev/null || true
    for _ in $(seq 1 30); do kill -0 "$LLM_PID" 2>/dev/null || break; sleep 1; done
    kill -9 "$LLM_PID" 2>/dev/null || true
    wait "$LLM_PID" 2>/dev/null || true
    LLM_PID=""
  fi
  release_model_lock
}

# ---------------------------------------------------------------- the backend's deferred reload (backend 89f07593)
# The backend serves no game-data version: /api/health's gameData carries degraded, tables and stale only (a stale
# entry's resVersion is the skipped input's), and no reload time is exposed. So `daily` reads what the backend's
# reload guard reads (core/reload_guard.rs `check`): the model lock, a pid first and alive by /proc/<pid> or `ps -p`,
# then MemAvailable against the EN floor. A refused reload retries every RELOAD_RETRY_SECS, so once the guard has
# admitted EN for RELOAD_RETRY_SECS plus RELOAD_BUILD_SECS without a break, a deferred reload has run and swapped (or
# none was deferred). An inference, not a report: a reload whose build FAILED looks the same.

# guard_refusal: why the backend's guard would refuse an EN reload now, or nothing. Always exits 0 (it runs inside a
# substitution, where bash 3.2 runs the ERR trap on a failing last command).
guard_refusal() {
  local line="" pid="" label="" avail=""
  [[ ! -r "$TREVOR_LLM_LOCK" ]] || line="$(cat "$TREVOR_LLM_LOCK" 2>/dev/null || true)"
  read -r pid label <<< "$line" || true
  if [[ "$pid" =~ ^[0-9]+$ ]] && (( 10#$pid > 0 )); then
    if [[ -d /proc/self ]]; then
      if [[ -d "/proc/$pid" ]]; then echo "the model lock names live pid $pid${label:+ ($label)}"; return 0; fi
    elif ps -p "$pid" >/dev/null 2>&1; then
      echo "the model lock names live pid $pid${label:+ ($label)}"; return 0
    fi
  fi
  if (( RELOAD_EN_MIN_FREE_MB > 0 )) && [[ -r /proc/meminfo ]]; then
    avail="$(awk '/^MemAvailable:/ {print int($2 / 1024); exit}' /proc/meminfo 2>/dev/null || true)"
    if [[ -n "$avail" ]] && (( avail < RELOAD_EN_MIN_FREE_MB )); then
      echo "MemAvailable $avail MiB is under the EN floor of $RELOAD_EN_MIN_FREE_MB MiB"
    fi
  fi
  return 0
}

# wait_for_backend_reload RES: 0 once the guard has admitted EN for RELOAD_RETRY_SECS + RELOAD_BUILD_SECS without a
# break, 1 when BACKEND_RELOAD_WAIT_MIN ran out first (RELOAD_WAIT_WHY says why). PLAN=1 prints the check, returns 0.
RELOAD_WAIT_WHY=""
wait_for_backend_reload() {
  local res="$1" need=$(( RELOAD_RETRY_SECS + RELOAD_BUILD_SECS )) limit why open_since="" now start=$SECONDS last=-1
  limit="$(python3 -c 'import sys; print(int(float(sys.argv[1]) * 60))' "$BACKEND_RELOAD_WAIT_MIN")"
  if [[ -n "$PLAN" ]]; then
    why="$(guard_refusal)"
    echo "PLAN: wait up to $BACKEND_RELOAD_WAIT_MIN min for the backend's EN reload of $res (the reload guard ${why:+refuses now: $why}${why:-admits now}; it must admit for ${need}s without a break)"
    return 0
  fi
  log "waiting for the backend to load $res: its reload guard must admit EN for ${need}s without a break (RELOAD_RETRY_SECS $RELOAD_RETRY_SECS + RELOAD_BUILD_SECS $RELOAD_BUILD_SECS), at most $BACKEND_RELOAD_WAIT_MIN min"
  while :; do
    now=$SECONDS
    why="$(guard_refusal)"
    if [[ -n "$why" ]]; then open_since=""; elif [[ -z "$open_since" ]]; then open_since=$now; fi
    if [[ -n "$open_since" ]] && (( now - open_since >= need )); then
      log "the reload guard admitted EN for $(( now - open_since ))s; taking $res as loaded"
      return 0
    fi
    if (( now - start >= limit )); then
      RELOAD_WAIT_WHY="${why:-the reload guard admitted EN for only $(( now - open_since ))s of ${need}s}"
      return 1
    fi
    if (( last < 0 || now - last >= 60 )); then
      log "still waiting for the backend's reload of $res ($(( (now - start) / 60 )) of $BACKEND_RELOAD_WAIT_MIN min): ${why:-the guard has admitted EN for $(( now - open_since ))s of ${need}s}"
      last=$now
    fi
    sleep "$BACKEND_RELOAD_POLL_SECS"
  done
}

# reload_wait_skipped RES: the skip line, $STATE/reload-wait.json ({res, nights, lastNight}: consecutive UTC nights
# that skipped; a second run the same night counts once), and one embed on the night the count reaches 2.
reload_wait_skipped() {
  local res="$1" second
  log "backend has not loaded $res yet (serves an older version: $RELOAD_WAIT_WHY); skipping tonight, the next run retries"
  second="$(python3 - "$STATE/reload-wait.json" "$res" <<'PY' 2>&1 || true
import datetime, json, os, sys
path, res = sys.argv[1], sys.argv[2]
today = datetime.datetime.now(datetime.timezone.utc).date()
try:
    s = json.load(open(path))
    last, n = datetime.date.fromisoformat(s['lastNight']), int(s['nights'])
except (OSError, ValueError, KeyError, TypeError):
    last, n = None, 0
if last == today:
    print('same night')
    sys.exit()
n = n + 1 if last == today - datetime.timedelta(days=1) else 1
os.makedirs(os.path.dirname(path), exist_ok=True)
tmp = f'{path}.tmp.{os.getpid()}'
with open(tmp, 'w') as fh:
    json.dump({'res': res, 'nights': n, 'lastNight': today.isoformat()}, fh)
os.replace(tmp, path)
print('SECOND' if n == 2 else f'night {n}')
PY
)"
  if [[ "$second" == "SECOND" ]]; then
    post_embed "Trevor: the backend has not loaded the new game data" "Two nights running, \`trevor-vps-update.sh daily\` waited $BACKEND_RELOAD_WAIT_MIN min for the backend to reload EN res_version $res and skipped the build: $RELOAD_WAIT_WHY. The backend defers a reload while the Trevor model lock names a live pid or memory is short (RELOAD_GUARD, core/reload_guard.rs): check what holds $TREVOR_LLM_LOCK and the backend log for \"held back by the reload guard\". Trevor's corpus lags the game until the backend reloads." \
      15105570 "reload_wait · $(hostname 2>/dev/null || echo vps) · trevor-vps-update.sh $CMD"
  fi
  return 0
}

pending_summaries() {  # units with no summary row, in the current directory's tree
  python3 scripts/summaries_cf.py --plan | tail -1 | python3 -c 'import json,sys; c=json.load(sys.stdin); print(c["story"]["todo"] + c["group"]["todo"])'
}

# ---------------------------------------------------------------- commands
# ingested_res: the game-data version the active release was built from (active.json builtFrom), for a publish that
# fetched nothing (summaries, rebuild). The asset watcher's .version may be newer and not yet ingested; stamping it
# would make the next daily say "up to date" and never ingest it. Empty: a publish without a fetch cannot vouch for any
# version (panels.py would fall back to .version), so it stops; the first daily writes builtFrom.
ingested_res() {
  local b; b="$(json_get "$STATE/active.json" builtFrom)"
  [[ -n "$b" || -n "$PLAN" ]] || die "no builtFrom in $STATE/active.json: run \`$0 daily\` first (a $CMD publishes no game-data version of its own)"
  echo "$b"
}

cmd_daily() {
  local res res_age active_from
  [[ -f "$RES_VERSION_FILE" ]] || die "no EN res_version at $RES_VERSION_FILE (the asset watcher writes it after an extract)"
  res="$(tr -d '[:space:]' < "$RES_VERSION_FILE")"
  res_age=$(( ($(date +%s) - $(mtime "$RES_VERSION_FILE")) / 60 ))
  active_from="$(json_get "$STATE/active.json" builtFrom)"
  disk_check
  stale_tables_check
  watcher_age_check
  if [[ "$res" == "$active_from" && "$FORCE" != "1" ]]; then
    log "up to date: EN res_version $res is the active version's builtFrom"; return 0
  fi
  if (( res_age < SETTLE_MIN )) && [[ "$FORCE" != "1" ]]; then
    log "EN res_version $res is $res_age min old (< $SETTLE_MIN); the next run takes it"; return 0
  fi
  # The run lock is held (the stage runner pauses and frees the model lock), so a deferred reload can run now. No
  # fetch (SKIP_FETCH) reads nothing from the backend and needs no wait.
  if [[ "$BACKEND_RELOAD_WAIT_MIN" != "0" && -z "$SKIP_FETCH" ]]; then
    if ! wait_for_backend_reload "$res"; then reload_wait_skipped "$res"; return 0; fi
    [[ -n "$PLAN" ]] || rm -f "$STATE/reload-wait.json"
  fi
  log "EN res_version $res; active version built from ${active_from:-nothing}"
  if [[ -n "$PLAN" ]]; then WORK="$TREVOR_DIR"; else prepare_stage; fi
  deterministic_build 1
  panels_and_gates "$res"
  publish_stage
}

cmd_summaries() {  # summaries for new stories, then the bank's pending answers; one model server for both
  local res before after written=0 n bank_before bank_after bank_done=0 cf_spent=0
  res="$(ingested_res)"
  if [[ -n "$PLAN" ]]; then
    cd "$TREVOR_DIR"
    python3 scripts/summaries_cf.py --plan
    echo "PLAN: summaries_cf.py --backend cf, then (if any remain) llama-server $E2B_GGUF on :$SUMMARY_PORT and --backend local --until $SUMMARY_UNTIL, then the rebuild, gates and publish"
    if [[ "$REGEN_ANSWERS" != "0" ]]; then
      PLAN=1 python3 scripts/bank_regen.py run
      echo "PLAN: bank_regen.py run --backend cf (after the summaries), then --backend local on the same llama-server until $SUMMARY_UNTIL"
    fi
    return 0
  fi
  prepare_stage
  cd "$WORK"
  before="$(pending_summaries)"
  bank_before=0
  [[ "$REGEN_ANSWERS" == "0" ]] || bank_before="$(python3 scripts/bank_regen.py count | tail -1)"
  if [[ "$before" == "0" && "$bank_before" == "0" ]]; then log "no story or group lacks a summary and no bank unit is pending"; return 0; fi
  if [[ "$before" != "0" ]]; then
    step "summaries: $before units (Cloudflare first)"
    python3 scripts/summaries_cf.py --backend cf --status "$LOG/summaries-cf.json" || log "cf summaries ended with an error"
    n="$(json_get "$LOG/summaries-cf.json" written)"; written=$((written + ${n:-0}))
    cf_spent="$(json_get "$LOG/summaries-cf.json" neurons)"
  fi
  if [[ "$bank_before" != "0" ]]; then
    step "bank answers: $bank_before pending units on Cloudflare (after ${cf_spent:-0} summary Neurons)"
    python3 scripts/bank_regen.py run --backend cf --cf-spent "${cf_spent:-0}" --status "$LOG/bank-cf.json" \
      || log "cf bank answers ended with an error"
    n="$(json_get "$LOG/bank-cf.json" units)"; bank_done=$((bank_done + ${n:-0}))
  fi
  after="$(pending_summaries)"
  bank_after=0
  [[ "$REGEN_ANSWERS" == "0" ]] || bank_after="$(python3 scripts/bank_regen.py count | tail -1)"
  if [[ ( "$after" != "0" || "$bank_after" != "0" ) && "$LOCAL_SUMMARIES" == "1" ]]; then
    step "local E2B until $SUMMARY_UNTIL UTC: $after summary units, then $bank_after bank units"
    if start_local_llm; then
      if [[ "$after" != "0" ]]; then
        TREVOR_SUMMARY_LLM_URL="http://127.0.0.1:$SUMMARY_PORT" python3 scripts/summaries_cf.py --backend local \
          --until "$SUMMARY_UNTIL" --status "$LOG/summaries-local.json" || log "local summaries ended with an error"
        n="$(json_get "$LOG/summaries-local.json" written)"; written=$((written + ${n:-0}))
      fi
      if [[ "$bank_after" != "0" ]]; then
        STEP="bank answers (local)"
        TREVOR_SUMMARY_LLM_URL="http://127.0.0.1:$SUMMARY_PORT" python3 scripts/bank_regen.py run --backend local \
          --until "$SUMMARY_UNTIL" --status "$LOG/bank-local.json" || log "local bank answers ended with an error"
        n="$(json_get "$LOG/bank-local.json" units)"; bank_done=$((bank_done + ${n:-0}))
      fi
      stop_local_llm
    fi
  fi
  log "summaries written: $written; still pending: $(pending_summaries); bank units done: $bank_done, pending: $( [[ "$REGEN_ANSWERS" == "0" ]] && echo 0 || python3 scripts/bank_regen.py count | tail -1)"
  if (( written == 0 && bank_done == 0 )); then return 0; fi
  deterministic_build 0
  panels_and_gates "$res"
  publish_stage
}

cmd_rollback() {
  require_live_release
  local target="$ARG2" rel
  if [[ -z "$target" ]]; then
    target="$(python3 -c 'import json,sys
rows=[json.loads(l) for l in open(sys.argv[1]) if l.strip()]
cur=rows[-1]["version"] if rows else None
prev=[r["version"] for r in rows if r["version"] != cur]
print(prev[-1] if prev else "")' "$STATE/history.jsonl" 2>/dev/null || true)"
  fi
  [[ -n "$target" ]] || die "no earlier version in $STATE/history.jsonl; name one: $0 rollback VERSION"
  rel="$(python3 -c 'import json,sys,os
rows=[json.loads(l) for l in open(sys.argv[1]) if l.strip()]
m=[r["release"] for r in rows if r["version"] == sys.argv[2] and os.path.isdir(r["release"])]
print(m[-1] if m else "")' "$STATE/releases.jsonl" "$target" 2>/dev/null || true)"
  log "rollback to $target (release ${rel:-not kept: panels only, the worker keeps its artifacts})"
  if [[ -n "$PLAN" ]]; then echo "PLAN: publish.py --activate $target; flip to $rel"; return 0; fi
  [[ -z "$rel" ]] || { flip_live "$rel/artifacts"; echo "$rel/artifacts" > "$STATE/stage-base"; sleep "$WORKER_SETTLE"; }
  python3 "$TREVOR_DIR/scripts/publish.py" --activate "$target" --how rollback
  notify "rolled back to $target"
}

cmd_status() {
  echo "active:  $(cat "$STATE/active.json" 2>/dev/null | tr -d '\n' || echo none)"
  echo "live:    $LIVE -> $CURRENT -> $(abspath "$LIVE")"
  echo "assets:  EN res_version $(cat "$RES_VERSION_FILE" 2>/dev/null || echo '?')"
  echo "backend: $(curl -s --connect-timeout 5 -m 10 "$TREVOR_BACKEND/api/trevor/status" || echo unreachable)"
  find "$RELEASES" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort -r | head -5 | sed 's/^/release: /'
}

cmd_prune() { prune_step "$(json_get "$STATE/active.json" version)"; disk_check; }

# rebuild: the stage runner's publish (scripts/stages.py --publish, 2026-10-19): the deterministic rebuilds, panels
# and gates over the stage tree the runner's model stages wrote into, then the same atomic publish as daily, without
# a fetch and without the game-data version gate. PLAN=1 runs the plan over the live tree and publishes nothing.
# Exit 4 when another update holds the run lock (the runner retries in its next pass).
cmd_rebuild() {
  local built; built="$(ingested_res)"
  if [[ -n "$PLAN" ]]; then WORK="$TREVOR_DIR"; else prepare_stage; fi
  deterministic_build 0; panels_and_gates "$built"; publish_stage
}

cmd_init() {  # run once, as a user that may write the code directory (root); then chown TREVOR_HOME to trevor
  [[ -L "$LIVE" ]] && die "$LIVE is already a symlink"
  [[ -d "$LIVE" ]] || die "no $LIVE to adopt: copy the Mac's artifacts there first"
  mkdir -p "$RELEASES" "$STATE"
  local rel="$RELEASES/$STAMP-initial"
  mkdir -p "$rel"
  mv "$LIVE" "$rel/artifacts"
  ln -sfn "$rel/artifacts" "$CURRENT"
  ln -s "$CURRENT" "$LIVE"
  echo "$rel/artifacts" > "$STATE/stage-base"
  log "adopted the artifacts as $rel; $LIVE -> $CURRENT -> $rel/artifacts"
}

case "$CMD" in
  daily|summaries|rebuild)
    if [[ "$CMD" == "rebuild" ]]; then take_run_lock 4; else take_run_lock; fi
    [[ -n "$PLAN" ]] || open_log
    clean_partial_releases
    require_live_release
    "cmd_$CMD" ;;
  rollback) take_run_lock; [[ -n "$PLAN" ]] || open_log; cmd_rollback ;;
  status) cmd_status ;;
  prune) take_run_lock; cmd_prune ;;
  init) take_run_lock; cmd_init ;;
  *) die "usage: $0 daily|summaries|rebuild|rollback [VERSION]|status|prune|init" ;;
esac
log "done ($CMD)"
