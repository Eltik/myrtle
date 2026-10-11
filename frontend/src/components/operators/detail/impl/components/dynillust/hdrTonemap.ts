import * as PIXI from "pixi.js";

/**
 * HDR bloom pass for the dynamic-illustration compositor.
 *
 * Dynchar scenes stack many ADDITIVE particle and light layers (Hoshiguma's
 * blue ice-flames, Wiš'adel's supernova bloom). In an 8-bit target additive
 * blending clips each channel at 1.0, so a stack of blue flames
 * `(0.3,0.6,3.0)` clips to `(1,1,1)`: a hard WHITE blob over the character
 * where the game shows a translucent blue glow. The game accumulates in HDR
 * and tonemaps, and so do we:
 *
 *  1. Render the scene into a HALF-FLOAT target, where additive blends
 *     accumulate past 1.0 without clipping.
 *  2. Blit it to the screen through a hue-preserving highlight compressor:
 *     values at/under the knee pass untouched (backdrop and character are
 *     authored ≤1), and a stack whose brightest channel exceeds the knee is
 *     scaled down as a whole, so blue stays blue and the peak rolls toward 1
 *     instead of clipping to white.
 *
 * Returns null when float render targets aren't available (no WebGL2 /
 * `EXT_color_buffer_float`); the caller then composites in 8-bit.
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

// The scene is rendered PREMULTIPLIED over a transparent (page-dark) canvas, so `rgb`
// is the actual light contribution. Only the highlight TAIL is compressed: identity
// below `uKnee`; above it rgb scales by `tone(m)/m` (m = max channel), so the colour
// dims together and the peak asymptotes to 1 instead of clipping per channel to white.
// The blurred bloom (the >1 energy spread into a glow) is added last, so thin bright
// additive effects (Hoshiguma's ice-crystal beams) read as glowing shards the way the
// game's bloom shows them, not the thin sliver the tonemap clamps to.
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
uniform float uDstA;
uniform sampler2D uCov;
uniform float uCovOn;
uniform float uCovLo;
uniform float uCovHi;
void main() {
    vec4 c = texture2D(uSampler, vUV);
    vec3 rgb = c.rgb;
    float m = max(max(rgb.r, rgb.g), rgb.b);
    // DIAGNOSTIC (?hdrprobe=<scale>): skip the tonemap and output the raw half-float
    // target divided by <scale>, so a captured PNG reads back as HDR magnitude
    // (pixel/255*scale). Tells an over-bright stack from LDR white already passing
    // through the achroma release.
    if (uProbe > 0.0) {
        gl_FragColor = vec4(rgb / uProbe, 1.0);
        return;
    }
    if (m > uKnee) {
        // soft-compress the tail [knee, +inf) -> [knee, 1)
        float e = (m - uKnee) / max(1e-4, 1.0 - uKnee);
        float t = uKnee + (1.0 - uKnee) * (e / (1.0 + e));
        // The knee preserves HUE on >1 additive stacks. An ACHROMATIC highlight (a
        // fullscreen white reveal-flash) has no hue to preserve, yet the knee still greys
        // it: at the old 0.9 knee a pure-white 1.0 mapped to ~0.95 = 242, never the game's
        // 255. So the compression is released as the pixel nears neutral; anything with
        // real chroma keeps it. Saturation-gated, not per-skin.
        float mn = min(min(rgb.r, rgb.g), rgb.b);
        float sat = (m > 1e-4) ? (m - mn) / m : 0.0;
        float achroma = 1.0 - smoothstep(0.05, 0.25, sat); // 1 = white/grey, 0 = coloured
        // COVERAGE GATE (?cov=1, off by default, see coverageOn). The achroma release above
        // exists so a full-frame white REVEAL FLASH reaches an opaque 255 instead of the
        // knee's ~242, but it also fires on ordinary bright white ART: Virtuosa blows 11.03%
        // of her t=2 frame to pure white where the game blows 1.10%.
        //
        // Magnitude can't separate the two: Mlynar's HELD FLASH sits at exactly 1.0 just as
        // Virtuosa's interior does, and there the game wants 255. What differs is how much
        // of the neighbourhood is white, ~12% scattered vs 100%. uCov is a blurred
        // near-white mask, ~1 inside a real white-out and low around isolated highlights.
        if (uCovOn > 0.5) {
            float cov = texture2D(uCov, vUV).r;
            achroma *= smoothstep(uCovLo, uCovHi, cov);
        }
        float scale = mix(t / m, 1.0, achroma);
        // DIAGNOSTIC (?tmclip=1): hue-preserving CLIP instead of the soft roll-off.
        //
        // The roll-off has 1.0 as an ASYMPTOTE, so a pixel above the knee is darkened even
        // when it isn't over-bright: at the old 0.9 knee a saturated 1.0 came out at 0.95.
        // The achroma release only rescues near-NEUTRAL pixels, which is why white flashes
        // were fixed and saturated ones weren't. Measured on Virtuosa's apple at knee 0.9:
        // peak red capped at 241 where the game reaches 255.
        //
        // The clip is identity at m <= 1 and scales by 1/m above, so LDR passes untouched
        // and over-bright stacks keep their hue.
        if (uClip > 0.5) scale = (m > 1.0) ? (1.0 / m) : 1.0;
        rgb *= scale;
    }
    // NO cinematic vignette. A brightness-gated radial falloff used to live here, to match
    // a believed "~209 corner vs 255 centre" on the game's white transition flash. That
    // falloff doesn't exist: on the held flash frames (t = 14.5, 15.0, 15.5, 16.0) the game
    // is 255 in all four corners, at the centre and between (min 254 over the inner window,
    // 0.0% of pixels below 250). The 209 was almost certainly read off a RAMP frame, which
    // is uniformly dim (~202 at t = 14). The vignette held our margins at a flat 237 for the
    // whole hold. Don't bring it back without re-measuring the HELD frames.
    // EXPERIMENT (?dsta=1): composite by the scene's own DESTINATION ALPHA, modelling the
    // game UI drawing the RT with straight alpha over a near-black backdrop. Only means
    // anything with the carve blend (SceneIllust patchAdditiveBlendAlpha) under the same
    // param; uDstA is 0 otherwise.
    if (uDstA > 0.5) {
        rgb *= clamp(c.a, 0.0, 1.0);
    }
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

// Bright-pass: keep only the energy ABOVE the threshold in the half-float scene target.
// Painted panels authored ≤1 never reach it, so only additive stacks that accumulated
// past it (blades, flames, gem cores) bloom.
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

/** Only premultiplied energy above this blooms. 1.0 keeps painted panels (authored ≤1)
 *  out, so only genuinely over-bright additive peaks glow. */
