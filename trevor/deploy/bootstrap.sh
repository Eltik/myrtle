#!/usr/bin/env bash
# Trevor bootstrap: the one command a fresh Ubuntu 24.04 x86_64 server runs after `git clone` to become a serving
# Trevor instance with no model vendor (the cache and cpu rungs, passages as the floor;
# design/trevor-self-contained-deploy.md). Run as root from the clone:
#
#   sudo trevor/deploy/bootstrap.sh --snapshot URL|FILE [--sha SHA] [options]
#
# Steps, in order (each checks its own done-state, so a rerun continues where the last one stopped):
#   preflight     OS and arch, tools (missing apt packages are installed), memory and disk thresholds, the trevor user,
#                 /var/lib/trevor, /etc/trevor, /run/trevor (tmpfiles)
#   release       scripts/fetch-release.sh --components trevor,assets[,backend] SHA: the CI-built binaries. The bundle
#                 exists only for commits built on main by the Release Bundle workflow; exit 3 (no bundle) stops here,
#                 nothing is ever compiled on the box. backend is added when backend/target/release/backend is missing
#   llama         llama.cpp at the pin in deploy/pins.env, sha256-checked, into /opt/llama.cpp/<tag>, `current` symlink,
#                 /usr/local/bin/llama-server (the path trevor-llama.service runs)
#   models        scripts/fetch-model.sh with WITH_FP32=1 WITH_E2B=1, and WITH_LLM=1 (the 12B and Qwen, 12.7 GB) only with
#                 16 GB of memory or more, into trevor/models; the E2B file is linked under the name the units use
#   snapshot      scripts/snapshot.py install FILE --into trevor/artifacts, from --snapshot URL or path. --no-snapshot
#                 starts from an empty tree, and the first update then builds everything: weeks on a CPU
#   env           /etc/trevor/trevor.env from deploy/trevor.env.example (paths, BM25 serving, a fresh TREVOR_WORKER_KEY,
#                 the Cloudflare keys left unset), and the same TREVOR_WORKER_KEY in backend/.env
#   database      the backend applies its migrations on startup (backend/README.md, Database), so this (re)starts the
#                 backend under pm2 and waits until it answers and accepts the worker key
#   units         installs trevor-llama, trevor-worker, trevor-worker-restart.{path,service}, trevor-update.{service,timer};
#                 daemon-reload; enables and starts llama, the worker and the update timer. Not trevor-summaries.timer:
#                 that is the Cloudflare summaries path, unused here
#   first-update  trevor-vps-update.sh init (adopts trevor/artifacts as the first release), then `daily` once in the
#                 foreground (FORCE=1), log under /var/lib/trevor/logs/<stamp>-daily/run.log
#   verify        the worker heartbeats (/api/trevor/status online, cpu rung), `ask --route-only` of one question, and
#                 a one-screen summary
#
# Options:
#   --sha SHA              the commit whose release bundle to install (default: the checked-out HEAD)
#   --snapshot URL|FILE    the corpus snapshot (snapshot.py pack); a URL is downloaded to /var/lib/trevor/snapshot/
#   --snapshot-sha256 HEX  checked against the snapshot file before install (the manifest inside only proves the
#                          archive is whole, not that it is the one you meant)
#   --no-snapshot          no snapshot: an empty artifacts tree (prints why that is weeks of CPU)
#   --database-url URL     only when backend/.env does not exist: create it from backend/.env.example with this
#                          DATABASE_URL and fresh JWT_SECRET, SERVICE_KEY and GAME_CREDENTIAL_KEY
#   --webhook URL          DISCORD_ALERT_WEBHOOK for the update script's alerts (optional)
#   --only STEP[,STEP]     run only these steps (repeatable);  --skip STEP[,STEP]  skip these (repeatable)
#   --dry-run              print every command that would run and run nothing (read-only checks still run)
#   -h, --help             this text
#
# Environment: DESTDIR prefixes the files the env step writes (/etc/trevor/trevor.env, backend/.env) for a test run
# without root; BACKEND_ENV overrides the backend's env file; WITH_LLM=0|1 overrides the 12B memory rule.
#
# Exit: 0 done; 1 a step failed (the message names it); 2 usage; 3 no release bundle for the sha.
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TREVOR_DIR="$ROOT/trevor"
DEPLOY="$TREVOR_DIR/deploy"
UNITS_SRC="$DEPLOY/systemd"
TREVOR_HOME=/var/lib/trevor
STATE="$TREVOR_HOME/state"
DESTDIR="${DESTDIR:-}"
ETC_DIR="$DESTDIR/etc/trevor"
ENV_FILE="$ETC_DIR/trevor.env"
BACKEND_ENV="${BACKEND_ENV:-$DESTDIR$ROOT/backend/.env}"
UNIT_DIR=/etc/systemd/system
LLAMA_ROOT=/opt/llama.cpp
LLAMA_BIN=/usr/local/bin
BACKEND_URL=http://127.0.0.1:3060
LLAMA_URL=http://127.0.0.1:8081
UNITS_PATH=/var/www/myrtle.moe    # the repo path the units are written for
ASSETS_DIR="$ROOT/assets"
RES_VERSION_FILE="$ASSETS_DIR/ArkAssets/en/.version"
E2B_GGUF_NAME=gemma-4-e2b-it-q4_k_m.gguf  # the name fetch-model.sh writes; the units and trevor.env.example use it too
ROUTE_QUESTION="Who is Amiya?"
# Thresholds of design/trevor-self-contained-deploy.md section 5 (memory) and the sizes below (disk): models 4 GB
# without the 12B, 17 GB with it; the snapshot file 0.2 GB, its tree 1.2 GB, and the live release, the stage and the
# release copies of the update about three trees more.
MEM_FAIL_GB=8; MEM_LLM_GB=16; MEM_WARN_GB=32; CORES_WARN=8
DISK_FAIL_GB=15; DISK_WARN_GB=40

