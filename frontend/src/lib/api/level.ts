import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

/** A grid coordinate in game space (`row`/`col`). */
export interface IPosition {
    row: number;
    col: number;
    reachOffset?: { x: number; y: number } | null;
}

export interface IRawTile {
    tileKey: string;
    heightType: string;
    buildableType: string;
    passableMask: string;
    playerSideMask?: string;
}

export interface ICheckpoint {
    type: string | number;
    position: IPosition;
    time?: number;
    reachOffset?: { x: number; y: number } | null;
}

export interface IRouteDef {
    motionMode: string | number;
    startPosition: IPosition;
    endPosition: IPosition;
    checkpoints: ICheckpoint[] | null;
    allowDiagonalMove?: boolean;
    visitEveryCheckPoint?: boolean;
}

export interface IMapData {
    /** Row-major tile-index grid, top row first. */
    map: number[][];
    tiles: IRawTile[];
}

export interface IWaveAction {
    actionType: string;
    key?: string;
    routeIndex?: number | null;
    count?: number;
    interval?: number;
    preDelay: number;
    hiddenGroup?: string | null;
}

export interface IFragment {
    preDelay: number;
    actions: IWaveAction[];
}

export interface IWave {
    preDelay: number;
    postDelay: number;
    maxTimeWaitingForNextWave?: number;
    fragments: IFragment[];
}

export interface ITokenInst {
    position: IPosition;
    hidden?: boolean;
    inst?: { characterKey?: string };
}

export interface IEnemyDbRef {
    id: string;
    useDb?: boolean;
}

/** The level `Options` block (stage rules). Serializable scalars only. */
export interface ILevelOptions {
    characterLimit?: number;
    maxLifePoint?: number;
    initialCost?: number;
    maxCost?: number;
    costIncreaseTime?: number;
    moveMultiplier?: number;
    steeringEnabled?: boolean;
    isTrainingLevel?: boolean;
    isHardTrainingLevel?: boolean;
    isPredefinedCardsSelectable?: boolean;
    displayRestTime?: boolean;
    maxPlayTime?: number;
    functionDisableMask?: string;
}

export interface ILevel {
    mapData: IMapData;
    routes: IRouteDef[];
    waves: IWave[];
    predefines?: { tokenInsts?: ITokenInst[] };
    enemyDbRefs?: IEnemyDbRef[];
    tilesDisallowToLocate?: number[];
    options?: ILevelOptions;
}

export const getLevelFn = createServerFn({ method: "GET" })
    .inputValidator((data: { stageId: string }) => data)
    .handler(async ({ data: { stageId } }) => {
        const res = await backendFetch(`/level/${encodeURIComponent(stageId)}`);
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load level ${stageId}: ${res.status}`);
        return (await res.json()) as ILevel;
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
