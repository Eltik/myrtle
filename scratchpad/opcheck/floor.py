#!/usr/bin/env python3
"""Per-beat TEMPORAL FLOOR of a reference capture, in mad.py's exact metric.

Ranking beats by raw MADC is misleading: a beat where the game is moving fast carries a large
irreducible error from sub-frame timing alone, so a big number there may mean nothing is wrong.
This measures that irreducible part directly — MADC between the game's own adjacent frames,
halved (a render lands on average half a frame away from the sampled index). Everything our
render scores ABOVE this is headroom that is actually ours to win.

Same pipeline as mad.py: `--inner` column crop, 2x2 decimation, MADC = MAD_Y + (MAD_Cb+MAD_Cr)/2.
"""
import subprocess, sys
import numpy as np

mp4, times = sys.argv[1], sys.argv[2]
beats = [float(x) for x in times.split(",")]

def ycc(a):
    y = 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]
    return y, 128 + 0.564 * (a[..., 2] - y), 128 + 0.713 * (a[..., 0] - y)

def madc(a, b):
    a, b = a[:, 36:864], b[:, 36:864]
    a, b = a[::2, ::2], b[::2, ::2]
    ya, cba, cra = ycc(a)
    yb, cbb, crb = ycc(b)
    return np.abs(np.round(ya) - np.round(yb)).mean() + (np.abs(cba - cbb).mean() + np.abs(cra - crb).mean()) / 2

w, h = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
                       "stream=width,height", "-of", "csv=p=0", mp4],
                      capture_output=True, text=True).stdout.strip().split(",")
w, h = int(w), int(h)
raw = subprocess.run(["ffmpeg", "-v", "error", "-i", mp4, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
                     capture_output=True).stdout
n = len(raw) // (w * h * 3)
G = np.frombuffer(raw, np.uint8)[: n * w * h * 3].reshape(n, h, w, 3).astype(np.float32)

for b in beats:
    gi = int(round(b * 15))
    if gi >= n:
        print(f"  t={b:<6} beyond clip"); continue
    d = []
    if gi - 1 >= 0: d.append(madc(G[gi], G[gi - 1]))
    if gi + 1 < n:  d.append(madc(G[gi], G[gi + 1]))
    fl = float(np.mean(d)) / 2
    print(f"  t={b:<6} floor={fl:6.3f}")
