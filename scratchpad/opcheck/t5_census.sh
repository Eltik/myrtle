#!/bin/zsh
set -e
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
export NODE_PATH=$OP/node_modules
SKEL="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
BD=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/char_245_cello/'+urllib.parse.quote('char_245_cello_sale#12',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
for s in {0..57}; do
  d=$SP/C5_$s
  [[ -d "$d" && -n "$(ls -A $d 2>/dev/null)" ]] && continue
  rm -rf $d; mkdir -p $d
  EXTRA="${BD}&psonly=$s" node $OP/rec.js "/spine/DynIllust/${SKEL}" $d "5.0171" 900 416 >/dev/null 2>&1 || echo "FAIL $s"
done
echo done
