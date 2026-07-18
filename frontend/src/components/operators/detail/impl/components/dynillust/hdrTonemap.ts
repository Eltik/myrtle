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
void main() {
    vec4 c = texture2D(uSampler, vUV);
    vec3 rgb = c.rgb;
    float m = max(max(rgb.r, rgb.g), rgb.b);
    if (m > uKnee) {
        // soft-compress the tail [knee, +inf) -> [knee, 1)
        float e = (m - uKnee) / max(1e-4, 1.0 - uKnee);
        float t = uKnee + (1.0 - uKnee) * (e / (1.0 + e));
        rgb *= t / m;
    }
    vec3 bloom = texture2D(uBloom, vUV).rgb * uBloomIntensity;
    gl_FragColor = vec4(rgb + bloom, max(c.a, 0.0));
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
/** How strongly the blurred bloom is added back on top of the tonemapped scene. */
const BLOOM_INTENSITY = 0.85;
/** Bloom RTs run at 1/N res — cheaper and gives a wider, softer glow for free. */
const BLOOM_DOWNSCALE = 2;
/** Gaussian blur strength (in bloom-RT pixels) applied to the bright-pass. */
const BLOOM_BLUR = 10;

/** Highlights at/under this (premultiplied) level pass through untouched; brighter
 *  stacks are compressed toward 1 with hue preserved. 0.9 keeps near-white LDR
 *  regions almost intact (a pure-white 1.0 maps to ~0.95) while taming the >1
 *  additive blowouts hard. */
const DEFAULT_KNEE = 0.9;

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
export function createHDRScene(renderer: PIXI.IRenderer, width: number, height: number, resolution: number, knee: number = DEFAULT_KNEE): IHDRScene | null {
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
    const brightMesh = new PIXI.Mesh(makeQuad(bw, bh), PIXI.Shader.from(TONEMAP_VERT, BRIGHT_FRAG, { uSampler: target, uThreshold: BLOOM_THRESHOLD }));
    const brightContainer = new PIXI.Container();
    brightContainer.addChild(brightMesh);
    const blur = new PIXI.BlurFilter(BLOOM_BLUR);
    blur.quality = 4;
    brightContainer.filters = [blur];

    const geometry = makeQuad(width, height);
    const shader = PIXI.Shader.from(TONEMAP_VERT, TONEMAP_FRAG, { uSampler: target, uBloom: bloomRT, uKnee: knee, uBloomIntensity: BLOOM_INTENSITY });
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
        },
        resize(w: number, h: number, res: number) {
            target.resize(w, h, true);
            target.baseTexture.setResolution(res);
            const positions = mesh.geometry.getBuffer("aVertexPosition");
            positions.data = new Float32Array([0, 0, w, 0, w, h, 0, h]) as unknown as typeof positions.data;
            positions.update();
            const nbw = Math.max(1, Math.round(w / BLOOM_DOWNSCALE));
            const nbh = Math.max(1, Math.round(h / BLOOM_DOWNSCALE));
            bloomRT.resize(nbw, nbh, true);
            bloomRT.baseTexture.setResolution(res);
            setBrightQuad(nbw, nbh);
        },
        destroy() {
            mesh.destroy();
            brightMesh.destroy();
            brightContainer.destroy();
            blur.destroy();
            bloomRT.destroy(true);
            target.destroy(true);
        },
    };
}
