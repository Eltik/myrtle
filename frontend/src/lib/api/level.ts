import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IEnemyLevel } from "./enemies";

/** A single board tile, already normalized to a rendering `kind`. */
export interface ITileCell {
    kind: string;
    tileKey: string;
    heightType: string;
    buildable: string;
    passable: boolean;
}

export interface IPoint {
    x: number;
    y: number;
}

export interface IRoute {
    /** `WALK`, `FLY`, or `OTHER`. */
    motion: string;
    /** Polyline in screen space (y = 0 is the top row): spawn → checkpoints → goal. */
    points: IPoint[];
}

export interface IWaveMarker {
    at: number;
    label: string;
}

export interface ISpawn {
    route: number;
    enemyId: string;
    t0: number;
    speed: number;
    level: IEnemyLevel;
}

export interface IRosterEntry {
    enemyId: string;
    name: string;
    level: IEnemyLevel;
    damageType: string[];
    attackType: string | null;
    motion: string;
    count: number;
}

export interface IStageMap {
    stageId: string;
    levelId: string;
    code: string;
    name: string | null;
    width: number;
    height: number;
    /** Row-major, top row first: `tiles[y][x]`. */
    tiles: ITileCell[][];
    routes: IRoute[];
    waves: IWaveMarker[];
    spawns: ISpawn[];
    roster: IRosterEntry[];
    duration: number;
}

export const getLevelFn = createServerFn({ method: "GET" })
    .inputValidator((data: { stageId: string }) => data)
    .handler(async ({ data: { stageId } }) => {
        const res = await backendFetch(`/level/${encodeURIComponent(stageId)}`);
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load level ${stageId}: ${res.status}`);
        return (await res.json()) as IStageMap;
    });

export function levelQueryOptions(stageId: string | null) {
    return queryOptions({
        queryKey: ["level", stageId],
        queryFn: () => (stageId ? getLevelFn({ data: { stageId } }) : Promise.resolve(null)),
        enabled: !!stageId,
        staleTime: 60 * 60 * 1000,
        gcTime: 6 * 60 * 60 * 1000,
    });
}
