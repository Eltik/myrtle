#!/usr/bin/env python3
"""Do DIFFERENT REGIONS of the frame need DIFFERENT shifts?

  regionshift.py <game.mp4> <ours_dir> <beats> <refoff>

Run this before calling a residual "parallax" or "architectural". If one rigid (dx,dy) aligns
every band, the fault is a single GLOBAL error (camera track, timing) and is fixable. If bands
disagree, content at different depths moves at different rates, which a depth-less 2D composite
cannot reproduce.

It has already overturned one conclusion: whitw2's residual pan was called parallax, and this
showed her top/middle/bottom bands all wanting the SAME dx (t=6: -112/-117/-119) against a
control that is 0 everywhere. The real cause was a 0.55s reference mis-trim, worth -22 MADC.

If one rigid (dx,dy) aligns every region, the fault is the CAMERA. If regions disagree, content
at different depths is moving at different rates — parallax — which a depth-less 2D composite
cannot reproduce and no camera-curve fix will repair.
"""
import subprocess, sys
import numpy as np
from PIL import Image
from scipy.signal import fftconvolve

GAME, OURS, BEATS, OFF = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4])
probe=subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries",
  "stream=width,height","-of","csv=p=0",GAME],capture_output=True,text=True).stdout.strip().split(",")
GW,GH=int(probe[0]),int(probe[1])
raw=subprocess.run(["ffmpeg","-v","error","-i",GAME,"-f","rawvideo","-pix_fmt","rgb24","-"],capture_output=True).stdout
n=len(raw)//(GW*GH*3); G=np.frombuffer(raw,np.uint8)[:n*GW*GH*3].reshape(n,GH,GW,3)

def best_shift(img, patch, rng=170):
    p = patch - patch.mean(); pn=np.sqrt((p*p).sum())
    if pn<1e-6: return None
    num=fftconvolve(img,p[::-1,::-1],mode='valid')
    ones=np.ones_like(p)
    s1=fftconvolve(img,ones,mode='valid'); s2=fftconvolve(img*img,ones,mode='valid')
    var=s2-s1*s1/p.size
    floor=0.05*float((p*p).sum()); bad=var<floor
    r=num/(np.sqrt(np.maximum(var,floor))*pn); r[bad]=-1
    iy,ix=np.unravel_index(np.argmax(r),r.shape)
    return ix,iy,float(np.clip(r.max(),-1,1))

for bt in [float(x) for x in BEATS.split(',')]:
    o=np.asarray(Image.open(f"{OURS}/t{bt+OFF:.2f}.png").convert("L")).astype(np.float32)
    gi=int(round(bt*30))
    if gi>=n: continue
    g=np.asarray(Image.fromarray(G[gi]).convert("L").resize((o.shape[1],o.shape[0]),Image.LANCZOS)).astype(np.float32)
    H,W=g.shape
    out=[]
    # three horizontal bands: top (usually far), middle, bottom (usually near)
    for name,(y0,y1) in [("top",(30,140)),("mid",(150,260)),("bot",(280,395))]:
        pw=380; gx=(W-pw)//2
        patch=g[y0:y1, gx:gx+pw]
        res=best_shift(o,patch)
        if res is None: out.append(f"{name}: flat"); continue
        ix,iy,r=res
        out.append(f"{name}: dx={ix-gx:+5.0f} dy={iy-y0:+4.0f} r={r:.2f}")
    print(f"t={bt:<6} " + "   ".join(out))
