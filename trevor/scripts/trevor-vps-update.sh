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
# <trevor>/artifacts, a symlink that moves to the new release only at the end (flipped back if the activate fails).
# A failed run leaves the stage as it is, so the next run continues from it (summaries are not lost);
# RESET_STAGE=1 copies the live release over it first. One run at a time (flock); nice 19, ionice idle.
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
#   NO_PUBLISH=1  publish.py --dry-run into the log dir; no upload, no switch;  KEEP_RELEASES [3]
#   WORKER_SETTLE [30]  seconds between the worker restart and the activate call;  SETTLE_MIN [15] minutes a new
#              .version must be old before `daily` uses it (the backend hot-reloads after the watcher's extract)
#   PRUNE [1]  0 = no prune step after a daily publish (the script before 2026-10-19). The step runs scripts/retention.py
#              (logs, eval runs, the spool, releases, asset manifests; policy in its docstring and RETAIN_* variables)
#              as a dry run that lists what it would delete; PRUNE_APPLY=1 deletes, PRUNE_APPLY_ASSETS=1 also the old
#              asset manifests
#   DISK_CHECK [1]  0 = no disk check; else every `daily` run (and `prune`) warns, and posts one disk_low embed, when a
#              filesystem of TREVOR_HOME, TREVOR_DIR or ASSETS_DIR has less free space than the smaller of
#              DISK_MIN_FREE_GB [10] GB and DISK_MIN_FREE_PCT [10] percent
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
    | curl -sS -m 20 -H 'content-type: application/json' -d @- "$ALERT_WEBHOOK" >/dev/null 2>&1 \
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
  local rc="$1" msg="${2:-}" tail_lines=""
  local outcome="${3:-The last good version stays active; the stage keeps the work for the next run.}"
  [[ -e "$ALERT_MARK" ]] && return 0
  : > "$ALERT_MARK" 2>/dev/null || true
  log "update_failed: step '$STEP', exit $rc${msg:+: $msg}"
  [[ -n "$LOG" && -f "$LOG/run.log" ]] && { sleep 0.5; tail_lines="$(tail -n "$ALERT_LOG_LINES" "$LOG/run.log" | sed 's/```/` ` `/g')"; }
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
trap 'stop_local_llm; release_model_lock; rm -f "$ALERT_MARK"' EXIT
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
open_log() {
  mkdir -p "$TREVOR_HOME/logs" "$STATE"
  LOG="$TREVOR_HOME/logs/$STAMP-$CMD"
  mkdir -p "$LOG"
  exec > >(tee -a "$LOG/run.log") 2>&1
  find "$TREVOR_HOME/logs" -mindepth 1 -maxdepth 1 -type d -mtime +30 -exec rm -rf {} + 2>/dev/null || true
}

take_run_lock() {
  mkdir -p "$STATE"
  exec 9>"$STATE/update.lock"
  if command -v flock >/dev/null 2>&1; then
    flock -n 9 || { log "another update holds $STATE/update.lock; nothing to do"; exit 0; }
  else
    log "WARNING: no flock(1) here; runs are not serialized (util-linux provides it on Ubuntu)"
  fi
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
  if [[ ! -d "$STAGE/trevor/artifacts" || "$base" != "$live_target" || "${RESET_STAGE:-0}" == "1" ]]; then
    log "stage: copying the live release ($live_target) into the stage"
    rsync -a --delete "$live_target/" "$STAGE/trevor/artifacts/"
    echo "$live_target" > "$STATE/stage-base"
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
    local code; code="$(curl -s -o /dev/null -w '%{http_code}' "$TREVOR_BACKEND/" || true)"
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
  [[ "${ENEMY_INTROS:-1}" == "1" ]] && run python3 scripts/enemy_intros.py --gamedata "$TREVOR_GAMEDATA"
  # shellcheck disable=SC2086
  run python3 scripts/panels.py --built-from "$res" $PANELS_ARGS
  step "gates"
  local g=()
  [[ "$CANARY" == "1" ]] || g+=(--no-canary)
  # shellcheck disable=SC2086
  run python3 scripts/gates.py ${g[@]+"${g[@]}"} $GATES_ARGS
}

# ---------------------------------------------------------------- publish and switch
flip_live() {  # flip_live RELEASE_ARTIFACTS_DIR: atomically point $CURRENT at it, then restart the worker
  python3 -c 'import os,sys; t=sys.argv[2]+".next"
if os.path.lexists(t): os.remove(t)
os.symlink(sys.argv[1], t); os.replace(t, sys.argv[2])' "$1" "$CURRENT"
  date -u +%FT%TZ > "$STATE/worker-restart"   # trevor-worker-restart.path restarts trevor-worker on this
  log "live artifacts -> $1; worker restart requested"
}

