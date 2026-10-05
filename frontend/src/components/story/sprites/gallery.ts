/**
 * The CHARACTER GALLERY's pure half: who a card says it is, what a search
 * matches it by, how the grid orders it, how a sheet groups its expressions,
 * where the crop sits on a body plate, how the download is laid out and how
 * an arrow key moves through the sheet. Nothing here touches the DOM, so
 * every rule is tested against hand-built entries (`gallery.test.ts`).
 *
 * The names come from the backend's census (`GET /story/sprites`), which
 * attributes every named line to the sprite LIT when it is spoken; see
 * `docs/story-reader.md`, "Character gallery".
 */

import { type IPreparedQuery, type IPreparedTarget, type IScoreTarget, prepareQuery, prepareTarget, scorePrepared } from "#/lib/search/fuzzy";
import type { StorySpriteEntry } from "#/types/generated/StorySpriteEntry";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";

export type SpriteKindFilter = "all" | "operator" | "npc";
export type SpriteSort = "appearances" | "name" | "firstSeen";

export const SPRITE_KIND_FILTERS: readonly SpriteKindFilter[] = ["all", "operator", "npc"];
export const SPRITE_SORTS: readonly SpriteSort[] = ["appearances", "name", "firstSeen"];

/**
 * The name a card leads with: the name the scripts speak this sprite under
 * most, else the operator's table name, else the folder. An operator's own
 * name is NOT preferred over the scripts', because a folder is one outfit or
 * one set and the scripts name the person in it (`avg_npc_043_1` is "Nine").
 */
export function primaryName(entry: Pick<StorySpriteEntry, "names" | "operatorName" | "base">): string {
    return entry.names[0]?.name ?? entry.operatorName ?? entry.base;
}

/** Every other name the card answers to: the remaining script names, then the operator's own when it differs. */
export function aliasesOf(entry: Pick<StorySpriteEntry, "names" | "operatorName" | "base">): string[] {
    const primary = primaryName(entry);
    const out: string[] = [];
    for (const n of entry.names) if (n.name !== primary && !out.includes(n.name)) out.push(n.name);
    if (entry.operatorName && entry.operatorName !== primary && !out.includes(entry.operatorName)) out.push(entry.operatorName);
    return out;
}

/**
 * What a card is searched by. The display names score on the name tiers, so
 * "jie" and "nine" hit their cards first; the ids sit in `extra`, which is
 * substring only, so "npc_043" and "char_002" still find a folder without a
 * subsequence pulling unrelated ids in.
 */
export function spriteSearchTarget(entry: StorySpriteEntry): IScoreTarget {
    return { name: primaryName(entry), aliases: aliasesOf(entry), extra: [entry.base, entry.charId ?? ""].join(" ") };
}

export interface ISpriteSearch {
    entries: readonly StorySpriteEntry[];
    prepared: readonly IPreparedTarget[];
    /** The primary name alone, so a card CALLED the query beats one that was once called it. */
    primary: readonly IPreparedTarget[];
}

/** Prepare every haystack ONCE per list, so a keystroke only normalises the query. */
export function prepareSpriteSearch(entries: readonly StorySpriteEntry[]): ISpriteSearch {
    return { entries, prepared: entries.map((e) => prepareTarget(spriteSearchTarget(e))), primary: entries.map((e) => prepareTarget({ name: primaryName(e) })) };
}

/**
 * The fuzzy score folded into TIERS. Inside a tier the reader's sort decides;
 * across tiers the better match always leads, which is what makes "jie" open
 * on Jie rather than on the 300 names that hold j, i and e in that order.
 * The cut points are the scorer's own tier constants (exact 1000, prefix 600,
 * word prefix 450, contains 300, subsequence 180, `extra` 90) less the
 * position penalty each tier can take.
 */
export function matchTier(score: number): number {
    if (score >= 1000) return 5;
    if (score >= 600) return 4;
    if (score >= 400) return 3;
    if (score >= 250) return 2;
    if (score >= 150) return 1;
    return 0;
}