STEPS="preflight release llama models snapshot env database units first-update verify"
SHA=""; SNAPSHOT=""; SNAPSHOT_SHA=""; NO_SNAPSHOT=0; DATABASE_URL_NEW=""; WEBHOOK=""
ONLY=""; SKIP=""; DRY_RUN=0
STEP="start"; WARNINGS=()

usage() { sed -n '2,/^set -Eeuo/p' "$0" | sed '$d; s/^# \{0,1\}//'; }
while [ $# -gt 0 ]; do
  case "$1" in
    --sha) SHA="${2:?--sha needs a commit}"; shift ;;
    --snapshot) SNAPSHOT="${2:?--snapshot needs a URL or a file}"; shift ;;
    --snapshot-sha256) SNAPSHOT_SHA="${2:?--snapshot-sha256 needs a hex digest}"; shift ;;
    --no-snapshot) NO_SNAPSHOT=1 ;;
    --database-url) DATABASE_URL_NEW="${2:?--database-url needs a URL}"; shift ;;
    --webhook) WEBHOOK="${2:?--webhook needs a URL}"; shift ;;
    --only) ONLY="$ONLY,${2:?--only needs a step}"; shift ;;
    --skip) SKIP="$SKIP,${2:?--skip needs a step}"; shift ;;
    --dry-run) DRY_RUN=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "bootstrap: unknown argument $1 (--help lists them)" >&2; exit 2 ;;
  esac
  shift
done
for s in ${ONLY//,/ } ${SKIP//,/ }; do
  case " $STEPS " in *" $s "*) ;; *) echo "bootstrap: unknown step '$s' (steps: $STEPS)" >&2; exit 2 ;; esac
done
if [ -n "$SNAPSHOT" ] && [ "$NO_SNAPSHOT" = 1 ]; then echo "bootstrap: --snapshot and --no-snapshot exclude each other" >&2; exit 2; fi

# ---------------------------------------------------------------- helpers
say()  { printf '%s\n' "$*"; }
info() { printf '    %s\n' "$*"; }
warn() { printf '    WARNING: %s\n' "$*" >&2; WARNINGS+=("$STEP: $*"); }
# fail MSG [CODE]: stop with the step's name; under --dry-run report what would stop and go on, so the plan is whole.
fail() {
  if [ "$DRY_RUN" = 1 ]; then printf '    DRY-RUN: a real run would stop here: %s\n' "$1"; return 0; fi
  printf 'bootstrap: step "%s" failed: %s\n' "$STEP" "$1" >&2
  exit "${2:-1}"
}
on_err() {
  local code=$?
  printf 'bootstrap: step "%s" failed: line %s exited %s (the command is above); fix it and rerun, done steps are skipped\n' \
    "$STEP" "$1" "$code" >&2
  exit 1
}
trap 'on_err $LINENO' ERR
# run CMD...: print it, then run it (not under --dry-run).
run() {
  printf '  + '; printf '%q ' "$@"; printf '\n'
  [ "$DRY_RUN" = 1 ] || "$@"
}
# run_sh 'shell text': the same for a pipeline or a redirect.
run_sh() {
  printf '  + %s\n' "$1"
  [ "$DRY_RUN" = 1 ] || bash -c "$1"
}
selected() {
  local s="$1"
  if [ -n "$ONLY" ]; then case ",$ONLY," in *",$s,"*) ;; *) return 1 ;; esac; fi
  case ",$SKIP," in *",$s,"*) return 1 ;; esac
  return 0
}
have() { command -v "$1" >/dev/null 2>&1; }
sha256_of() { if have sha256sum; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi; }
mem_gb() {
  if [ -r /proc/meminfo ]; then awk '/^MemTotal:/ {printf "%d", $2 / 1048576}' /proc/meminfo
  elif have sysctl; then echo $(( $(sysctl -n hw.memsize 2>/dev/null || echo 0) / 1073741824 ))
  else echo 0; fi
}
free_gb() {  # free space at PATH or its nearest existing parent
  local p="$1"
  while [ ! -e "$p" ] && [ "$p" != / ]; do p="$(dirname "$p")"; done
  df -Pk "$p" | awk 'NR == 2 {printf "%d", $4 / 1048576}'
}
# wait_http URL SECONDS [WHAT]: until URL answers 2xx.
wait_http() {
  local url="$1" secs="$2" what="${3:-$1}" end
  printf '  + wait up to %ss for %s (%s)\n' "$secs" "$what" "$url"
  [ "$DRY_RUN" = 1 ] && return 0
  end=$(( $(date +%s) + secs ))
  until curl -fsS -m 10 -o /dev/null "$url" 2>/dev/null; do
    [ "$(date +%s)" -lt "$end" ] || return 1
    sleep 5
  done
}
# env_value FILE KEY: the value of KEY in a dotenv file (quotes stripped), empty when absent.
env_value() {
  [ -f "$1" ] || return 0
  python3 - "$1" "$2" <<'PY'
import sys
path, key = sys.argv[1:3]
val = ''
for line in open(path, encoding='utf-8'):
    s = line.strip()
    if s.startswith('export '):
        s = s[7:].lstrip()
    if s.startswith(key + '='):
        val = s[len(key) + 1:].strip()
        if len(val) >= 2 and val[0] == val[-1] and val[0] in '"\'':
            val = val[1:-1]
print(val)
PY
}
need_root() {
  [ "$DRY_RUN" = 1 ] && return 0
  [ "$(id -u)" = 0 ] || fail "run as root (sudo $0 ...): this step $1"
}
as_trevor() { run runuser -u trevor -- "$@"; }
begin() { STEP="$1"; say ""; say "== $1: $2"; }
done_already() { info "already done: $1"; }

