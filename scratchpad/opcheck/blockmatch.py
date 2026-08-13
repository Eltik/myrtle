#!/usr/bin/env python3
"""Symmetric BLOCK MATCHING between our render and the game capture.

Rebuilt 2026-08-12 (the original lived in a session scratchpad and is gone). This is the tool that
localised Cello's displaced instrument: a global shift sweep says "registration is right" while a
single COLUMN of blocks is displaced 6-8 px, because one held object swings wrong while the body
stays pixel-aligned. A whole-frame metric cannot see that; a per-block one can.

  blockmatch.py <ourdir> <game.mp4> <beats> [--offset=S] [--block=N] [--search=N]

Prints, per beat, a grid of best (dx,dy) and the MAE gain from applying it, so a block that is
genuinely displaced (large gain, coherent direction) is distinguishable from noise (zero gain).
"""
import sys, os, glob, subprocess, tempfile
import numpy as np
from PIL import Image

def luma(p):
    a = np.asarray(Image.open(p).convert("L")).astype(float)
    return a

def main():
    ourdir, mp4, beats = sys.argv[1], sys.argv[2], sys.argv[3]
    off = 0.0; blk = 64; search = 14
    for a in sys.argv[4:]:
        if a.startswith("--offset="): off = float(a.split("=")[1])
        elif a.startswith("--block="): blk = int(a.split("=")[1])
        elif a.startswith("--search="): search = int(a.split("=")[1])
    td = tempfile.mkdtemp()
    subprocess.run(["ffmpeg", "-y", "-i", mp4, "-vf", "fps=30", f"{td}/g%04d.png", "-loglevel", "error"], check=True)
    g = sorted(glob.glob(f"{td}/*.png"))
    ours = {round(float(os.path.basename(p)[1:-4]) - off, 3): p for p in glob.glob(f"{ourdir}/*.png")}
    for bs in beats.split(","):
        beat = round(float(bs), 3)
        key = min(ours, key=lambda k: abs(k - beat)) if ours else None
        if key is None or abs(key - beat) > 0.02:
            print(f"beat {beat}: no frame"); continue
        gi = int(round(beat * 30))
        if gi >= len(g):
            print(f"beat {beat}: beyond capture"); continue
        O, G = luma(ours[key]), luma(g[gi])
        H, W = O.shape
        print(f"\n=== beat {beat}  (block {blk}px, search +-{search})")
        for by in range(search, H - blk - search, blk):
            row = []
            for bx in range(max(search, 37), W - blk - search - 37, blk):
                ref = G[by:by + blk, bx:bx + blk]
                base = np.abs(O[by:by + blk, bx:bx + blk] - ref).mean()
                best = (0, 0, base)
                for dy in range(-search, search + 1, 2):
                    for dx in range(-search, search + 1, 2):
                        v = np.abs(O[by + dy:by + dy + blk, bx + dx:bx + dx + blk] - ref).mean()
                        if v < best[2]: best = (dx, dy, v)
                row.append(f"{best[0]:+3d},{best[1]:+3d}/{base - best[2]:4.1f}")
            print("  " + " ".join(row))
main()
