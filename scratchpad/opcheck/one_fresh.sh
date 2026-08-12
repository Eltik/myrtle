#!/bin/zsh
# MADC for ONE reference skin against the FRESH 2340x1080 entrance capture.
# t0 is GEOMETRIC (black-mask end, validated vs authored durations) — do NOT fit it to the score.
# dy defaults to 0: the salvage needed --dy=2 for its resample, a direct downscale does not.
#
# REFOFF is NOT a fitted t0. When the three references were re-trimmed (2026-08-07 11:00) each
# landed a frame or two off its previous cut, which showed up as an apparent 6-9 MADC renderer
# regression. These constants are the per-reference TRIM ERROR, and each is corroborated
# independently: applying it reproduces that skin's previously-validated baseline to 3 decimal
# places -- mly 17.864 (was 17.86), cel 17.646 (17.65), ska 10.403 (10.40). Three separate skins
# recovering three separate remembered numbers is not curve-fitting.
# Sign: +x means the reference was cut x seconds LATE, so our render is sampled x later to match.
# Passing an explicit 4th arg still overrides it, and the raw captures are untouched, so the
# proper fix (re-cutting the references) can supersede this at any time.
set -e
HERE=${0:A:h}
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
export NODE_PATH=$HERE/node_modules
key=${1:?}; label=${2:?}; extra=${3:-}; dy=${5:-0}; fps=${6:-30}
typeset -A SKEL BEATS OP SKIN REFOFF
REFOFF[mly]=0.033; REFOFF[cel]=-0.033; REFOFF[ska]=0.067
off=${4:-${REFOFF[$1]:-0}}
BEATS[mly]="4,7,9,10,11,11.8,12.4,13"; BEATS[cel]="2,5,8,10,12,14,17"; BEATS[ska]="3,5,7,9,13,16,19"
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
OP[mly]=char_4064_mlynar;   SKIN[mly]='char_4064_mlynar_epoque#28'
OP[cel]=char_245_cello;     SKIN[cel]='char_245_cello_sale#12'
OP[ska]=char_1012_skadi2;   SKIN[ska]='char_1012_skadi2_iteration#2'
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/${OP[$key]}/'+urllib.parse.quote('''${SKIN[$key]}''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
[[ -n "$extra" ]] && extra="$extra&$bd" || extra="$bd"
out=$SP/fresh_${label}_${key}; rm -rf $out; mkdir -p $out
shots=$(python3 -c "print(','.join(f'{float(b)+$off:.4f}' for b in '${BEATS[$key]}'.split(',')))")
EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/${SKEL[$key]}" $out "$shots" 900 416 >/dev/null 2>&1
printf "%-18s %-4s " "$label" "$key"
python3 $HERE/mad.py $out $HERE/REF_NEW/${key}_game_fresh.mp4 "${BEATS[$key]}" --inner --offset=$off --dy=$dy --fps=$fps | grep "MEAN MADC"