const BLOOM_THRESHOLD = 1.0;
/** DIAGNOSTIC (`?bloomthr=<f>`): lower the bright-pass threshold so painted (<=1) panels can
 *  glow too. The shipped 1.0 deliberately excludes them; this makes that choice measurable. */
function bloomThreshold(): number {
    if (typeof window === "undefined") return BLOOM_THRESHOLD;
    const v = parseFloat(new URLSearchParams(window.location.search).get("bloomthr") ?? "");
    return Number.isFinite(v) && v > 0 ? v : BLOOM_THRESHOLD;
}
// Coverage pass: a blurred NEAR-white mask that tells a full-frame white-out from scattered
// highlights (see `uCovOn` in the tonemap). The bloom's bright-pass can't supply it: it
// thresholds at 1.0 and both cases sit at exactly 1.0, so it's zero in exactly the frames
// that need telling apart.
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
/** Coarser than the bloom: coverage is a low-frequency signal and the wide blur below is
 *  what gives it reach. */
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
/** Bloom RTs run at 1/N res: cheaper, and a wider, softer glow for free. */
const BLOOM_DOWNSCALE = 2;
/** Gaussian blur strength (in bloom-RT pixels) applied to the bright-pass. */
const BLOOM_BLUR = 10;

/** Highlights at/under this (premultiplied) level pass through untouched; brighter stacks
 *  are compressed toward 1 with hue preserved. `?knee=<f>` overrides.
 *
 *  2 since 2026-09-06, by Ian's ruling on Nian E2's flames: the Additive program's own x2
 *  (`col = 2 * tint * tex`) under the hue-preserving roll-off read as saturated cartoon
 *  orange (left flame 249.5/183.9/135.8 against the static art's 208.9/207.4/181.2 in its
 *  most saturated tenth). At knee 2 the stack clips per channel toward pale
 *  (249.6/213.4/170.4), the way the game's LDR target does. Stacks up to 2x pass untouched;
 *  only a pixel above 2 is still rolled off. `?knee=0.99` restores the previous default.
 *
 *  0.99 before that, raised from 0.9. The roll-off maps [knee, inf) -> [knee, 1) with 1.0 as
 *  an ASYMPTOTE, so everything between the knee and 1.0 is darkened though it isn't
 *  over-bright: at 0.9 a saturated 1.0 came out at 0.95 (242, not 255). The achroma release
 *  only rescues near-NEUTRAL pixels. Measured on Virtuosa's apple (beat t ~ 6.27): peak red
 *  241/244 at 0.9 against the game's 255, 253/254 at 0.99, with position and size already
 *  matching. Corpus cost +0.009 mean MADC (mly +0.015, cel -0.005, ska +0.016). That move
 *  kept the >1 protection intact; only the LDR shoulder moved. */
