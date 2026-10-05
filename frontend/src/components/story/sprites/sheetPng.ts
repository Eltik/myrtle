/**
 * "Download sheet": every expression of one folder on ONE canvas at the
 * plate's own resolution (a body is 256 to 2,048 px on a side over the 12,107
 * EN expressions, 1,024 for 8,134 of them), each
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
import { type ISheetLayout, SHEET_METRICS, sheetLayout } from "./gallery";

export interface ISheetColours {
    background: string;
    cell: string;
    ink: string;
    muted: string;
}

/** The page's own colours, read off the live theme, so a dark page saves a dark sheet. */
export function readThemeColours(): ISheetColours {
    const style = getComputedStyle(document.documentElement);
    const read = (v: string, fallback: string) => style.getPropertyValue(v).trim() || fallback;
    return { background: read("--background", "#111"), cell: read("--secondary", "#222"), ink: read("--foreground", "#eee"), muted: read("--muted-foreground", "#999") };
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
 * Compose and save the sheet at full size, or smaller only when the canvas
 * limits force it (`note`). Resolves with the canvas size, the PNG's bytes and
 * the wall time once the download has been handed to the browser; rejects if
 * any body fails to load.
 */
export async function downloadSheet(opts: { base: string; title: string; variants: readonly StorySpriteVariant[]; colours: ISheetColours }): Promise<ISheetResult> {
    const started = performance.now();
    const { base, title, variants, colours } = opts;
    const images = await Promise.all(
        variants.map(async (v) => {
            const body = await loadImage(asset(v.bodyUrl), { crossOrigin: true });
            const face = v.faceUrl ? await loadImage(asset(v.faceUrl), { crossOrigin: true }).catch(() => null) : null;
            return { body, face };
        }),
    );
    const plate = Math.max(1, ...images.map((i) => Math.max(i.body.naturalWidth, i.body.naturalHeight)));
    const layout = sheetLayout(variants.length, plate, 1);
    const canvas = document.createElement("canvas");
    canvas.width = layout.width;
    canvas.height = layout.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no 2d context");
    ctx.imageSmoothingQuality = "high";
    ctx.fillStyle = colours.background;
    ctx.fillRect(0, 0, layout.width, layout.height);
    const px = (measure: number) => Math.round(measure * layout.scale);
    ctx.textBaseline = "alphabetic";
    ctx.fillStyle = colours.ink;
    ctx.font = `600 ${px(SHEET_METRICS.titleFont)}px system-ui, sans-serif`;
    ctx.fillText(title, layout.gap, layout.head * 0.55);
    ctx.fillStyle = colours.muted;
    ctx.font = `${px(SHEET_METRICS.folderFont)}px ui-monospace, monospace`;
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
        ctx.font = `${px(SHEET_METRICS.keyFont)}px ui-monospace, monospace`;
        ctx.fillText(v.key, x + px(SHEET_METRICS.keyInset), y + layout.cell + layout.label * 0.72);
    });
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/png"));
    if (!blob) throw new Error("canvas.toBlob returned null");
    downloadBlob(blob, `${base}-expressions.png`);
    return { width: layout.width, height: layout.height, bytes: blob.size, ms: Math.round(performance.now() - started), note: layout.note };
}
