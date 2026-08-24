#!/usr/bin/env python3
"""Locate a known skin tile by SLIDING-WINDOW normalised cross-correlation.
No label/geometry assumptions — works on any FLOT layout. Scrolls and reports
the best match position in native coords."""
import subprocess, io, sys, time, urllib.request, urllib.parse
import numpy as np
from PIL import Image
BUNDLES=[a for a in sys.argv[1:] if not a.startswith("--")]
STEPS=int(next((a.split("=")[1] for a in sys.argv if a.startswith("--steps=")),18))
THRESH=float(next((a.split("=")[1] for a in sys.argv if a.startswith("--thresh=")),0.45))
DS=4                      # downscale factor for speed
TW,TH=276//DS,514//DS     # tile size in downscaled px
def portrait(b):
    u="http://localhost:3060/api/skin-portrait/"+urllib.parse.quote(b,safe="")
    return Image.open(io.BytesIO(urllib.request.urlopen(u,timeout=20).read())).convert("L")
def cap():
    r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("L") if len(r)>1000 else None
def zn(a):
    a=a.astype(np.float32); s=a.std()
    return (a-a.mean())/(s if s>1e-6 else 1.0)
T={b:zn(np.asarray(portrait(b).resize((TW,TH),Image.LANCZOS))) for b in BUNDLES}
print(f"[slide] templates {list(T)} tile {TW}x{TH} (downscale {DS}x)")
prev=None
for step in range(STEPS):
    im=cap()
    if im is None: break
    small=np.asarray(im.resize((im.width//DS,im.height//DS),Image.LANCZOS)).astype(np.float32)
    H,W=small.shape
    best=(0,None,None)
    for y in range(0,H-TH+1,3):
        for x in range(0,W-TW+1,3):
            w=small[y:y+TH,x:x+TW]
            s=w.std()
            if s<8: continue                     # skip flat background
            wz=(w-w.mean())/s
            for b,t in T.items():
                v=float((wz*t).mean())
                if v>best[0]: best=(v,b,(x*DS+276//2,y*DS+514//2))
    print(f"step {step}: best NCC {best[0]:.3f} {best[1]} at {best[2]}")
    if best[0]>=THRESH:
        print(f">>> MATCH {best[1]} NCC={best[0]:.3f} tap={best[2]}"); sys.exit(0)
    sig=round(best[0],3)
    if sig==prev: print("(view static)"); break
    prev=sig
    subprocess.run(["adb","shell","input","swipe","1170","760","1170","480","900"],capture_output=True)
    time.sleep(1.5)
print("[slide] no match"); sys.exit(2)
