#!/usr/bin/env python3
"""Exhaustively tap every tile in the EPOQUE grid and OCR the PREVIEW page.
Preview OCR is reliable (unlike faded tile labels / image matching), and the
collection is small (~50 skins), so brute force is practical and correct."""
import subprocess, io, sys, time, json
import numpy as np
from PIL import Image
TGT={k.lower():v for k,v in json.load(open("scoreable_skins.json")).items()}
WANT=[t.lower() for t in sys.argv[1:]] or list(TGT)
def cap():
    r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("RGB") if len(r)>1000 else None
def tap(x,y): subprocess.run(["adb","shell","input","tap",str(x),str(y)],capture_output=True)
def back(): subprocess.run(["adb","shell","input","keyevent","4"],capture_output=True)
def scroll(): subprocess.run(["adb","shell","input","swipe","1170","760","1170","480","900"],capture_output=True)
def preview_text():
    im=cap()
    if im is None: return ""
    rt=im.crop((int(im.width*0.55),0,im.width,im.height))
    rt.resize((rt.width*2,rt.height*2),Image.LANCZOS).save("_pv.png")
    return subprocess.run(["tesseract","_pv.png","-","--psm","3"],capture_output=True).stdout.decode("utf-8","ignore").lower()
def rows(im):
    g=np.asarray(im.convert("L")).astype(float)
    rb=(g>235).mean(axis=1); bands=[];run=None
    for y,v in enumerate(rb):
        if v>0.30 and run is None: run=y
        elif v<=0.30 and run is not None:
            if 70<=y-run<=90: bands.append((run,y))
            run=None
    out=[]
    for (y0,y1) in bands:
        if y0<400: continue                      # tile above must be on-screen
        st=(g[y0:y1]>235).mean(axis=0); cols=[];run=None
        for x,v in enumerate(st):
            if v>0.5 and run is None: run=x
            elif v<=0.5 and run is not None:
                if x-run>120: cols.append(((run+x)//2, y0-250))
                run=None
        out.extend(cols)
    return out
seen=set()
for page in range(26):
    im=cap()
    if im is None: break
    cells=rows(im)
    print(f"page {page}: {len(cells)} tiles")
    for (cx,cy) in cells:
        key=(page,cx)
        if key in seen: continue
        seen.add(key)
        tap(cx,cy); time.sleep(3.2)
        t=preview_text()
        if "outfit" not in t:                    # didn't open a preview
            continue
        name=None
        for w in WANT:
            if w in t: name=w; break
        shown=" ".join(t.split())[:60]
        print(f"   tile x{cx}: {shown}")
        if name:
            print(f">>> FOUND '{name}' -> {TGT[name]} at ({cx},{cy}) page {page}")
            sys.exit(0)
        back(); time.sleep(2.2)
    scroll(); time.sleep(1.6)
print("not found"); sys.exit(2)
