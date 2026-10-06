/**
 * WebGL availability, probed without importing PixiJS.
 *
 * pixi.js 7 ships no canvas fallback: with WebGL unavailable (hardware acceleration off, a
 * blocklisted GPU, a VM or remote desktop) `new PIXI.Application` throws "Unable to
 * auto-detect a suitable renderer". Surfaces that only decorate static art check this
 * first so they neither download the renderer chunk nor reach that throw.
 */

let supported: boolean | undefined;

/**
 * Mirrors PixiJS's own `isWebGLSupported` (a `webgl` context with a stencil buffer, no
 * performance-caveat failure) so the two agree, and caches the answer the same way: PixiJS
 * keeps its first verdict for the page's lifetime, so a later fresh probe could not help.
 */
export function hasWebGL(): boolean {
    if (supported !== undefined) return supported;
    if (typeof document === "undefined") return false;
    try {
        const canvas = document.createElement("canvas");
        const options: WebGLContextAttributes = { stencil: true, failIfMajorPerformanceCaveat: false };
        const gl = (canvas.getContext("webgl", options) ?? canvas.getContext("experimental-webgl", options)) as WebGLRenderingContext | null;
        supported = !!gl?.getContextAttributes()?.stencil;
        // Hand the probe's context back now; browsers cap live contexts (~16).
        gl?.getExtension("WEBGL_lose_context")?.loseContext();
    } catch {
        supported = false;
    }
    return supported;
}
