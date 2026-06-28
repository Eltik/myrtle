import { STAIR_TILES, Tile } from "./tile/tile-model";
import type { ICheckpoint, IMapData, IPosition, IRouteDef, ITokenInst } from "./types";
import { encodeMotionMode, MOVE_TYPE } from "./util/enums";

export interface IWaypoint {
    row: number;
    col: number;
    type?: string | number;
    time?: number;
    reachOffset?: { x: number; y: number } | null;
}

function normalizeCheckpoint(c: ICheckpoint): IWaypoint {
    return { row: c.position.row, col: c.position.col, type: c.type, time: c.time, reachOffset: c.reachOffset };
}

const INHERIT_POS_TYPES: (string | number)[] = [MOVE_TYPE.WAIT_FOR_SECONDS, MOVE_TYPE.WAIT_FOR_PLAY_TIME, MOVE_TYPE.WAIT_CURRENT_FRAGMENT_TIME, MOVE_TYPE.WAIT_CURRENT_WAVE_TIME, MOVE_TYPE.DISAPPEAR, MOVE_TYPE.ALERT, 1, 3, 5, 7];
const ENDPOINT_CARRY_TYPES: (string | number)[] = [MOVE_TYPE.MOVE, MOVE_TYPE.WAIT_FOR_SECONDS, MOVE_TYPE.WAIT_CURRENT_FRAGMENT_TIME, MOVE_TYPE.DISAPPEAR, 0, 1, 3, 5];
const PORTAL_TYPES: (string | number)[] = [MOVE_TYPE.DISAPPEAR, MOVE_TYPE.APPEAR_AT_POS, 5, 6];

function flattenRouteWaypoints(routes: IRouteDef[] | undefined): IPosition[][] {
    const polylines = (routes || [])
        .filter(Boolean)
        .filter((r) => r.motionMode !== "E_NUM")
        .map(({ startPosition, endPosition, checkpoints }) => {
            const pts: IPosition[] = [startPosition];
            if (checkpoints) pts.push(...checkpoints.filter((c) => c.type === MOVE_TYPE.MOVE || c.type === 0).map(normalizeCheckpoint));
            pts.push(endPosition);
            return pts;
        });
    const seen: Record<string, boolean> = {};
    for (const pts of polylines) seen[pts.map(({ row, col }) => `(${row},${col})`).join("-")] = true;
    return Object.keys(seen).map((k) =>
        k.split("-").map((p) => {
            const [row, col] = p.replace(/[()]/g, "").split(",").map(Number);
            return { row, col };
        }),
    );
}

interface IFindPathArgs {
    startPosition: IPosition;
    checkpoints?: ICheckpoint[] | null;
    endPosition: IPosition | null;
    motionMode: string | number;
    allowDiagonalMove?: boolean;
    visitEveryCheckPoint?: boolean;
}

interface ISegmentArgs {
    motionMode?: number;
    allowDiagonalMove?: boolean;
    visitEveryCheckPoint?: boolean;
    point_data?: IWaypoint;
}

export class GameMap {
    public initialized: boolean;
    public tiles_ref: IMapData["tiles"];
    public width: number;
    public height: number;
    public tiles: Tile[][];
    public start_points: { row: number; col: number; tileKey: string }[];
    public end_points: { row: number; col: number; tileKey: string }[];
    public check_paths: IFindPathArgs[];
    public crates: IPosition[];

