/**
 * "Download sheet": every expression of one folder composed onto ONE canvas,
 * body plate plus face patch exactly as `Body` stacks them, in the same head
 * crop the sheet's grid shows, each captioned with its `#N$M` key.
 *
 * The images are fetched again with `crossOrigin="anonymous"`, because the
 * assets are served from the backend's origin and a canvas that draws a
 * non-CORS image cannot be read back; the backend answers those requests with
 * CORS headers (the library's palette sampler reads the same files the same way).
 */
import { asset } from "#/components/operators/detail/impl/assets";
import { downloadBlob, loadImage } from "#/lib/utils";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";
import { CELL_CROP, cropForVariant, sheetLayout } from "./gallery";

const CELL = 220;
const LABEL = 26;
const HEAD = 64;
const GAP = 12;
const MAX_COLS = 6;

/** Draw one expression into a `cell` px square whose top-left is (x, y). */
function drawCell(ctx: CanvasRenderingContext2D, variant: StorySpriteVariant, body: HTMLImageElement, face: HTMLImageElement | null, x: number, y: number, cell: number): void {
    const crop = cropForVariant(variant, CELL_CROP);
    const plate = (crop.size / 100) * cell;
    const px = x + (crop.left / 100) * cell;
    const py = y + (crop.top / 100) * cell;
    ctx.save();
    ctx.beginPath();
    ctx.rect(x, y, cell, cell);
    ctx.clip();
    ctx.drawImage(body, px, py, plate, plate);
    const pos = variant.facePos;
    const bw = variant.bodySize?.w ?? body.naturalWidth;
    const bh = variant.bodySize?.h ?? body.naturalHeight;
    if (face && pos && bw > 0 && bh > 0) {
        ctx.drawImage(face, px + (pos.x / bw) * plate, py + (pos.y / bh) * plate, (pos.w / bw) * plate, (pos.h / bh) * plate);
    }
    ctx.restore();
}

/** Compose and save the sheet. Resolves when the download has been handed to the browser; rejects if any body fails to load. */
export async function downloadSheet(opts: { base: string; title: string; variants: readonly StorySpriteVariant[]; background: string; ink: string; muted: string }): Promise<void> {
    const { base, title, variants, background, ink, muted } = opts;
    const layout = sheetLayout(variants.length, { cell: CELL, label: LABEL, head: HEAD, gap: GAP, maxCols: MAX_COLS });
    const images = await Promise.all(
        variants.map(async (v) => {
            const body = await loadImage(asset(v.bodyUrl), { crossOrigin: true });
            const face = v.faceUrl ? await loadImage(asset(v.faceUrl), { crossOrigin: true }).catch(() => null) : null;
            return { body, face };
        }),
    );
    const canvas = document.createElement("canvas");
    canvas.width = layout.width;
    canvas.height = layout.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no 2d context");
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, layout.width, layout.height);
    ctx.fillStyle = ink;
    ctx.font = "600 26px system-ui, sans-serif";
    ctx.textBaseline = "middle";
    ctx.fillText(title, GAP, HEAD / 2 - 6);
    ctx.fillStyle = muted;
    ctx.font = "13px ui-monospace, monospace";
    ctx.fillText(base, GAP, HEAD / 2 + 18);
    variants.forEach((v, i) => {
        const img = images[i];
        if (!img) return;
        const col = i % layout.cols;
        const row = Math.floor(i / layout.cols);
        const x = GAP + col * (CELL + GAP);
        const y = HEAD + GAP + row * (CELL + LABEL + GAP);
        drawCell(ctx, v, img.body, img.face, x, y, CELL);
        ctx.fillStyle = muted;
        ctx.font = "13px ui-monospace, monospace";
        ctx.fillText(v.key, x + 2, y + CELL + LABEL / 2);
    });
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/png"));
    if (!blob) throw new Error("canvas.toBlob returned null");
    downloadBlob(blob, `${base}-expressions.png`);
}
