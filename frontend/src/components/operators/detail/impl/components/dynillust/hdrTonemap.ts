import * as PIXI from "pixi.js";

/**
 * HDR bloom pass for the dynamic-illustration compositor.
 *
 * Dynchar scenes stack many ADDITIVE particle systems / light layers (Hoshiguma's
 * blue ice-flames, Wiš'adel's supernova bloom). Composited in an 8-bit target,
 * additive blending clips each channel at 1.0 — so a stack of blue flames
 * `(0.3,0.6,3.0)` clips to `(1,1,1)` = a hard WHITE blob that blows out the
 * character, instead of the translucent blue glow the game shows. The game
 * accumulates in HDR and tonemaps; we do the same:
 *
 *  1. Render the whole scene into a HALF-FLOAT render target, where additive
 *     blends accumulate PAST 1.0 without clipping (the colour survives).
 *  2. Blit that target to the screen through a hue-preserving highlight
 *     compressor: values at/under the knee pass through untouched (the LDR
 *     backdrop + character are authored ≤1, so they're preserved), while a
 *     stack whose brightest channel exceeds 1 is scaled down as a whole —
 *     preserving its HUE (blue stays blue) and rolling the peak toward 1
 *     instead of clipping to white.
 *
 * Falls back to null when float render targets aren't available (no WebGL2 /
 * `EXT_color_buffer_float`); the caller then composites in 8-bit as before.
 */

const TONEMAP_VERT = `
precision highp float;
attribute vec2 aVertexPosition;
attribute vec2 aUV;
uniform mat3 projectionMatrix;
uniform mat3 translationMatrix;
varying vec2 vUV;
void main() {
    gl_Position = vec4((projectionMatrix * translationMatrix * vec3(aVertexPosition, 1.0)).xy, 0.0, 1.0);
    vUV = aUV;
}
`;

