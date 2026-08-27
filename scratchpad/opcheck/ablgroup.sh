#!/bin/zsh
# ABLATION BY DRAW GROUP across a skin's broken window, to name what content differs.
#
# 🚨 CONTROL, and no table from this script is read without it: the `base` arm must reproduce the
# skin's KNOWN per-beat r. kalts t=3 0.623 / t=4 0.481 / t=5 0.279, whitw2 t=2 0.308 / t=4 0.496 /
# t=6 0.638. If base does not match, the beats or the trim are wrong and every other arm is noise.
#
# ⚠️ `wholeart=0` on EVERY arm including base. Whole-art framing is derived from painted content, so
# an ablation silently re-frames the shot and an ablated render cannot otherwise be compared with an
# unablated one. This inverted a real result once.
#
# RESULT: the two structural outliers point at DIFFERENT groups. kalts is scene FOREGROUND
# (dMADC -1.236, dr +0.191), whitw2 is particle FOREGROUND (dMADC -11.577, dr +0.116). Both improve
# on BOTH metrics, so neither is a sign disagreement. They share a shape, foreground over-draw early
# in the cinematic, not a subsystem.
cd /Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
arm(){ key=$1; d=$2; beats=$3; off=$4; label=$5; extra=$6
  skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$d"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
  op=$(python3 -c "import re;print(re.match(r'(char_\d+_[a-z0-9]+)','$d').group(1))")
  enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$d''',safe=''))")
  fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
  bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$d''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
  out=$SP/abl_${key}_${label}; rm -rf $out; mkdir -p $out
  shots=$(python3 -c "print(','.join(f'{float(b)+$off:.4f}' for b in '$beats'.split(',')))")
  NODE_PATH=$PWD/node_modules EXTRA="rt2048=0&wholeart=0&$bd$extra" node rec.js "/spine/DynIllust/$enc/$fenc" $out "$shots" 900 416 >/dev/null 2>&1
  echo "@@ $key $label"
  python3 mad.py $out REF_NEW/${key}_game_fresh.mp4 "$beats" --inner --offset=$off --dy=0 --fps=30 2>/dev/null | grep -E "t=|MEAN MADC|MEAN r"
}
K='char_003_kalts_boc#6'; KB="3,4,5"; KO=0.400
W='char_1038_whitw2_sale#15'; WB="2,4,6"; WO=0.550
for a in base scenebg scenefg partbg partfg spine; do
  case $a in base) e="";; *) e="&abl=$a";; esac
  arm kalts  $K "$KB" $KO $a "$e"
done
for a in base scenebg scenefg partbg partfg spine; do
  case $a in base) e="";; *) e="&abl=$a";; esac
  arm whitw2 $W "$WB" $WO $a "$e"
done
echo ABL-DONE
