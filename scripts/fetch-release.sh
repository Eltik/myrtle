#!/usr/bin/env bash
# Install the CI-built bundle for one commit instead of compiling on the box.
#
#   bash scripts/fetch-release.sh                    # bundle for the checked-out HEAD
#   bash scripts/fetch-release.sh <sha>              # bundle for that commit
#   bash scripts/fetch-release.sh --components backend,frontend [<sha>]
#   bash scripts/fetch-release.sh --check [<sha>]    # only ask whether the bundle exists
#   bash scripts/fetch-release.sh --wait 1500 [<sha>]  # poll up to 1500 s for CI to publish it
#
# The bundle is published by .github/workflows/release.yml as the assets of
# the prerelease `build-<full sha>`: myrtle-<sha>.tar.gz plus its .sha256. The
# repo is public, so this needs curl and nothing else: no token, no gh.
#
# Components and where they land (the paths vps-update.sh and the pm2
# ecosystem files already use):
#   backend   backend/target/release/backend
#   discord   discord/target/release/discord, discord/target/release/commands
#   assets    assets/binaries/downloader, assets/binaries/unpacker
#   frontend  frontend/.output
#
# Exit codes, so a caller can fall back to building:
#   0  installed (or, with --check, the bundle exists)
#   3  no bundle published for this commit (yet): build locally
#   1  anything else: download, checksum, layout or install failed
#   2  usage
# On any non-zero exit nothing installed is touched, except that a failure
# part-way through the INSTALL step can leave some components new and the rest
# old; every component is swapped by rename, so none is ever half written.
#
# The caller restarts what changed, at once. A running binary keeps its old
# inode and is unaffected by the swap, but a running FRONTEND is not: it loads
# hashed chunks from .output lazily and serves .output/public by path, so
# between the swap and `pm2 restart myrtle-frontend` it can fail a lazy import
# or serve the new build's assets. That window existed before too (vite build
# rewrote .output in place for a minute); here it is the seconds until restart.
#
# Binaries are installed only when their bytes differ, and a changed binary
# gets the current time as its mtime, which is what a local cargo build gives
# it: the asset watchers re-extract when the unpacker is newer than their last
# extract (run.mjs unpackerIsNewer), so an unchanged unpacker must keep its
# old mtime and a changed one must not carry CI's older build time. Each is
# staged beside its destination and renamed into place, so a running process
# keeps its old inode (no ETXTBSY).
#
# frontend/.output is swapped as a directory; the previous build is kept as
# frontend/.output.prev for a hand rollback:
#   mv frontend/.output frontend/.output.bad && mv frontend/.output.prev frontend/.output
#
# Env overrides:
#   MYRTLE_RELEASE_REPO      owner/repo (default Eltik/myrtle)
#   MYRTLE_RELEASE_BASE_URL  download base, default https://github.com/$REPO/releases/download
#                            (the bundle is fetched from $BASE/build-<sha>/myrtle-<sha>.tar.gz)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${MYRTLE_RELEASE_REPO:-Eltik/myrtle}"
BASE_URL="${MYRTLE_RELEASE_BASE_URL:-https://github.com/$REPO/releases/download}"
ALL_COMPONENTS="backend,discord,assets,frontend"

COMPONENTS="$ALL_COMPONENTS"
CHECK_ONLY=0
WAIT_S=0
SHA=""
while [ $# -gt 0 ]; do
    case "$1" in
        --components) COMPONENTS="${2:?--components needs a list}"; shift ;;
        --components=*) COMPONENTS="${1#*=}" ;;
        --check) CHECK_ONLY=1 ;;
        --wait) WAIT_S="${2:?--wait needs seconds}"; shift ;;
        --wait=*) WAIT_S="${1#*=}" ;;
        -h|--help) sed -n '2,52p' "$0"; exit 0 ;;
        -*) echo "fetch-release: unknown option $1" >&2; exit 2 ;;
        *) [ -z "$SHA" ] || { echo "fetch-release: one sha only" >&2; exit 2; }; SHA="$1" ;;
    esac
    shift
done

if [ -z "$SHA" ]; then
    SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null)" || { echo "fetch-release: no sha given and $ROOT is not a git checkout" >&2; exit 2; }
fi
# A short sha would name a release that does not exist: require the full one.
if ! printf '%s' "$SHA" | grep -Eq '^[0-9a-f]{40}$'; then
    resolved="$(git -C "$ROOT" rev-parse --verify --quiet "$SHA^{commit}" 2>/dev/null || true)"
    [ -n "$resolved" ] || { echo "fetch-release: '$SHA' is not a full commit sha and does not resolve in this checkout" >&2; exit 2; }
    SHA="$resolved"