// The scene is rendered PREMULTIPLIED over a transparent (→ page-dark) canvas, so
// `rgb` is the actual light contribution. Compress only the highlight TAIL: below
// `uKnee` it's identity (LDR base untouched); above, scale rgb by `tone(m)/m` where
// `m` is the max channel, so the whole colour dims together (hue preserved) and the
// peak asymptotes to 1 rather than per-channel clipping to white. Finally ADD the
// blurred bloom (the >1 HDR energy spread into a glow), so bright thin additive
// effects (Hoshiguma's ice-crystal beams) read as bold glowing shards the way the
// game's bloom shows them, rather than the raw thin sliver the tonemap clamps to.
const TONEMAP_FRAG = `
precision highp float;
varying vec2 vUV;
uniform sampler2D uSampler;
uniform sampler2D uBloom;
uniform float uKnee;
uniform float uBloomIntensity;
uniform float uGamma;
uniform float uClip;
uniform float uProbe;
uniform sampler2D uCov;
uniform float uCovOn;
uniform float uCovLo;
uniform float uCovHi;
void main() {
    vec4 c = texture2D(uSampler, vUV);
    vec3 rgb = c.rgb;
    float m = max(max(rgb.r, rgb.g), rgb.b);
    // DIAGNOSTIC (?hdrprobe=<scale>): bypass the tonemap and output the RAW half-float
    // target divided by <scale>, so the captured PNG can be read back as HDR magnitude
    // (pixel/255*scale). Answers "does anything actually exceed 1 before the knee?" —
    // which decides whether a clipped highlight is an over-bright stack or already-LDR
    // white passing through the achroma release.
    if (uProbe > 0.0) {
        gl_FragColor = vec4(rgb / uProbe, 1.0);
        return;
    }
    if (m > uKnee) {
        // soft-compress the tail [knee, +inf) -> [knee, 1)
        float e = (m - uKnee) / max(1e-4, 1.0 - uKnee);
        float t = uKnee + (1.0 - uKnee) * (e / (1.0 + e));
        // The knee exists to preserve HUE while taming >1 additive stacks (blue flames
        // stay blue instead of clipping to white). An ACHROMATIC highlight — a fullscreen
        // white reveal-flash — has no hue to preserve, yet the knee still greys it (a
        // pure-white 1.0 maps to ~0.95 = 242, never the game's opaque 255). So release the
        // compression as the pixel approaches neutral: at low saturation it passes through
        // (output-clamped to pure white), while any pixel with real chroma keeps the full
        // hue-preserving compression unchanged. Saturation-gated, not per-skin.
        float mn = min(min(rgb.r, rgb.g), rgb.b);
        float sat = (m > 1e-4) ? (m - mn) / m : 0.0;
        float achroma = 1.0 - smoothstep(0.05, 0.25, sat); // 1 = white/grey, 0 = coloured
        // COVERAGE GATE (?cov=1). The achroma release above exists so a full-frame white
        // REVEAL FLASH reaches an opaque 255 instead of the knee's ~242. But it fires for any
        // near-neutral pixel at/above the knee, so ordinary bright white ART clips too:
        // Virtuosa blows 11.03% of her t=2 frame to pure white where the game blows 1.10%.
        //
        // Magnitude cannot separate the two — measured, Mlynar's HELD FLASH sits at exactly
        // 1.0 just as her interior does, and there the game DOES want 255. The only property
        // that differs is how much of the neighbourhood is white: ~12% scattered vs 100%.
        // uCov is a blurred near-white mask, so it reads ~1 inside a genuine white-out and
        // low around isolated highlights; release only where it is high.
        if (uCovOn > 0.5) {
            float cov = texture2D(uCov, vUV).r;
            achroma *= smoothstep(uCovLo, uCovHi, cov);
        }
        float scale = mix(t / m, 1.0, achroma);
        // DIAGNOSTIC (?tmclip=1): hue-preserving CLIP instead of the soft roll-off.
        //
        // The roll-off maps [knee, inf) -> [knee, 1) with 1.0 as an ASYMPTOTE, so a pixel at
        // exactly 1.0 comes out at 0.95 — a saturated LDR colour is darkened 5% and can never
        // reach 255, even though it is not over-bright at all. (The achroma release rescues only
        // near-NEUTRAL pixels, which is why white flashes were fixed and saturated ones were not.)
        // Measured on Virtuosa's apple: our peak red caps at 241 where the game reaches 255.
        //
        // The clip is identity at m <= 1 and scales by 1/m above, so LDR passes through untouched
        // and over-bright stacks still keep their hue.
        if (uClip > 0.5) scale = (m > 1.0) ? (1.0 / m) : 1.0;
        rgb *= scale;
    }
    // NO cinematic vignette. A brightness-gated radial falloff used to live here, added to
    // match a believed "~209 corner vs 255 centre" falloff on the game's white transition
    // flash. That falloff does not exist: measured on the held flash frames (t = 14.5, 15.0,
    // 15.5, 16.0) the game is pure 255 in all four corners, at the centre, and everywhere
    // between — min 254 over the whole inner window, 0.0% of pixels below 250. The 209 was
    // almost certainly read off a RAMP frame, where the entire frame is uniformly dim
    // (~202 at t = 14) rather than dim only at the corners. The vignette cost us the flash:
    // it held our margins at a flat 237 for the full hold. Do not reintroduce it without
    // re-measuring the HELD frames, not the ramp.
    vec3 bloom = texture2D(uBloom, vUV).rgb * uBloomIntensity;
    vec3 outc = rgb + bloom;
    // Scene-composite transfer (see sceneCompositeGamma). Applied LAST, on the assembled
    // frame, because that is where it was measured. Values above 1 stay above 1 under a
    // >1 exponent and clamp to white exactly as they did before, so this only reshapes
    // the LDR range. uGamma == 1.0 is a no-op and skips the pow entirely.
    if (uGamma != 1.0) {
        outc = pow(max(outc, 0.0), vec3(uGamma));
    }
    gl_FragColor = vec4(outc, max(c.a, 0.0));
}
`;

// Bright-pass: sample the HALF-FLOAT scene target and keep only the energy ABOVE
// the threshold. Because the target is HDR, painted panels authored at ≤1 fall
// under the threshold and never bloom — only additive stacks that accumulated past
// it (blades, flames, gem cores) do. Output the excess, softly.
const BRIGHT_FRAG = `
precision highp float;
varying vec2 vUV;
uniform sampler2D uSampler;
uniform float uThreshold;
void main() {
    vec3 rgb = texture2D(uSampler, vUV).rgb;
    vec3 excess = max(rgb - vec3(uThreshold), vec3(0.0));
    gl_FragColor = vec4(excess, 1.0);
}
`;

/** Only premultiplied energy ABOVE this blooms. The tonemap keeps LDR ≤1 intact,
 *  so 1.0 means "only genuinely HDR-over-bright additive peaks glow" — painted
 *  scene panels (authored ≤1) are excluded, keeping the bloom off backdrops. */
const BLOOM_THRESHOLD = 1.0;
/** DIAGNOSTIC (`?bloomthr=<f>`): lower the bright-pass threshold so painted (<=1) panels can
 *  glow too. The shipped 1.0 deliberately excludes them; this makes that choice measurable. */