# ---------------------------------------------------------------- 1. preflight
step_preflight() {
  begin preflight "OS, tools, memory, disk, the trevor user and directories"
  local os_id="" os_ver="" arch mem cores missing=() pkgs=() t g
  if [ -r /etc/os-release ]; then
    os_id="$(sed -n 's/^ID=//p' /etc/os-release | tr -d '"')"
    os_ver="$(sed -n 's/^VERSION_ID=//p' /etc/os-release | tr -d '"')"
  fi
  arch="$(uname -m)"
  info "host: $(uname -s) $arch, ${os_id:-unknown} ${os_ver:-}"
  [ "$(uname -s)" = Linux ] || fail "Linux only (this is $(uname -s)): the units are systemd and the bundle is a Linux build"
  [ "$arch" = x86_64 ] || fail "x86_64 only (this is $arch): the Release Bundle and the pinned llama.cpp archive are x86_64"
  if [ "$os_id" != ubuntu ] || [ "$os_ver" != 24.04 ]; then
    warn "built for Ubuntu 24.04, this is ${os_id:-unknown} ${os_ver:-}: the bins need its glibc and libstdc++"
  fi

  for t in curl tar zstd python3 rsync flock sha256sum openssl git runuser; do have "$t" || missing+=("$t"); done
  for t in ${missing[@]+"${missing[@]}"}; do
    case "$t" in flock|runuser) pkgs+=(util-linux) ;; sha256sum) pkgs+=(coreutils) ;; *) pkgs+=("$t") ;; esac
  done
  if [ ${#pkgs[@]} -gt 0 ]; then
    info "missing: ${missing[*]}"
    need_root "installs packages"
    run apt-get update
    run env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates "${pkgs[@]}"
  else
    info "tools: curl tar zstd python3 rsync flock sha256sum openssl git runuser present"
  fi
  for t in systemctl systemd-tmpfiles systemd-run; do have "$t" || fail "no $t: systemd is required (the units, tmpfiles, the first update)"; done

  mem="$(mem_gb)"; cores="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 0)"
  info "memory: $mem GB, cores: $cores"
  [ "$mem" -ge "$MEM_FAIL_GB" ] || fail "$mem GB of memory, under the $MEM_FAIL_GB GB floor (E2B, the embedder, the indexes, the backend and Postgres)"
  [ "$mem" -ge "$MEM_LLM_GB" ] || warn "under $MEM_LLM_GB GB: the 12B is not fetched; the model stages, when they come, run on E2B"
  [ "$mem" -ge "$MEM_WARN_GB" ] || warn "under the planned $MEM_WARN_GB GB (section 5): serving fits, the 12B beside it may not"
  [ "$cores" -ge "$CORES_WARN" ] || warn "$cores cores, under the planned $CORES_WARN: answers queue sooner"
  for t in "$ROOT" "$TREVOR_HOME"; do
    g="$(free_gb "$t")"
    info "disk: $g GB free at $t"
    [ "$g" -ge "$DISK_FAIL_GB" ] || fail "$g GB free at $t, under $DISK_FAIL_GB GB (models, snapshot, releases, stage)"
    [ "$g" -ge "$DISK_WARN_GB" ] || warn "$g GB free at $t, under $DISK_WARN_GB GB: the 12B, releases and logs will press on it"
  done
  [ "$ROOT" = "$UNITS_PATH" ] || warn "the clone is at $ROOT, not $UNITS_PATH: the units are installed with that path rewritten"
  [ -f "$BACKEND_ENV" ] || [ -n "$DATABASE_URL_NEW" ] || \
    warn "no $BACKEND_ENV: the env step needs it or --database-url (backend/README.md, Quick Start)"
  [ -f "$RES_VERSION_FILE" ] || warn "no $RES_VERSION_FILE yet: the asset watcher (assets/ecosystem.config.cjs) must extract EN before the first update"

  need_root "creates the trevor user and its directories"
  if id trevor >/dev/null 2>&1; then
    info "user trevor exists"
  else
    run useradd --system --home-dir "$TREVOR_HOME" --shell /usr/sbin/nologin trevor
  fi
  run install -d -o trevor -g trevor -m 755 "$TREVOR_HOME" "$STATE" "$TREVOR_HOME/spool" "$TREVOR_HOME/logs" "$TREVOR_HOME/snapshot"
  run install -d -o root -g root -m 700 /etc/trevor
  run install -m 644 "$UNITS_SRC/trevor.tmpfiles.conf" /etc/tmpfiles.d/trevor.conf
  run systemd-tmpfiles --create /etc/tmpfiles.d/trevor.conf
}

# ---------------------------------------------------------------- 2. release
step_release() {
  begin release "the CI-built binaries for the commit"
  local sha comps="trevor,assets" code=0 marker="$TREVOR_DIR/target/release/.release-installed"
  sha="$SHA"
  if [ -z "$sha" ]; then
    sha="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null)" \
      || fail "git cannot read the checkout at $ROOT (owned by another user? git config --global --add safe.directory $ROOT), or pass --sha"
  fi
  sha="$(git -C "$ROOT" rev-parse --verify --quiet "$sha^{commit}" 2>/dev/null || echo "$sha")"
  info "commit $sha (the bundle exists only for commits the Release Bundle workflow built on main)"
  [ -x "$ROOT/backend/target/release/backend" ] || comps="$comps,backend"
  if [ -f "$marker" ] && grep -qx "sha=$sha" "$marker" && [ -x "$ROOT/assets/binaries/unpacker" ] \
      && [ -x "$ROOT/assets/binaries/downloader" ] && [ -x "$ROOT/backend/target/release/backend" ]; then
    done_already "bundle ${sha:0:12} installed ($marker)"; return 0
  fi
  need_root "writes the binaries into the clone"
  printf '  + %q --components %s %s\n' "$ROOT/scripts/fetch-release.sh" "$comps" "$sha"
  [ "$DRY_RUN" = 1 ] && return 0
  bash "$ROOT/scripts/fetch-release.sh" --components "$comps" "$sha" || code=$?
  case "$code" in
    0) ;;
    3) fail "no release bundle for ${sha:0:12}. Bundles exist only for commits pushed to main and built by .github/workflows/release.yml (the newest 20 are kept). Check out such a commit, or pass --sha of one; this box never compiles Rust." 3 ;;
    *) fail "fetch-release.sh exited $code (download, checksum or layout; its message is above)" ;;
  esac
}

