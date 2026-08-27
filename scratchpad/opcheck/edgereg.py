"""Is the SETTLED camera error systematic, or per skin?

Registers our settled frame onto the game's by scale and offset, scored on EDGE MAGNITUDE. Luma
correlation cannot do this job here: our settled backdrop is dark where the game's is white on at
least one subject, and that flat anti-correlated area dominates at every alignment (measured on
chyue: scale >= 1 stalled at r 0.275 and scale < 1 at 0.136, both at a range edge, while edges
found an interior optimum at 0.79 / dy -108).

Usage: python3 edgereg.py <key> <renders-dir> [beat]
"""
import subprocess
import sys
import glob
import re
import numpy as np
from PIL import Image

def edges(im):
    a = np.asarray(im).astype(np.float32)
    l = 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]
    gy, gx = np.gradient(l)
    return np.log1p(np.hypot(gx, gy))   # compress so a few strong edges cannot dominate

def game_frames(ref):
    p = subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries",
                        "stream=width,height","-of","csv=p=0",ref],
                       capture_output=True, text=True).stdout.strip().split(",")
    w, h = int(p[0]), int(p[1])
    raw = subprocess.run(["ffmpeg","-v","error","-i",ref,"-f","rawvideo","-pix_fmt","rgb24","-"],
                         capture_output=True).stdout
    n = len(raw) // (w * h * 3)
    return np.frombuffer(raw, np.uint8)[: n * w * h * 3].reshape(n, h, w, 3), n

def register(o_im, g_im):
    ge = edges(g_im); GH, GW = ge.shape
    def sc(s, dx, dy):
        w = max(8, int(round(GW * s))); h = max(8, int(round(GH * s)))
        canv = Image.new("RGB", (GW, GH), (0, 0, 0))
        canv.paste(o_im.resize((w, h), Image.LANCZOS),
                   (int(round((GW - w) / 2 + dx)), int(round((GH - h) / 2 + dy))))
        oe = edges(canv)
        return None if oe.std() < 1e-6 else float(np.corrcoef(oe.ravel(), ge.ravel())[0, 1])
    best = (-2, None)
    for s in np.arange(0.40, 1.45, 0.05):
        for dx in range(-140, 141, 20):
            for dy in range(-140, 141, 20):
                v = sc(s, dx, dy)
                if v is not None and v > best[0]: best = (v, (round(s, 3), dx, dy))
    s0, dx0, dy0 = best[1]; fine = best
    for s in np.arange(max(0.2, s0 - 0.06), s0 + 0.07, 0.01):
        for dx in range(dx0 - 16, dx0 + 17, 4):
            for dy in range(dy0 - 16, dy0 + 17, 4):
                v = sc(s, dx, dy)
                if v is not None and v > fine[0]: fine = (v, (round(s, 3), dx, dy))
    return sc(1.0, 0, 0), fine

def main():
    key, d = sys.argv[1], sys.argv[2]
    ref = f"REF_NEW/{key}_settled.mp4"
    import os
    if not os.path.exists(ref): ref = f"REF_NEW/{key}_game_fresh.mp4"
    off = float(subprocess.run(["python3","-c",
        f"import re;src=open('all8.sh').read();print(dict(re.findall(r'REFOFF\\[(\\w+)\\]=([-\\d.]+)',src)).get('{key}','0'))"],
        capture_output=True, text=True).stdout.strip())
    G, n = game_frames(ref)
    files = sorted(glob.glob(d + "/t*.png"), key=lambda f: float(re.search(r"t([\d.]+)\.png", f).group(1)))
    for f in files:
        t = float(re.search(r"t([\d.]+)\.png", f).group(1))
        gi = int(round((t - off) * 30))
        if gi >= n: print(f"  {key:<8} t={t:7.2f}  game frame {gi} beyond {n}"); continue
        o = Image.open(f).convert("RGB")
        g = Image.fromarray(G[gi]).resize(o.size, Image.LANCZOS)
        ident, (r, (s, dx, dy)) = register(o, g)
        edge_ = "  <- AT RANGE EDGE" if s <= 0.41 or s >= 1.44 or abs(dx) >= 156 or abs(dy) >= 156 else ""
        print(f"  {key:<8} t={t:7.2f}  identity {ident:+.4f} -> {r:+.4f}   scale {s:5.3f}  dx {dx:+4d}  dy {dy:+5d}{edge_}")

main()