function bloomThreshold(): number {
    if (typeof window === "undefined") return BLOOM_THRESHOLD;
    const v = parseFloat(new URLSearchParams(window.location.search).get("bloomthr") ?? "");
    return Number.isFinite(v) && v > 0 ? v : BLOOM_THRESHOLD;
}
// Coverage pass: a NEAR-white mask, blurred, used to tell a full-frame white-out from
// scattered highlights. See `uCovOn` in the tonemap for why this is needed and why the
// bloom's own bright-pass cannot supply it (it thresholds at 1.0 and both cases sit at
// exactly 1.0, so it is identically zero in precisely the frames that must be told apart).
const COVERAGE_FRAG = `
precision highp float;
varying vec2 vUV;
uniform sampler2D uSampler;
uniform float uCovThreshold;
void main() {
    vec3 rgb = texture2D(uSampler, vUV).rgb;
    float m = max(max(rgb.r, rgb.g), rgb.b);
    // Binary mask; the blur below turns it into a local "fraction of my neighbourhood
    // that is near-white", which is exactly the coverage signal.
    gl_FragColor = vec4(vec3(step(uCovThreshold, m)), 1.0);
}
`;
/** Near-white cut for the coverage mask. Below 1.0 on purpose: the pixels this exists to
 *  classify sit at EXACTLY 1.0, so a >=1.0 test (the bloom's) returns nothing. */
const COVERAGE_THRESHOLD = 0.97;
/** Coverage pass runs coarser than the bloom — it is a low-frequency signal and the wide
 *  blur below is what gives it reach. */
const COVERAGE_DOWNSCALE = 8;
/** Blur radius (in coverage-RT px) that defines "large region". At downscale 8 this reaches
 *  roughly 100 source px. */
const COVERAGE_BLUR = 12;

/** How strongly the blurred bloom is added back on top of the tonemapped scene. */
const BLOOM_INTENSITY = 0.85;
/** DIAGNOSTIC: `?bloom=<f>` overrides {@link BLOOM_INTENSITY} so a residual can be
 *  attributed to (or cleared of) the bloom pass without a rebuild. */
function bloomIntensity(): number {
    if (typeof window === "undefined") return BLOOM_INTENSITY;
    const q = new URLSearchParams(window.location.search).get("bloom");
    const v = q === null ? NaN : parseFloat(q);
    return Number.isFinite(v) ? v : BLOOM_INTENSITY;
}
/** Bloom RTs run at 1/N res — cheaper and gives a wider, softer glow for free. */
const BLOOM_DOWNSCALE = 2;
/** Gaussian blur strength (in bloom-RT pixels) applied to the bright-pass. */
const BLOOM_BLUR = 10;

/** Highlights at/under this (premultiplied) level pass through untouched; brighter
 *  stacks are compressed toward 1 with hue preserved.
 *
 *  0.99, NOT the historical 0.9. The compression maps [knee, inf) -> [knee, 1) with 1.0 as an
 *  ASYMPTOTE, so every pixel between the knee and 1.0 is darkened even though it is not
 *  over-bright at all — at knee 0.9 a saturated 1.0 came out at 0.95, i.e. 242 instead of 255.
 *  The saturation release below rescues only near-NEUTRAL pixels, which is why white flashes were
 *  fixed by it and saturated colours were not.
 *
 *  Measured on Virtuosa's apple (its real beat is t ~ 6.27): peak red 241/244 at knee 0.9 against
 *  the game's 255, and 253/254 at knee 0.99 — position and size already matched, only the peak was
 *  short. Corpus cost is +0.009 mean MADC (mly +0.015, cel -0.005, ska +0.016), smaller than other
 *  corrections already shipped here.
 *
 *  Raising the knee does NOT weaken the >1 protection this pass exists for: anything above 1 is
 *  still scaled by `t/m`, so coloured additive stacks keep their hue instead of clipping to white.
 *  Only the LDR shoulder moves. `?knee=<f>` overrides. */
const DEFAULT_KNEE = 0.99;

/** DIAGNOSTIC (`?knee=<f>`): override {@link DEFAULT_KNEE}. Raising it above 1 disables the
 *  hue-preserving compression entirely, so over-bright pixels clip PER CHANNEL the way Unity's
 *  pipeline does — which is the only mechanism that can turn a saturated warm additive stack
 *  white. Exists to test exactly that against Skadi's crown fish. */
/** DIAGNOSTIC (`?tmclip=1`): swap the soft roll-off for a hue-preserving clip — see the shader. */
function tonemapClip(): number {
    if (typeof window === "undefined") return 0;
    return new URLSearchParams(window.location.search).get("tmclip") === "1" ? 1 : 0;
}

