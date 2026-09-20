#!/usr/bin/env bash
# Run the backend with raw syncData dumping on, so one login + refresh on the
# frontend writes the whole account payload to disk for inspection.
#
#   backend/scripts/sync-dump.sh            # dumps to backend/sync-dumps/
#   backend/scripts/sync-dump.sh /some/dir  # dumps there instead
#
# Then log in on the frontend (or press Refresh on your profile). Each sync
# writes <server>_<uid>_<unix>.json into the dump dir. Stop with Ctrl-C; the
# dump is off whenever MYRTLE_SYNC_DUMP_DIR is unset, so a normal start never
# writes anything. The files hold the entire account: do not commit them
# (backend/sync-dumps/ is gitignored).
set -euo pipefail

backend="$(cd "$(dirname "$0")/.." && pwd)"
dir="${1:-$backend/sync-dumps}"
mkdir -p "$dir"

echo "syncData dumps -> $dir"
echo "log in or refresh on the frontend, then read the newest *.json there"
cd "$backend"
MYRTLE_SYNC_DUMP_DIR="$dir" exec cargo run --bin backend
