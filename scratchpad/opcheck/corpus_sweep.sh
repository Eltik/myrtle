#!/bin/zsh
# Corpus blow-out sweep for the simSpeed/prewarm default.
#
# For every deployed skin that carries either field, render the same beats twice — shipped
# (flags off) and candidate (flags on) — and diff the frames. There is no game reference for
# 74 of the 77, so this is NOT a parity score: it asks the only question a corpus render can
# answer, which is whether the change destroys or blows out any skin.
set -e
HERE=${0:A:h}
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
export NODE_PATH=$OP/node_modules
OUT=$HERE/sweep
SHOTS="2,6,10,14"

mkdir -p $OUT
skins=$(python3 -c "import json;print('\n'.join(json.load(open('/tmp/affected_skins.json'))))")

i=0
echo "$skins" | while read -r sk; do
  [[ -z "$sk" ]] && continue
  i=$((i+1))
  # The skel FILE name is not derivable from the DIRECTORY name — a dir may carry a `_2`
  # suffix its skel does not, and one skin differs only by capitalisation. Glob it.
  skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$sk" | grep '\.skel$' | head -1)
  [[ -z "$skel" ]] && { echo "NO-SKEL $sk"; continue; }
  enc=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  encs=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  url="/spine/DynIllust/${enc}/${encs}"
  for variant in off on; do
    d=$OUT/${sk}__${variant}
    [[ -d "$d" && -n "$(ls -A $d 2>/dev/null)" ]] && continue
    rm -rf $d; mkdir -p $d
    ex=""
    [[ $variant == on ]] && ex="simspeed=1&prewarm=1"
    EXTRA="$ex" node $OP/rec.js "$url" $d "$SHOTS" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk $variant"
  done
  printf "\r  %3d rendered" $i
done
echo ""
echo "done"