# ---------------------------------------------------------------- 3. llama
step_llama() {
  begin llama "llama.cpp at the pin in deploy/pins.env"
  # shellcheck source=pins.env
  . "$DEPLOY/pins.env"
  local dir="$LLAMA_ROOT/$LLAMA_CPP_TAG" tmp="$LLAMA_ROOT/.download" got bin
  [ -n "${LLAMA_CPP_SHA256:-}" ] || fail "LLAMA_CPP_SHA256 is empty in deploy/pins.env: fill it from the release's asset digest before the first run"
  info "pin: $LLAMA_CPP_TAG $LLAMA_CPP_ASSET sha256 ${LLAMA_CPP_SHA256:0:16}..."
  if [ -f "$dir/.verified" ] && [ "$(readlink "$LLAMA_ROOT/current" 2>/dev/null)" = "$LLAMA_CPP_TAG" ] \
      && [ -x "$LLAMA_BIN/llama-server" ] && "$LLAMA_BIN/llama-server" --version >/dev/null 2>&1; then
    done_already "$LLAMA_BIN/llama-server -> $LLAMA_ROOT/current -> $LLAMA_CPP_TAG"; return 0
  fi
  need_root "installs into $LLAMA_ROOT and $LLAMA_BIN"
  run install -d -m 755 "$LLAMA_ROOT" "$tmp"
  run curl -fL --retry 3 --max-time 900 -o "$tmp/$LLAMA_CPP_ASSET" "$LLAMA_CPP_BASE_URL/$LLAMA_CPP_TAG/$LLAMA_CPP_ASSET"
  printf '  + sha256 %s must equal the pin\n' "$tmp/$LLAMA_CPP_ASSET"
  if [ "$DRY_RUN" = 0 ]; then
    got="$(sha256_of "$tmp/$LLAMA_CPP_ASSET")"
    [ "$got" = "$LLAMA_CPP_SHA256" ] || { rm -f "$tmp/$LLAMA_CPP_ASSET"; fail "sha256 of $LLAMA_CPP_ASSET is $got, the pin says $LLAMA_CPP_SHA256"; }
  fi
  run rm -rf "$dir.new"
  run install -d -m 755 "$dir.new"
  run tar -xzf "$tmp/$LLAMA_CPP_ASSET" -C "$dir.new"
  # The archive's top directory has changed name across releases: take the directory that holds llama-server.
  printf '  + find the directory holding llama-server in %s, move it to %s\n' "$dir.new" "$dir"
  if [ "$DRY_RUN" = 0 ]; then
    bin="$(find "$dir.new" -type f -name llama-server | head -1)"
    [ -n "$bin" ] || fail "no llama-server in $LLAMA_CPP_ASSET"
    rm -rf "$dir"
    mv "$(dirname "$bin")" "$dir"
    rm -rf "$dir.new"
    sha256_of "$tmp/$LLAMA_CPP_ASSET" > "$dir/.verified"
    rm -f "$tmp/$LLAMA_CPP_ASSET"
  fi
  run ln -sfn "$LLAMA_CPP_TAG" "$LLAMA_ROOT/current"
  run ln -sfn "$LLAMA_ROOT/current/llama-server" "$LLAMA_BIN/llama-server"
  run ln -sfn "$LLAMA_ROOT/current/llama-bench" "$LLAMA_BIN/llama-bench"
  run "$LLAMA_BIN/llama-server" --version
}

# ---------------------------------------------------------------- 4. models
step_models() {
  begin models "the embedder, the reranker, E2B, and the 12B when memory allows"
  local mem llm marker="$STATE/bootstrap-models" want models="$TREVOR_DIR/models"
  mem="$(mem_gb)"
  if [ -n "${WITH_LLM:-}" ]; then llm="$WITH_LLM"
  elif [ "$mem" -ge "$MEM_LLM_GB" ]; then llm=1
  else llm=0; fi
  info "WITH_LLM=$llm ($mem GB of memory; the 12B and Qwen are fetched at $MEM_LLM_GB GB or more)"
  want="$(sha256_of "$TREVOR_DIR/scripts/fetch-model.sh") fp32=1 e2b=1 llm=$llm"
  if [ -f "$marker" ] && [ "$(cat "$marker")" = "$want" ] && [ -f "$models/llm/$E2B_GGUF_NAME" ]; then
    done_already "fetch-model.sh ran with these flags ($marker)"; return 0
  fi
  need_root "writes into $models"
  run env WITH_FP32=1 WITH_E2B=1 WITH_LLM="$llm" nice -n 10 bash "$TREVOR_DIR/scripts/fetch-model.sh" "$models"
  run_sh "printf '%s\\n' '$want' > '$marker'"
}

