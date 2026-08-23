#!/usr/bin/env python3
"""Scroll the FLOT brand grid and tap the tile whose label matches a target brand.
Brand labels are WHITE-ON-DARK bars (inverse of skin tiles) at fixed columns, so we
invert before OCR and use known column centres instead of fragile run-detection.
Never use a hardcoded tap: the brand grid RETAINS its scroll position."""
import subprocess, io, sys, time
import numpy as np
from PIL import Image, ImageOps
TARGET=sys.argv[1].lower() if len(sys.argv)>1 else "epoque"
COLS=[412,790,1168,1546,1926]; W=170
def cap():
    r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("RGB") if len(r)>1000 else None
def bands(g):
    rd=(g<90).mean(axis=1); out=[];run=None
    for y,v in enumerate(rd):
        if v>0.25 and run is None: run=y
        elif v<=0.25 and run is not None:
            if 25<=y-run<=70: out.append((run,y))
            run=None
    return [b for b in out if b[0]>120]      # skip the top nav bar
def ocr(im,x,y0,y1):
    c=ImageOps.invert(im.crop((max(0,x-W),y0,min(im.width,x+W),y1)).convert("L"))
    c.resize((c.width*4,c.height*4),Image.LANCZOS).save("_bt.png")
    return subprocess.run(["tesseract","_bt.png","-","--psm","7"],capture_output=True).stdout.decode("utf-8","ignore").strip()
def up():   subprocess.run(["adb","shell","input","swipe","1170","420","1170","800","250"],capture_output=True)
def down(): subprocess.run(["adb","shell","input","swipe","1170","760","1170","480","900"],capture_output=True)
for _ in range(12): up(); time.sleep(0.3)     # brand grid to top
time.sleep(2)
for page in range(10):
    im=cap()
    if im is None: sys.exit(1)
    g=np.asarray(im.convert("L")).astype(float)
    found=[]
    for (y0,y1) in bands(g):
        for cx in COLS:
            t=ocr(im,cx,y0,y1)
            if not t: continue
            found.append(t)
            if TARGET in t.lower():
                ty=max(0,y0-220)
                print(f">>> '{t}' at ({cx},{ty}) page {page} — tapping")
                subprocess.run(["adb","shell","input","tap",str(cx),str(ty)],capture_output=True)
                time.sleep(6)
                sys.exit(0)
    print(f"page {page}: {found}")
    down(); time.sleep(1.5)
print("brand not found"); sys.exit(2)
