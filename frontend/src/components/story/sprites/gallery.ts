/**
 * The CHARACTER GALLERY's pure half: who a card says it is, what a search
 * matches it by, how the grid orders it, how a sheet groups its expressions,
 * and where the crop sits on a body plate. Nothing here touches the DOM, so
 * every rule is tested against hand-built entries (`gallery.test.ts`).
 *
 * The names come from the backend's census (`GET /story/sprites`), which
 * attributes every named line to the sprite LIT when it is spoken; see
 * `docs/story-reader.md`, "Character gallery".
 */

import { compactForSearch, type IPreparedQuery, type IPreparedTarget, type IScoreTarget, prepareQuery, prepareTarget, scorePrepared } from "#/lib/search/fuzzy";
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

/** The `#N` face a variant key addresses, or null for an alias key. */
export function faceIndexOf(key: string): number | null {
    const m = /#(\d+)/.exec(key);
    return m ? Number(m[1]) : null;
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

/**
 * Where the face sits on the body plate, as fractions of the plate. Read off
 * `facePos` over `bodySize` (the texture's own pixels, never the 1024 canvas
 * plate). A whole-body or legacy sprite carries no face box, and there the
 * crop falls back to the head height the thumbnail service measured as the
 * median over the EN face-carrying hubs: 0.5 across, about 0.3 down.
 */
export function faceCentre(sprite: Pick<StorySpriteVariant, "facePos" | "bodySize">): { x: number; y: number; measured: boolean } {
    const p = sprite.facePos;
    const s = sprite.bodySize;
    if (p && s && s.w > 0 && s.h > 0 && p.w > 0 && p.h > 0) {
        const x = (p.x + p.w / 2) / s.w;
        const y = (p.y + p.h / 2) / s.h;
        if (x >= 0 && x <= 1 && y >= 0 && y <= 1) return { x, y, measured: true };
    }
    return { x: 0.5, y: 0.3, measured: false };
}

export interface ICrop {
    /** The plate's side as a percentage of the frame's WIDTH. */
    size: number;
    /** The plate's left edge, percent of the frame's width (zero or negative). */
    left: number;
    /** The plate's top edge, percent of the frame's HEIGHT (zero or negative). */
    top: number;
}

/**
 * A head-and-shoulders window onto a square body plate.
 *
 * `zoom` is the plate's side over the frame's width; `aspect` the frame's
 * height over its width; the face centre lands `anchorY` of the way down the
 * frame and centred across it, clamped so the plate always covers the frame.
 */
export function cropFor(face: { x: number; y: number }, opts: { zoom: number; aspect: number; anchorY: number }): ICrop {
    const { zoom, aspect, anchorY } = opts;
    const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
    // In units of the frame's width.
    const left = clamp(0.5 - face.x * zoom, 1 - zoom, 0);
    const topW = clamp(anchorY * aspect - face.y * zoom, Math.min(0, aspect - zoom), 0);
    return { size: zoom * 100, left: left * 100, top: (topW / aspect) * 100 };
}

/**
 * The window for one expression. A face the hub does not place is a GUESS, so
 * the zoom is held to {@link GUESSED_ZOOM}: a legacy whole sprite (Nine's
 * `avg_npc_043_1`) draws her head at 0.15 of the plate, and the 3.6x sheet
 * window around a guessed 0.3 framed her shoulder.
 */
export function cropForVariant(sprite: Pick<StorySpriteVariant, "facePos" | "bodySize">, crop: { zoom: number; aspect: number; anchorY: number }): ICrop {
    const face = faceCentre(sprite);
    if (face.measured) return cropFor(face, crop);
    return cropFor({ x: 0.5, y: 0.3 }, { ...crop, zoom: Math.min(crop.zoom, GUESSED_ZOOM), anchorY: Math.min(crop.anchorY, 0.36) });
}

/** The most a crop zooms around a face it had to guess. */
export const GUESSED_ZOOM = 1.6;

/** The card's window: a 3:4 frame, the plate at 1.7x the frame's width, the face at 34% of its height. */
export const CARD_CROP = { zoom: 1.7, aspect: 4 / 3, anchorY: 0.34 } as const;
/** The sheet cell's window: square, tighter on the face, which is what an expression sheet compares. */
export const CELL_CROP = { zoom: 3.6, aspect: 1, anchorY: 0.45 } as const;

/** A weighted line count for display: whole numbers stay whole, a split keeps one decimal. */
export function lineCount(n: number): number {
    return Math.round(n * 10) / 10;
}

/** Normalise a folder name for a URL comparison (`AVG_NPC_043_1` is `avg_npc_043_1`). */
export function sameBase(a: string, b: string): boolean {
    return compactForSearch(a) === compactForSearch(b);
}

export interface ISheetLayout {
    cols: number;
    rows: number;
    width: number;
    height: number;
    cell: number;
    label: number;
    head: number;
    gap: number;
}

/**
 * The downloaded sheet's geometry: `count` square cells of `cell` px with a
 * caption strip of `label` px under each, `cols` across at most, under a
 * `head` px title band.
 */
export function sheetLayout(count: number, opts: { cell: number; label: number; head: number; gap: number; maxCols: number }): ISheetLayout {
    const cols = Math.max(1, Math.min(opts.maxCols, count));
    const rows = Math.max(1, Math.ceil(count / cols));
    const width = opts.gap + cols * (opts.cell + opts.gap);
    const height = opts.head + opts.gap + rows * (opts.cell + opts.label + opts.gap);
    return { cols, rows, width, height, cell: opts.cell, label: opts.label, head: opts.head, gap: opts.gap };
}