const DEFAULT_KNEE = 2;

/** DIAGNOSTIC (`?tmclip=1`): swap the soft roll-off for a hue-preserving clip; see the shader. */
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
 *  RE-MEASURED 2026-08-07 against the CLEAN 2340x1080 / 100 Mbps entrance captures. The old
 *  values were sweep minima against the SALVAGED reference, which is systematically DARKER
 *  than the game (+5.7 / +4.3 / +1.1 luma fresh-minus-salvage on the three skins), so tuning
 *  to it darkened the renderer to match a degraded copy. The residual showed as a per-skin
 *  gamma of ~1.04 against the clean capture, near zero at black and growing through the
 *  midtones: an exponent error, not an offset. Every exponent dropped:
 *
 *      Skadi    cs 1000   1.02 -> 10.679   1.03 -> 10.478  [1.04 -> 10.403]  1.05 -> 10.468   1.06 -> 10.650
 *      Virtuosa cs 1050   1.01 -> 17.667  [1.02 -> 17.646]  1.03 -> 17.670   1.04 -> 17.782   1.06 -> 18.179
 *      Mlynar   cs 1111   0.92 -> 17.904  [0.93 -> 17.864]  0.94 -> 17.866   0.95 -> 17.911   0.98 -> 18.300
 *
 *  All three minima are INTERIOR, bracketed on both sides. Against the clean reference:
 *  **mly 18.77 -> 17.86, cel 18.57 -> 17.65, ska 12.89 -> 10.40**.
 *
 *  The points aren't collinear (slope -0.0004 then -0.0014 per px), so interpolation stays
 *  piecewise; a straight line would fit only two.
 *
 *  ⚠️ A much larger move than the previous re-anchoring (|delta gamma| up to 0.08 vs 0.02),
 *  and it brightens, so the corpus risk is highlight clipping rather than crush. See the
 *  saturation sweep recorded alongside it before widening the range further.
 *
 *  New points need no owned skin: the FLOT Lookbook viewer's play button plays the entrance
 *  for ANY skin, so the set grows by capture rather than inference. */
const GAMMA_CAL: readonly (readonly [number, number])[] = [
    [1000, 1.04], // Skadi the Corrupting Heart
    [1050, 1.02], // Virtuosa
    [1100, 1.02], // Muelsyse, the fourth capture (see sceneCompositeGamma)
    [1111, 0.93], // Mlynar
    // fugue "Wintry Sojourn": the FIRST capture above the old calibrated range, and why the
    // range was extended rather than clamped. Everything at cs > 1111 used to inherit
    // Mlynar's 0.93; fugue at cs 1250 measures **1.03**, an interior minimum bracketed on
    // both sides over her eight beats:
    //   0.93 -> 14.270  0.96 -> 13.021  0.99 -> 11.961  1.02 -> 11.165  [1.03 -> 11.064]
    //   1.05 -> 11.181  1.08 -> 11.833  1.11 -> 12.752
    // Worth **14.270 -> 11.064**.
    //
    // ⚠️ She BREAKS "gamma falls as cameraSize rises". Three of the four earlier points sit
    // at 1.02-1.04 and fugue rejoins them at 1.02-1.03, so Mlynar's 0.93 is the outlier, not
    // the trend's endpoint. `cameraSizePx` ORDERS the set well enough to interpolate but does
    // NOT extrapolate: capture a new skin, don't infer it from the slope.
    //
    // Scope: only 13 deployed entrance scenes carry a `cameraSizePx` and only TWO are above
    // 1111, fugue (1250) and pasngr (1150). All eight reference skins are <= 1111, so this
    // row can't move the corpus (verified: 11.759 before and after). pasngr has no capture
    // and now interpolates to ~0.96 instead of inheriting 0.93: a small move toward the
    // consensus, but unmeasured.
    [1250, 1.03], // fugue
];