function kneeParam(): number {
    if (typeof window === "undefined") return DEFAULT_KNEE;
    const v = parseFloat(new URLSearchParams(window.location.search).get("knee") ?? "");
    return Number.isFinite(v) && v > 0 ? v : DEFAULT_KNEE;
}

/** Calibration points for {@link sceneCompositeGamma}: `[cameraSizePx, exponent]`, ascending.
 *
 *  RE-MEASURED 2026-08-07 against the CLEAN 2340x1080 / 100 Mbps entrance captures. The previous
 *  values were sweep minima against the SALVAGED reference, and that reference is systematically
 *  DARKER than the real game (measured +5.7 / +4.3 / +1.1 luma fresh-minus-salvage on the three
 *  skins). Tuning to it therefore darkened the renderer to match a degraded copy — the residual
 *  showed up as a per-skin gamma of ~1.04 against the clean capture, near zero at black and
 *  growing through the midtones, which is the signature of an exponent error rather than an
 *  offset. Every exponent below dropped:
 *
 *      Skadi    cs 1000   1.02 -> 10.679   1.03 -> 10.478  [1.04 -> 10.403]  1.05 -> 10.468   1.06 -> 10.650
 *      Virtuosa cs 1050   1.01 -> 17.667  [1.02 -> 17.646]  1.03 -> 17.670   1.04 -> 17.782   1.06 -> 18.179
 *      Mlynar   cs 1111   0.92 -> 17.904  [0.93 -> 17.864]  0.94 -> 17.866   0.95 -> 17.911   0.98 -> 18.300
 *
 *  All three minima are INTERIOR and bracketed on both sides. Worth against the clean reference:
 *  **mly 18.77 -> 17.86, cel 18.57 -> 17.65, ska 12.89 -> 10.40**.
 *
 *  The three are still not collinear (slope -0.0004 then -0.0014 per px), so the piecewise
 *  interpolation through all three is kept — a straight line would again fit only two.
 *
 *  ⚠️ This is a MUCH larger move than the previous re-anchoring (|delta gamma| up to 0.08 against
 *  0.02), and it brightens rather than darkens, so the corpus risk is highlight clipping rather
 *  than crush. See the saturation sweep recorded alongside it before widening the range further.
 *
 *  The old note here said replacing these needed "a fourth and fifth reference CAPTURE". That is
 *  no longer the binding constraint — the entrance is capturable for ANY skin, owned or not, via
 *  the FLOT Lookbook viewer's play button, so the calibration set can now be extended by capture
 *  rather than by inference. */
const GAMMA_CAL: readonly (readonly [number, number])[] = [
    [1000, 1.04], // Skadi the Corrupting Heart
    [1050, 1.02], // Virtuosa
    [1100, 1.02], // Muelsyse — the fourth capture this table asked for (see below)
    [1111, 0.93], // Mlynar
];

