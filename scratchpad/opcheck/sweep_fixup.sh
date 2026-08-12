#!/bin/zsh
# Re-render the skins the first sweep pass missed.
#
# Cause: a skin DIRECTORY may carry a `_2` suffix its skel FILE does not
# (`char_2026_yu_2/dyn_illust_char_2026_yu.skel`), so building the URL as
# `dyn_illust_<dir>.skel` 404s and the recorder writes pure black. Discover the
# actual skel from disk instead of deriving it from the directory name.
set -e
HERE=${0:A:h}
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
export NODE_PATH=$OP/node_modules
OUT=$HERE/sweep
SHOTS="2,6,10,14"

for sk in "$@"; do
  skel=$(ls "$ROOT/$sk" | grep '\.skel$' | head -1)
  if [[ -z "$skel" ]]; then echo "NO-SKEL $sk"; continue; fi
  enc_dir=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  enc_skel=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  url="/spine/DynIllust/${enc_dir}/${enc_skel}"
  for variant in off on; do
    d=$OUT/${sk}__${variant}
    rm -rf $d; mkdir -p $d
    ex=""
    [[ $variant == on ]] && ex="simspeed=1&prewarm=1"
    EXTRA="$ex" node $OP/rec.js "$url" $d "$SHOTS" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk $variant"
  done
  echo "  redone $sk  ($skel)"
done
