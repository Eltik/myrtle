#!/bin/zsh
# MADC for all three reference skins at the scored beats, under one EXTRA= query string.
#   score3.sh <label> [EXTRA]
#
# REBUILT 2026-08-01 to be self-contained inside the repo, after the previous harness
# (game captures + mad.py + node_modules, all under a job tmp dir) was deleted mid-session.
#
# WARNING: REF/*_game_salvaged.mp4 are SALVAGED from the bottom half of the side-by-side
# clips (720x333, CRF 36, upscaled back to 900x416), because the original captures were
# lost. They carry a degradation floor of roughly +5 MADC and are valid for A/B SIGN only —
# numbers from them are NOT comparable to any figure published before 2026-08-01.
# Validated: the known gamma win reproduces as 17.048 -> 13.754 (true 12.039 -> 7.442).
set -e
HERE=${0:A:h}
REPO=/Users/eltik/Documents/Coding/myrtle
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
export NODE_PATH=$HERE/node_modules
label=${1:?}; extra=${2:-}
typeset -A SKEL OFF BEATS
# OFF RE-DERIVED 2026-08-02 against the re-aligned salvaged clips. Mlynar's +0.051 was
# measured on the PRE-LOSS oracle; the salvage was re-encoded and trimmed, so his true
# offset is -0.085 (swept: +0.051->30.06, 0.00->26.90, -0.05->22.23, -0.085->17.72,
# -0.10->20.01, -0.15->25.46). Virtuosa 0.0171 and Skadi 0.0 re-checked and still optimal.
OFF[mly]=-0.085; OFF[cel]=0.0171; OFF[ska]=0
BEATS[mly]="4,7,9,10,11,11.8,12.4,13"; BEATS[cel]="2,5,8,10,12,14,17"; BEATS[ska]="3,5,7,9,13,16,19"
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
for key in mly cel ska; do
  out=$SP/s3_${label}_${key}; rm -rf $out; mkdir -p $out
  shots=$(python3 -c "print(','.join(f'{float(b)+${OFF[$key]}:.4f}' for b in '${BEATS[$key]}'.split(',')))")
  EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/${SKEL[$key]}" $out "$shots" 900 416 >/dev/null 2>&1
  printf "%s %-6s " "$label" "$key"
  # --dy=2 corrects the salvaged clips' 2-row vertical misalignment (see mad.py). The optimum is
# dy=+2 on ALL THREE skins, so it is a harness artifact of the bottom-half cut, not a render bug.
python3 $HERE/mad.py $out $HERE/REF/${key}_game_salvaged.mp4 "${BEATS[$key]}" --inner --offset=${OFF[$key]} --dy=2 \
    | grep "MEAN MADC"
done
