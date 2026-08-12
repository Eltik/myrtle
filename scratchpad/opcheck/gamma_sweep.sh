#!/bin/zsh
# Corpus sweep for moving the composite-gamma HI anchor 1.02 -> 1.00.
#
# The exponent is INTERPOLATED per skin from its authored `cameraSizePx`, so the anchor move
# does not apply one value corpus-wide — each affected skin gets its own new exponent. The
# candidate variant therefore passes a PER-SKIN `?gamma=`, read from /tmp/gamma_new.json, while
# the control renders with no override (the shipped interpolation).
set -e
HERE=${0:A:h}
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
export NODE_PATH=$OP/node_modules
OUT=$HERE/gsweep
SHOTS="2,6,10,14"

mkdir -p $OUT
skins=$(python3 -c "import json;print('\n'.join(sorted(json.load(open('/tmp/gamma_new.json')))))")

i=0
echo "$skins" | while read -r sk; do
  [[ -z "$sk" ]] && continue
  i=$((i+1))
  # The skel FILE name is not derivable from the DIRECTORY name (a dir may carry a `_2`
  # suffix its skel does not; one skin differs only by capitalisation). Glob it.
  skel=$(ls "$ROOT/$sk" | grep '\.skel$' | head -1)
  [[ -z "$skel" ]] && { echo "NO-SKEL $sk"; continue; }
  enc=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  encs=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  url="/spine/DynIllust/${enc}/${encs}"
  g=$(python3 -c "import json,sys;print(json.load(open('/tmp/gamma_new.json'))[sys.argv[1]])" "$sk")
  for variant in off on; do
    d=$OUT/${sk}__${variant}
    [[ -d "$d" && -n "$(ls -A $d 2>/dev/null)" ]] && continue
    rm -rf $d; mkdir -p $d
    ex=""
    [[ $variant == on ]] && ex="gamma=$g"
    EXTRA="$ex" node $OP/rec.js "$url" $d "$SHOTS" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk $variant"
  done
  printf "\r  %3d rendered" $i
done
echo ""
echo "done"
