#!/usr/bin/env python3
"""Register our render against a GAME WINDOWED-PAGE still, and recover the page's camera.

WHY THE OBVIOUS VERSION FAILS. A plain edge-NCC registration against a page still reads 0.09 to
0.27, against the 0.36 to 0.71 that `edgetrim.py` reports for two frames of the same thing, and it
cannot carry a 10 px claim: a scale error of 0.02 at scale 0.5 across a 1474 px frame already
displaces content by ~15 px. Three fixes take it to 0.54 to 0.76.

  1. MASK FROM MOTION, not by hand. Page chrome is fixed-position UI: the brand banner, the chibi,
     the thumbnail cards, the description card, together ~32.5% of the art rect. The set of pixels
     that CHANGED between two page stills cannot contain any of it, because chrome never moves.
     ⚠️ This is the same signal the motion BOX was retired for, used the other way round. As a
     layout measure it was a subset of unknown tightness and therefore wrong; as a chrome exclusion
     being a subset costs coverage, not correctness.
  2. LUMA, not edges. Edges are exactly what chrome contributes most.
  3. SEARCH OUR IDLE PHASE. Both sides run the same spine animation at unknown relative phase, and
     NCC against a mismatched pose is depressed for a reason that has nothing to do with the fit.
     The winning phases here were 3.00, 6.00 and 2.00 s, all different.

Translation is solved by FFT for all shifts at once; a brute-force grid over phase x scale x
translation does not finish.

"""
from PIL import Image
import numpy as np
from numpy.fft import rfft2, irfft2
SD="/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/751065f0-d8ca-4714-a615-795c1f6925be/scratchpad"
W,H=1474,1080
DS=4
PH=["1.00","2.00","3.00","4.00","5.00","6.00"]
def lum(a): return 0.299*a[...,0]+0.587*a[...,1]+0.114*a[...,2]
print("MASKED, LUMA-BASIS, PHASE-SEARCHED REGISTRATION, FFT translation")
print("mask = pixels that changed between the two page stills, so chrome cannot be in it")
print(f"basis = luma at 1/{DS} scale; translation resolved to {DS} full px by FFT peak\n")
print(f"{'skin':7s} {'bestNCC':>8s} {'k':>6s} {'dx':>7s} {'dy':>7s} {'phase':>6s} {'edge/prev':>10s}")
PREV={"cet":0.2310,"whitw2":0.2680,"eyja":0.1334}
for key in ("cet","whitw2","eyja"):
    p1=np.asarray(Image.open(f"{SD}/page_{key}_1.png").convert("RGB"))[0:H,0:W].astype(int)
    p2=np.asarray(Image.open(f"{SD}/page_{key}_2.png").convert("RGB"))[0:H,0:W].astype(int)
    mask=(np.abs(p1-p2).max(axis=2)>10)
    Pg=lum(p1.astype(np.float32))
    gw,gh=W//DS,H//DS
    Ps=np.asarray(Image.fromarray(Pg.astype("uint8")).resize((gw,gh),Image.LANCZOS)).astype(np.float32)
    ms=np.asarray(Image.fromarray((mask*255).astype("uint8")).resize((gw,gh),Image.NEAREST))>128
    g=np.zeros_like(Ps); gv=Ps[ms]; g[ms]=(gv-gv.mean())/(gv.std()+1e-9)
    Gf=rfft2(g)
    best=(-9,None,None,None,None)
    for ph in PH:
        im=Image.open(f"{SD}/ph_{key}/t{ph}.png").convert("RGB")
        for kk in np.arange(0.80,1.81,0.02):
            w=max(gw,int(round(gw*kk))); h=max(gh,int(round(gh*kk)))
            R=np.asarray(im.resize((w,h),Image.LANCZOS).convert("RGB")).astype(np.float32)
            Rs=lum(R)
            # pad/crop the candidate to the page grid, correlate for all shifts at once
            can=np.zeros((h,w),np.float32); can[:,:]=Rs
            # correlate the page (small) against the candidate (>= page): slide via FFT on the candidate grid
            gpad=np.zeros((h,w),np.float32); gpad[:gh,:gw]=g
            mpad=np.zeros((h,w),bool);      mpad[:gh,:gw]=ms
            cc=irfft2(rfft2(can)*np.conj(rfft2(gpad)), s=(h,w))
            # normalise by the candidate's local energy under the mask, approximated globally
            idx=int(np.argmax(cc)); dy,dx=divmod(idx,w)
            a=can[dy:dy+gh, dx:dx+gw] if dy+gh<=h and dx+gw<=w else None
            if a is None: continue
            av=a[ms]
            if av.std()<1e-6: continue
            az=(av-av.mean())/av.std()
            v=float((az*g[ms]).mean())
            if v>best[0]: best=(v,kk,dx*DS,dy*DS,ph)
    v,kk,dx,dy,ph=best
    print(f"{key:7s} {v:8.4f} {kk:6.3f} {dx:7d} {dy:7d} {ph:>6s} {PREV[key]:10.4f}")