    constructor({ tiles, width, height }: { map?: number[][]; tiles: IMapData["tiles"]; width: number; height: number }, { routes, predefines }: { routes?: IRouteDef[]; predefines?: { tokenInsts?: ITokenInst[] } } = {}) {
        this.initialized = false;
        this.tiles_ref = tiles.slice();
        const h = height;
        const w = width;
        this.width = w;
        this.height = h;
        this.tiles = Array(h)
            .fill(0)
            .map((_, row) =>
                Array(w)
                    .fill(0)
                    .map((__, col) => new Tile({ ...this.tiles_ref[row * w + col], row, col, map: this })),
            );
        this.start_points = this.tiles_ref.map(({ tileKey }, i) => ({ row: Math.floor(i / w), col: i % w, tileKey })).filter(({ tileKey }) => tileKey === "tile_start");
        this.end_points = this.tiles_ref.map(({ tileKey }, i) => ({ row: Math.floor(i / w), col: i % w, tileKey })).filter(({ tileKey }) => tileKey === "tile_end");
        this.check_paths = flattenRouteWaypoints(routes)
            .filter((pts) => pts.length >= 2)
            .map((pts) => ({
                startPosition: pts[0],
                checkpoints: pts.slice(1, -1).map((p) => ({ type: MOVE_TYPE.MOVE, position: { row: p.row, col: p.col } })),
                endPosition: pts[pts.length - 1],
                motionMode: 0,
            }));
        this.crates = [];
        if (predefines?.tokenInsts) {
            for (const { position, hidden, inst } of predefines.tokenInsts) {
                if (!hidden && inst && inst.characterKey && inst.characterKey.startsWith("trap")) {
                    this.setTilePassableMaskOverride({ ...position, value: 2 });
                    this.crates.push(position);
                }
            }
        }
        this.initialized = true;
    }

    public getTileAt({ row, col }: { row: number; col: number }): Tile | null {
        if (row < 0 || col < 0 || row >= this.height || col >= this.width) return null;
        return this.tiles[row][col];
    }

    public setTilePassableMaskOverride({ row, col, value = 2 }: { row: number; col: number; value?: number }): void {
        this.tiles[row][col].passableMaskOverride = 3 & value;
    }

    public forEachTile(fn: (e: { tile: Tile; row: number; col: number }) => void): void {
        for (let row = 0; row < this.height; row++) for (let col = 0; col < this.width; col++) fn({ tile: this.tiles[row][col], row, col });
    }

    public resetAllTile(): void {
        this.forEachTile(({ tile }) => {
            tile.visited = false;
            tile.pointer = null;
        });
    }

    public findPathBetween(from: IWaypoint, to: IWaypoint, { motionMode = 0, allowDiagonalMove = false, visitEveryCheckPoint, point_data }: ISegmentArgs): IWaypoint[] | null {
        if (from.row < 0 || from.col < 0 || from.row >= this.height || from.col >= this.width || to.row < 0 || to.col < 0 || to.row >= this.height || to.col >= this.width) return null;
        let result: IWaypoint[] | null = null;
        let queue: { dist: number; tile: Tile }[] = [{ dist: 0, tile: this.getTileAt(from) as Tile }];
        while (queue.length > 0) {
            const { dist, tile } = queue.shift() as { dist: number; tile: Tile };
            tile.visited = true;
            if (tile.row === to.row && tile.col === to.col) {
                result = tile.connectPointers();
                break;
            }
            tile.getAvailableNeib({ motionMode, allowDiagonalMove, visitEveryCheckPoint })
                .filter(({ tile: t }) => !t.visited)
                .forEach(({ dist: step, tile: t }) => {
                    const nd = dist + step;
                    t.pointTo(tile, nd);
                    if (queue.length === 0 || queue[queue.length - 1].dist < nd) {
                        queue.push({ dist: nd, tile: t });
                        return;
                    }
                    let idx = queue.length;
                    while (idx > 0 && queue[idx - 1].dist > nd) idx--;
                    queue.splice(idx, 0, { dist: nd, tile: t });
                    queue = queue.filter((e, n) => e.tile !== t || n === idx);
                });
        }
        this.resetAllTile();
        if (!result) throw new Error("Invalid path");
        result[0].reachOffset = from.reachOffset;
        result[result.length - 1].reachOffset = to.reachOffset;
        if (to.type !== undefined && ENDPOINT_CARRY_TYPES.includes(to.type)) result[result.length - 1] = { ...result[result.length - 1], ...point_data };
        return result;
    }

