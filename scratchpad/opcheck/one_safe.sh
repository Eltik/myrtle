#!/bin/zsh
# score_new.sh with the standing dropout guard applied.
#
# A render that comes back with an ALL-ZERO frame, or with fewer frames than beats requested,
# is a black-frame dropout, not a measurement — it has faked at least one confident wrong
# conclusion per session (most recently "psonly=4 paints the frame black", which on retry was
# 68.2/83.2/83.2/76.1). Retry up to 3x and report DROPOUT rather than hand back a bad frame.
#
#   one_safe.sh <key> <skinDir> <beats> <label> [EXTRA] [off]
set -e
HERE=${0:A:h}
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
key=${1:?}; dir=${2:?}; beats=${3:?}; label=${4:?}; extra=${5:-}; off=${6:-0}
want=$(python3 -c "print(len('$beats'.split(',')))")
for attempt in 1 2 3; do
  rm -rf $SP/new_${label}_${key}
  $HERE/score_new.sh "$key" "$dir" "$beats" "$label" "$extra" "$off" >/dev/null 2>&1 || true
  ok=$(python3 - "$SP/new_${label}_${key}" "$want" <<'PY'
import sys, glob, os
import numpy as np
from PIL import Image
d, want = sys.argv[1], int(sys.argv[2])
fs = sorted(glob.glob(os.path.join(d, "*.png")))
if len(fs) != want:
    print("0"); raise SystemExit
# An all-zero frame is the dropout signature. A legitimately black frame does exist
# (mid-fade), so this is a retry trigger, not a hard failure.
print("0" if any(np.array(Image.open(f)).max() == 0 for f in fs) else "1")
PY
)
  [[ "$ok" == "1" ]] && { echo "OK $label ($attempt)"; exit 0; }
done
echo "DROPOUT $label after 3 attempts" >&2
exit 1