prune_releases() {  # keep the live release and the KEEP_RELEASES - 1 newest others
  local live_target keep=0 r
  live_target="$(abspath "$LIVE")"
  while IFS= read -r r; do
    [[ -n "$r" && "$r/artifacts" != "$live_target" ]] || continue
    keep=$((keep + 1))
    if (( keep >= KEEP_RELEASES )); then
      case "$r" in "$RELEASES"/*) log "pruning release $r"; rm -rf -- "$r" ;; esac
    fi
  done < <(python3 -c 'import os,sys
d=sys.argv[1]
rs=[os.path.join(d,x) for x in os.listdir(d) if os.path.isdir(os.path.join(d,x))] if os.path.isdir(d) else []
print("\n".join(sorted(rs, key=os.path.getmtime, reverse=True)))' "$RELEASES")
}

publish_stage() {
  cd "$WORK"
  local version active
  version="$(json_get artifacts/panels/manifest.json version)"
  active="$(json_get "$STATE/active.json" version)"
  if [[ -n "$PLAN" ]]; then echo "PLAN: publish, promote, activate"; return 0; fi
  [[ -n "$version" ]] || die "no version in the panels manifest"
  if [[ "$version" == "$active" ]]; then
    log "version $version is already active; nothing to publish"
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
  mkdir -p "$rel"
  rsync -a --delete --link-dest="$prev_target/" "$STAGE/trevor/artifacts/" "$rel/artifacts/"
  echo "{\"version\": \"$version\", \"release\": \"$rel\", \"at\": \"$STAMP\"}" >> "$STATE/releases.jsonl"
  flip_live "$rel/artifacts"
  echo "$rel/artifacts" > "$STATE/stage-base"
  sleep "$WORKER_SETTLE"
  step "activate $version"
  if ! python3 scripts/publish.py --activate "$version"; then
    flip_live "$prev_target"
    echo "$prev_target" > "$STATE/stage-base"
    die "activate failed; the worker is back on $prev_target"
  fi
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
  low="$(printf '%s\n' "$out" | grep 'WARNING: disk low' | sed 's/^== [0-9:]* //')" || true
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
seen = json.load(open(seen_path)) if os.path.exists(seen_path) else {}
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
    json.dump(seen, open(seen_path, 'w'), indent=1)
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
    curl -sf "http://127.0.0.1:$SUMMARY_PORT/health" >/dev/null 2>&1 && return 0
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

pending_summaries() {  # units with no summary row, in the current directory's tree
  python3 scripts/summaries_cf.py --plan | tail -1 | python3 -c 'import json,sys; c=json.load(sys.stdin); print(c["story"]["todo"] + c["group"]["todo"])'
}

# ---------------------------------------------------------------- commands
cmd_daily() {
  local res res_age active_from
  [[ -f "$RES_VERSION_FILE" ]] || die "no EN res_version at $RES_VERSION_FILE (the asset watcher writes it after an extract)"
  res="$(tr -d '[:space:]' < "$RES_VERSION_FILE")"
  res_age=$(( ($(date +%s) - $(mtime "$RES_VERSION_FILE")) / 60 ))
  active_from="$(json_get "$STATE/active.json" builtFrom)"
  disk_check
  stale_tables_check
  if [[ "$res" == "$active_from" && "$FORCE" != "1" ]]; then
    log "up to date: EN res_version $res is the active version's builtFrom"; return 0
  fi
  if (( res_age < SETTLE_MIN )) && [[ "$FORCE" != "1" ]]; then
    log "EN res_version $res is $res_age min old (< $SETTLE_MIN); the next run takes it"; return 0
  fi
  log "EN res_version $res; active version built from ${active_from:-nothing}"
  if [[ -n "$PLAN" ]]; then WORK="$TREVOR_DIR"; else prepare_stage; fi
  deterministic_build 1
  panels_and_gates "$res"
  publish_stage
}

cmd_summaries() {  # summaries for new stories, then the bank's pending answers; one model server for both
  local res before after written=0 n bank_before bank_after bank_done=0 cf_spent=0
  res="$(tr -d '[:space:]' < "$RES_VERSION_FILE" 2>/dev/null || true)"
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
  python3 "$TREVOR_DIR/scripts/publish.py" --activate "$target"
  notify "rolled back to $target"
}

cmd_status() {
  echo "active:  $(cat "$STATE/active.json" 2>/dev/null | tr -d '\n' || echo none)"
  echo "live:    $LIVE -> $CURRENT -> $(abspath "$LIVE")"
  echo "assets:  EN res_version $(cat "$RES_VERSION_FILE" 2>/dev/null || echo '?')"
  echo "backend: $(curl -s -m 10 "$TREVOR_BACKEND/api/trevor/status" || echo unreachable)"
  find "$RELEASES" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort -r | head -5 | sed 's/^/release: /'
}

cmd_prune() { prune_step "$(json_get "$STATE/active.json" version)"; disk_check; }

# rebuild: the stage runner's publish (scripts/stages.py --publish, 2026-10-19): the deterministic rebuilds, panels
# and gates over the stage tree the runner's model stages wrote into, then the same atomic publish as daily, without
# a fetch and without the game-data version gate. PLAN=1 runs the plan over the live tree and publishes nothing.
cmd_rebuild() {
  local res; res="$(tr -d '[:space:]' < "$RES_VERSION_FILE" 2>/dev/null || true)"
  if [[ -n "$PLAN" ]]; then WORK="$TREVOR_DIR"; else prepare_stage; fi
  deterministic_build 0; panels_and_gates "$res"; publish_stage
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
    take_run_lock
    [[ -n "$PLAN" ]] || open_log
    require_live_release
    "cmd_$CMD" ;;
  rollback) take_run_lock; [[ -n "$PLAN" ]] || open_log; cmd_rollback ;;
  status) cmd_status ;;
  prune) take_run_lock; cmd_prune ;;
  init) take_run_lock; cmd_init ;;
  *) die "usage: $0 daily|summaries|rebuild|rollback [VERSION]|status|prune|init" ;;
esac
log "done ($CMD)"