function compareBy(sort: SpriteSort, collator: Intl.Collator): (a: StorySpriteEntry, b: StorySpriteEntry) => number {
    const byName = (a: StorySpriteEntry, b: StorySpriteEntry) => collator.compare(primaryName(a), primaryName(b)) || collator.compare(a.base, b.base);
    if (sort === "name") return byName;
    if (sort === "firstSeen") {
        // Undated sprites (mainline only, or never shown) sort after every dated
        // one, then by library order, then by name.
        const date = (e: StorySpriteEntry) => e.firstSeen ?? Number.POSITIVE_INFINITY;
        const order = (e: StorySpriteEntry) => e.firstOrder ?? Number.POSITIVE_INFINITY;
        return (a, b) => date(a) - date(b) || order(a) - order(b) || byName(a, b);
    }
    return (a, b) => b.storyCount - a.storyCount || b.lines - a.lines || byName(a, b);
}

/** Keep the kind the filter asks for, match the query, order by tier and then the sort. A blank query keeps every entry of the kind. */
export function filterSprites(search: ISpriteSearch, query: string, kind: SpriteKindFilter, sort: SpriteSort, collator: Intl.Collator): StorySpriteEntry[] {
    const q: IPreparedQuery | null = query.trim() === "" ? null : prepareQuery(query);
    const kept: { entry: StorySpriteEntry; tier: number }[] = [];
    search.entries.forEach((entry, at) => {
        if (kind !== "all" && entry.kind !== kind) return;
        if (q === null) {
            kept.push({ entry, tier: 0 });
            return;
        }
        const target = search.prepared[at];
        const score = target ? scorePrepared(q, target) : 0;
        if (score <= 0) return;
        // Twice the tier, plus one when the primary name alone reaches that
        // tier: "chun" puts the sprite called Chun before Jie's, whose
        // scripts call her Chun three times.
        const primary = search.primary[at];
        const own = primary ? matchTier(scorePrepared(q, primary)) : 0;
        const tier = matchTier(score);
        kept.push({ entry, tier: tier * 2 + (own === tier ? 1 : 0) });
    });
    const cmp = compareBy(sort, collator);
    kept.sort((a, b) => b.tier - a.tier || cmp(a.entry, b.entry));
    return kept.map((k) => k.entry);
}

/** The `$M` body a variant key addresses; an alias key or an unreadable one is body 1. */
export function bodyIndexOf(key: string): number {
    const m = /\$(\d+)/.exec(key);
    const n = m ? Number(m[1]) : 1;
    return Number.isFinite(n) && n >= 1 ? n : 1;
}

export interface IVariantGroup {
    body: number;
    variants: StorySpriteVariant[];
}

/** The sheet's sections: one per body `$M`, in body order, each in face order as the hub lists them. */
export function groupVariants(variants: readonly StorySpriteVariant[]): IVariantGroup[] {
    const groups = new Map<number, StorySpriteVariant[]>();
    for (const v of variants) {
        const body = bodyIndexOf(v.key);
        const list = groups.get(body);
        if (list) list.push(v);
        else groups.set(body, [v]);
    }
    return [...groups.entries()].sort((a, b) => a[0] - b[0]).map(([body, list]) => ({ body, variants: list }));
}

/** The variant a sheet opens on: the card's own expression, else the most used, else the first. */
export function initialVariant(variants: readonly StorySpriteVariant[], thumbKey: string | undefined): number {
    if (thumbKey !== undefined) {
        const at = variants.findIndex((v) => v.key === thumbKey);
        if (at >= 0) return at;
    }
    let best = -1;
    variants.forEach((v, i) => {
        if (v.uses > 0 && (best < 0 || v.uses > (variants[best]?.uses ?? 0))) best = i;
    });
    return best >= 0 ? best : 0;
}

/** A point on the body plate, as fractions of its width and height. */
export interface IPlatePoint {
    x: number;
    y: number;
}

/**
 * The face centre a sprite with no face box is read at: the head height the
 * thumbnail service measured as the median over the EN face-carrying hubs,
 * 0.5 across and about 0.3 down.
 */
const GUESSED_FACE: IPlatePoint = { x: 0.5, y: 0.3 };

/**
 * Where the face sits on the body plate. Read off `facePos` over `bodySize`
 * (the texture's own pixels, never the 1024 canvas plate). A whole-body or
 * legacy sprite carries no face box and falls back to {@link GUESSED_FACE}.
 */
