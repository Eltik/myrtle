#!/bin/zsh
# PER-BEAT r for every entrance reference, which the corpus mean cannot show.
#
# WHY. MADC and r answer different questions. A high MADC with a high r is a level or detail
# difference; a LOW r is a structural mismatch, content in the wrong place, and that is what
# "absolute parity" means we do not have. The corpus prints one r per skin, which hides where in
# the cinematic the structure breaks.
#
# 🚨 THE BEATS AND TRIMS ARE all8.sh's, NOT guessed. A first pass at this table used invented beat
# sets and zero trims and read wis at r 0.237 against her known 0.972, with eight of thirteen wrong.
# A per-beat number computed on the wrong beats is not a weaker measurement, it is a different one.
# The control that caught it was comparing each skin's MEAN r here against its corpus value.
#
# RESULT, aligned by fraction of `entranceDuration`: the low-r window is IDIOSYNCRATIC. Ten of
# thirteen run 0.806 to 0.972 throughout. Only whitw2 (0.717) and kalts (0.593) break, both in the
# first third, both recovering by the last. cel (0.806) has a separate MID-entrance dip, 0.504 at
# 0.4 and 0.449 at 0.6, which her mean hides entirely.
#
#   perbeat.sh        prints the per-beat MADC and r lines for all thirteen
cd /Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
run(){ key=$1; d=$2; beats=$3; off=$4; sfx=$5
  skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$d"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
  op=$(python3 -c "import re;print(re.match(r'(char_\d+_[a-z0-9]+)','$d').group(1))")
  enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$d''',safe=''))")
  fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
  bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$d''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
  out=$SP/pb_$key; rm -rf $out; mkdir -p $out
  shots=$(python3 -c "print(','.join(f'{float(b)+$off:.4f}' for b in '$beats'.split(',')))")
  NODE_PATH=$PWD/node_modules EXTRA="rt2048=0&$bd" node rec.js "/spine/DynIllust/$enc/$fenc" $out "$shots" 900 416 >/dev/null 2>&1
  echo "### $key"
  python3 mad.py $out REF_NEW/${key}${sfx}.mp4 "$beats" --inner --offset=$off --dy=0 --fps=30 2>/dev/null | grep "t="
}
run ska    'char_1012_skadi2_iteration#2' "3,5,7,9,13,16,19"     0      _game_fresh
run exc    'char_1032_excu2_sale#12'      "1,2,3,4,5,6"          0.100  _game_fresh
run cel    'char_245_cello_sale#12'       "2,5,8,10,12,14,17"    0      _game_fresh
run mly    'char_4064_mlynar_epoque#28'   "4,7,9,10,11,11.8,12.4,13" 0.033 _game_fresh
run mue    'char_249_mlyss_boc#8'         "3,6,9,12,15,18"       0      _game_fresh
run eyja   'char_1016_agoat2_epoque#34'   "2,4,6,8"              0.100  _game_fresh
run cet    'char_4134_cetsyr_epoque#50'   "2,5,8,11,14,17,18.5"  -0.100 _game_fresh
run wis    'char_1035_wisdel_sale#14'     "2,4,6,8,10,12"        -0.200 _game_fresh
run whitw2 'char_1038_whitw2_sale#15'     "2,4,6,8,10,12,13.5"   0.550  _game_fresh
run fugue  'char_113_cqbw_epoque#7'       "1,1.8,2.8,4,5.5,6.8,7.5,8.2" 0.083 _game_fresh
run kalts  'char_003_kalts_boc#6'         "1,2,3,4,5,6,8,12"     0.400  _game_fresh
run chyue  'char_2024_chyue_cfa#1'        "1,3,5,7,8.5,12.5,15,17" 0.267 _game_fresh
run excunew 'char_1032_excu2_sale#12'     "1,2,3,4,4.8"          0.1667 _game_fresh
echo PERBEAT-DONE
