#!/usr/bin/env python3
"""Geometric instrument for THIN BRIGHT RIDGES (Skadi's coral threads, red streaks).

Why this exists: MADC samples a handful of beats and averages per pixel, so a structure that
is present but drawn as a broad dim smear costs almost nothing — the beat metric is blind to
it. Colour alone is equally useless here: Skadi's dress is a huge red field, so any "count the
red pixels" test reports that we OVER-render. The discriminating property is GEOMETRY — a
thread is warm, THIN in the vertical direction, and LONG in the horizontal one.

Method, per frame:
  1. warm mask            r - g > WARM  (and a floor on r, so near-black noise is excluded)
  2. thin                 the warmth must STOP within THICK rows both above and below, so a
                          broad field (the dress) fails while a ridge passes
  3. long                 horizontal erosion of length RUN, so only structures that actually
                          span RUN columns survive

Static UI columns are DERIVED from the capture's own temporal variance (a column that never
changes across the clip is chrome), never hardcoded — see the recorder/UI-border traps.

Usage:  linets.py <ours_dir> <game_mp4> <fps> [--offset S]
        ours_dir holds tN.NN.png frames named by their absolute time.
"""
import sys, subprocess, re, pathlib
import numpy as np
from PIL import Image

WARM, RFLOOR, THICK, RUN = 22, 55, 7, 34


def game_frames(mp4):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", mp4, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
        capture_output=True).stdout
    n = len(raw) // (416 * 900 * 3)
    return np.frombuffer(raw[: n * 416 * 900 * 3], np.uint8).reshape(n, 416, 900, 3)


def static_cols(frames):
    """Columns whose content never varies across the clip = capture chrome."""
    v = frames.astype(np.float32).std(axis=0).mean(axis=(0, 2))
    return v < 1.0


def ridge_mask(img, keep_cols):
    r = img[..., 0].astype(np.int16)
    g = img[..., 1].astype(np.int16)
    warm = ((r - g) > WARM) & (r > RFLOOR)
    warm &= keep_cols[None, :]
    # thin: warmth must stop within THICK rows above AND below
    up = np.zeros_like(warm)
    dn = np.zeros_like(warm)
    for k in range(1, THICK + 1):
        up |= ~np.roll(warm, k, axis=0)
        dn |= ~np.roll(warm, -k, axis=0)
    thin = warm & up & dn
    # long: survive horizontal erosion of length RUN
    out = thin.copy()
    for k in range(1, RUN):
        out &= np.roll(thin, k, axis=1)
    # dilate back so the count reflects the ridge, not just its right end
    grown = np.zeros_like(out)
    for k in range(RUN):
        grown |= np.roll(out, -k, axis=1)
    return grown & thin


def main():
    ours_dir, mp4, fps = sys.argv[1], sys.argv[2], float(sys.argv[3])
    off = 0.0
    if "--offset" in sys.argv:
        off = float(sys.argv[sys.argv.index("--offset") + 1])
    G = game_frames(mp4)
    keep = ~static_cols(G)
    print(f"derived {int((~keep).sum())} static chrome columns of {G.shape[2]}")
    print(f"{'t':>7} {'ours':>7} {'game':>7} {'ratio':>7}")
    tot_o = tot_g = 0
    for f in sorted(pathlib.Path(ours_dir).glob("t*.png"),
                    key=lambda p: float(re.findall(r"t([\d.]+)\.png", p.name)[0])):
        t = float(re.findall(r"t([\d.]+)\.png", f.name)[0])
        idx = int(round((t + off) * fps))
        if not (0 <= idx < len(G)):
            continue
        ours = np.asarray(Image.open(f).convert("RGB"), np.uint8)
        gm = G[idx]
        h = min(ours.shape[0], gm.shape[0])
        o = int(ridge_mask(ours[:h], keep).sum())
        g = int(ridge_mask(gm[:h], keep).sum())
        tot_o += o
        tot_g += g
        print(f"{t:7.2f} {o:7} {g:7} {(g / o if o else float('inf')):7.2f}")
    print(f"{'MEAN':>7} {tot_o:7} {tot_g:7} {(tot_g / tot_o if tot_o else float('inf')):7.2f}"
          f"   <- game/ours; 1.00 = parity")


if __name__ == "__main__":
    main()
