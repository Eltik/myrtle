#!/usr/bin/env python3
"""Auto-find the first SCOREABLE dynamic (animated L2D) skin in the open FLOT lookbook.

Scoreable = the skin is DYNAMIC (has an entrance) AND its bundle is in the release
manifest (so we can export it and score against it). Upcoming / "Test Collection"
previews (e.g. Lappland "Unruly Humbleness", Ling "A Gallant Dream") are NOT in the
manifest and are skipped.

For each candidate cell it OCRs the preview page's skin name (tesseract), matches it
against scoreable_skins.json (built from the current skin_table ∩ device manifest),
and only then opens the viewer and motion-checks it. Prints `FOUND <bundle> | <skin>`
and leaves the game IN the viewer, ready to record. Exit 2 if none found.

Usage: python3 autofind_l2d.py [--max-scroll=N] [--motion=T]
"""
import subprocess, sys, time, io, json, os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SCOREABLE = json.load(open(f"{HERE}/scoreable_skins.json"))          # {skinName: bundle}
SCLOW = {n.lower(): b for n, b in SCOREABLE.items()}
MAX_SCROLL = int(next((a.split("=")[1] for a in sys.argv if a.startswith("--max-scroll=")), 8))
MOTION_T   = float(next((a.split("=")[1] for a in sys.argv if a.startswith("--motion=")), 5.0))
COLS = [76, 228, 377, 521, 665]
ROWS = [180, 300, 420]
MAG  = (29, 498)

def sh(*a): subprocess.run(["adb","shell",*a], capture_output=True)
def _size():
    o=subprocess.run(["adb","shell","wm","size"],capture_output=True,text=True).stdout
    m=[x for x in o.replace("\r","").split("\n") if "x" in x]
    w,h=m[-1].split(":")[-1].strip().split("x"); return int(w),int(h)
_W,_H=_size(); SCALE_X,SCALE_Y=_W/1170.0,_H/540.0   # coords authored at 1170x540
def tap(x,y): sh("input","tap",str(int(x*SCALE_X)),str(int(y*SCALE_Y)))
def back(): sh("input","keyevent","4")
def scroll_down(): sh("input","swipe",str(int(500*SCALE_X)),str(int(430*SCALE_Y)),str(int(500*SCALE_X)),str(int(110*SCALE_Y)),"350")
def grab():
    r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
    return np.asarray(Image.open(io.BytesIO(r)).convert("L")).astype(np.int16) if len(r)>1000 else None
def diff(a,b):
    if a is None or b is None: return 0.0
    return 999.0 if a.shape!=b.shape else float(np.abs(a-b).mean())
def ocr():
    """OCR the current screen. Tesseract on this UI is flaky, so try several
    preprocessings and return the first non-trivial read (retrying the grab once)."""
    for attempt in range(2):
        r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
        if len(r) < 1000: time.sleep(1.0); continue
        im=Image.open(io.BytesIO(r)).convert("RGB")
        # the preview's identifying text lives in the RIGHT panel ("Operator X Outfit")
        right=im.crop((int(im.width*0.55), 0, im.width, im.height))
        for src in (right, im):
            for scale in (3, 2):
                src.resize((src.width*scale, src.height*scale), Image.LANCZOS).save(f"{HERE}/_af_ocr.png")
                t=subprocess.run(["tesseract",f"{HERE}/_af_ocr.png","-","--psm","3"],
                                 capture_output=True).stdout.decode("utf-8","ignore").lower()
                if len(t.split()) >= 3:
                    return t
        time.sleep(0.8)
    return ""
def match_scoreable(text):
    for name, bundle in SCLOW.items():
        if len(name) >= 5 and name in text:            # skin name appears on the preview
            return SCOREABLE_ORIG.get(name, name), bundle
    return None, None
SCOREABLE_ORIG = {n.lower(): n for n in SCOREABLE}

def is_preview(text):
    # every skin PREVIEW page says "Operator <X> Outfit" — the lookbook grid does NOT.
    return "outfit" in text and ("operator" in text or "preview only" in text)
def is_lookbook(gray=None):
    """NON-OCR: the lookbook's centre is a flat bright grey panel (std~6, >99% bright);
    previews/menus are dense art (std 60-70, ~20-26% bright). OCR fails on this
    thin light-grey-on-white UI entirely, so key on the pixel signature instead."""
    g = grab() if gray is None else gray
    if g is None: return False
    # Key on the FLOT HEADER strip, which is SCROLL-INVARIANT (identical at top and
    # scrolled: 93.3% bright) unlike the content band, whose density grows as rows
    # fill and caused false bails. Preview/menu headers are 46-70% bright.
    h = g[int(8*SCALE_Y):int(44*SCALE_Y), int(300*SCALE_X):int(1000*SCALE_X)].astype(float)
    return bool((h > 200).mean() > 0.85)

def main():
    print(f"[find] {len(SCOREABLE)} scoreable dynamic skins loaded")
    for sp in range(MAX_SCROLL):
        for (x,y) in [(x,y) for y in ROWS for x in COLS]:
            tap(x,y); time.sleep(2.5)
            txt = ocr()
            if not is_preview(txt):
                # tap didn't open a preview (animated grid thumb / empty cell). NEVER back here.
                if not is_lookbook():
                    print(f"[find] drifted off lookbook — bailing to avoid exit-dialog"); return 3
                continue
            # a PREVIEW is confirmed open → safe to inspect + back
            name, bundle = match_scoreable(txt)
            if not bundle:
                print(f"[find] scroll {sp} ({x},{y}) preview — not scoreable/released, skip")
                back(); time.sleep(1.5); continue
            tap(*MAG); time.sleep(2.5)                  # open viewer
            a = grab(); time.sleep(1.1); b = grab(); m = diff(a,b)
            print(f"[find] scroll {sp} ({x},{y}) '{name}' -> {bundle}, viewer motion Δ={m:.1f}")
            if m >= MOTION_T:
                print(f"FOUND {bundle} | {name}"); return 0
            back(); time.sleep(1.5); back(); time.sleep(2)   # viewer→preview→lookbook
        scroll_down(); time.sleep(1.2)
    print("[find] no scoreable dynamic skin found in scan bounds"); return 2

sys.exit(main())
