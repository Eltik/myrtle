/**
 * "Download sheet": every expression of one folder on ONE canvas at the
 * plate's own resolution (the 1024 px bodies, 1280 for the large ones), each
 * the body plate with its face patch composited by the same math as `Body`
 * (`facePos / bodySize` of the drawn plate), captioned with its `#N$M` key,
 * under the character's name and folder, in the page's own colours.
 *
 * The images are fetched again with `crossOrigin="anonymous"`, because the
 * assets are served from the backend's origin and a canvas that draws a
 * non-CORS image cannot be read back; the backend answers those requests with
 * CORS headers (the library's palette sampler reads the same files the same way).
 */
import { asset } from "#/components/operators/detail/impl/assets";
import { downloadBlob, loadImage } from "#/lib/utils";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";
import { type ISheetLayout, sheetLayout } from "./gallery";

export interface ISheetColours {
    background: string;
    cell: string;
    ink: string;
    muted: string;
}

export interface ISheetResult {
    width: number;
    height: number;
    bytes: number;
    ms: number;
    note: ISheetLayout["note"];
}

/** Draw one expression as its whole plate into a `cell` px square at (x, y). */
function drawPlate(ctx: CanvasRenderingContext2D, variant: StorySpriteVariant, body: HTMLImageElement, face: HTMLImageElement | null, x: number, y: number, cell: number): void {
    ctx.drawImage(body, x, y, cell, cell);
    const pos = variant.facePos;
    const bw = variant.bodySize?.w ?? body.naturalWidth;
    const bh = variant.bodySize?.h ?? body.naturalHeight;
    if (face && pos && bw > 0 && bh > 0) {
        ctx.drawImage(face, x + (pos.x / bw) * cell, y + (pos.y / bh) * cell, (pos.w / bw) * cell, (pos.h / bh) * cell);
    }
}

/**
 * Compose and save the sheet at `scale` (1 full, 0.5 half). Resolves with the
 * canvas size, the PNG's bytes and the wall time once the download has been
 * handed to the browser; rejects if any body fails to load.
 */
export async function downloadSheet(opts: { base: string; title: string; variants: readonly StorySpriteVariant[]; scale: number; colours: ISheetColours }): Promise<ISheetResult> {
    const started = performance.now();
    const { base, title, variants, scale, colours } = opts;
    const images = await Promise.all(
        variants.map(async (v) => {
            const body = await loadImage(asset(v.bodyUrl), { crossOrigin: true });
            const face = v.faceUrl ? await loadImage(asset(v.faceUrl), { crossOrigin: true }).catch(() => null) : null;
            return { body, face };
        }),
    );
    const plate = Math.max(1, ...images.map((i) => Math.max(i.body.naturalWidth, i.body.naturalHeight)));
    const layout = sheetLayout(variants.length, plate, scale);
    const canvas = document.createElement("canvas");
    canvas.width = layout.width;
    canvas.height = layout.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no 2d context");
    ctx.imageSmoothingQuality = "high";
    ctx.fillStyle = colours.background;
    ctx.fillRect(0, 0, layout.width, layout.height);
    const s = layout.scale;
    ctx.textBaseline = "alphabetic";
    ctx.fillStyle = colours.ink;
    ctx.font = `600 ${Math.round(76 * s)}px system-ui, sans-serif`;
    ctx.fillText(title, layout.gap, layout.head * 0.55);
    ctx.fillStyle = colours.muted;
    ctx.font = `${Math.round(34 * s)}px ui-monospace, monospace`;
    ctx.fillText(base, layout.gap, layout.head * 0.88);
    variants.forEach((v, i) => {
        const img = images[i];
        if (!img) return;
        const col = i % layout.cols;
        const row = Math.floor(i / layout.cols);
        const x = layout.gap + col * (layout.cell + layout.gap);
        const y = layout.head + layout.gap + row * (layout.cell + layout.label + layout.gap);
        ctx.fillStyle = colours.cell;
        ctx.fillRect(x, y, layout.cell, layout.cell);
        drawPlate(ctx, v, img.body, img.face, x, y, layout.cell);
        ctx.fillStyle = colours.muted;
        ctx.font = `${Math.round(44 * s)}px ui-monospace, monospace`;
        ctx.fillText(v.key, x + Math.round(4 * s), y + layout.cell + layout.label * 0.72);
    });
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/png"));
    if (!blob) throw new Error("canvas.toBlob returned null");
    downloadBlob(blob, `${base}-expressions${scale < 1 ? "-half" : ""}.png`);
    return { width: layout.width, height: layout.height, bytes: blob.size, ms: Math.round(performance.now() - started), note: layout.note };
}
