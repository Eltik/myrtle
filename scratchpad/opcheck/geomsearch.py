#!/usr/bin/env python3
"""Is the mismatch GEOMETRY? Search scale + translation for the alignment that maximises
normalised cross-correlation between our render and the game frame.

  geomsearch.py <game.mp4> <ours_dir> <beats> <refoff>

Run this whenever `mad.py` reports a low `MEAN r`. It answers the one question a diff map
cannot: is the content in the WRONG PLACE (a geometry fault) or genuinely ABSENT? A skin that
is merely missing content still aligns at scale 1.0 / dx 0 / dy 0; a skin with a camera fault
reports a consistent non-unit scale or a drifting offset.

Found two real bugs on its first day: Kal'tsit's dropped camera track (offset decaying
165 -> 8 px across t=2..8) and whitw2's sqrt(3) framing error (a constant 1.6-1.8x zoom).

Avoids the two documented traps:
  - no zero-padding: a central GAME patch is matched INSIDE our (resampled) frame, so every
    candidate position has real data on both sides.
  - reports the correlation VALUE, not just the argmax, so a flat/periodic surface is visible.
"""
import subprocess, sys
import numpy as np
from PIL import Image
from scipy.signal import fftconvolve

if len(sys.argv) < 5:
    print(__doc__)
    raise SystemExit(2)
GAME, OURS, BEATS, OFF = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4])

probe = subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries",
    "stream=width,height","-of","csv=p=0",GAME],capture_output=True,text=True).stdout.strip().split(",")
GW,GH=int(probe[0]),int(probe[1])
raw=subprocess.run(["ffmpeg","-v","error","-i",GAME,"-f","rawvideo","-pix_fmt","rgb24","-"],capture_output=True).stdout
n=len(raw)//(GW*GH*3)
G=np.frombuffer(raw,np.uint8)[:n*GW*GH*3].reshape(n,GH,GW,3)

def ncc_map(img, patch):
    """Normalised cross-correlation of `patch` over `img` (valid positions only)."""
    p = patch - patch.mean()
    pn = np.sqrt((p*p).sum())
    if pn == 0: return None
    num = fftconvolve(img, p[::-1,::-1], mode='valid')
    ones = np.ones_like(p)
    s1 = fftconvolve(img, ones, mode='valid')
    s2 = fftconvolve(img*img, ones, mode='valid')
    cnt = p.size
    var = s2 - s1*s1/cnt
    # A near-flat window makes `var` tiny (or negative from rounding) and the ratio explodes —
    # that is how this returned "NCC = 916". Require the window to carry at least 5% of the
    # patch's own variance before its correlation is meaningful, and clip to the valid range.
    floor = 0.05 * float((p*p).sum())
    bad = var < floor
    var = np.maximum(var, floor)
    r = num / (np.sqrt(var)*pn)
    r[bad] = -1.0
    return np.clip(r, -1.0, 1.0)

for bt in [float(x) for x in BEATS.split(',')]:
    o_full = np.asarray(Image.open(f"{OURS}/t{bt+OFF:.2f}.png").convert("L")).astype(np.float32)
    g_full = np.asarray(Image.fromarray(G[int(round(bt*30))]).convert("L")
                        .resize((o_full.shape[1],o_full.shape[0]),Image.LANCZOS)).astype(np.float32)
    H,W = g_full.shape
    # central game patch (60% of frame), well inside the letterbox columns
    ph,pw = int(H*0.55), int(W*0.55)
    gy,gx = (H-ph)//2, (W-pw)//2
    patch = g_full[gy:gy+ph, gx:gx+pw]
    best = (-2, None)
    for s in np.arange(0.60, 1.85, 0.05):
        nh,nw = int(H*s), int(W*s)
        if nh < ph or nw < pw: continue
        o = np.asarray(Image.fromarray(o_full).resize((nw,nh),Image.LANCZOS)).astype(np.float32)
        m = ncc_map(o, patch)
        if m is None: continue
        r = float(m.max())
        if r > best[0]:
            iy,ix = np.unravel_index(m.argmax(), m.shape)
            best = (r, (round(s,2), ix - gx*s, iy - gy*s))
    sc, ddx, ddy = best[1] if best[1] else (float("nan"),) * 3
    flag = ""
    if abs(sc - 1.0) > 0.06:
        flag = "   <== ZOOM error"
    elif max(abs(ddx), abs(ddy)) > 12:
        flag = "   <== OFFSET error"
    print(f"t={bt:<6} best NCC={best[0]:6.3f}  scale={sc:5.2f}  dx={ddx:+7.1f}  dy={ddy:+7.1f}{flag}")