/**
 * Exponent applied to the assembled scene composite, keyed per scene on its authored
 * orthographic camera size. `?gamma=<f>` forces one value.
 *
 * WHAT IT CORRECTS. We render midtones brighter than the game. The error is a power law on
 * the encoded value, not an offset or a multiply: pooling FLAT patches (no edges, so no
 * registration or filtering component) across Skadi the Corrupting Heart's scored beats,
 * `game/ours` rises 0.902 -> 0.939 -> 0.961 -> 0.994 with level, where a multiply would hold
 * it constant. A mid-grey we draw at 117 the game draws at 106. Correcting it was worth 38%
 * of her total error (MADC 12.04 -> 7.44).
 *
 * WHY cameraSizePx. The exponent differs per skin and no other scene quantity predicts it.
 * Layer count, drawn-layer count, coverage-weighted overdraw, spine slot count,
 * semi-transparent slot count and summed alpha were all measured against the three
 * reference skins and none is even monotone; "compositing depth" runs BACKWARDS (Mlynar has
 * the most slots, 688, and the most semi-transparent ones, 301, yet needs the least
 * correction). Camera size was the one datum that ordered the three, and in the original
 * fit (salvaged reference, since re-measured: see GAMMA_CAL) a line through the two
 * extremes predicted the middle skin to within 0.005:
 *
 *     cameraSizePx 1000 (Skadi)   measured 1.120   derived 1.120
 *     cameraSizePx 1050 (Cello)   measured 1.070   derived 1.075
 *     cameraSizePx 1111 (Mlynar)  measured 1.020   derived 1.020
 *
 * STATUS: read before extending, and don't restore the mechanistic reading. Camera size has
 * been **FALSIFIED as the driver**. The entrance dollies, so each skin's LIVE camera size
 * sweeps far wider than the between-skin spread (Skadi 1.38->3.48, ~2.5×, against
 * 10.0->11.11 = 1.11× between skins). A camera-size driver would have to track that sweep
 * with this slope. It fails in BOTH directions:
 *
 *     skin  live-ortho span   gamma span this line PREDICTS   gamma span MEASURED
 *     ska      1.38 - 3.48            0.190                          0.060
 *     mly      2.93 - 5.58            0.239                          0.115
 *     cel      1.53 - 1.91            0.035                          0.105
 *
 * So the between-skin agreement is a COINCIDENCE and this is a bare curve fit on a scalar
 * that doesn't cause the effect. It stays because it reproduced the three measured
 * exponents better than any constant (mean MADC 14.338 against 15.011 for a flat 1.08,
 * which costs Mlynar 16.993 against his 15.877), the same calibrated-on-references basis as
 * every other tuned constant here.
 *
 * Strictly, that test rules out the LIVE camera size, not the authored per-scene value read
 * once at load. But nothing proposes that mechanism either: the bundles carry no colour
 * grading, LUT, tonemap or exposure data, the captures share one transfer, and our tonemap,
 * bloom, canvas alpha, environment fill and texture flags were each eliminated by
 * measurement. The cause is believed to live in the client's compositing stage, which we
 * can't observe yet. **If a real driver is found, delete this keying rather than adding
 * terms to it.**
 *
 * The INPUT is clamped to the table's span, flat at the end rows (today 1.04 at <= 1000 and
 * 1.03 at >= 1250). Extrapolated, the original line was catastrophic: the corpus runs from
 * cameraSizePx 889 to 1800, and 1800 gave an exponent of 0.40. Under the original fit the
 * clamp bounded every scene to [1.02, 1.12], bracketing a flat 1.08, and 16% of the
 * 115-scene corpus took an endpoint value. CLAMP VERIFIED then on the two uncaptured
 * extremes: Nian #7 (1800 -> 1.02) moved at most 2 code values, Texas the Omertosa
 * (889 -> 1.12) at most 11. Neither degenerated.
 *
 * 2026-08-09: FIVE MORE CAPTURES, saying two things. Per-skin sweep optima against them:
 *
 *     skin  cameraSizePx  this table PREDICTED  MEASURED  verdict
 *     wis       1000              1.04            ~1.03    agrees
 *     exc       1040              1.024            1.02     agrees
 *     cet       1050              1.02             1.02     agrees
 *     eyja      1050              1.02             1.02     agrees
 *     mue       1100              0.946            1.02     ** WRONG by 0.074 **
 *
 * Four of five land where the line predicts, so the three-point fit wasn't a fluke. But
 * Muelsyse breaks it where it matters: she and Mlynar are ELEVEN pixels apart in camera size
 * and want exponents 0.09 apart. No smooth function of this key can do that, which confirms
 * camera size as a proxy, not a cause. Seven of the eight measured skins want 1.02-1.04;
 * Mlynar alone wants 0.93.
 *
 * Muelsyse is a measured point, not smoothed over: she's the ONLY capture at 1100, and the
 * alternative is predicting a value her own reference refutes. That makes 1100 -> 1111 a
 * cliff (1.02 -> 0.93), honest about this being a lookup rather than a law. No corpus scene
 * falls strictly between those keys.
 *
 * TO REPLACE THIS: a driver, not more points. Mlynar is the skin that would falsify or
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

/** Coverage gate on the achroma release. **DISABLED: MEASURED AND REJECTED.**
 *
 *  The diagnosis is right and the gate works. The achroma release lets any near-neutral
 *  pixel at/above the knee through to an opaque 255: needed so a white reveal FLASH reaches
 *  255 instead of the knee's ~242, but it also clips ordinary bright white ART. Coverage is
 *  the only property separating the two (magnitude can't: Mlynar's held flash sits at
 *  exactly 1.0 just as Virtuosa's interior does). Gating the release on a blurred near-white
 *  mask does what it should:
 *
 *      clipped fraction (luma >= 254)      OFF       ON      game
 *        cel t=2                         11.03%    0.91%    1.10%
 *        cel t=5                          6.71%    0.00%    0.21%
 *        mly t=15 (HELD FLASH)          100.00%   92.68%   99.29%
 *        mly t=16 (HELD FLASH)          100.00%  100.00%   99.29%
 *
 *  Virtuosa's spurious blow-out is essentially gone and the flash survives. Parity still
 *  doesn't improve:
 *
 *      baseline (off)        mly 17.721  cel 19.422  ska 10.505   sum 47.648
 *      cov, knee 0.99            17.726      19.471      10.500       47.697
 *      cov, knee 0.97            17.727      19.449      10.494       47.670
 *      cov, knee 0.94            17.732      19.489      10.517       47.738
 *      cov, knee 0.90            17.747      19.647      10.656       48.050
 *
 *  Rolling whites further DOWN, toward the game's p90 of 242, is monotonically worse, which
 *  is the tell: the metric is PIXELWISE and the clipping census is DISTRIBUTIONAL. Part of
 *  the game's lower p90 is the salvaged reference's own peak suppression (degrading our own
 *  frame takes 11.03% to 4.91%), so matching its distribution moves us AWAY from its pixel
 *  values where the game is still bright. A distributional win that costs pixelwise error
 *  isn't a parity win.
 *
 *  Kept because the mechanism is established and the gate is correct. If the ORIGINAL
 *  captures come back this is the first thing to re-measure, since the peak suppression that
 *  defeats it is a salvage artifact. `?cov=1`, plus `?covlo=`, `?covhi=`, `?covthr=`. */
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

