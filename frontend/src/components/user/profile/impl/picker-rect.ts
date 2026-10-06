/**
 * Where the floating background picker sits from `sm` up: a rectangle in viewport
 * pixels the author drags by its title bar and resizes by its edges, kept inside
 * the viewport and remembered per browser.
 */
export interface IRect {
    x: number;
    y: number;
    width: number;
    height: number;
}

/** The gap kept between the picker and every viewport edge. */
export const RECT_MARGIN = 16;
/**
 * The smallest picker. 520 px tall, not 420: at 490 x 420 the fixed parts
 * (title, description, source tabs, sliders, hint, footer) left the body 36 px
 * (measured 2026-10-06). The compact layout under {@link COMPACT_HEIGHT} raises
 * that to 188 px at 420, of which the Character art tab's kind tabs, search
 * and rarity/class filters take about 164, leaving 24 px of tiles; 100 px more
 * height shows one full row (a 76 px tile and its two-line name).
 */
export const RECT_MIN_WIDTH = 480;
export const RECT_MIN_HEIGHT = 520;
/** Below this height the picker drops its description and hint line to give the tiles the room. */
export const COMPACT_HEIGHT = 600;
/**
 * The size it opens at before the author has moved it, clamped to the viewport. At 960 the
 * Gallery tab's 224 px rail leaves the tile grid 664 px, three 176 px-minimum tiles a row
 * at 216 px each, and the width is past {@link WIDE_WIDTH}, so a fresh picker opens with
 * the rail. It covers more of the header's art than the 720 x 680 it replaced; the author
 * drags it aside, and it reopens where it was left.
 */
const DEFAULT_WIDTH = 960;
const DEFAULT_HEIGHT = 720;
/** From this width up the Gallery tab lays its sources, categories and stories in a left rail beside the tiles; under it they stack in rows above them. */
export const WIDE_WIDTH = 760;

/** Which edges a resize moves: a corner moves two. */
export interface IEdges {
    left?: boolean;
    right?: boolean;
    top?: boolean;
    bottom?: boolean;
}

/** The largest picker the viewport holds; the minimum yields to it on a small window, so the picker never leaves the screen. */
function limits(viewportWidth: number, viewportHeight: number) {
    const maxWidth = Math.max(0, viewportWidth - 2 * RECT_MARGIN);
    const maxHeight = Math.max(0, viewportHeight - 2 * RECT_MARGIN);
    return { maxWidth, maxHeight, minWidth: Math.min(RECT_MIN_WIDTH, maxWidth), minHeight: Math.min(RECT_MIN_HEIGHT, maxHeight) };
}

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

/** `rect` sized into the min..max range and moved fully inside the viewport's margins. */
export function clampRect(rect: IRect, viewportWidth: number, viewportHeight: number): IRect {
    const { maxWidth, maxHeight, minWidth, minHeight } = limits(viewportWidth, viewportHeight);
    const width = clamp(Math.round(rect.width), minWidth, maxWidth);
    const height = clamp(Math.round(rect.height), minHeight, maxHeight);
    return {
        width,
        height,
        x: clamp(Math.round(rect.x), RECT_MARGIN, viewportWidth - RECT_MARGIN - width),
        y: clamp(Math.round(rect.y), RECT_MARGIN, viewportHeight - RECT_MARGIN - height),
    };
}

/**
 * Where the picker opens: docked to the lower LEFT. From `sm` up the header's
 * text side is under a scrim of at least 84% card for its first 760 px and the
 * art shows to the right of it, so a picker on the left covers the least of
 * the preview it drives.
 */
export function defaultRect(viewportWidth: number, viewportHeight: number): IRect {
    return clampRect({ x: RECT_MARGIN, y: viewportHeight, width: DEFAULT_WIDTH, height: DEFAULT_HEIGHT }, viewportWidth, viewportHeight);
}

/** `start` moved by `(dx, dy)`, inside the viewport. */
export function moveRect(start: IRect, dx: number, dy: number, viewportWidth: number, viewportHeight: number): IRect {
    return clampRect({ ...start, x: start.x + dx, y: start.y + dy }, viewportWidth, viewportHeight);
}

/**
 * `start` with the given edges dragged by `(dx, dy)`. The opposite edge stays
 * put: a left or top drag that hits the minimum stops instead of pushing the
 * picker across the screen, and none crosses the viewport's margin.
 */
export function resizeRect(start: IRect, edges: IEdges, dx: number, dy: number, viewportWidth: number, viewportHeight: number): IRect {
    const { maxWidth, maxHeight, minWidth, minHeight } = limits(viewportWidth, viewportHeight);
    let { x, y, width, height } = start;
    if (edges.right) width = clamp(start.width + dx, minWidth, Math.min(maxWidth, viewportWidth - RECT_MARGIN - start.x));
    if (edges.bottom) height = clamp(start.height + dy, minHeight, Math.min(maxHeight, viewportHeight - RECT_MARGIN - start.y));
    if (edges.left) {
        const right = start.x + start.width;
        x = clamp(start.x + dx, Math.max(RECT_MARGIN, right - maxWidth), right - minWidth);
        width = right - x;
    }
    if (edges.top) {
        const bottom = start.y + start.height;
        y = clamp(start.y + dy, Math.max(RECT_MARGIN, bottom - maxHeight), bottom - minHeight);
        height = bottom - y;
    }
    return clampRect({ x, y, width, height }, viewportWidth, viewportHeight);
}

/** The remembered rectangle, or `null` when none is stored, it is not one, or storage is unavailable. */
export function parseRect(raw: string | null): IRect | null {
    if (raw === null) return null;
    try {
        const v: unknown = JSON.parse(raw);
        if (typeof v !== "object" || v === null) return null;
        const { x, y, width, height } = v as Record<string, unknown>;
        if ([x, y, width, height].every((n) => typeof n === "number" && Number.isFinite(n))) return { x, y, width, height } as IRect;
        return null;
    } catch {
        return null;
    }
}
