/**
 * Stage-viewer simulation core: polyline geometry (smooth token motion between
 * waypoints), tile-grid BFS re-routing around placed blockers, attack-range
 * projection, and heatmap accumulation. Ported to TS and adapted to the backend
 * board model (baked tile-coord polylines + `passable` cells).
 */

import type { IPoint } from "#/lib/api/level";
import type { IBoardCell, IBoardEnemy, IBoardModel } from "#/lib/api/level-adapter";
import { tileKey } from "#/lib/api/level-adapter";

export type Pixel = { x: number; y: number; tp?: boolean };

export const key = tileKey;

/** Tile-coord polyline → pixel polyline (tile centers). */
export function toPixelPolyline(points: IPoint[], T: number): Pixel[] {
    return points.map((p) => ({ x: p.x * T + T / 2, y: p.y * T + T / 2 }));
}

export function polylineLength(pts: Pixel[]): number {
    let L = 0;
    for (let i = 1; i < pts.length; i++) {
        if (!pts[i]?.tp) L += Math.hypot((pts[i]?.x ?? 0) - (pts[i - 1]?.x ?? 0), (pts[i]?.y ?? 0) - (pts[i - 1]?.y ?? 0));
    }
    return L;
}

/** Point at fraction f∈[0,1] along the polyline (smooth between waypoints). */
export function pointAtFraction(pts: Pixel[], f: number): Pixel {
    if (pts.length === 0) return { x: 0, y: 0 };
    if (pts.length === 1) return { ...(pts[0] as Pixel) };
    const total = polylineLength(pts) || 1;
    let target = Math.max(0, Math.min(1, f)) * total;
    for (let i = 1; i < pts.length; i++) {
        const a = pts[i - 1] as Pixel;
        const b = pts[i] as Pixel;
        const d = Math.hypot(b.x - a.x, b.y - a.y);
        if (target <= d || i === pts.length - 1) {
            const t = d === 0 ? 0 : target / d;
            return { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
        }
        target -= d;
    }
    return { ...(pts[pts.length - 1] as Pixel) };
}

/** Even cumulative timestamps at each polyline vertex, scaled to `travel` secs. */
export function vertexTimings(pts: Pixel[], travel: number): number[] {
    const total = polylineLength(pts) || 1;
    let acc = 0;
    return pts.map((p, i) => {
        if (i > 0) acc += Math.hypot(p.x - (pts[i - 1] as Pixel).x, p.y - (pts[i - 1] as Pixel).y);
        return Math.round((acc / total) * travel);
    });
}

/** Unique tile coords a polyline covers (sampled finely; handles diagonals). */
export function routeTiles(points: IPoint[]): IPoint[] {
    if (points.length === 0) return [];
    const out: IPoint[] = [];
    const seen = new Set<string>();
    const push = (x: number, y: number) => {
        const k = key(x, y);
        if (seen.has(k)) return;
        seen.add(k);
        out.push({ x, y });
    };
    for (let i = 0; i < points.length - 1; i++) {
        const a = points[i] as IPoint;
        const b = points[i + 1] as IPoint;
        const steps = Math.max(1, Math.round(Math.hypot(b.x - a.x, b.y - a.y)) * 2);
        for (let s = 0; s <= steps; s++) {
            const t = s / steps;
            push(Math.round(a.x + (b.x - a.x) * t), Math.round(a.y + (b.y - a.y) * t));
        }
    }
    return out;
}

// ── Tile-grid BFS (for Squad Formation re-routing around blockers) ──────────

const DIRS: Array<[number, number]> = [
    [1, 0],
    [-1, 0],
    [0, 1],
    [0, -1],
];

/** Shortest passable path a→b avoiding `blocked` tile keys. Endpoints inclusive. */
export function bfs(grid: Record<string, IBoardCell>, a: IPoint, b: IPoint, blocked: Set<string>): IPoint[] | null {
    if (a.x === b.x && a.y === b.y) return [{ x: a.x, y: a.y }];
    const startK = key(a.x, a.y);
    const goalK = key(b.x, b.y);
    const q: IPoint[] = [a];
    const prev: Record<string, string | null> = { [startK]: null };
    while (q.length) {
        const cur = q.shift() as IPoint;
        const ck = key(cur.x, cur.y);
        if (ck === goalK) break;
        for (const [dx, dy] of DIRS) {
            const nx = cur.x + dx;
            const ny = cur.y + dy;
            const nk = key(nx, ny);
            if (nk in prev) continue;
            const t = grid[nk];
            if (!t || !t.passable) continue;
            if (blocked.has(nk) && nk !== goalK) continue;
            prev[nk] = ck;
            q.push({ x: nx, y: ny });
        }
    }
    if (!(goalK in prev)) return null;
    const path: IPoint[] = [];
    let k: string | null = goalK;
    while (k) {
        const [x, y] = k.split(",").map(Number);
        path.unshift({ x: x as number, y: y as number });
        k = prev[k] ?? null;
    }
    return path;
}

/**
 * Snap a WALK route's polyline to the road grid by BFS-ing tile-by-tile between
 * its checkpoints over passable tiles (avoiding any `blocked` tiles). This makes
 * the drawn line hug the road with orthogonal turns instead of cutting straight
 * across tiles between sparse checkpoints — and naturally re-routes around placed
 * blockers. Fliers (FLY) ignore the grid and keep their straight checkpoints.
 * Falls back to the raw segment when a stretch is unreachable.
 */
export function rerouteAround(grid: Record<string, IBoardCell>, points: IPoint[], motion: string, blocked: Set<string>): IPoint[] {
    if (motion === "FLY" || points.length < 2) return points;
    const out: IPoint[] = [];
    for (let i = 0; i < points.length - 1; i++) {
        const seg = bfs(grid, points[i] as IPoint, points[i + 1] as IPoint, blocked);
        const piece = seg ?? [points[i] as IPoint, points[i + 1] as IPoint];
        piece.forEach((p, j) => {
            if (i === 0 && j === 0) out.push(p);
            else if (j > 0) out.push(p);
        });
    }
    return collinearSimplify(out);
}

/** Drop interior points that lie on a straight run, keeping only the turns. */
function collinearSimplify(tiles: IPoint[]): IPoint[] {
    if (tiles.length <= 2) return tiles;
    const out: IPoint[] = [tiles[0] as IPoint];
    for (let i = 1; i < tiles.length - 1; i++) {
        const a = out[out.length - 1] as IPoint;
        const p = tiles[i] as IPoint;
        const b = tiles[i + 1] as IPoint;
        // keep p only if it's a turn (not collinear with a→b)
        if ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x) !== 0) out.push(p);
    }
    out.push(tiles[tiles.length - 1] as IPoint);
    return out;
}