export function faceCentre(sprite: Pick<StorySpriteVariant, "facePos" | "bodySize">): IPlatePoint & { measured: boolean } {
    const p = sprite.facePos;
    const s = sprite.bodySize;
    if (p && s && s.w > 0 && s.h > 0 && p.w > 0 && p.h > 0) {
        const x = (p.x + p.w / 2) / s.w;
        const y = (p.y + p.h / 2) / s.h;
        if (x >= 0 && x <= 1 && y >= 0 && y <= 1) return { x, y, measured: true };
    }
    return { ...GUESSED_FACE, measured: false };
}

/**
 * A head-and-shoulders window onto a square body plate. `zoom` is the plate's
 * side over the frame's width; `aspect` the frame's height over its width;
 * the face centre lands `anchorY` of the way down the frame.
 */
export interface ICropWindow {
    zoom: number;
    aspect: number;
    anchorY: number;
}

export interface ICrop {
    /** The plate's side as a percentage of the frame's WIDTH. */
    size: number;
    /** The plate's left edge, percent of the frame's width (zero or negative). */
    left: number;
    /** The plate's top edge, percent of the frame's HEIGHT (zero or negative). */
    top: number;
}

/** Place the plate so `face` sits at the window's anchor, centred across, clamped so the plate always covers the frame. */
export function cropFor(face: IPlatePoint, crop: ICropWindow): ICrop {
    const { zoom, aspect, anchorY } = crop;
    const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
    // In units of the frame's width.
    const left = clamp(0.5 - face.x * zoom, 1 - zoom, 0);
    const topW = clamp(anchorY * aspect - face.y * zoom, Math.min(0, aspect - zoom), 0);
    return { size: zoom * 100, left: left * 100, top: (topW / aspect) * 100 };
}

/** The most a crop zooms around a face it had to guess. */
export const GUESSED_ZOOM = 1.6;
/** The lowest a guessed face sits in its window. */
const GUESSED_ANCHOR_Y = 0.36;

/**
 * The window for one expression. A face the hub does not place is a GUESS, so
 * the zoom is held to {@link GUESSED_ZOOM}: a legacy whole sprite (Nine's
 * `avg_npc_043_1`) draws her head at 0.15 of the plate, and the 3.6x sheet
 * window around a guessed 0.3 framed her shoulder.
 */
export function cropForVariant(sprite: Pick<StorySpriteVariant, "facePos" | "bodySize">, crop: ICropWindow): ICrop {
    const face = faceCentre(sprite);
    if (face.measured) return cropFor(face, crop);
    return cropFor(GUESSED_FACE, { ...crop, zoom: Math.min(crop.zoom, GUESSED_ZOOM), anchorY: Math.min(crop.anchorY, GUESSED_ANCHOR_Y) });
}

/** The card's window: a 3:4 frame, the plate at 1.7x the frame's width, the face at 34% of its height. */
export const CARD_CROP = { zoom: 1.7, aspect: 4 / 3, anchorY: 0.34 } as const satisfies ICropWindow;
/** The sheet cell's window: square, tighter on the face, which is what an expression sheet compares. */
export const CELL_CROP = { zoom: 3.6, aspect: 1, anchorY: 0.45 } as const satisfies ICropWindow;

/**
 * A weighted line count for display, as a whole number: the half lines of a
 * two-lit split are SUMMED on the backend first and rounded only here.
 */
export function lineCount(n: number): number {
    return Math.round(n);
}

/** The largest canvas side every current engine draws (Chrome, Firefox, Safari agree on 16,384 px for a 2D canvas). */
export const CANVAS_MAX_SIDE = 16_384;
/** The largest canvas area Chrome allocates, ~268 MP. */
export const CANVAS_MAX_AREA = 268_435_456;

/**
 * The downloaded sheet's measures in px at full scale. Every one scales with
 * the layout's `scale`, so a caption reads at the same proportion on a
 * half-size sheet.
 */
export const SHEET_METRICS = {
    /** The caption band under each cell. */
    label: 76,
    /** The title block above the grid. */
    head: 170,
    gap: 32,
    titleFont: 76,
    folderFont: 34,
    keyFont: 44,
    /** The caption's inset from the cell's left edge. */
    keyInset: 4,
} as const;
/** The sheet is near square, ceil(sqrt(count)) columns, up to this many. */
const SHEET_MAX_COLS = 6;
/** The scale the halving fallback stops at. */
const SHEET_MIN_SCALE = 0.05;

