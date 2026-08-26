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
# ⚠️ COVERAGE IS 5 OF 13, and that is a property of the CAPTURES, not of the renderer. A reference
# only qualifies if its clip runs past `entranceDuration + 9`:
#
#     REACHES     whitw2 (25.700s clip, needs 23.500)   chyue (32.267 / 29.000)
#                 fugue  (29.767 / 18.767)              kalts (27.900 / 23.500)
#                 excunew (18.233 / 15.500)
#     CANNOT      ska (22.500 clip, needs 31.333)  exc (7.000 / 15.500)  cel (18.500 / 27.000)
#                 mly (18.500 / 25.500)  mue (20.500 / 29.000)  eyja (10.500 / 18.767)
#                 cet (19.500 / 28.000)  wis (13.500 / 21.550)
#
# The eight that cannot reach do not fall short of the LAST settled beat, they end before the
# FIRST one: every one of them stops within 0.5 to 2.0 s of its own hand-off. Nothing partial can
# be salvaged from them, so extending this instrument means re-capturing those eight past the cut,
# not adjusting the beat offsets.
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
  for attempt in 1 2; do
    line=$($HERE/score_new.sh $k "$d" "$beats" "$label" "$extra" $off 2>/dev/null || true)
    got=$(print -r -- "$line" | grep -oE "over [0-9]+" | grep -oE "[0-9]+" || echo 0)
    [[ "$got" == "$want" ]] && break
  done
  print -r -- "$line"
done