fi
case "$WAIT_S" in ''|*[!0-9]*) echo "fetch-release: --wait takes whole seconds" >&2; exit 2 ;; esac

for c in ${COMPONENTS//,/ }; do
    case ",$ALL_COMPONENTS," in *",$c,"*) ;; *) echo "fetch-release: unknown component '$c' (known: $ALL_COMPONENTS)" >&2; exit 2 ;; esac
done
want() { case ",$COMPONENTS," in *",$1,"*) return 0 ;; *) return 1 ;; esac; }

NAME="myrtle-$SHA"
URL="$BASE_URL/build-$SHA/$NAME.tar.gz"

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
    else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# HTTP status of the checksum file: 200 = published, 404 = not (yet).
probe() {
    local c
    c="$(curl -sSL -o /dev/null -w '%{http_code}' --max-time 20 "$URL.sha256" 2>/dev/null)" || true
    printf '%s' "${c:-000}"
}

# State of the newest Release Bundle run for this commit, from the public API
# (unauthenticated: 60 requests an hour per IP, so it is asked sparingly):
# "none", "queued", "in_progress", "success", "failure", ... or "unknown".
run_state() {
    local json status conclusion http
    json="$(curl -sS --max-time 20 -H 'Accept: application/vnd.github+json' -w '\n%{http_code}' \
        "https://api.github.com/repos/$REPO/actions/workflows/release.yml/runs?head_sha=$SHA&per_page=1" 2>/dev/null)" || { echo unknown; return; }
    http="${json##*$'\n'}"
    json="${json%$'\n'*}"
    # 404: the workflow does not exist on the default branch, so no run will come.
    if [ "$http" = "404" ]; then echo none; return; fi
    if [ "$http" != "200" ]; then echo unknown; return; fi
    if printf '%s' "$json" | grep -Eq '"total_count": *0[,}]'; then echo none; return; fi
    status="$(printf '%s' "$json" | grep -Eo '"status": *"[a-z_]+"' | head -1 | sed -E 's/.*"([a-z_]+)"$/\1/')"
    conclusion="$(printf '%s' "$json" | grep -Eo '"conclusion": *"[a-z_]+"' | head -1 | sed -E 's/.*"([a-z_]+)"$/\1/')"
    if [ "$status" = "completed" ]; then echo "${conclusion:-unknown}"; else echo "${status:-unknown}"; fi
}

code="$(probe)"
if [ "$code" = "404" ] && [ "$WAIT_S" -gt 0 ]; then
    # Only wait for a bundle that is actually being built: a commit no run
    # covers (not on main, or the workflow did not trigger) and a run that
    # already failed both mean "build locally", now rather than after the wait.
    #
    # `success` with a 404 means the bundle existed and was pruned (the workflow
    # keeps the newest 20), which is the usual case for a rollback.
    #
    # `none` gets a 90 s grace: a deploy started right after the push can ask
    # before GitHub has created the run.
    state="$(run_state)"
    grace_end=$(( $(date +%s) + 90 ))
    while [ "$state" = "none" ] && [ "$(date +%s)" -lt "$grace_end" ]; do
        sleep 30
        code="$(probe)"
        [ "$code" = "404" ] || break
        state="$(run_state)"
    done
    if [ "$code" = "404" ]; then
        case "$state" in
            none|success|failure|cancelled|timed_out|startup_failure|skipped|neutral|action_required|stale)
                echo "fetch-release: no bundle coming for ${SHA:0:12} (Release Bundle run: $state)" >&2
                exit 3 ;;
        esac
    fi
    echo "==> waiting up to ${WAIT_S}s for CI to publish build-${SHA:0:12} (run: $state)"
    deadline=$(( $(date +%s) + WAIT_S ))
    polls=0
    while [ "$code" = "404" ] && [ "$(date +%s)" -lt "$deadline" ]; do
        sleep 30
        code="$(probe)"
        polls=$((polls + 1))
        # Re-ask the API every 5 minutes, not every poll.
        if [ "$code" = "404" ] && [ $((polls % 10)) -eq 0 ]; then
            state="$(run_state)"
            case "$state" in
                success|failure|cancelled|timed_out|startup_failure|skipped|neutral|action_required|stale)
                    echo "fetch-release: Release Bundle run for ${SHA:0:12} ended: $state" >&2
                    exit 3 ;;
            esac
        fi
    done
fi
case "$code" in
    200) ;;
    404) echo "fetch-release: no bundle published for ${SHA:0:12} ($URL)" >&2; exit 3 ;;
    *)   echo "fetch-release: could not reach $URL.sha256 (HTTP $code)" >&2; exit 1 ;;