# ---------------------------------------------------------------- 5. snapshot
step_snapshot() {
  begin snapshot "the corpus snapshot into trevor/artifacts"
  local live="$TREVOR_DIR/artifacts" file got
  if [ -L "$live" ]; then done_already "$live is the release link (adopted by trevor-vps-update.sh init)"; return 0; fi
  if [ -f "$live/snapshot.json" ]; then done_already "$live holds an installed snapshot ($live/snapshot.json)"; return 0; fi
  if [ "$NO_SNAPSHOT" = 1 ]; then
    warn "--no-snapshot: the first update builds every derived table from nothing. On a CPU that is weeks (design section 6 step 3), and until the model stages finish, answers lack summaries, dossiers, topics and the bank."
    [ -e "$live" ] && { done_already "$live exists (no snapshot asked)"; return 0; }
    run install -d -o trevor -g trevor -m 755 "$live"
    return 0
  fi
  [ -n "$SNAPSHOT" ] || fail "no --snapshot URL or file. A fresh box starts from one (snapshot.py pack on the build machine, after the fp32 refresh); --no-snapshot starts empty, which is weeks of CPU"
  [ -n "$SNAPSHOT" ] || SNAPSHOT=SNAPSHOT.tar.zst   # only reached under --dry-run: a placeholder for the plan
  need_root "writes trevor/artifacts"
  case "$SNAPSHOT" in
    http://*|https://*)
      file="$TREVOR_HOME/snapshot/$(basename "${SNAPSHOT%%\?*}")"
      if [ -f "$file" ] && [ -n "$SNAPSHOT_SHA" ] && [ "$(sha256_of "$file")" = "$SNAPSHOT_SHA" ]; then
        info "already downloaded: $file"
      else
        run curl -fL --retry 3 --max-time 3600 -o "$file.part" "$SNAPSHOT"
        run mv -f "$file.part" "$file"
      fi ;;
    *) file="$SNAPSHOT"
       [ -f "$file" ] || fail "no snapshot file at $file" ;;
  esac
  if [ -n "$SNAPSHOT_SHA" ]; then
    printf '  + sha256 %s must equal --snapshot-sha256\n' "$file"
    if [ "$DRY_RUN" = 0 ]; then
      got="$(sha256_of "$file")"
      [ "$got" = "$SNAPSHOT_SHA" ] || fail "sha256 of $file is $got, --snapshot-sha256 says $SNAPSHOT_SHA"
    fi
  fi
  [ -e "$live" ] && fail "$live exists but holds no snapshot.json: move it aside and rerun, or --skip snapshot to adopt that tree as it is"
  run nice -n 19 python3 "$TREVOR_DIR/scripts/snapshot.py" install "$file" --into "$live"
}