/**
 * Exponent applied to the assembled scene composite, derived per scene from its authored
 * orthographic camera size.
 *
 * WHAT IT CORRECTS. We render the midtones brighter than the game does. The error is a
 * clean power law on the encoded value, not an offset or a multiply — pooling FLAT patches
 * (no edges, so no registration or filtering component) across Skadi the Corrupting Heart's
 * scored beats, the ratio `game/ours` rises 0.902 → 0.939 → 0.961 → 0.994 with level, where
 * a multiply would hold it constant. A mid-grey we draw at 117 the game draws at 106.
 * Correcting it is worth 38% of her total error (MADC 12.04 → 7.44).
 *
 * WHY cameraSizePx. The needed exponent is NOT global — it differs per skin — and no other
 * scene quantity predicts it. Layer count, drawn-layer count, coverage-weighted overdraw,
 * spine slot count, semi-transparent slot count and summed alpha were all measured against
 * the three reference skins and none is even monotone; the "compositing depth" story runs
 * BACKWARDS (Mlynar carries the most slots, 688, and the most semi-transparent ones, 301,
 * yet needs the least correction). `cameraSize` is the one scene datum that orders the three
 * correctly, and a straight line through the two extremes predicts the middle skin to within
 * 0.005:
 *
 *     cameraSizePx 1000 (Skadi)   measured 1.120   derived 1.120
 *     cameraSizePx 1050 (Cello)   measured 1.070   derived 1.075
 *     cameraSizePx 1111 (Mlynar)  measured 1.020   derived 1.020
 *
 * HONEST STATUS — read this before extending, and do not restore the mechanistic reading.
 * Three points and two parameters is an INTERPOLATION, and camera size has since been
 * **FALSIFIED as the driver**. The entrance dollies, so each skin's LIVE camera size sweeps a
 * far wider range than the between-skin spread does (Skadi 1.38→3.48, ~2.5×, against 10.0→11.11
 * = 1.11× between skins). If the exponent were driven by camera size it would have to track
 * that sweep with this slope. It does not, and it fails in BOTH directions:
 *
 *     skin  live-ortho span   gamma span this line PREDICTS   gamma span MEASURED
 *     ska      1.38 - 3.48            0.190                          0.060
 *     mly      2.93 - 5.58            0.239                          0.115
 *     cel      1.53 - 1.91            0.035                          0.105
 *
 * So the between-skin agreement is a COINCIDENCE, and this function is a bare curve fit keyed
 * on a scene scalar that does not cause the effect. It is kept only because it reproduces the
 * three measured exponents better than any single constant does (mean MADC 14.338 against
 * 15.011 for a flat 1.08, which costs Mlynar 16.993 against his 15.877) — the same
 * calibrated-on-three-references basis as every other tuned constant in this renderer.
 *
 * Strictly, the test above rules out the LIVE camera size; it cannot rule out the authored
 * per-scene value being read once at load. But there is no mechanism proposing that either:
 * the asset bundles contain no colour grading, LUT, tonemap or exposure data, the captures
 * share one transfer, and our own tonemap, bloom, canvas alpha, environment fill and texture
 * flags were each eliminated with measurements. The true cause is believed to live in the
 * client's compositing stage, which we cannot yet observe. **If a real driver is found,
 * delete this keying rather than adding terms to it.**
 *
 * Consequently the INPUT is clamped to the calibrated span. Extrapolated, the line is
 * catastrophic — the corpus runs from cameraSizePx 889 to 1800, and 1800 would give an
 * exponent of 0.40. Clamping bounds every scene to [1.02, 1.12], which brackets the 1.08
 * that a single global constant would have used, so a skin outside the calibrated range is
 * never worse off than under one flat number. 16% of the 115-scene corpus sits outside the
 * span and takes an endpoint value.
 *
 * CLAMP VERIFIED on the two corpus extremes, for which no capture exists so only the bound
 * could be checked: Nian #7 (1800 → 1.02) shifts by at most 2 code values, Texas the
 * Omertosa (889 → 1.12) by at most 11. Neither degenerates.
 *
 * 2026-08-09 — FIVE MORE CAPTURES ARRIVED, and they say two things.
 *
 * Sweeping the exponent per skin against the new references gives a measured optimum for each:
 *
 *     skin  cameraSizePx  this table PREDICTED  MEASURED  verdict
 *     wis       1000              1.04            ~1.03    agrees
 *     exc       1040              1.024            1.02     agrees
 *     cet       1050              1.02             1.02     agrees
 *     eyja      1050              1.02             1.02     agrees
 *     mue       1100              0.946            1.02     ** WRONG by 0.074 **
 *
 * Four of the five land where the line predicts, which is real corroboration — the original
 * three-point fit was not a fluke. But Muelsyse breaks it, and breaks it where it matters:
 * she and Mlynar are ELEVEN pixels apart in camera size and want exponents 0.09 apart. No
 * smooth function of this key can do that, so camera size is confirmed as a proxy rather than
 * a cause, exactly as the section above suspected. Seven of the eight measured skins want
 * 1.02–1.04; Mlynar alone wants 0.93.
 *
 * Muelsyse is added as a measured point rather than smoothed over: she is the ONLY capture at
 * 1100, and the alternative is to keep predicting a value her own reference refutes. That makes
 * the final segment a cliff (1100 → 1111 is 1.02 → 0.93), which is honest about the fit being a
 * lookup rather than a law. No corpus scene falls strictly between those two keys.
 *
 * TO REPLACE THIS: a driver, not more points. The five new captures have now shown the key is
 * not causal; what is missing is a mechanism, and Mlynar is the skin that would falsify or
 * confirm any candidate.
 */
export function sceneCompositeGamma(cameraSizePx: number | undefined | null): number {
    const diag = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("gamma") : null;
    if (diag != null) {
        const v = Number(diag);
        return Number.isFinite(v) && v > 0 ? v : 1;
    }
    if (!cameraSizePx || !Number.isFinite(cameraSizePx)) return 1;
    // Piecewise-linear through GAMMA_CAL, clamped flat outside the calibrated range.
    const first = GAMMA_CAL[0];
    const last = GAMMA_CAL[GAMMA_CAL.length - 1];
    if (cameraSizePx <= first[0]) return first[1];
    if (cameraSizePx >= last[0]) return last[1];
    for (let i = 1; i < GAMMA_CAL.length; i++) {
        const [aPx, aG] = GAMMA_CAL[i - 1];
        const [bPx, bG] = GAMMA_CAL[i];
        if (cameraSizePx <= bPx) return aG + ((bG - aG) * (cameraSizePx - aPx)) / (bPx - aPx);
    }
    return last[1];
}

