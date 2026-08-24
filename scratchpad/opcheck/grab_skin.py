#!/usr/bin/env python3
"""Find a named skin in the CURRENT FLOT brand grid, open it, and capture its ENTRANCE.

Assumes the brand collection grid (e.g. EPOQUE) is on screen at native 2340x1080.
Scrolls the grid, OCRs each tile's LABEL box to locate the target, taps it, then runs
the verified recipe: record → magnifier → reveal chrome → ▶ Play → assert luma blackout
→ anchor t0 → write 900x416/30 reference.

  grab_skin.py "<Skin Name>" <outname> [--dur=34] [--max-page=14]
"""
import subprocess, sys, io, time, os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
NAME   = sys.argv[1]
OUT    = sys.argv[2]
DUR    = int(next((a.split("=")[1] for a in sys.argv if a.startswith("--dur=")), 34))
MAXPG  = int(next((a.split("=")[1] for a in sys.argv if a.startswith("--max-page=")), 14))
RAW    = f"/private/tmp/ak_{OUT}_raw.mp4"

# grid geometry measured at native 2340x1080 (7 cols x 2 visible rows)
COLX   = [266, 568, 870, 1172, 1474, 1776, 2078]
TILEY  = [220, 720]           # tile centres
LABY   = [(350, 400), (1012, 1062)]   # label box y-ranges per row
MAG    = (72, 1008); REVEAL = (1170, 540); PLAY = (1971, 1011)

def sh(*a): return subprocess.run(["adb","shell",*a], capture_output=True)
def tap(x,y): sh("input","tap",str(x),str(y))
def swipe(): sh("input","swipe","1170","800","1170","420","300")
def cap():
    r=subprocess.run(["adb","exec-out","screencap","-p"],capture_output=True).stdout
    return Image.open(io.BytesIO(r)).convert("RGB") if len(r)>1000 else None
def ocr(img, scale=3):
    img.resize((img.width*scale, img.height*scale), Image.LANCZOS).save(f"{HERE}/_gs.png")
    return subprocess.run(["tesseract",f"{HERE}/_gs.png","-","--psm","6"],
                          capture_output=True).stdout.decode("utf-8","ignore").lower()

def find_tile(im, target):
    """OCR each label box; return (x,y) tile centre whose label contains `target`."""
    for r,(y0,y1) in enumerate(LABY):
        for c,cx in enumerate(COLX):
            box = im.crop((max(0,cx-140), y0, min(im.width,cx+140), y1))
            if target in ocr(box):
                return COLX[c], TILEY[r]
    return None

def main():
    tgt = NAME.lower()
    for page in range(MAXPG):
        im = cap()
        if im is None: return 1
        if tgt in ocr(im.resize((im.width//2, im.height//2)), scale=2):   # cheap page check
            pos = find_tile(im, tgt)
            if pos:
                print(f"[grab] '{NAME}' found at tile {pos} (page {page})")
                tap(*pos); time.sleep(5)
                got = ocr(cap().crop((int(2340*0.55),0,2340,1080)), scale=2)
                print(f"[grab] preview says: {' '.join(got.split())[:90]}")
                if tgt not in got:
                    print("[grab] ⛔ opened the WRONG skin"); return 2
                break
            print(f"[grab] page {page}: name on page but tile not located, scrolling")
        swipe(); time.sleep(1.7)
    else:
        print(f"[grab] ⛔ '{NAME}' not found in {MAXPG} pages"); return 2

    # ---- capture ----
    sh("rm","-f","/sdcard/g.mp4")
    subprocess.Popen(["adb","shell",f"screenrecord --bit-rate 100000000 --time-limit {DUR+8} /sdcard/g.mp4"])
    time.sleep(1.5)
    print("[grab] magnifier"); tap(*MAG);    time.sleep(4)
    print("[grab] reveal");    tap(*REVEAL); time.sleep(1.3)
    print("[grab] ▶ PLAY");    tap(*PLAY)
    time.sleep(DUR)
    sh("pkill","-INT","screenrecord"); time.sleep(3)
    subprocess.run(["adb","pull","/sdcard/g.mp4",RAW],capture_output=True)

    # ---- verify + process ----
    pr=subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries",
        "stream=width,height","-of","csv=p=0",RAW],capture_output=True,text=True).stdout.strip().split(",")
    W,H=int(pr[0]),int(pr[1])
    dur=float(subprocess.run(["ffprobe","-v","error","-show_entries","format=duration","-of","csv=p=0",RAW],
        capture_output=True,text=True).stdout.strip() or 0)
    g=subprocess.run(["ffmpeg","-v","error","-i",RAW,"-f","rawvideo","-pix_fmt","gray","-"],capture_output=True).stdout
    n=len(g)//(W*H); G=np.frombuffer(g,np.uint8)[:n*W*H].reshape(n,H,W); fps=n/dur if dur else 60
    lum=G.reshape(n,-1).mean(axis=1)
    print(f"[grab] {W}x{H} {n}f {dur:.1f}s {fps:.0f}fps luma min={lum.min():.2f}")
    dark=np.where(lum<6)[0]
    if len(dark)==0:
        print("[grab] ⛔ NO BLACKOUT — ▶ missed revealed chrome. RETAKE."); return 3
    b1=int(dark[-1]); post=np.where(lum[b1:]>12)[0]
    t0=int(b1+post[0]) if len(post) else b1
    print(f"[grab] blackout {dark[0]/fps:.2f}-{b1/fps:.2f}s → t0={t0/fps:.3f}s ✅")
    t0s=t0/fps
    fresh=f"{HERE}/REF_NEW/{OUT}_game_fresh.mp4"; ent=f"{HERE}/REF_NEW/{OUT}_entrance.mp4"
    subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",RAW,"-c","copy",ent])
    subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",RAW,
        "-vf","scale=900:416:flags=lanczos,fps=30","-c:v","libx264","-crf","12","-pix_fmt","yuv420p",fresh])
    print(f"[grab] ✅ wrote {OUT}_game_fresh.mp4")
    return 0

sys.exit(main())
