import * as PIXI from "pixi.js";

/**
 * Save the frame an animation viewer is showing as a transparent PNG.
 *
 * The frame is drawn again into an offscreen render texture and read back from
 * that, never from the viewer's canvas: the canvas is created without
 * `preserveDrawingBuffer`, so between two of the viewer's renders its buffer
 * may already be cleared, and it carries the page's device pixel ratio and
 * whatever the viewer happens to crop. Drawing the target alone also leaves
 * out anything behind it, so the background stays transparent.
 *
 * The target is not touched: its own transform is undone through the render's
 * projection, so position, scale and the viewer's next render are unchanged.
 */

/** A rectangle in the target's LOCAL units (for a spine skeleton, skeleton units). */
export interface IFrameRegion {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface IFrameSize {
    width: number;
    height: number;
    /** Output pixels per local unit after any shrink to fit `maxDim`. */
    scale: number;
}

/** Transparent margin, in output pixels, so a filtered edge on the region's border is not cut. */
export const FRAME_PNG_PADDING = 4;

/** Past this a render texture is too large for some GPUs to allocate. */
export const FRAME_PNG_MAX_DIM = 4096;

/**
 * The PNG's pixel size for `region` drawn at `scale` output pixels per local
 * unit. A size past `maxDim` on either side shrinks the scale to fit rather
 * than failing the save.
 */
export function framePngSize(region: IFrameRegion, scale: number, maxDim: number = FRAME_PNG_MAX_DIM): IFrameSize {
    const pad = FRAME_PNG_PADDING * 2;
    const longest = Math.max(region.width, region.height) * scale + pad;
    const fitted = longest > maxDim ? (scale * (maxDim - pad)) / (longest - pad) : scale;
    return {
        width: Math.max(1, Math.ceil(region.width * fitted) + pad),
        height: Math.max(1, Math.ceil(region.height * fitted) + pad),
        scale: fitted,
    };
}

/**
 * The projection that maps the target's world position (its own transform
 * applied to a local point) onto the PNG: `region`'s top-left lands at the
 * padding and one local unit spans `scale` pixels. Undoing `localTransform`
 * here is what lets the target render as-is.
 */
export function frameProjection(localTransform: PIXI.Matrix, region: IFrameRegion, scale: number, flipX = false): PIXI.Matrix {
    // Mirrored, the region's right edge lands at the padding instead of its left.
    const toFrame = flipX ? new PIXI.Matrix(-scale, 0, 0, scale, FRAME_PNG_PADDING + (region.x + region.width) * scale, FRAME_PNG_PADDING - region.y * scale) : new PIXI.Matrix(scale, 0, 0, scale, FRAME_PNG_PADDING - region.x * scale, FRAME_PNG_PADDING - region.y * scale);
    return localTransform.clone().invert().prepend(toFrame);
}

/** Union of two regions; `null` on either side is ignored. */
export function unionRegion(a: IFrameRegion | null, b: IFrameRegion | null): IFrameRegion | null {
    if (!a || a.width <= 0 || a.height <= 0) return b && b.width > 0 && b.height > 0 ? b : null;
    if (!b || b.width <= 0 || b.height <= 0) return a;
    const x = Math.min(a.x, b.x);
    const y = Math.min(a.y, b.y);
    return { x, y, width: Math.max(a.x + a.width, b.x + b.width) - x, height: Math.max(a.y + a.height, b.y + b.height) - y };
}

/** The renderer's own texture-size limit, capped at {@link FRAME_PNG_MAX_DIM}. */
function maxDimOf(renderer: PIXI.IRenderer): number {
    const gl = renderer instanceof PIXI.Renderer ? renderer.gl : null;
    const limit = gl ? Number(gl.getParameter(gl.MAX_TEXTURE_SIZE)) : FRAME_PNG_MAX_DIM;
    return Math.min(FRAME_PNG_MAX_DIM, Number.isFinite(limit) && limit > 0 ? limit : FRAME_PNG_MAX_DIM);
}

/** The size {@link renderFramePng} will produce on this renderer. */
export function framePngSizeOn(renderer: PIXI.IRenderer, region: IFrameRegion, scale: number): IFrameSize {
    return framePngSize(region, scale, maxDimOf(renderer));
}

/**
 * Draw `target` as it stands now, cropped to `region` (local units) at
 * `scale` output pixels per unit, mirrored when `flipX`, and encode it as a
 * PNG. The caller applies the pose it wants first; this only draws and reads back.
 */
export async function renderFramePng(renderer: PIXI.IRenderer, target: PIXI.Container, region: IFrameRegion, scale: number, flipX = false): Promise<Blob> {
    const size = framePngSizeOn(renderer, region, scale);
    target.transform.updateLocalTransform();
    const transform = frameProjection(target.transform.localTransform, region, size.scale, flipX);

    const renderTexture = PIXI.RenderTexture.create({ width: size.width, height: size.height, resolution: 1 });
    let canvas: PIXI.ICanvas;
    try {
        renderer.render(target, { renderTexture, clear: true, transform });
        canvas = renderer.extract.canvas(renderTexture);
    } finally {
        renderTexture.destroy(true);
    }

    return new Promise<Blob>((resolve, reject) => {
        if (!canvas.toBlob) {
            reject(new Error("frame-png: the extract canvas cannot encode"));
            return;
        }
        canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error("frame-png: PNG encoding failed"))), "image/png");
    });
}

/** Characters a file name cannot carry on some system, plus `#` and whitespace. */
const UNSAFE_FILENAME_RE = /[\\/:*?"<>|#\s]+/g;

/** The parts made file-safe and joined with `_`, empty ones dropped: `amiya_default_front_Idle`. */
export function fileStem(parts: readonly (string | null | undefined)[]): string {
    return parts
        .map((part) =>
            (part ?? "")
                .trim()
                .replace(UNSAFE_FILENAME_RE, "-")
                .replace(/^-+|-+$/g, ""),
        )
        .filter((part) => part.length > 0)
        .join("_");
}

export interface IFramePngName {
    /** Subject first, most general to most specific: operator, skin, view, animation. Empty parts are dropped. */
    parts: readonly (string | null | undefined)[];
    /** 0-based frame, as the frame stepper reports it. */
    frame: number;
    total: number;
    seconds: number;
    scale: number;
}

/**
 * `amiya_default_front_Idle_f07-of-40_0.20s.png`, with `@2x` before the
 * extension past 1x. The frame counts from 1 to match the stepper's readout
 * and is zero-padded so a stepped sequence sorts in order.
 */
export function framePngFilename({ parts, frame, total, seconds, scale }: IFramePngName): string {
    const subject = fileStem(parts);
    const digits = String(total).length;
    const position = `f${String(frame + 1).padStart(digits, "0")}-of-${total}_${seconds.toFixed(2)}s`;
    const suffix = scale === 1 ? "" : `@${scale}x`;
    return `${subject ? `${subject}_` : ""}${position}${suffix}.png`;
}