# ---------------------------------------------------------------- 6. env
step_env() {
  begin env "$ENV_FILE and the backend's TREVOR_WORKER_KEY"
  local key="" backend_key="" hook="$WEBHOOK"
  if [ -z "$DESTDIR" ]; then need_root "writes /etc/trevor and backend/.env"; fi
  if [ "$DRY_RUN" = 1 ]; then
    info "would keep an existing TREVOR_WORKER_KEY (from $ENV_FILE, else $BACKEND_ENV), else generate one (python3 secrets, 32 bytes hex)"
    printf '  + write %s (mode 600) from %s: paths under %s, TREVOR_RETRIEVAL=bm25, EMBED=0, TREVOR_CF_* commented out\n' \
      "$ENV_FILE" "$DEPLOY/trevor.env.example" "$ROOT"
    if [ -f "$BACKEND_ENV" ]; then printf '  + set TREVOR_WORKER_KEY in %s (the same value)\n' "$BACKEND_ENV"
    else printf '  + create %s from backend/.env.example with --database-url and fresh JWT_SECRET, SERVICE_KEY, GAME_CREDENTIAL_KEY, plus TREVOR_WORKER_KEY\n' "$BACKEND_ENV"; fi
    [ -f "$BACKEND_ENV" ] || [ -n "$DATABASE_URL_NEW" ] || fail "no $BACKEND_ENV and no --database-url"
    return 0
  fi
  key="$(env_value "$ENV_FILE" TREVOR_WORKER_KEY)"
  backend_key="$(env_value "$BACKEND_ENV" TREVOR_WORKER_KEY)"
  if [ -n "$key" ] && [ "$key" = "$backend_key" ] && { [ -z "$WEBHOOK" ] || [ "$(env_value "$ENV_FILE" DISCORD_ALERT_WEBHOOK)" = "$WEBHOOK" ]; }; then
    done_already "$ENV_FILE and $BACKEND_ENV hold the same TREVOR_WORKER_KEY (edit $ENV_FILE by hand from here on)"; return 0
  fi
  [ -n "$key" ] || key="$backend_key"
  [ -n "$key" ] || key="$(python3 -c 'import secrets; print(secrets.token_hex(32))')"
  [ -n "$hook" ] || hook="$(env_value "$ENV_FILE" DISCORD_ALERT_WEBHOOK)"
  [ -f "$BACKEND_ENV" ] || [ -n "$DATABASE_URL_NEW" ] || \
    fail "no $BACKEND_ENV: the backend's own secrets come first (DATABASE_URL, JWT_SECRET, SERVICE_KEY, GAME_CREDENTIAL_KEY; backend/README.md, Quick Start), or pass --database-url to create it"
  mkdir -p "$ETC_DIR"
  [ -n "$DESTDIR" ] || chmod 700 "$ETC_DIR"
  printf '  + write %s (mode 600) from deploy/trevor.env.example\n' "$ENV_FILE"
  KEY="$key" HOOK="$hook" ROOT_DIR="$ROOT" TREVOR_HOME="$TREVOR_HOME" STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    python3 - "$DEPLOY/trevor.env.example" "$ENV_FILE" <<'PY'
import os, sys
src, dst = sys.argv[1:3]
root, home = os.environ['ROOT_DIR'], os.environ['TREVOR_HOME']
set_to = {
    'TREVOR_WORKER_KEY': os.environ['KEY'],
    'DISCORD_ALERT_WEBHOOK': os.environ['HOOK'],
    'TREVOR_DIR': f'{root}/trevor',
    'TREVOR_HOME': home,
    'ASSETS_DIR': f'{root}/assets',
    'TREVOR_SPOOL_DIR': f'{home}/spool',
    'TREVOR_CANARY_FILE': f'{home}/state/cf-canary.json',
    'TREVOR_RETRIEVAL': 'bm25',
    'EMBED': '0',
    'E2B_GGUF': f'{root}/trevor/models/llm/gemma-4-e2b-it-q4_k_m.gguf',
    'LLAMA_SERVER': '/usr/local/bin/llama-server',
}
out = [
    f'# Generated by trevor/deploy/bootstrap.sh at {os.environ["STAMP"]} from deploy/trevor.env.example.',
    '# A rerun of the env step leaves this file alone while it and backend/.env hold the same TREVOR_WORKER_KEY.',
]
seen, cf_noted = set(), False
for line in open(src, encoding='utf-8').read().splitlines():
    key = line.split('=', 1)[0] if '=' in line and not line.startswith('#') else None
    if key and key.startswith('TREVOR_CF_'):
        if not cf_noted:
            out.append('# NOT USED IN THIS DEPLOYMENT: the self-contained shape serves the cache and cpu rungs only, with no model')
            out.append('# vendor (design/trevor-self-contained-deploy.md). Leave every TREVOR_CF_* unset: the worker then reports')
            out.append('# cf:false and admission goes cpu, deferred cpu, passages.')
            cf_noted = True
        out.append('#' + line)
        continue
    if key in set_to:
        out.append(f'{key}={set_to[key]}')
        seen.add(key)
        continue
    out.append(line)
for key in sorted(set(set_to) - seen):
    out.append(f'{key}={set_to[key]}')
tmp = dst + '.tmp'
fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
with os.fdopen(fd, 'w', encoding='utf-8') as f:
    f.write('\n'.join(out) + '\n')
os.replace(tmp, dst)
PY
  [ -n "$DESTDIR" ] || chown root:root "$ENV_FILE"
  check_env_file "$ENV_FILE"

  printf '  + set TREVOR_WORKER_KEY in %s\n' "$BACKEND_ENV"
  mkdir -p "$(dirname "$BACKEND_ENV")"
  KEY="$key" DBURL="$DATABASE_URL_NEW" python3 - "$BACKEND_ENV" "$ROOT/backend/.env.example" <<'PY'
import os, secrets, sys
path, example = sys.argv[1:3]
key = os.environ['KEY']
if os.path.exists(path):
    lines = open(path, encoding='utf-8').read().splitlines()
    st = os.stat(path)
else:
    # A fresh box: the backend's required secrets, generated; everything else as backend/.env.example has it.
    fresh = {
        'DATABASE_URL': '"' + os.environ['DBURL'] + '"',
        'JWT_SECRET': '"' + secrets.token_hex(32) + '"',
        'SERVICE_KEY': '"' + secrets.token_hex(32) + '"',
        'GAME_CREDENTIAL_KEY': '"' + secrets.token_hex(32) + '"',
    }
    lines = []
    for line in open(example, encoding='utf-8').read().splitlines():
        k = line.split('=', 1)[0] if '=' in line and not line.startswith('#') else None
        lines.append(f'{k}={fresh[k]}' if k in fresh else line)
    st = None
done = False
for i, line in enumerate(lines):
    s = line.strip()
    if s.startswith('TREVOR_WORKER_KEY=') or s.startswith('export TREVOR_WORKER_KEY='):
        lines[i] = f'TREVOR_WORKER_KEY={key}'
        done = True
if not done:
    lines += ['', '# Trevor worker key (trevor/deploy/bootstrap.sh); the same value as /etc/trevor/trevor.env.',
              f'TREVOR_WORKER_KEY={key}']
tmp = path + '.tmp'
fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
with os.fdopen(fd, 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines) + '\n')
if st is not None:
    os.chmod(tmp, st.st_mode & 0o777)
    if os.geteuid() == 0:
        os.chown(tmp, st.st_uid, st.st_gid)
os.replace(tmp, path)
PY
  [ "$(env_value "$BACKEND_ENV" TREVOR_WORKER_KEY)" = "$key" ] || fail "TREVOR_WORKER_KEY did not land in $BACKEND_ENV"
  info "TREVOR_WORKER_KEY: the same value in both files (${#key} characters)"
}

# check_env_file FILE: every required key set, no Cloudflare key set.
check_env_file() {
  python3 - "$1" <<'PY'
import sys
req = ['TREVOR_WORKER_KEY', 'TREVOR_DIR', 'TREVOR_HOME', 'ASSETS_DIR', 'TREVOR_BACKEND', 'TREVOR_LLM_URL',
       'TREVOR_LLM_LOCK', 'TREVOR_SPOOL_DIR', 'TREVOR_RETRIEVAL', 'EMBED', 'EMBED_ONNX', 'E2B_GGUF', 'LLAMA_SERVER']
vals, bad = {}, []
for line in open(sys.argv[1], encoding='utf-8'):
    s = line.strip()
    if s and not s.startswith('#') and '=' in s:
        k, v = s.split('=', 1)
        vals[k] = v
bad += [f'{k} is empty or missing' for k in req if not vals.get(k)]
bad += [f'{k} is set' for k in vals if k.startswith('TREVOR_CF_')]
if bad:
    sys.exit('env file check failed: ' + '; '.join(bad))
print(f'    env file check: {len(req)} required keys set, no TREVOR_CF_* set')
PY
}