esac
if [ "$CHECK_ONLY" -eq 1 ]; then echo "bundle available: $URL"; exit 0; fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/myrtle-release.XXXXXX")"
STAGED=()
cleanup() {
    for s in ${STAGED[@]+"${STAGED[@]}"}; do rm -rf "$s"; done
    rm -rf "$TMP"
}
trap cleanup EXIT

echo "==> downloading $NAME.tar.gz"
curl -fsSL --retry 3 --retry-delay 5 --max-time 600 -o "$TMP/$NAME.tar.gz.sha256" "$URL.sha256"
curl -fsSL --retry 3 --retry-delay 5 --max-time 900 -o "$TMP/$NAME.tar.gz" "$URL"

expected="$(awk '{print $1}' "$TMP/$NAME.tar.gz.sha256")"
actual="$(sha256_of "$TMP/$NAME.tar.gz")"
if [ -z "$expected" ] || [ "$expected" != "$actual" ]; then
    echo "fetch-release: checksum mismatch for $NAME.tar.gz (expected ${expected:-nothing}, got $actual)" >&2
    exit 1
fi
echo "    sha256 ok ($actual)"

tar -C "$TMP" -xzf "$TMP/$NAME.tar.gz"
B="$TMP/$NAME"
[ -d "$B" ] || { echo "fetch-release: the archive does not hold $NAME/, so it is not this commit's bundle" >&2; exit 1; }
[ -f "$B/BUILD_INFO" ] || { echo "fetch-release: bundle has no BUILD_INFO" >&2; exit 1; }
grep -qx "sha=$SHA" "$B/BUILD_INFO" || { echo "fetch-release: bundle BUILD_INFO names a different commit" >&2; exit 1; }
# Per-file checksums, so a truncated or tampered entry fails here rather than at runtime.
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$B" && sha256sum -c SHA256SUMS >/dev/null) || { echo "fetch-release: per-file checksum failed" >&2; exit 1; }
else
    (cd "$B" && shasum -a 256 -c SHA256SUMS >/dev/null) || { echo "fetch-release: per-file checksum failed" >&2; exit 1; }
fi

BINS=()
want backend && BINS+=("backend/target/release/backend")
want discord && BINS+=("discord/target/release/discord" "discord/target/release/commands")
want assets && BINS+=("assets/binaries/downloader" "assets/binaries/unpacker")
for f in ${BINS[@]+"${BINS[@]}"}; do
    [ -x "$B/$f" ] || { echo "fetch-release: bundle is missing $f" >&2; exit 1; }
done
if want frontend; then
    [ -f "$B/frontend/.output/server/index.mjs" ] || { echo "fetch-release: bundle is missing frontend/.output" >&2; exit 1; }
fi
sed 's/^/    /' "$B/BUILD_INFO"

# --- stage everything beside its destination, then swap -----------------------
# All copies happen before the first rename, so a full disk or a permission
# error stops the run with every running build still in place.
CHANGED=()
for f in ${BINS[@]+"${BINS[@]}"}; do
    dst="$ROOT/$f"
    mkdir -p "$(dirname "$dst")"
    if [ -f "$dst" ] && cmp -s "$B/$f" "$dst"; then
        echo "    $f unchanged (mtime preserved)"
        continue
    fi
    tmp="$dst.new.$$"
    STAGED+=("$tmp")
    cp "$B/$f" "$tmp"
    chmod 755 "$tmp"
    touch "$tmp"
    CHANGED+=("$f")
done
FRONT_NEW=""
if want frontend; then
    FRONT_NEW="$ROOT/frontend/.output.new.$$"
    mkdir -p "$ROOT/frontend"
    STAGED+=("$FRONT_NEW")
    cp -R "$B/frontend/.output" "$FRONT_NEW"
fi

for f in ${CHANGED[@]+"${CHANGED[@]}"}; do
    mv -f "$ROOT/$f.new.$$" "$ROOT/$f"
    echo "    installed $f"
done
if [ -n "$FRONT_NEW" ]; then
    out="$ROOT/frontend/.output"
    note=""
    if [ -d "$out" ]; then
        rm -rf "$out.prev"
        mv "$out" "$out.prev"
        note=" (previous kept as frontend/.output.prev)"
    fi
    mv "$FRONT_NEW" "$out"
    echo "    installed frontend/.output$note"
fi

printf 'sha=%s\ninstalled_at=%s\ncomponents=%s\n' "$SHA" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$COMPONENTS" > "$ROOT/scripts/.release-installed"
echo "==> installed bundle ${SHA:0:12} ($COMPONENTS)"
