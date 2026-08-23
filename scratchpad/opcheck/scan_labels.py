#!/usr/bin/env python3
"""Per-LABEL-BOX OCR scan of a FLOT brand grid (native 2340x1080).

Whole-screen OCR only reads ~14 of the tile labels in a whole collection (thin
light-grey type). This crops each label box individually and upscales it, which
actually reads every tile. Label bands are detected dynamically per page, so it
survives arbitrary scroll positions.

  scan_labels.py [target ...]      # prints every label; announces a target hit
"""
import subprocess, sys, io, time, os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TARGETS = [t.lower() for t in sys.argv[1:]]

def cap():
    r = subprocess.run(["adb","exec-out","screencap","-p"], capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("RGB") if len(r) > 1000 else None
def swipe():
    subprocess.run(["adb","shell","input","swipe","1170","800","1170","420","300"], capture_output=True)
def ocr_box(img):
    """psm 6 on the full box, take the FIRST line: each label box holds the skin
    name on top and the brand ("EPOQUE") beneath; psm 7 grabbed the wrong one."""
    img.resize((img.width*4, img.height*4), Image.LANCZOS).save(f"{HERE}/_sl.png")
    t = subprocess.run(["tesseract", f"{HERE}/_sl.png", "-", "--psm", "6"],
                       capture_output=True).stdout.decode("utf-8","ignore")
    lines = [l.strip() for l in t.splitlines() if l.strip()]
    return lines[0] if lines else ""

def bands_and_cols(gray):
    """Detect label-row bands and the 7 label-box x-ranges within them."""
    rb = (gray > 235).mean(axis=1)
    bands, run = [], None
    for y, v in enumerate(rb):
        if v > 0.30 and run is None: run = y
        elif v <= 0.30 and run is not None:
            if 25 <= y - run <= 90: bands.append((run, y))
            run = None
    out = []
    for (y0, y1) in bands:
        strip = (gray[y0:y1] > 235).mean(axis=0)
        cols, run = [], None
        for x, v in enumerate(strip):
            if v > 0.5 and run is None: run = x
            elif v <= 0.5 and run is not None:
                if x - run > 120: cols.append((run, x))
                run = None
        if cols: out.append((y0, y1, cols))
    return out

def main():
    seen, prev_sig = set(), ""
    for page in range(20):
        im = cap()
        if im is None: return 1
        g = np.asarray(im.convert("L")).astype(float)
        found = bands_and_cols(g)
        page_labels = []
        for (y0, y1, cols) in found:
            for (x0, x1) in cols:
                txt = ocr_box(im.crop((x0, y0, x1, y1)))
                if not txt or len(txt) < 3: continue
                page_labels.append(txt)
                seen.add(txt)
                for t in TARGETS:
                    if t in txt.lower():
                        cx = (x0 + x1) // 2
                        ty = max(0, y0 - 150)          # tile sits above its label
                        print(f"\n>>> HIT '{txt}' → tap tile at ({cx},{ty})  [page {page}]")
                        return 0
        print(f"page {page}: {len(page_labels)} labels | {' · '.join(page_labels)[:150]}")
        sig = "|".join(page_labels)
        if sig and sig == prev_sig:
            print("(bottom of collection)"); break
        prev_sig = sig
        swipe(); time.sleep(1.6)
    print(f"\ntotal distinct labels read: {len(seen)}")
    if TARGETS: print("target(s) NOT found:", TARGETS)
    return 2

sys.exit(main())