# ---------------------------------------------------------------- 7. database
step_database() {
  begin database "the backend's migrations (applied by the backend on startup) and the worker key"
  local probe="$BACKEND_URL/api/trevor/worker/popular?version=bootstrap-probe" code key
  # The backend runs every pending migration in database::init before it serves (backend/README.md, Database); there is
  # no separate migrate command. It reads backend/.env at startup, so a new TREVOR_WORKER_KEY needs a restart too.
  if [ "$DRY_RUN" = 0 ]; then
    key="$(env_value "$BACKEND_ENV" TREVOR_WORKER_KEY)"
    code="$(curl -s -o /dev/null -w '%{http_code}' -m 10 -H "x-trevor-worker-key: $key" "$probe" 2>/dev/null || true)"
    case "$code" in 200|404) done_already "the backend answers and accepts the worker key (HTTP $code on worker/popular)"; return 0 ;; esac
    info "backend probe: HTTP ${code:-none} (401 = an old key, 503 = no key, 000 = not running)"
  fi
  need_root "restarts the backend"
  have pm2 || [ "$DRY_RUN" = 1 ] || fail "no pm2: the site runs the backend under pm2 (backend/ecosystem.config.cjs). Install node and pm2, or start $ROOT/backend/target/release/backend from $ROOT/backend under your own supervisor, then rerun with --skip database"
  if [ "$DRY_RUN" = 0 ] && pm2 describe myrtle-backend >/dev/null 2>&1; then
    run pm2 restart myrtle-backend --update-env
  elif [ "$DRY_RUN" = 0 ] && curl -fsS -m 10 -o /dev/null "$BACKEND_URL/api/health" 2>/dev/null; then
    # A backend answers but root's pm2 does not know it (another user's pm2, or another supervisor): starting a second
    # one would fight it for the port.
    fail "a backend answers on $BACKEND_URL but root's pm2 has no myrtle-backend: restart that backend yourself so it reads $BACKEND_ENV (TREVOR_WORKER_KEY), then rerun"
  else
    run pm2 start "$ROOT/backend/ecosystem.config.cjs"
    run pm2 save
  fi
  # Startup loads the game data first (tens of seconds, minutes on a cold disk), then migrates, then serves.
  wait_http "$BACKEND_URL/api/health" 900 "the backend to migrate and serve" \
    || fail "the backend did not answer /api/health in 15 minutes: pm2 logs myrtle-backend --lines 100 (DATABASE_URL, the game data under ASSETS_DIR)"
  printf '  + GET %s with the worker key: 200 or 404 = accepted\n' "$probe"
  [ "$DRY_RUN" = 1 ] && return 0
  code="$(curl -s -o /dev/null -w '%{http_code}' -m 10 -H "x-trevor-worker-key: $key" "$probe" || true)"
  case "$code" in
    200|404) info "worker key accepted (HTTP $code); migrations applied at startup" ;;
    *) fail "the backend refuses the worker key (HTTP $code): check TREVOR_WORKER_KEY in $BACKEND_ENV and that pm2 runs $ROOT/backend" ;;
  esac
}

# ---------------------------------------------------------------- 8. units
UNITS="trevor-llama.service trevor-worker.service trevor-worker-restart.path trevor-worker-restart.service trevor-update.service trevor-update.timer"
step_units() {
  begin units "systemd units and the update timer (not trevor-summaries: the Cloudflare summaries path)"
  local u changed=0 src
  need_root "installs into $UNIT_DIR"
  for u in $UNITS; do
    src="$UNITS_SRC/$u"
    if [ "$ROOT" = "$UNITS_PATH" ]; then
      if [ -f "$UNIT_DIR/$u" ] && cmp -s "$src" "$UNIT_DIR/$u"; then info "$u unchanged"; continue; fi
      run install -m 644 "$src" "$UNIT_DIR/$u"
    else
      if [ -f "$UNIT_DIR/$u" ] && cmp -s <(sed "s#$UNITS_PATH#$ROOT#g" "$src") "$UNIT_DIR/$u"; then info "$u unchanged"; continue; fi
      run_sh "sed 's#$UNITS_PATH#$ROOT#g' '$src' > '$UNIT_DIR/$u' && chmod 644 '$UNIT_DIR/$u'"
    fi
    changed=1
  done
  [ "$changed" = 0 ] || run systemctl daemon-reload
  run systemctl enable --now trevor-llama.service
  wait_http "$LLAMA_URL/health" 600 "llama-server to load E2B" \
    || fail "llama-server did not answer $LLAMA_URL/health in 10 minutes: journalctl -u trevor-llama -n 50"
  run systemctl enable --now trevor-worker-restart.path trevor-worker.service
  run systemctl enable --now trevor-update.timer
  info "trevor-summaries.timer is not enabled: it spends Cloudflare's daily Neurons, which this deployment does not use"
}

