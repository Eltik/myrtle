/**
 * Hex geometry for the game's Archives network.
 *
 * The tables use Unity coordinates (y up) and flat-topped hexes: territory
 * hexes have circumradius 50 (neighbours 86.6 apart), chart cells 100. SVG
 * runs y down, so every point is flipped once, here, with {@link toSvg}.
 *
 * A connection's `direction` names a neighbour: 1 is straight up and the rest
 * go clockwise in 60 degree steps, so direction d points at `150 - 60d`
 * degrees in Unity space (read off Rhodes Island's seven hexes: d1 -> (0, 86.6),
 * d2 -> (75, 43.3), d0 -> (-75, 43.3)).
 */

export const TERRITORY_R = 50;
export const CELL_R = 100;
const SQRT3 = Math.sqrt(3);

export interface IPoint {
    x: number;
    y: number;
}

export function toSvg(p: IPoint): IPoint {
    return { x: p.x, y: -p.y };
}

/** Unit vector of a connection direction, in SVG space (y down). */
export function directionVector(direction: number): IPoint {
    const deg = 150 - 60 * direction;
    const rad = (deg * Math.PI) / 180;
    return { x: Math.cos(rad), y: -Math.sin(rad) };
}

/** The six corners of a flat-topped hex, as an SVG `points` string. */
export function hexPoints(c: IPoint, r: number): string {
    const out: string[] = [];
    for (let i = 0; i < 6; i++) {
        const a = (Math.PI / 3) * i;
        out.push(`${(c.x + r * Math.cos(a)).toFixed(2)},${(c.y + r * Math.sin(a)).toFixed(2)}`);
    }
    return out.join(" ");
}

/**
 * The edges of a cluster's outline: every hex side with no hex of the same
 * cluster across it. Side i runs from corner i to corner i+1 and faces the
 * neighbour at `30 + 60i` degrees, `sqrt(3) r` away.
 */
export function outlineSegments(centers: readonly IPoint[], r: number): [IPoint, IPoint][] {
    const near = (p: IPoint) => centers.some((c) => Math.abs(c.x - p.x) < r * 0.2 && Math.abs(c.y - p.y) < r * 0.2);
    const segs: [IPoint, IPoint][] = [];
    for (const c of centers) {
        for (let i = 0; i < 6; i++) {
            const mid = (Math.PI / 6) * (2 * i + 1);
            const across = { x: c.x + SQRT3 * r * Math.cos(mid), y: c.y + SQRT3 * r * Math.sin(mid) };
            if (near(across)) continue;
            const a0 = (Math.PI / 3) * i;
            const a1 = (Math.PI / 3) * (i + 1);
            segs.push([
                { x: c.x + r * Math.cos(a0), y: c.y + r * Math.sin(a0) },
                { x: c.x + r * Math.cos(a1), y: c.y + r * Math.sin(a1) },
            ]);
        }
    }
    return segs;
}

export function centroid(points: readonly IPoint[]): IPoint {
    if (points.length === 0) return { x: 0, y: 0 };
    const sum = points.reduce((s, p) => ({ x: s.x + p.x, y: s.y + p.y }), { x: 0, y: 0 });
    return { x: sum.x / points.length, y: sum.y / points.length };
}

export interface IBox {
    x: number;
    y: number;
    w: number;
    h: number;
}

/** The box around `points`, grown by `pad` on every side. */
export function bounds(points: readonly IPoint[], pad: number): IBox {
    if (points.length === 0) return { x: -pad, y: -pad, w: pad * 2, h: pad * 2 };
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const p of points) {
        minX = Math.min(minX, p.x);
        minY = Math.min(minY, p.y);
        maxX = Math.max(maxX, p.x);
        maxY = Math.max(maxY, p.y);
    }
    return { x: minX - pad, y: minY - pad, w: maxX - minX + pad * 2, h: maxY - minY + pad * 2 };
}

/** `box` scaled by `factor` around `anchor` (both in the box's own space). */
export function zoomBox(box: IBox, factor: number, anchor: IPoint): IBox {
    const w = box.w * factor;
    const h = box.h * factor;
    return { x: anchor.x - (anchor.x - box.x) * factor, y: anchor.y - (anchor.y - box.y) * factor, w, h };
}

/** `#rrggbb` from the tables' bare hex, falling back when the table leaves it blank. */
export function hexColor(raw: string | null | undefined, fallback = "#8a8f98"): string {
    const v = (raw ?? "").trim().replace(/^#/, "");
    return /^[0-9a-fA-F]{6}$/.test(v) ? `#${v}` : fallback;
}