export interface ISheetLayout {
    cols: number;
    rows: number;
    width: number;
    height: number;
    /** The cell's side in px: the plate at `scale`. */
    cell: number;
    label: number;
    head: number;
    gap: number;
    /** 1 at full size, 0.5 at half; lower only when the fallback below had to shrink it. */
    scale: number;
    /** Why the layout differs from the one asked for, or null. */
    note: "moreColumns" | "halfSize" | null;
}

/**
 * The downloaded sheet's geometry at the PLATE's own size. `plate` is the
 * largest body texture side in px (256 to 2,048 on EN, 1,024 for 8,134 of the
 * 12,107 expressions), `scale` 1 (full) or 0.5 (half); every
 * other measure scales with it ({@link SHEET_METRICS}). Columns are
 * ceil(sqrt(count)), at most {@link SHEET_MAX_COLS}.
 *
 * Past the canvas limits ({@link CANVAS_MAX_SIDE}, {@link CANVAS_MAX_AREA}) it
 * first takes MORE columns (a taller sheet is the usual overflow), then
 * halves the scale until it fits, and says which it did.
 */
export function sheetLayout(count: number, plate: number, scale: number): ISheetLayout {
    const n = Math.max(1, count);
    const fits = (l: ISheetLayout) => l.width <= CANVAS_MAX_SIDE && l.height <= CANVAS_MAX_SIDE && l.width * l.height <= CANVAS_MAX_AREA;
    const layoutAt = (cols: number, sc: number): ISheetLayout => {
        const cell = Math.round(plate * sc);
        const label = Math.round(SHEET_METRICS.label * sc);
        const head = Math.round(SHEET_METRICS.head * sc);
        const gap = Math.round(SHEET_METRICS.gap * sc);
        const rows = Math.ceil(n / cols);
        return { cols, rows, width: gap + cols * (cell + gap), height: head + gap + rows * (cell + label + gap), cell, label, head, gap, scale: sc, note: null };
    };
    const asked = layoutAt(Math.min(SHEET_MAX_COLS, Math.ceil(Math.sqrt(n))), scale);
    if (fits(asked)) return asked;
    for (let cols = asked.cols + 1; cols <= n; cols++) {
        const wider = layoutAt(cols, scale);
        if (wider.width > CANVAS_MAX_SIDE) break;
        if (fits(wider)) return { ...wider, note: "moreColumns" };
    }
    let sc = scale;
    while (sc > SHEET_MIN_SCALE) {
        sc /= 2;
        const smaller = layoutAt(asked.cols, sc);
        if (fits(smaller)) return { ...smaller, note: "halfSize" };
    }
    return { ...layoutAt(asked.cols, sc), note: "halfSize" };
}

export interface ICellRect {
    left: number;
    top: number;
    width: number;
    height: number;
}

/** The keys the expression grid moves on. */
export const GRID_KEYS: ReadonlySet<string> = new Set(["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp", "Home", "End"]);

/**
 * The cell an arrow key moves to, read off the LAID-OUT grid rather than a
 * column count: the sheet's groups break rows, so "down" is the nearest cell
 * in the next visual row, by centre distance across.
 */
export function stepCell(rects: readonly ICellRect[], from: number, key: string): number {
    const n = rects.length;
    if (n === 0) return from;
    if (key === "ArrowRight") return Math.min(n - 1, from + 1);
    if (key === "ArrowLeft") return Math.max(0, from - 1);
    if (key === "Home") return 0;
    if (key === "End") return n - 1;
    const here = rects[from];
    if (!here || (key !== "ArrowDown" && key !== "ArrowUp")) return from;
    const cx = here.left + here.width / 2;
    const down = key === "ArrowDown";
    let best = from;
    let bestRow = Number.POSITIVE_INFINITY;
    let bestDx = Number.POSITIVE_INFINITY;
    rects.forEach((r, i) => {
        const dy = down ? r.top - here.top : here.top - r.top;
        if (dy <= here.height / 2) return;
        const dx = Math.abs(r.left + r.width / 2 - cx);
        if (dy < bestRow - 1 || (Math.abs(dy - bestRow) <= 1 && dx < bestDx)) {
            bestRow = dy;
            bestDx = dx;
            best = i;
        }
    });
    return best;
}