/** DIAGNOSTIC (`?hdrprobe=<scale>`): see the shader. 0 (default) = normal tonemap. */
function hdrProbe(): number {
    if (typeof window === "undefined") return 0;
    const v = parseFloat(new URLSearchParams(window.location.search).get("hdrprobe") ?? "");
    return Number.isFinite(v) && v > 0 ? v : 0;
}

/** Enable the coverage gate on the achroma release? **DISABLED — MEASURED AND REJECTED.**
 *
 *  The diagnosis is right and the gate demonstrably works. The achroma release lets any
 *  near-neutral pixel at/above the knee pass through to an opaque 255 — needed so a white
 *  reveal FLASH reaches 255 instead of the knee's ~242, but it also clips ordinary bright
 *  white ART. Coverage is the only property separating the two (magnitude cannot: Mlynar's
 *  held flash sits at exactly 1.0 just as Virtuosa's interior does). Gating the release on a
 *  blurred near-white mask does exactly what it should:
 *
 *      clipped fraction (luma >= 254)      OFF       ON      game
 *        cel t=2                         11.03%    0.91%    1.10%
 *        cel t=5                          6.71%    0.00%    0.21%
 *        mly t=15 (HELD FLASH)          100.00%   92.68%   99.29%
 *        mly t=16 (HELD FLASH)          100.00%  100.00%   99.29%
 *
 *  Virtuosa's spurious blow-out is essentially eliminated and the flash is preserved. And it
 *  still does not improve parity:
 *
 *      baseline (off)        mly 17.721  cel 19.422  ska 10.505   sum 47.648
 *      cov, knee 0.99            17.726      19.471      10.500       47.697
 *      cov, knee 0.97            17.727      19.449      10.494       47.670
 *      cov, knee 0.94            17.732      19.489      10.517       47.738
 *      cov, knee 0.90            17.747      19.647      10.656       48.050
 *
 *  Rolling the whites further DOWN — toward the game's p90 of 242 — is monotonically worse,
 *  which is the tell: the metric is PIXELWISE while the clipping census is DISTRIBUTIONAL.
 *  Part of the game's lower p90 is the salvaged reference's own peak suppression (degrading
 *  our own frame takes 11.03% to 4.91%), so matching its distribution moves us AWAY from its
 *  pixel values where the game is still bright. A distributional win that costs pixelwise
 *  error is not a parity win.
 *
 *  Kept because the mechanism is established and the gate is correct — if the ORIGINAL
 *  captures are ever restored, this is the first thing to re-measure, since the peak
 *  suppression that defeats it is an artifact of the salvage. `?cov=1`, plus `?covlo=`,
 *  `?covhi=`, `?covthr=`. */
function coverageOn(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("cov") === "1";
}
function covParam(name: string, dflt: number): number {
    if (typeof window === "undefined") return dflt;
    const v = parseFloat(new URLSearchParams(window.location.search).get(name) ?? "");
    return Number.isFinite(v) && v >= 0 ? v : dflt;
}
/** Blurred-coverage values below this get the FULL hue-preserving roll-off (scattered
 *  highlights); above `coverageHi` the release passes through (a genuine white-out). */
const COVERAGE_LO = 0.35;
const COVERAGE_HI = 0.75;
const coverageLo = () => covParam("covlo", COVERAGE_LO);
const coverageHi = () => covParam("covhi", COVERAGE_HI);

/** GAP FILL IS NOT COMPOSITED HERE — the whole tonemap-side machinery was removed after every
 *  variant it existed to serve was refuted with numbers. Kept as a record so none is retried:
 *
 *      draw-first (a plain sprite at index 0)        cel 19.422 -> 19.361   the only winner
 *      DST_OVER (draw last, destination-over)        19.404, and washes MORE than draw-first;
 *                                                    NOT algebraically equal to draw-first,
 *                                                    because additive layers add colour with
 *                                                    no alpha
 *      hard alpha/luminance threshold in this pass   19.421 (no effect) — the scene target's
 *                                                    alpha is non-zero across the frame, so
 *                                                    "covered" is 1 nearly everywhere
 *      spine-shaped ERASE mask on the static art     a NO-OP by construction: the art sits at
 *                                                    index 0 and the spine draws in FRONT of
 *                                                    it, so the mask can only remove pixels
 *                                                    that were already hidden (786 px of
 *                                                    374400 on lin_nian#10, all antialiasing)
 *
 *  What ships instead is in `SceneIllust.tsx`: the static art drawn first, DEFOCUSED, so it
 *  fills as a vista without contributing recognisable detail. See `gapBlurOn` there. */