/** GAP FILL IS NOT COMPOSITED HERE. The tonemap-side machinery was removed after every
 *  variant it served was refuted with numbers. Kept as a record so none is retried:
 *
 *      draw-first (a plain sprite at index 0)        cel 19.422 -> 19.361   the only winner
 *      DST_OVER (draw last, destination-over)        19.404, and washes MORE than draw-first;
 *                                                    NOT algebraically equal to draw-first,
 *                                                    because additive layers add colour with
 *                                                    no alpha
 *      hard alpha/luminance threshold in this pass   19.421 (no effect): the scene target's
 *                                                    alpha is non-zero across the frame, so
 *                                                    "covered" is 1 nearly everywhere
 *      spine-shaped ERASE mask on the static art     a NO-OP by construction: the art sits at
 *                                                    index 0 and the spine draws in FRONT of
 *                                                    it, so the mask can only remove pixels
 *                                                    that were already hidden (786 px of
 *                                                    374400 on lin_nian#10, all antialiasing)
 *
 *  What ships instead is in `SceneIllust.tsx`: the static art drawn first, DEFOCUSED, so it
 *  fills as a vista without recognisable detail. See `gapFillOn` (`?gapfill=0`) there. */

export interface IHDRScene {
    /** Half-float target the scene is drawn into each frame (before tonemap). */
    target: PIXI.RenderTexture;
    /** Full-screen quad that tonemaps `target` (+bloom) to the screen; add it to the stage. */
    mesh: PIXI.Mesh<PIXI.Shader>;
    /** Update the bloom texture from the current `target`. Call each frame AFTER
     *  drawing the scene into `target` and BEFORE rendering the stage. */
    prepare(renderer: PIXI.IRenderer): void;
    resize(width: number, height: number, resolution: number): void;
    /** Replace the composite-transfer exponent at runtime (see sceneCompositeGamma). Exists
     *  for the `?gammadomain=entrance` arm: the calibration table's every data point is
     *  entrance-domain, and this lets the handoff restore 1.0 for the settled phase so the
     *  fitted exponent is confined to the domain it has data for. */
    setGamma(g: number): void;
    destroy(): void;
}