    public findPath({ startPosition, checkpoints, endPosition, motionMode, allowDiagonalMove, visitEveryCheckPoint }: IFindPathArgs): IWaypoint[] {
        let last: IWaypoint | null = null;
        const waypoints: IWaypoint[] = [startPosition, ...(checkpoints || []).map(normalizeCheckpoint), endPosition]
            .filter((w): w is IWaypoint => !!w)
            .map((wp) => {
                if (wp.type !== undefined && INHERIT_POS_TYPES.includes(wp.type) && last) return { ...wp, row: last.row, col: last.col, reachOffset: last.reachOffset };
                last = wp;
                return wp;
            });
        const mm = encodeMotionMode(motionMode);
        if (waypoints.some((wp) => !this.checkTilePassable(this.getTileAt(wp), { motionMode: mm, visitEveryCheckPoint }))) return waypoints;
        const out: IWaypoint[] = [];
        for (let i = 0; i < waypoints.length - 1; i++) {
            if (waypoints[i].type !== undefined && PORTAL_TYPES.includes(waypoints[i].type as string | number)) out.push({ ...waypoints[i] });
            if (waypoints[i + 1].type !== undefined && PORTAL_TYPES.includes(waypoints[i + 1].type as string | number)) continue;
            const segment = this.findPathBetween(waypoints[i], waypoints[i + 1], { motionMode: mm, allowDiagonalMove, visitEveryCheckPoint, point_data: waypoints[i + 1] });
            if (!segment) continue;
            this.splitAtTransitionKey(segment).forEach((part, p) => {
                const merged = this.merge(part, { motionMode: mm, visitEveryCheckPoint });
                if (i !== 0 && p === 0) merged.unshift();
                out.push(...merged);
            });
        }
        return out;
    }

    public splitAtTransitionKey(pts: IWaypoint[]): IWaypoint[][] {
        if (pts.length <= 2) return [pts];
        const groups: IWaypoint[][] = [];
        let cur: IWaypoint[] = [];
        for (const p of pts) {
            const tile = this.tiles[p.row][p.col];
            if (Object.values(STAIR_TILES).includes(tile.tileKey)) {
                groups.push(cur);
                groups.push([p]);
                cur = [];
            } else cur.push(p);
        }
        if (cur && cur.length > 0) groups.push(cur);
        return groups;
    }

    public checkTilePassable(tile: Tile | null, { motionMode, visitEveryCheckPoint }: { motionMode: number; visitEveryCheckPoint?: boolean }): boolean {
        if (!tile) return false;
        const blockedKeys = [!visitEveryCheckPoint && "tile_hole", "tile_empty", "tile_fence_bound"].filter(Boolean) as string[];
        return (tile.passableMask & (1 << motionMode)) !== 0 && (motionMode !== 0 || !blockedKeys.includes(tile.tileKey));
    }

    public checkArea({ from, to, motionMode, visitEveryCheckPoint }: { from: IWaypoint; to: IWaypoint; motionMode: number; visitEveryCheckPoint?: boolean }): boolean {
        const cols = Math.abs(from.col - to.col) + 1;
        const rows = Math.abs(from.row - to.row) + 1;
        const c0 = Math.min(from.col, to.col);
        const r0 = Math.min(from.row, to.row);
        const flag = this.tiles[from.row][from.col].passFlag;
        return Array(rows)
            .fill(0)
            .every((_, r) =>
                Array(cols)
                    .fill(0)
                    .every((__, c) => {
                        const tile = this.tiles[r + r0][c + c0];
                        return this.checkTilePassable(tile, { motionMode, visitEveryCheckPoint }) && tile.passFlag === flag;
                    }),
            );
    }

    public merge(pts: IWaypoint[], { motionMode, visitEveryCheckPoint }: { motionMode: number; visitEveryCheckPoint?: boolean }): IWaypoint[] {
        if (pts.length <= 2) return pts;
        const out: IWaypoint[] = [];
        let anchor = pts[0];
        let candidate: IWaypoint | null = null;
        for (const p of pts) {
            if (out.length !== 0) {
                if (this.checkArea({ from: anchor, to: p, motionMode, visitEveryCheckPoint })) {
                    candidate = p;
                } else {
                    if (candidate) out.push(candidate);
                    anchor = candidate as IWaypoint;
                    if (this.checkArea({ from: anchor, to: p, motionMode, visitEveryCheckPoint })) candidate = p;
                }
            } else out.push(p);
        }
        out.push(pts[pts.length - 1]);
        return out;
    }
}
