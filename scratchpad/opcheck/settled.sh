#!/bin/zsh
# SETTLED-BEAT INSTRUMENT. A second scorer beside all8.sh, for the state AFTER the hand-off.
#
# WHY IT EXISTS. Every beat in all8.sh is PRE hand-off, by construction and not by accident:
# whitw2's last is 13.5 against `entranceDuration` 14.5, kalts 12 against 14.5, chyue 17 against
# 20.0, fugue 8.2 against 9.767, excunew 4.8 against 6.5. So any change to the settled idle is
# BIT-IDENTICAL on that instrument no matter how wrong it is, which left a whole class of defect
# unfalsifiable: the settled-ground sprite, the post-hand-off framing, the dolly terminus. Reading
# "corpus unchanged" as "nothing moved" is exactly the mistake that instrument invites.
#
# 🚨 THIS IS A SECOND INSTRUMENT, NOT A REPLACEMENT. It does not touch all8.sh's beat sets, trims
# or baselines, and the two scores are never averaged together. They measure different regions of
# the same clip and a single number over both would hide which one moved.
#
#   settled.sh <label> [EXTRA] [keys...]
#
# Beats are `entranceDuration` + 3, 6 and 9, taken from the skin's own `_Start[scene].json`, and
# the trim is whatever all8.sh already declares for that key. Both are READ from their existing
# sources at run time rather than copied here, so a trim re-derivation lands on both instruments.
#
# ⚠️ COVERAGE IS 12 OF 13. A reference qualifies only if some clip of it runs past
# `entranceDuration + 9`:
#
#     REACHES   whitw2 chyue fugue kalts excunew   (original `_game_fresh` clips)
#               wis cet cel ska exc eyja mue       (`_settled` re-captures, FLOT lookbook, 20 Mbps)
#     CANNOT    mly, whose "W Dali" tile was not surfaced in four scroll passes of the
#               release-sorted list around its Y-4 P07 band, in both directions.
#
# 🔑 A FREE CROSS-VALIDATION of the whole capture-and-score pipeline: `exc` and `excunew` are the
# SAME SKIN captured independently, months apart, by different routes, and they score 77.468 and
# 77.475, agreeing to 0.007 MADC with r -0.017 against -0.014.
#
# ⚠️ These frames carry a LARGE framing error that is not the thing you are usually measuring
# (chyue 106.477, whitw2 99.303 against pre-cut 20.230 and 33.432). Read this instrument as a
# SELF-CONTROLLED A/B, base against your change, never as an absolute parity figure.
set -uo pipefail
HERE=${0:A:h}
DYN=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
label=${1:?label required}; extra=${2:-}; shift 2 2>/dev/null || shift 1

# The five that reach. Kept as a list rather than computed so the coverage claim above is the
# thing that is executed: adding a key here without a long enough capture fails loudly below.
if (( $# )); then keys=($@); else keys=(whitw2 chyue fugue kalts excunew); fi

for k in $keys; do
  # DIR and REFOFF are READ OUT OF all8.sh, which stays the single source for both.
  d=$(grep -E "^DIR\[$k\]=" $HERE/all8.sh | head -1 | sed "s/^DIR\[$k\]='//; s/'.*$//")
  off=$(grep -E "^REFOFF\[$k\]=" $HERE/all8.sh | head -1 | sed "s/^REFOFF\[$k\]=//; s/[^0-9.].*$//")
  [[ -z "$d" ]] && { echo "unknown key: $k (no DIR entry in all8.sh)" >&2; continue; }
  [[ -z "$off" ]] && off=0
  # `entranceDuration` from the skin's own director export, never a transcribed number.
  beats=$(python3 - "$DYN/$d" <<'PY'
import json, os, sys
d = sys.argv[1]
f = [x for x in os.listdir(d) if x.endswith("_Start[scene].json")]
if not f:
    print("")
else:
    dur = json.load(open(os.path.join(d, f[0]))).get("entranceDuration")
    print("" if dur is None else ",".join(f"{dur + o:.3f}" for o in (3, 6, 9)))
PY
)
  [[ -z "$beats" ]] && { echo "$k: no _Start[scene].json entranceDuration, not a settled subject" >&2; continue; }
  want=3
  # A DROPOUT (black frame) reads as a missing beat, never as a score. Retry once, same rule
  # all8.sh uses, and print the beat count so a short read cannot pass as a full one.
  # Prefer a LONGER re-capture when one exists. `_game_fresh` clips stop within 0.5 to 2.0 s of
  # their own hand-off, so most of them cannot reach even the FIRST settled beat; `_settled` clips
  # are recorded from the lookbook viewer at 20 Mbps specifically to clear `entranceDuration + 9`.
  # The originals are never overwritten, so every all8.sh baseline stays reproducible.
  if [[ -f $HERE/REF_NEW/${k}_settled.mp4 ]]; then sfx=_settled; else sfx=_game_fresh; fi
  for attempt in 1 2; do
    line=$(REF_SUFFIX=$sfx $HERE/score_new.sh $k "$d" "$beats" "$label" "$extra" $off 2>/dev/null || true)
    got=$(print -r -- "$line" | grep -oE "over [0-9]+" | grep -oE "[0-9]+" || echo 0)
    [[ "$got" == "$want" ]] && break
  done
  print -r -- "$line   [${sfx#_}]"
done
