#!/bin/zsh
# Rebuild every parity clip with the CORRECT per-skin reference trim offset.
#
# ⚠️ The offsets are load-bearing and were only established on 2026-08-11: exc, wis and eyja were
# all mis-trimmed (wis by SIX frames). A clip built with the wrong offset shows a real renderer as
# if it were out of sync, which is exactly the artefact these clips exist to rule out.
# Keep in step with REFOFF in opcheck/all8.sh.
set -e
HERE=${0:A:h}
out=${1:?out dir}
mkdir -p $out
typeset -A DIR OFF
DIR[ska]='char_1012_skadi2_iteration#2';  OFF[ska]=0.067
DIR[exc]='char_1032_excu2_sale#12';       OFF[exc]=0.100
DIR[cel]='char_245_cello_sale#12';        OFF[cel]=-0.033
DIR[mly]='char_4064_mlynar_epoque#28';    OFF[mly]=0.033
DIR[mue]='char_249_mlyss_boc#8';          OFF[mue]=0
DIR[eyja]='char_1016_agoat2_epoque#34';   OFF[eyja]=0.100
DIR[cet]='char_4134_cetsyr_epoque#50';    OFF[cet]=-0.067
DIR[wis]='char_1035_wisdel_sale#14';      OFF[wis]=-0.200
keys=(${@:2})
(( ${#keys} )) || keys=(ska exc cel mly mue eyja cet wis whitw2)
for k in $keys; do
  echo "=== $k (off ${OFF[$k]})"
  FPS=${FPS:-20} $HERE/mkclip_any.sh $k "${DIR[$k]}" $out ${OFF[$k]}
done
