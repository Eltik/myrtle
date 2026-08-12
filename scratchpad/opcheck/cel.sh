#!/bin/zsh
# Cello-only MADC under one EXTRA= query string. Same harness as score3.sh, but the
# gap-fill tests need a per-skin `backdrop=` URL, so the three skins cannot share one
# EXTRA — this runs cello alone.
#   cel.sh <label> [EXTRA]
set -e
HERE=${0:A:h}
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
export NODE_PATH=$HERE/node_modules
label=${1:?}; extra=${2:-}
OFF=0.0171
BEATS="2,5,8,10,12,14,17"
SKEL="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
out=$SP/cel_${label}; rm -rf $out; mkdir -p $out
shots=$(python3 -c "print(','.join(f'{float(b)+${OFF}:.4f}' for b in '${BEATS}'.split(',')))")
EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/${SKEL}" $out "$shots" 900 416 >/dev/null 2>&1
printf "%-22s " "$label"
python3 $HERE/mad.py $out $HERE/REF/cel_game_salvaged.mp4 "$BEATS" --inner --offset=$OFF --dy=2
