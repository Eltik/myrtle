#!/usr/bin/env python3
"""Large-scale (smooth-field) error against the capture, for one render dir.

⚠️ USE FOR A/B ONLY. The control subtracted below is `madc(blur(game[i-1]), blur(game[i]))`,
which is a MOTION term, not a noise floor: it correlates +0.97 with the temporal floor, and the
resulting "excess" correlates -0.98 with it. Our own smooth error is nearly CONSTANT across
beats (cello mean 9.809, std 0.579), so the excess is essentially `constant - motion` and any
PER-BEAT RANKING it produces is a motion ranking. Comparing two VARIANTS at the same beats is
still valid — the control is identical in both arms and cancels — which is what this is for.

usage: smooth_excess.py <dir> <beats> <offset> [mly|cel|ska]   (default cel)

Whole-frame MADC is swamped by the capture's codec noise, which lives in the detail band and is
unmatchable by construction — sweeps judged against it are insensitive. Blurring BOTH images
17px suppresses that noise; subtracting the same measurement taken between the game and its own
neighbouring frame removes what remains. What is left is the part of the smooth lighting field
that is genuinely ours to fix.  usage: smooth_excess.py <dir> <beats> <offset>
"""
import subprocess, sys, numpy as np
from PIL import Image
d, beats, off = sys.argv[1], [float(x) for x in sys.argv[2].split(",")], float(sys.argv[3])
key = sys.argv[4] if len(sys.argv) > 4 else "cel"
OP = "/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck"
mp4 = f"{OP}/REF/{key}_game_salvaged.mp4"
w, h = subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries","stream=width,height","-of","csv=p=0",mp4],capture_output=True,text=True).stdout.strip().split(",")
w, h = int(w), int(h)
raw = subprocess.run(["ffmpeg","-v","error","-i",mp4,"-f","rawvideo","-pix_fmt","rgb24","-"],capture_output=True).stdout
n = len(raw)//(w*h*3)
G = np.frombuffer(raw,np.uint8)[:n*w*h*3].reshape(n,h,w,3).astype(np.float32)

def ycc(a):
    y=.299*a[...,0]+.587*a[...,1]+.114*a[...,2]
    return y,128+.564*(a[...,2]-y),128+.713*(a[...,0]-y)
def madc(o,g):
    yo,cbo,cro=ycc(o); yg,cbg,crg=ycc(g)
    return np.abs(np.round(yo)-np.round(yg)).mean()+(np.abs(cbo-cbg).mean()+np.abs(cro-crg).mean())/2
def blur(a,k=8):
    p=np.pad(a,((k,k),(k,k),(0,0)),mode='edge')
    c=np.cumsum(np.cumsum(p,0),1); c=np.pad(c,((1,0),(1,0),(0,0)))
    return (c[2*k+1:,2*k+1:]-c[:-2*k-1,2*k+1:]-c[2*k+1:,:-2*k-1]+c[:-2*k-1,:-2*k-1])/((2*k+1)**2)

ex=[]
for b in beats:
    gi=int(round(b*15))
    o=np.asarray(Image.open(f"{d}/t{b+off:.2f}.png").convert("RGB"),np.float32)
    g0,gp0=G[gi],G[gi-1]
    if (g0.shape[1],g0.shape[0])!=(o.shape[1],o.shape[0]):
        rs=lambda a: np.asarray(Image.fromarray(a.astype(np.uint8)).resize((o.shape[1],o.shape[0]),Image.LANCZOS),np.float32)
        g0,gp0=rs(g0),rs(gp0)
    o=o[:-2][:,36:864]; g=g0[2:][:,36:864]; gp=gp0[2:][:,36:864]
    ob,gb,pb=blur(o),blur(g),blur(gp)
    ex.append(madc(ob[::2,::2],gb[::2,::2]) - madc(pb[::2,::2],gb[::2,::2]))
print(" ".join(f"{v:.3f}" for v in ex) + f"   MEAN {np.mean(ex):.4f}")