export interface IHDRScene {
    /** Half-float target the scene is drawn into each frame (before tonemap). */
    target: PIXI.RenderTexture;
    /** Full-screen quad that tonemaps `target` (+bloom) to the screen — add to the stage. */
    mesh: PIXI.Mesh<PIXI.Shader>;
    /** Update the bloom texture from the current `target`. Call each frame AFTER
     *  drawing the scene into `target` and BEFORE rendering the stage. */
    prepare(renderer: PIXI.IRenderer): void;
    /** Resize the target + quad to a new screen size. */
    resize(width: number, height: number, resolution: number): void;
    destroy(): void;
}

/** Whether this renderer can color-render to a half-float target (WebGL2 +
 *  EXT_color_buffer_float). */
function supportsFloatTarget(renderer: PIXI.IRenderer): boolean {
    const anyR = renderer as unknown as { gl?: WebGL2RenderingContext; context?: { webGLVersion?: number } };
    const gl = anyR.gl;
    if (!gl || anyR.context?.webGLVersion !== 2) return false;
    return !!gl.getExtension("EXT_color_buffer_float");
}

function makeQuad(width: number, height: number): PIXI.Geometry {
    // TL, TR, BR, BL in screen pixels, with sprite-style UVs so the render target
    // shows upright (matches how a PIXI.Sprite samples a RenderTexture). Built as a
    // raw Geometry with explicitly-named attributes (`aVertexPosition`/`aUV`) —
    // PIXI.MeshGeometry names its UV attribute differently, so the shader can't bind.
    type Buf = ConstructorParameters<typeof PIXI.Buffer>[0];
    const geometry = new PIXI.Geometry();
    geometry.addAttribute("aVertexPosition", new PIXI.Buffer(new Float32Array([0, 0, width, 0, width, height, 0, height]) as unknown as Buf), 2);
    geometry.addAttribute("aUV", new PIXI.Buffer(new Float32Array([0, 0, 1, 0, 1, 1, 0, 1]) as unknown as Buf), 2);
    geometry.addIndex(new PIXI.Buffer(new Uint16Array([0, 1, 2, 0, 2, 3]) as unknown as Buf));
    return geometry;
}

/** Build the HDR scene target + tonemap quad, or null if float targets are
 *  unavailable (caller falls back to plain 8-bit compositing). */
