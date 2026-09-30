/**
 * THE BROWSER HALF OF THE EPUB: fetch an asset, resize it, pick its format,
 * load a font, compose the cover. Everything else in `book/` is pure.
 *
 * IMAGES: at most 1,200 px wide, JPEG at quality 0.8, which is what the plan
 * measured (a background 3.9 MB of PNG per story -> 0.6 MB). A PNG whose
 * pixels are not all opaque stays PNG, because a JPEG would paint its
 * transparent corners black: 37 backgrounds and 184 CGs over EN carry real
 * alpha. The test is a SAMPLE, a 64 x 64 downscale of the whole picture, so a
 * one-pixel hole can be missed and would be flattened onto white; it is a
 * trade for not reading 1.4 M pixels per image.
 */
import { asset } from "#/components/operators/detail/impl/assets";
import type { EpubDeps, EpubFont, EpubImage } from "./epub";
import type { PdfDeps, PdfFont } from "./pdf";
import type { Book, ImageVariant, Typeface } from "./types";

export const MAX_IMAGE_WIDTH = 1200;
export const JPEG_QUALITY = 0.8;
/** A scene background is a small uncaptioned figure above its rule: <= 600 px at quality 0.75. */
export const THUMB_WIDTH = 600;
export const THUMB_QUALITY = 0.75;
const ALPHA_SAMPLE = 64;

type AnyCanvas = OffscreenCanvas | HTMLCanvasElement;

function canvasOf(w: number, h: number): AnyCanvas {
    if (typeof OffscreenCanvas !== "undefined") return new OffscreenCanvas(w, h);
    const c = document.createElement("canvas");
    c.width = w;
    c.height = h;
    return c;
}

function context(c: AnyCanvas): CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D {
    const ctx = c.getContext("2d") as CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D | null;
    if (!ctx) throw new Error("2d canvas unavailable");
    return ctx;
}

async function encode(c: AnyCanvas, type: "image/jpeg" | "image/png", quality?: number): Promise<Uint8Array> {
    let blob: Blob | null;
    if ("convertToBlob" in c) blob = await c.convertToBlob({ type, quality });
    else blob = await new Promise<Blob | null>((resolve) => (c as HTMLCanvasElement).toBlob(resolve, type, quality));
    if (!blob) throw new Error("encode failed");
    return new Uint8Array(await blob.arrayBuffer());
}

/** True when any sampled pixel is not fully opaque. */
function hasAlpha(bitmap: ImageBitmap): boolean {
    const w = Math.min(ALPHA_SAMPLE, bitmap.width);
    const h = Math.min(ALPHA_SAMPLE, bitmap.height);
    const c = canvasOf(w, h);
    const ctx = context(c);
    ctx.clearRect(0, 0, w, h);
    ctx.drawImage(bitmap, 0, 0, w, h);
    const data = ctx.getImageData(0, 0, w, h).data;
    for (let i = 3; i < data.length; i += 4) if (data[i] < 255) return true;
    return false;
}

/** Fetch one asset path and prepare it for the book. */
export async function prepareImage(path: string, signal?: AbortSignal, variant: ImageVariant = "full"): Promise<EpubImage | null> {
    const maxWidth = variant === "thumb" ? THUMB_WIDTH : MAX_IMAGE_WIDTH;
    const quality = variant === "thumb" ? THUMB_QUALITY : JPEG_QUALITY;
    const res = await fetch(asset(path), { signal });
    if (!res.ok) return null;
    const blob = await res.blob();
    const bitmap = await createImageBitmap(blob);
    try {
        const alpha = blob.type === "image/png" && hasAlpha(bitmap);
        const scale = bitmap.width > maxWidth ? maxWidth / bitmap.width : 1;
        if (alpha && scale === 1) return { bytes: new Uint8Array(await blob.arrayBuffer()), mime: "image/png" };
        const w = Math.max(1, Math.round(bitmap.width * scale));
        const h = Math.max(1, Math.round(bitmap.height * scale));
        const c = canvasOf(w, h);
        const ctx = context(c);
        if (!alpha) {
            ctx.fillStyle = "#000";
            ctx.fillRect(0, 0, w, h);
        }
        ctx.imageSmoothingQuality = "high";
        ctx.drawImage(bitmap, 0, 0, w, h);
        return alpha ? { bytes: await encode(c, "image/png"), mime: "image/png" } : { bytes: await encode(c, "image/jpeg", quality), mime: "image/jpeg" };
    } finally {
        bitmap.close();
    }
}

