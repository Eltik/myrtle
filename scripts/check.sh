#!/usr/bin/env bash
# Run a toolchain and return ITS exit code.
#
# WHY THIS EXISTS. `npx tsc --noEmit | tail && echo OK` reports TAIL's status, not the compiler's,
# so a failing check reads as a passing one. That has now happened twice in this repo: seven
# commits' type-checks were never real evidence, and later a biome formatting failure was reported
# as clean. Both times the fix was "remember not to do it", and both times remembering failed.
#
# So the shape is removed rather than discouraged. This wrapper already prints a BOUNDED tail, so
# there is no reason left to pipe it anywhere, and it ends on an explicit PASS/FAIL line carrying
# the real code. Pipe THIS and you still get the summary line; the exit code stays the tool's
# because nothing runs after it.
#
#   scripts/check.sh tsc                 type-check the frontend
#   scripts/check.sh biome [paths...]    lint+format check (default: the whole frontend)
#   scripts/check.sh fix [paths...]      biome check --write, then re-check
#   scripts/check.sh cargo               cargo check the unpacker
#   scripts/check.sh backend             cargo check + clippy + test the backend
#   scripts/check.sh all                 tsc, biome and cargo; exits non-zero if ANY failed
#
# `set -e` is deliberately NOT used: a non-zero tool must reach the reporting line rather than
# abort the script, or the wrapper would hide failures the same way the pipe did.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAIL_LINES=${CHECK_TAIL:-25}

run() {
    # $1 label, $2 working directory, rest: the command
    local label=$1 dir=$2
    shift 2
    local out
    out=$(mktemp)
    (cd "$dir" && "$@") >"$out" 2>&1
    local code=$?
    local n
    n=$(wc -l <"$out" | tr -d ' ')
    if ((code != 0)); then
        echo "----- $label output (last $TAIL_LINES of $n lines) -----"
        tail -n "$TAIL_LINES" "$out"
    elif ((n > 0)); then
        echo "----- $label output (last 3 of $n lines) -----"
        tail -n 3 "$out"
    fi
    rm -f "$out"
    # The line a reader looks for, and the only one that matters. Printed for pass and fail alike
    # so a silent success is still positively confirmed rather than merely not-denied.
    printf '%-24s %s (exit %d)\n' "CHECK $label" "$( ((code == 0)) && echo PASS || echo FAIL )" "$code"
    return $code
}

cmd=${1:-all}
shift || true

case "$cmd" in
tsc)
    run "tsc" "$ROOT/frontend" npx tsc --noEmit
    ;;
biome)
    if (($#)); then run "biome" "$ROOT/frontend" npx biome check "$@"; else run "biome" "$ROOT/frontend" npx biome check src; fi
    ;;
fix)
    if (($#)); then
        run "biome --write" "$ROOT/frontend" npx biome check --write "$@"
        run "biome" "$ROOT/frontend" npx biome check "$@"
    else
        run "biome --write" "$ROOT/frontend" npx biome check --write src
        run "biome" "$ROOT/frontend" npx biome check src
    fi
    ;;
cargo)
    # RUSTC_WRAPPER is cleared: a configured sccache without permissions makes cargo exit 101
    # before it compiles anything, which is a environment failure masquerading as a build failure.
    run "cargo" "$ROOT/assets/unpacker" env RUSTC_WRAPPER= cargo check
    ;;
backend)
    # The server crate. `cargo` above does NOT cover it - that target is the unpacker.
    # --all-targets so the examples and tests are linted too; the crate opts into
    # pedantic/nursery in src/lib.rs.
    rc=0
    run "clippy backend" "$ROOT/backend" env RUSTC_WRAPPER= cargo clippy --all-targets || rc=1
    run "test backend" "$ROOT/backend" env RUSTC_WRAPPER= cargo test --lib || rc=1
    printf '%-24s %s (exit %d)\n' "CHECK backend" "$( ((rc == 0)) && echo PASS || echo FAIL )" "$rc"
    exit $rc
    ;;
all)
    rc=0
    run "tsc" "$ROOT/frontend" npx tsc --noEmit || rc=1
    run "biome" "$ROOT/frontend" npx biome check src || rc=1
    run "cargo" "$ROOT/assets/unpacker" env RUSTC_WRAPPER= cargo check || rc=1
    printf '%-24s %s (exit %d)\n' "CHECK all" "$( ((rc == 0)) && echo PASS || echo FAIL )" "$rc"
    exit $rc
    ;;
*)
    echo "usage: scripts/check.sh {tsc|biome|fix|cargo|backend|all} [paths...]" >&2
    exit 2
    ;;
esac