function supportsFloatTarget(renderer: PIXI.IRenderer): boolean {
    const anyR = renderer as unknown as { gl?: WebGL2RenderingContext; context?: { webGLVersion?: number } };
    const gl = anyR.gl;
    if (!gl || anyR.context?.webGLVersion !== 2) return false;
    return !!gl.getExtension("EXT_color_buffer_float");
}

function makeQuad(width: number, height: number): PIXI.Geometry {
    // TL, TR, BR, BL in screen pixels, with sprite-style UVs so the render target
    // shows upright (matches how a PIXI.Sprite samples a RenderTexture). Raw Geometry
    // with explicit `aVertexPosition`/`aUV`: PIXI.MeshGeometry names its UV attribute
    // differently, so the shader can't bind.
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
    // DIAGNOSTIC (`?ldr=1`): render into a fixed-point RGBA8 target instead of half-float.
    // A fixed-point framebuffer SATURATES on every blend, so each additive draw clamps to
    // [0,1] as it accumulates, like Unity's LDR path (the dynchar camera doesn't serialize
    // m_AllowHDR; the mobile tier default renders LDR). The ceiling is the target's own 1.0,
    // a GL semantic, nothing chosen here. The bright-pass (threshold 1.0) is then empty, so
    // bloom is off in this arm too, as LDR implies.
    const ldrArm = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("ldr") === "1";
    let target: PIXI.RenderTexture;
    try {
        target = PIXI.RenderTexture.create({
            width,
            height,
            resolution,
            format: PIXI.FORMATS.RGBA,
            type: ldrArm ? PIXI.TYPES.UNSIGNED_BYTE : PIXI.TYPES.HALF_FLOAT,
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
    // before the first scene pass therefore reads uninitialised GPU memory - zero-filled on
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
        uDstA: typeof window !== "undefined" && new URLSearchParams(window.location.search).get("dsta") === "1" ? 1 : 0,
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
        setGamma(g: number) {
            shader.uniforms.uGamma = g;
        },
        resize(w: number, h: number, res: number) {
            // ⚠️ setResolution BEFORE resize, never after. `BaseTexture.setResolution` RESCALES the
            // texture's screen dimensions to preserve its pixel size (`width = width*oldRes/res`),
            // so resizing first and setting the resolution second leaves every target off by
            // `oldRes/res`. Latent until the renderer's resolution actually changed at runtime -
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