const FONT_FILES: Record<Exclude<Typeface, "device">, { regular: string; bold: string; format: "woff" | "woff2" }> = {
    inter: { regular: "/fonts/inter-latin-400-normal.woff", bold: "/fonts/inter-latin-700-normal.woff", format: "woff" },
    opendyslexic: { regular: "/opendyslexic/OpenDyslexic-Regular.woff2", bold: "/opendyslexic/OpenDyslexic-Bold.woff2", format: "woff2" },
};

export async function loadBookFont(typeface: Exclude<Typeface, "device">, signal?: AbortSignal): Promise<EpubFont | null> {
    const files = FONT_FILES[typeface];
    const [a, b] = await Promise.all([fetch(files.regular, { signal }), fetch(files.bold, { signal })]);
    if (!a.ok || !b.ok) return null;
    return { regular: new Uint8Array(await a.arrayBuffer()), bold: new Uint8Array(await b.arrayBuffer()), format: files.format };
}

async function bitmapOf(path: string | undefined, signal?: AbortSignal): Promise<ImageBitmap | null> {
    if (!path) return null;
    try {
        const res = await fetch(asset(path), { signal });
        if (!res.ok) return null;
        return await createImageBitmap(await res.blob());
    } catch (err) {
        if (signal?.aborted) throw err;
        return null;
    }
}

export const COVER_W = 1200;
export const COVER_H = 1600;

/**
 * The cover: the key visual (or the chapter cover) filling a 1200 x 1600 page,
 * darkened toward the foot, with the group's LOGOTYPE over it, and the written
 * title when there is no logotype. Null only when there is no art at all.
 */
export async function composeCover(book: Book, signal?: AbortSignal): Promise<EpubImage | null> {
    const { titleImageUrl, bannerUrl, coverUrl } = book.meta.cover;
    const [art, title] = await Promise.all([bitmapOf(bannerUrl ?? coverUrl, signal).then((b) => b ?? (bannerUrl ? bitmapOf(coverUrl, signal) : null)), bitmapOf(titleImageUrl, signal)]);
    if (!art && !title) return null;
    const c = canvasOf(COVER_W, COVER_H);
    const ctx = context(c);
    ctx.fillStyle = "#101114";
    ctx.fillRect(0, 0, COVER_W, COVER_H);
    if (art) {
        const scale = Math.max(COVER_W / art.width, COVER_H / art.height);
        const w = art.width * scale;
        const h = art.height * scale;
        ctx.drawImage(art, (COVER_W - w) / 2, (COVER_H - h) / 2, w, h);
        art.close();
    }
    const shade = ctx.createLinearGradient(0, COVER_H * 0.45, 0, COVER_H);
    shade.addColorStop(0, "rgba(0,0,0,0)");
    shade.addColorStop(1, "rgba(0,0,0,0.78)");
    ctx.fillStyle = shade;
    ctx.fillRect(0, 0, COVER_W, COVER_H);
    if (title) {
        const w = COVER_W * 0.8;
        const h = (title.height / title.width) * w;
        ctx.drawImage(title, (COVER_W - w) / 2, COVER_H - h - 140, w, h);
        title.close();
    } else {
        ctx.fillStyle = "#fff";
        ctx.textAlign = "center";
        ctx.font = "700 72px sans-serif";
        ctx.fillText(book.meta.title, COVER_W / 2, COVER_H - 200, COVER_W - 160);
    }
    return { bytes: await encode(c, "image/jpeg", 0.85), mime: "image/jpeg" };
}

export const browserEpubDeps: EpubDeps = { loadImage: prepareImage, loadFont: loadBookFont, cover: composeCover };

async function bytesOf(url: string, signal?: AbortSignal): Promise<Uint8Array | null> {
    const res = await fetch(url, { signal });
    return res.ok ? new Uint8Array(await res.arrayBuffer()) : null;
}

/** The PDF's faces: the EPUB's files plus Inter's italics; OpenDyslexic ships no italic and reads upright there. */
export async function loadPdfFont(typeface: Exclude<Typeface, "device">, signal?: AbortSignal): Promise<PdfFont | null> {
    const base = await loadBookFont(typeface, signal);
    if (!base) return null;
    if (typeface !== "inter") return base;
    // Copied into `public/fonts` from `@fontsource/inter` (the package the site already depends on); a `?url` import of the package file 404'd on the dev server.
    const [italic, boldItalic] = await Promise.all([bytesOf("/fonts/inter-latin-400-italic.woff", signal), bytesOf("/fonts/inter-latin-700-italic.woff", signal)]);
    return { ...base, italic: italic ?? undefined, boldItalic: boldItalic ?? undefined };
}

export const browserPdfDeps: PdfDeps = { loadImage: prepareImage, loadFont: loadPdfFont, cover: composeCover };