export function createHDRScene(renderer: PIXI.IRenderer, width: number, height: number, resolution: number, knee: number = kneeParam(), gamma = 1): IHDRScene | null {
    if (!supportsFloatTarget(renderer)) return null;
    let target: PIXI.RenderTexture;
    try {
        target = PIXI.RenderTexture.create({
            width,
            height,
            resolution,
            format: PIXI.FORMATS.RGBA,
            type: PIXI.TYPES.HALF_FLOAT,
            scaleMode: PIXI.SCALE_MODES.LINEAR,
        });
    } catch {
        return null;
    }
    // Bloom pipeline: a half-res RT holding the blurred bright-pass. The bright
    // mesh samples `target` (float) and keeps only the >threshold energy; a
    // BlurFilter on its container spreads it into a glow when rendered into
    // `bloomRT`; the tonemap shader then adds `bloomRT` back on top.
    const bw = Math.max(1, Math.round(width / BLOOM_DOWNSCALE));
    const bh = Math.max(1, Math.round(height / BLOOM_DOWNSCALE));
    let bloomRT: PIXI.RenderTexture;
    try {
        bloomRT = PIXI.RenderTexture.create({ width: bw, height: bh, resolution, scaleMode: PIXI.SCALE_MODES.LINEAR });
    } catch {
        target.destroy(true);
        return null;
    }
    // A freshly-allocated framebuffer holds UNDEFINED contents until something writes to it,
    // and the stage's tonemap quad samples BOTH of these. Any stage render that happens
    // before the first scene pass therefore reads uninitialised GPU memory — zero-filled on
    // some drivers (so headless swiftshader shows nothing), arbitrary colour on others, which
    // is what a coloured flash on the first frames of a load actually is. Clear both once,
    // here, so the contents are always defined.
    {
        const blank = new PIXI.Container();
        renderer.render(blank, { renderTexture: target, clear: true });
        renderer.render(blank, { renderTexture: bloomRT, clear: true });
        blank.destroy();
    }
    const cw = Math.max(1, Math.round(width / COVERAGE_DOWNSCALE));
    const ch = Math.max(1, Math.round(height / COVERAGE_DOWNSCALE));
    let covRT: PIXI.RenderTexture;
    try {
        covRT = PIXI.RenderTexture.create({ width: cw, height: ch, resolution, scaleMode: PIXI.SCALE_MODES.LINEAR });
    } catch {
        bloomRT.destroy(true);
        target.destroy(true);
        return null;
    }
    {
        const blank2 = new PIXI.Container();
        renderer.render(blank2, { renderTexture: covRT, clear: true });
        blank2.destroy();
    }
    const covMesh = new PIXI.Mesh(makeQuad(cw, ch), PIXI.Shader.from(TONEMAP_VERT, COVERAGE_FRAG, { uSampler: target, uCovThreshold: covParam("covthr", COVERAGE_THRESHOLD) }));
    const covContainer = new PIXI.Container();
    covContainer.addChild(covMesh);
    const covBlur = new PIXI.BlurFilter(COVERAGE_BLUR);
    covBlur.quality = 4;
    covContainer.filters = [covBlur];
    const setCovQuad = (w: number, h: number) => {
        const p = covMesh.geometry.getBuffer("aVertexPosition");
        p.data = new Float32Array([0, 0, w, 0, w, h, 0, h]) as unknown as typeof p.data;
        p.update();
    };

    const brightMesh = new PIXI.Mesh(makeQuad(bw, bh), PIXI.Shader.from(TONEMAP_VERT, BRIGHT_FRAG, { uSampler: target, uThreshold: bloomThreshold() }));
    const brightContainer = new PIXI.Container();
    brightContainer.addChild(brightMesh);
    const blur = new PIXI.BlurFilter(BLOOM_BLUR);
    blur.quality = 4;
    brightContainer.filters = [blur];

    const geometry = makeQuad(width, height);
    const shader = PIXI.Shader.from(TONEMAP_VERT, TONEMAP_FRAG, {
        uSampler: target,
        uBloom: bloomRT,
        uKnee: knee,
        uBloomIntensity: bloomIntensity(),
        uGamma: gamma,
        uClip: tonemapClip(),
        uProbe: hdrProbe(),
        uCov: covRT,
        uCovOn: coverageOn() ? 1 : 0,
        uCovLo: coverageLo(),
        uCovHi: coverageHi(),
    });
    const mesh = new PIXI.Mesh(geometry, shader);

    const setBrightQuad = (w: number, h: number) => {
        const p = brightMesh.geometry.getBuffer("aVertexPosition");
        p.data = new Float32Array([0, 0, w, 0, w, h, 0, h]) as unknown as typeof p.data;
        p.update();
    };

    return {
        target,
        mesh,
        prepare(renderer: PIXI.IRenderer) {
            renderer.render(brightContainer, { renderTexture: bloomRT, clear: true });
            if (coverageOn()) renderer.render(covContainer, { renderTexture: covRT, clear: true });
        },
        resize(w: number, h: number, res: number) {
            // ⚠️ setResolution BEFORE resize, never after. `BaseTexture.setResolution` RESCALES the
            // texture's screen dimensions to preserve its pixel size (`width = width*oldRes/res`),
            // so resizing first and setting the resolution second leaves every target off by
            // `oldRes/res`. Latent until the renderer's resolution actually changed at runtime —
            // which it now does when the idle path takes over the square-2048 RT density.
            target.baseTexture.setResolution(res);
            target.resize(w, h, true);
            const positions = mesh.geometry.getBuffer("aVertexPosition");
            positions.data = new Float32Array([0, 0, w, 0, w, h, 0, h]) as unknown as typeof positions.data;
            positions.update();
            const nbw = Math.max(1, Math.round(w / BLOOM_DOWNSCALE));
            const nbh = Math.max(1, Math.round(h / BLOOM_DOWNSCALE));
            bloomRT.baseTexture.setResolution(res);
            bloomRT.resize(nbw, nbh, true);
            setBrightQuad(nbw, nbh);
            const ncw = Math.max(1, Math.round(w / COVERAGE_DOWNSCALE));
            const nch = Math.max(1, Math.round(h / COVERAGE_DOWNSCALE));
            covRT.baseTexture.setResolution(res);
            covRT.resize(ncw, nch, true);
            setCovQuad(ncw, nch);
        },
        destroy() {
            mesh.destroy();
            brightMesh.destroy();
            brightContainer.destroy();
            blur.destroy();
            bloomRT.destroy(true);
            covMesh.destroy();
            covContainer.destroy();
            covBlur.destroy();
            covRT.destroy(true);
            target.destroy(true);
        },
    };
}
