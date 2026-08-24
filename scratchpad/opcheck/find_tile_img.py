#!/usr/bin/env python3
"""Locate a specific skin's tile in a FLOT row by IMAGE matching, not OCR.

Unowned tiles are faded behind a grey "Not Yet Owned" overlay, so their labels OCR
into garbage and fuzzy text matching yields false positives ("Miraculous Moment"
matched "Dream In A Moment" at 0.60). Matching the tile ARTWORK instead is immune
to that: we already own every skin's portrait via the backend.

Uses zero-mean/unit-variance normalised cross-correlation, which is invariant to the
brightness/contrast shift the fade applies.

  find_tile_img.py <bundle> [<bundle> ...] [--charid=auto] [--steps=14] [--thresh=0.35]
      --horiz   scroll the On Sale strip horizontally (default: vertical grid)
"""
import subprocess, sys, io, time, os, re, urllib.request, urllib.parse
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
BUNDLES = [a for a in sys.argv[1:] if not a.startswith("--")]
STEPS  = int(next((a.split("=")[1] for a in sys.argv if a.startswith("--steps=")), 14))
THRESH = float(next((a.split("=")[1] for a in sys.argv if a.startswith("--thresh=")), 0.35))
HORIZ  = any(a == "--horiz" for a in sys.argv)
BACKEND = "http://localhost:3060/api"

def portrait(bundle):
    """Fetch the skin portrait (the same art the FLOT tile shows)."""
    url = f"{BACKEND}/skin-portrait/{urllib.parse.quote(bundle, safe='')}"
    with urllib.request.urlopen(url, timeout=20) as r:
        return Image.open(io.BytesIO(r.read())).convert("L")

def cap():
    r = subprocess.run(["adb","exec-out","screencap","-p"], capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("RGB") if len(r) > 1000 else None

def norm(a, size=(48, 88)):
    a = np.asarray(a.resize(size, Image.LANCZOS)).astype(np.float32)
    s = a.std()
    return (a - a.mean()) / (s if s > 1e-6 else 1.0)

def ncc(a, b):           # both pre-normalised, same shape
    return float((a * b).mean())

def tiles(im):
    """Tile boxes: label bands locate the columns; the artwork sits ABOVE each band."""
    g = np.asarray(im.convert("L")).astype(float)
    rb = (g > 235).mean(axis=1)
    bands, run = [], None
    for y, v in enumerate(rb):
        if v > 0.30 and run is None: run = y
        elif v <= 0.30 and run is not None:
            if 25 <= y - run <= 90: bands.append((run, y))
            run = None
    out = []
    for (y0, y1) in bands:
        strip = (g[y0:y1] > 235).mean(axis=0)
        cols, run = [], None
        for x, v in enumerate(strip):
            if v > 0.5 and run is None: run = x
            elif v <= 0.5 and run is not None:
                if x - run > 120: cols.append((run, x))
                run = None
        ty0 = max(0, y0 - 500)
        for (x0, x1) in cols:
            out.append((x0, ty0, x1, y0))
    return out

def scroll(horiz, tile_y=None):
    if horiz:
        y = tile_y if tile_y else 200
        subprocess.run(["adb","shell","input","swipe","1800",str(y),"400",str(y),"600"], capture_output=True)
    else:
        subprocess.run(["adb","shell","input","swipe","1170","760","1170","480","900"], capture_output=True)

def main():
    refs = {}
    for b in BUNDLES:
        try:
            refs[b] = norm(portrait(b)); print(f"[img] template loaded: {b}")
        except Exception as e:
            print(f"[img] ⛔ portrait fetch failed for {b}: {e}"); return 1
    prev = ""
    for step in range(STEPS):
        im = cap()
        if im is None: return 1
        boxes = tiles(im)
        best = (0.0, None, None)
        sig = []
        for (x0, y0, x1, y1) in boxes:
            crop = norm(im.crop((x0, y0, x1, y1)).convert("L"))
            for b, t in refs.items():
                s = ncc(crop, t)
                sig.append(f"{s:.2f}")
                if s > best[0]: best = (s, b, ((x0+x1)//2, (y0+y1)//2))
        print(f"step {step}: {len(boxes)} tiles, best NCC {best[0]:.3f}" + (f" ({best[1]})" if best[1] else ""))
        if best[0] >= THRESH:
            print(f">>> MATCH {best[1]}  NCC={best[0]:.3f}  tap=({best[2][0]},{best[2][1]})")
            return 0
        s2 = "|".join(sig)
        if s2 == prev:
            print("(view stopped changing)"); break
        prev = s2
        ty = boxes[0][1] + 250 if (HORIZ and boxes) else None
        scroll(HORIZ, ty); time.sleep(1.4)
    print("[img] no match above threshold")
    return 2

sys.exit(main())