// ── Attack-range projection (Squad Formation) ───────────────────────────────

export type Facing = "up" | "down" | "left" | "right";

/** Rotate a facing-right offset by a facing direction. */
export function rotateOffset(x: number, y: number, facing: Facing): [number, number] {
    switch (facing) {
        case "up":
            return [y, -x];
        case "down":
            return [-y, x];
        case "left":
            return [-x, -y];
        default:
            return [x, y];
    }
}

export interface RangeHit extends IPoint {
    self: boolean;
}

/** Absolute tiles covered by a range grid placed at `at`, facing a direction. */
export function rangeTiles(grids: Array<{ col: number; row: number }>, at: IPoint, facing: Facing): RangeHit[] {
    return grids.map((g) => {
        const [dx, dy] = rotateOffset(g.col, g.row, facing);
        return { x: at.x + dx, y: at.y + dy, self: g.col === 0 && g.row === 0 };
    });
}

// ── Heatmap ─────────────────────────────────────────────────────────────────

export const HEAT_METRICS = ["Count", "HP", "Atk", "Def", "Magic res", "Atk time", "Move speed", "Mass level", "HP recovery"] as const;
export type HeatMetric = (typeof HEAT_METRICS)[number];

function metricValue(e: IBoardEnemy, metric: HeatMetric): number {
    const s = e.stats;
    if (!s) return metric === "Count" ? 1 : 0;
    switch (metric) {
        case "Count":
            return 1;
        case "HP":
            return s.hp;
        case "Atk":
            return s.atk;
        case "Def":
            return s.def;
        case "Magic res":
            return s.res;
        case "Atk time":
            return s.atkTime;
        case "Move speed":
            return s.moveSpeed;
        case "Mass level":
            return s.mass === "High" ? 3 : s.mass === "Normal" ? 2 : 1;
        case "HP recovery":
            return s.hpRecover;
        default:
            return 1;
    }
}

export interface HeatCell {
    raw: number;
    norm: number;
    pct: number;
}

export interface Heatmap {
    tiles: Record<string, HeatCell>;
    max: number;
}

/** Accumulate a metric across every schedule entry's route tiles. */
export function computeHeatmap(model: IBoardModel, routeTileLists: IPoint[][], metric: HeatMetric): Heatmap {
    const acc: Record<string, number> = {};
    for (const s of model.schedule) {
        const tiles = routeTileLists[s.route];
        const e = model.enemies[s.enemyId];
        if (!tiles || !e) continue;
        const val = metricValue(e, metric) * s.count;
        const seen = new Set<string>();
        for (const t of tiles) {
            const k = key(t.x, t.y);
            if (seen.has(k)) continue;
            seen.add(k);
            acc[k] = (acc[k] ?? 0) + val;
        }
    }
    const max = Math.max(1, ...Object.values(acc));
    const tiles: Record<string, HeatCell> = {};
    for (const [k, v] of Object.entries(acc)) tiles[k] = { raw: v, norm: v / max, pct: Math.round((v / max) * 100) };
    return { tiles, max };
}

// ── Tone / label maps ───────────────────────────────────────────────────────

export const LEVEL_TONE: Record<string, string> = {
    NORMAL: "var(--enemy-normal)",
    ELITE: "var(--enemy-elite)",
    BOSS: "var(--enemy-boss)",
};
export const DMG_TONE: Record<string, string> = {
    PHYSIC: "var(--dmg-physic)",
    MAGIC: "var(--dmg-magic)",
    HEAL: "var(--dmg-heal)",
    NO_DAMAGE: "var(--dmg-none)",
};
export const DMG_LABEL: Record<string, string> = {
    PHYSIC: "Physical",
    MAGIC: "Arts",
    HEAL: "Heal",
    NO_DAMAGE: "No Damage",
};

export const orderLabel = (o: [number, number]): string => (o[0] === o[1] ? `${o[0]}` : `${o[0]} ~ ${o[1]}`);

export function titleCase(s: string): string {
    if (!s) return s;
    return s[0] + s.slice(1).toLowerCase();
}

export function compact(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(n >= 10000 ? 0 : 1)}k`;
    return `${Math.round(n)}`;
}
