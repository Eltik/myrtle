import { encodeBuildableType, encodeHeightType, encodePassableMask } from "../util/enums";

export const NEIGHBOR_OFFSETS: Record<number, { row: number; col: number }> = {
    0: { row: 1, col: 0 },
    1: { row: 1, col: 1 },
    2: { row: 0, col: 1 },
    3: { row: -1, col: 1 },
    4: { row: -1, col: 0 },
    5: { row: -1, col: -1 },
    6: { row: 0, col: -1 },
    7: { row: 1, col: -1 },
};

export const STAIR_TILES: Record<number, string> = { 1: "tile_stairs" };

export interface ITileMapRef {
    getTileAt(p: { row: number; col: number }): Tile | null;
}

interface INeibOptions {
    allowDiagonalMove?: boolean;
    visitEveryCheckPoint?: boolean;
    motionMode?: number;
}

interface ITileInit {
    row: number;
    col: number;
    tileKey: string;
    heightType: string;
    buildableType: string;
    passableMask: string;
    map: ITileMapRef;
}

export class Tile {
    public heightType: number;
    public buildableType: number;
    public visited: boolean;
    public passableMaskOverride: number | null;
    public pointer: { tile: Tile; dist: number; direction: string } | null;
    public tileKey: string;
    public passFlag: number;

    public row: number;
    public col: number;

    private map: ITileMapRef;
    private passableMask_raw: number;

    constructor({ row, col, tileKey, heightType, buildableType, passableMask, map }: ITileInit) {
        this.map = map;
        this.row = row;
        this.col = col;
        this.tileKey = tileKey;
        this.heightType = encodeHeightType(heightType);
        this.buildableType = encodeBuildableType(buildableType);
        this.passableMask_raw = encodePassableMask(passableMask);
        this.passableMaskOverride = null;
        this.passFlag = 0;
        if (tileKey === "tile_passable_wall" || tileKey === "tile_passable_wall_forbidden") this.passFlag |= 1;
        this.visited = false;
        this.pointer = null;
    }

    public get passableMask(): number {
        return this.passableMaskOverride || this.passableMask_raw;
    }

    public getTile(dir: number): Tile | null {
        if (!this.map) return null;
        const d = NEIGHBOR_OFFSETS[dir];
        return this.map.getTileAt({ row: this.row + d.row, col: this.col + d.col });
    }

    public getDirection(tile: Tile): [string, { row: number; col: number }] {
        const dRow = tile.row - this.row;
        const dCol = tile.col - this.col;
        const entry = Object.entries(NEIGHBOR_OFFSETS).find(([, o]) => o.row === dRow && o.col === dCol);
        if (!entry) throw new Error("Invalid direction");
        return entry as [string, { row: number; col: number }];
    }

    public checkNeibPassFlagChanged(tile: Tile, { ignore_transition = false }: { ignore_transition?: boolean } = {}): boolean {
        const diff = this.passFlag ^ tile.passFlag;
        return diff > 0 && (ignore_transition || Object.entries(STAIR_TILES).some(([bit, key]) => !(diff & (Number(bit) > 0 ? 1 : 0)) || (this.tileKey !== key && tile.tileKey !== key)));
    }

    public getAvailableNeib({ allowDiagonalMove, visitEveryCheckPoint, motionMode = 0 }: INeibOptions): { dist: number; tile: Tile }[] {
        if (!allowDiagonalMove) {
            return Array(4)
                .fill(0)
                .map((_, n) => 2 * n) // orthogonal directions 0,2,4,6
                .map((dir) => ({ dist: 1, tile: this.getTile(dir) }))
                .filter((e): e is { dist: number; tile: Tile } => !!e.tile && (e.tile.passableMask & (1 << motionMode)) !== 0)
                .filter(({ tile }) => (visitEveryCheckPoint ? true : !(tile.tileKey === "tile_hole" && motionMode === 0)));
        }
        const neighbors = Array(8)
            .fill(0)
            .map((_, dir) => {
                const tile = this.getTile(dir);
                let penalty = 0;
                if (tile) {
                    if (tile.tileKey === "tile_yinyang_switch") penalty = -1;
                    if (tile.tileKey === "tile_hole") penalty = 1e6;
                }
                return { dist: (dir % 2 === 0 ? 1 : Math.SQRT2) + penalty, tile };
            });
        const allowed = Array(8).fill(true);
        Array(4)
            .fill(0)
            .forEach((_, q) => {
                const ortho = neighbors[2 * q];
                if (ortho.tile) {
                    const isHole = ortho.tile.tileKey === "tile_hole" && motionMode === 0;
                    const blocked = !(ortho.tile.passableMask & (1 << motionMode));
                    const flagChanged = this.checkNeibPassFlagChanged(ortho.tile, { ignore_transition: true });
                    if (blocked || flagChanged || (!visitEveryCheckPoint && isHole)) {
                        allowed[(2 * q - 1 + 8) % 8] = false;
                        allowed[(2 * q + 0 + 8) % 8] = false;
                        allowed[(2 * q + 1 + 8) % 8] = false;
                    }
                    if (flagChanged && (Object.values(STAIR_TILES).includes(this.tileKey) || Object.values(STAIR_TILES).includes(ortho.tile.tileKey))) {
                        allowed[(2 * q + 0 + 8) % 8] = true;
                    }
                } else {
                    allowed[(2 * q + 0 + 8) % 8] = false;
                }
            });
        return neighbors
            .filter((_, n) => allowed[n])
            .filter((e): e is { dist: number; tile: Tile } => !!e.tile && (e.tile.passableMask & (1 << motionMode)) !== 0)
            .filter(({ tile }) => !this.checkNeibPassFlagChanged(tile))
            .filter(({ tile }) => (visitEveryCheckPoint ? true : !(tile.tileKey === "tile_hole" && motionMode === 0)));
    }

    public pointTo(tile: Tile, dist: number): void {
        const [direction] = this.getDirection(tile);
        if (!this.pointer || dist < this.pointer.dist || (dist === this.pointer.dist && direction < this.pointer.direction)) {
            this.pointer = { tile, dist, direction };
        }
    }

    public connectPointers(): { row: number; col: number; reachOffset?: { x: number; y: number } | null; type?: string | number }[] {
        const path: { row: number; col: number }[] = [{ row: this.row, col: this.col }];
        let node: Tile = this;
        while (node.pointer !== null) {
            path.unshift({ row: node.pointer.tile.row, col: node.pointer.tile.col });
            node = node.pointer.tile;
        }
        return path;
    }
}