# ---------------------------------------------------------------- 9. first update
step_first_update() {
  begin first-update "adopt the artifacts as the first release, then one daily update"
  local live="$TREVOR_DIR/artifacts" logdir
  need_root "moves trevor/artifacts into /var/lib/trevor/releases"
  if [ -L "$live" ]; then
    info "already adopted: $live -> $(readlink "$live")"
  else
    [ -d "$live" ] || [ "$DRY_RUN" = 1 ] || fail "no $live to adopt (the snapshot step installs it)"
    run env TREVOR_HOME="$TREVOR_HOME" "$TREVOR_DIR/scripts/trevor-vps-update.sh" init
    run chown -R trevor:trevor "$TREVOR_HOME"
    run systemctl try-restart trevor-worker.service
  fi
  if [ -f "$STATE/active.json" ]; then
    done_already "a version is active ($STATE/active.json)"; return 0
  fi
  [ -f "$RES_VERSION_FILE" ] || fail "no $RES_VERSION_FILE: the asset watcher has not extracted EN yet (pm2 start $ROOT/assets/ecosystem.config.cjs, then rerun --only first-update)"
  info "the first daily update runs now, in the foreground (minutes to an hour; the gates' canary asks E2B 10 questions)"
  info "log: $TREVOR_HOME/logs/<UTC stamp>-daily/run.log (the newest *-daily directory)"
  run systemd-run --unit=trevor-bootstrap-daily --uid=trevor --gid=trevor --wait --pipe --collect \
    -p EnvironmentFile="$ENV_FILE" -p Nice=19 -p IOSchedulingClass=idle --setenv=FORCE=1 \
    "$TREVOR_DIR/scripts/trevor-vps-update.sh" daily
  if [ "$DRY_RUN" = 0 ]; then
    logdir="$(find "$TREVOR_HOME/logs" -mindepth 1 -maxdepth 1 -type d -name '*-daily' | sort | tail -1)"
    info "log: ${logdir:-?}/run.log"
    [ -f "$STATE/active.json" ] || fail "the daily update ended without an active version (NO_PUBLISH or a held gate): read ${logdir:-$TREVOR_HOME/logs}/run.log"
  fi
}

# ---------------------------------------------------------------- 10. verify
step_verify() {
  begin verify "the worker heartbeat, a route-only question, the summary"
  local status="" end online="?" cpu="?" cf="?" version="?" route=""
  printf '  + GET %s until online and the cpu rung is up (3 minutes)\n' "$BACKEND_URL/api/trevor/status"
  if [ "$DRY_RUN" = 0 ]; then
    end=$(( $(date +%s) + 180 ))
    while :; do
      status="$(curl -fsS -m 10 "$BACKEND_URL/api/trevor/status" 2>/dev/null || true)"
      [ -n "$status" ] || status='{}'
      read -r online cpu cf version < <(printf '%s' "$status" | python3 -c 'import json,sys
try: d = json.load(sys.stdin)
except Exception: d = {}
r = d.get("rungs") or {}
print(str(d.get("online", "?")).lower(), str(r.get("cpu", "?")).lower(), str(r.get("cf", "?")).lower(), d.get("activeVersion") or "none")')
      [ "$online" = true ] && [ "$cpu" = true ] && break
      [ "$(date +%s)" -lt "$end" ] || break
      sleep 10
    done
    if [ "$online" != true ] || [ "$cpu" != true ]; then fail "no worker heartbeat with the cpu rung in 3 minutes (status: ${status:-unreachable}); journalctl -u trevor-worker -n 50"; fi
    [ "$cf" != true ] || warn "the worker reports the cf rung: a TREVOR_CF_* key is set in $ENV_FILE"
  fi
  printf '  + cd %s && runuser -u trevor -- target/release/ask --route-only %q\n' "$TREVOR_DIR" "$ROUTE_QUESTION"
  if [ "$DRY_RUN" = 0 ]; then
    route="$(cd "$TREVOR_DIR" && runuser -u trevor -- target/release/ask --route-only "$ROUTE_QUESTION" 2>&1 | head -5)" \
      || fail "ask --route-only failed: $route"
  fi
  say ""
  say "==================================================================== Trevor is up"
  say "  backend   $BACKEND_URL  status: online=$online cpu=$cpu cf=$cf (cf stays off: no vendor)"
  say "  version   ${version}   (trevor-vps-update.sh status; /api/trevor/status activeVersion)"
  say "  route     $ROUTE_QUESTION -> $(printf '%s' "${route:-(dry run)}" | head -1)"
  say "  serving   llama-server $LLAMA_URL (E2B, the cpu rung), BM25 retrieval; hybrid needs fp32 vectors (deploy/README.md, Embeddings)"
  say "  updates   trevor-update.timer, daily 19:30 UTC; logs $TREVOR_HOME/logs/<stamp>-daily/run.log"
  say "  secrets   $ENV_FILE (TREVOR_WORKER_KEY, DISCORD_ALERT_WEBHOOK); the same key in $BACKEND_ENV"
  say "  watch     journalctl -u trevor-worker -u trevor-llama -u trevor-update --since today"
  say "  read next trevor/design/trevor-vps-runbook.md (failures, rollback, monitoring); deploy/README.md"
  say "  rerun     $0 [--only STEP] is safe: every step checks its own done-state"
}

# ---------------------------------------------------------------- main
[ "$DRY_RUN" = 1 ] && say "DRY RUN: nothing is changed; every command is printed. Repo $ROOT."
for s in $STEPS; do
  selected "$s" || { say ""; say "== $s: skipped"; continue; }
  "step_${s//-/_}"
done
STEP="end"
say ""
if [ ${#WARNINGS[@]} -gt 0 ]; then
  say "warnings:"; for w in "${WARNINGS[@]}"; do say "  - $w"; done
fi
say "bootstrap: done ($( [ "$DRY_RUN" = 1 ] && echo 'dry run' || echo 'all selected steps'))"
