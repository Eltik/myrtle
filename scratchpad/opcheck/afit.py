"""ANISOTROPIC registration fit — (sx, sy, dx, dy) of our frame against a game frame.

Why anisotropic: an ISOTROPIC scale fit cannot see the reference clips' geometric distortion
(`capture_oracle.sh` forces `scale=900:416` from a 2340x1080 device frame, stretching every
reference vertically by 1.001481). It splits that error across both axes and reports ~0.999,
which reads as a framing bug and sends you hunting in the renderer. Split into (sx, sy) it is
unmistakable: sy lands on 1.0000 and sx on 0.9985 = 1/1.001481.
See `dynchar-reference-clips-aspect-distorted`.

⚠️ The obvious implementation — PIL resize then centre-crop — is WRONG for s<1: it zero-pads,
and the black border flatters dark regions. That bug reported `0.9955, gain 2.53` at Mlynar's
t=11.8 where the correct centred warp says `0.9910, gain 8.14`. Warp with an inner margin.

TWO-OFFSET USE — the sensitive instrument for a clock offset (see `dynchar-recorder-dt-clamp` #8):
render the same beats at two capture offsets, fit both, and take each beat's zero crossing of
sy -> 1.0. The SENSITIVITY (d sy / d offset) is proportional to the camera's zoom RATE, so beats
where the camera is static cannot constrain the offset AT ALL — and a MADC trim sweep, which those
beats dominate, will confidently report the wrong answer. Executor is the trap case: her fastest
zoom beat has sensitivity exactly 0.0000 because her scope APERTURE is pinned to the camera centre
and dominates the fit.

    import sys; sys.path.insert(0, "<opcheck>")
    from afit import y, fit
    sx, sy, dx, dy, mad = fit(our_luma, game_luma)
"""

import numpy as np


def y(a):
    """BT.601 luma from an HxWx3 float array."""
    return 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]


def warp(img,sx,sy,dx,dy):
    H,W=img.shape; cx,cy=W/2.0,H/2.0
    ys,xs=np.mgrid[0:H,0:W]
    ux=np.clip((xs-cx-dx)/sx+cx,0,W-1.001); uy=np.clip((ys-cy-dy)/sy+cy,0,H-1.001)
    x0=ux.astype(int); y0=uy.astype(int); fx=ux-x0; fy=uy-y0
    return (img[y0,x0]*(1-fx)*(1-fy)+img[y0,x0+1]*fx*(1-fy)+img[y0+1,x0]*(1-fx)*fy+img[y0+1,x0+1]*fx*fy)
def fit(oy, gy, M=20, maxshift=8):
    """Best (sx, sy, dx, dy, mad). `maxshift` bounds |dx|,|dy| in PIXELS.

    ⚠️ Stage 2 is a GREEDY hill-climb and `best` mutates inside the loop, so without the clamp
    the shift walks far past its seed range — it reached dy=22 on cello, where the frame carries
    ~44 px vertically periodic structure and dy is only determined MODULO 44. Raise `maxshift`
    deliberately when you want a large shift, and cross-check an exhaustive per-block search
    before believing it."""
    sl=(slice(M,oy.shape[0]-M), slice(36+M,864-M))
    def cost(sx,sy,dx,dy): return np.abs(warp(oy,sx,sy,dx,dy)[sl]-gy[sl]).mean()
    best=(1.0,1.0,0,0,cost(1,1,0,0))
    # stage 1: integer shift only
    for dx in range(-5,6):
        for dy in range(-6,7):
            c=cost(1,1,dx,dy)
            if c<best[4]: best=(1.0,1.0,dx,dy,c)
    # stage 2: scales at the found shift, coarse then fine
    for step,rng in ((0.002,0.010),(0.0005,0.0025)):
        sx0,sy0=best[0],best[1]
        for sx in np.arange(sx0-rng,sx0+rng+1e-9,step):
            for sy in np.arange(sy0-rng,sy0+rng+1e-9,step):
                for dx in (best[2]-1,best[2],best[2]+1):
                    for dy in (best[3]-1,best[3],best[3]+1):
                        if abs(dx) > maxshift or abs(dy) > maxshift:
                            continue
                        c=cost(sx,sy,dx,dy)
                        if c<best[4]: best=(sx,sy,dx,dy,c)
    return best
