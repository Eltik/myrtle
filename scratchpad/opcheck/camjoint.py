#!/usr/bin/env python3
"""Test a page-camera anchor hypothesis as a CONSTRAINED JOINT FIT, not as an eyeballed spread.

Free per-subject centres are the ceiling. Re-solve with the hypothesis imposed (one shared world Y
across all subjects, or the horizontal pinned to `bodyCx`) and compare total NCC. A constraint that
costs little is supported; one that costs a subject's worth of correlation is refuted.

Measured at n=4: free total 2.3209, shared constant Y 1.7839 (cost 0.5370), horizontal pinned to
bodyCx 1.9987 (cost 0.3223). Mean free NCC is 0.580 per subject, so the shared-Y constraint costs
nearly one whole subject. Both anchors refuted.

⚠️ A spread computed from INDEPENDENT solves was always going to be luck at small n; that is how a
3.10 px "constant" survived three points and died on the fourth.
"""
from PIL import Image
import numpy as np, math
from numpy.fft import rfft2, irfft2
SD="/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/751065f0-d8ca-4714-a615-795c1f6925be/scratchpad"
W,H=1474,1080; DS=4; MARGIN=0.95
SUBJ={
 "cet":    dict(cb=(-883.2265553474426,-1763.805726369222,1748.7519062360127,2061.442644437154), bodyCx=50.148,  k=1.2922, ph="3.00"),
 "whitw2": dict(cb=(-995.4691594441732,-1606.498878479004,2043.3466669718425,1836.7161051432292), bodyCx=8.888,  k=1.1799, ph="6.00"),
 "eyja":   dict(cb=(-1028.1900634765625,-1342.0552164713542,2059.966796875,1351.8532104492188),  bodyCx=-2.599, k=1.1266, ph="2.00"),
 "reed2":  dict(cb=(-970.2621459960938,-1486.72119140625,2045.3073263168335,1493.3990001678467), bodyCx=69.147, k=1.0710, ph="1.00"),
}
def lum(a): return 0.299*a[...,0]+0.587*a[...,1]+0.114*a[...,2]
CACHE={}
def prep(key):
    if key in CACHE: return CACHE[key]
    p1=np.asarray(Image.open(f"{SD}/page_{key}_1.png").convert("RGB"))[0:H,0:W].astype(int)
    p2=np.asarray(Image.open(f"{SD}/page_{key}_2.png").convert("RGB"))[0:H,0:W].astype(int)
    mask=(np.abs(p1-p2).max(axis=2)>10)
    gw,gh=W//DS,H//DS
    Ps=np.asarray(Image.fromarray(lum(p1.astype(np.float32)).astype("uint8")).resize((gw,gh),Image.LANCZOS)).astype(np.float32)
    ms=np.asarray(Image.fromarray((mask*255).astype("uint8")).resize((gw,gh),Image.NEAREST))>128
    g=np.zeros_like(Ps); gv=Ps[ms]; g[ms]=(gv-gv.mean())/(gv.std()+1e-9)
    im=Image.open(f"{SD}/ph_{key}/t{SUBJ[key]['ph']}.png").convert("RGB")
    CACHE[key]=(g,ms,gw,gh,im); return CACHE[key]
def grid(key,kk):
    g,ms,gw,gh,im=prep(key)
    w=max(gw,int(round(gw*kk))); h=max(gh,int(round(gh*kk)))
    Rs=lum(np.asarray(im.resize((w,h),Image.LANCZOS)).astype(np.float32))
    return Rs,g,ms,gw,gh,w,h
def ncc_at(key,kk,dxp,dyp):
    Rs,g,ms,gw,gh,w,h=grid(key,kk)
    dx=int(round(dxp/DS)); dy=int(round(dyp/DS))
    if dx<0 or dy<0 or dy+gh>h or dx+gw>w: return -9
    a=Rs[dy:dy+gh, dx:dx+gw]; av=a[ms]
    if av.std()<1e-6: return -9
    az=(av-av.mean())/av.std()
    return float((az*g[ms]).mean())
def best_free(key,kk):
    Rs,g,ms,gw,gh,w,h=grid(key,kk)
    gpad=np.zeros((h,w),np.float32); gpad[:gh,:gw]=g
    cc=irfft2(rfft2(Rs)*np.conj(rfft2(gpad)), s=(h,w))
    idx=int(np.argmax(cc)); dy,dx=divmod(idx,w)
    return ncc_at(key,kk,dx*DS,dy*DS), dx*DS, dy*DS
def sc_of(d):
    cx,cy,cw,ch=d["cb"]; return min(W/cw,H/ch)*MARGIN
def dxy_for_centre(d,kk,px,py):
    cx,cy,cw,ch=d["cb"]; ccx=cx+cw/2; ccy=cy+ch/2; sc=sc_of(d)
    dx=(px-ccx+W/(2*sc))*(kk*sc)-W/2
    dy=(py-ccy+H/(2*sc))*(kk*sc)-H/2
    return dx,dy
print("JOINT CONSTRAINED FITS. Free per-subject centres are the ceiling; a constraint that")
print("costs little NCC is supported, one that costs a lot is refuted.\n")
free={}
for key,d in SUBJ.items():
    v,dx,dy=best_free(key,d["k"])
    cx,cy,cw,ch=d["cb"]; sc=sc_of(d)
    px=cx+cw/2+(W/2+dx)/(d["k"]*sc)-W/(2*sc)
    py=cy+ch/2+(H/2+dy)/(d["k"]*sc)-H/(2*sc)
    free[key]=(v,px,py)
tot_free=sum(v for v,_,_ in free.values())
print(f"{'skin':7s} {'free ncc':>9s} {'free x':>9s} {'free y':>9s}")
for key in SUBJ: print(f"{key:7s} {free[key][0]:9.4f} {free[key][1]:9.1f} {free[key][2]:9.1f}")
print(f"{'TOTAL':7s} {tot_free:9.4f}\n")
# shared constant Y
best=(-99,None)
for Y in np.arange(-760,-600,1.0):
    t=0
    for key,d in SUBJ.items():
        # free dx at the dy the shared Y implies
        Rs,g,ms,gw,gh,w,h=grid(key,d["k"])
        _,dyc=dxy_for_centre(d,d["k"],0,Y)
        bestv=-9
        for dxs in range(0,max(1,w-gw+1)):
            v=ncc_at(key,d["k"],dxs*DS,dyc)
            if v>bestv: bestv=v
        t+=bestv
    if t>best[0]: best=(t,Y)
print(f"SHARED CONSTANT Y: best Y {best[1]:.1f}, total ncc {best[0]:.4f} vs free {tot_free:.4f}, cost {tot_free-best[0]:.4f}")
# shared horizontal = bodyCx
t=0
for key,d in SUBJ.items():
    _,dyf=dxy_for_centre(d,d["k"],free[key][1],free[key][2])
    dxc,_=dxy_for_centre(d,d["k"],d["bodyCx"],0)
    t+=ncc_at(key,d["k"],dxc,dyf)
print(f"HORIZONTAL PINNED TO bodyCx: total ncc {t:.4f} vs free {tot_free:.4f}, cost {tot_free-t:.4f}")
