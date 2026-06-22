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

/** Grouped spawn-schedule entry (one per SPAWN action): the spawn-table row. */
export interface IScheduleEntry {
    /** 1-based row number in spawn order. */
    index: number;
    /** Inclusive `[first, last]` 1-based enemy ordinals this action emits. */
    order: [number, number];
    enemyId: string;
    route: number;
    /** Absolute time of the action's first token (seconds from stage start). */
    t0: number;
    /** Time of the action relative to its wave's start. */
    waveTime: number;
    /** Seconds between successive tokens. */
    interval: number;
    count: number;
    /** 1-based wave number. */
    wave: number;
}

/** A conditional / branch route the board can reveal (from `ExtraRoutes`). */
export interface IHiddenRoute {
    index: number;
    motion: string;
    points: IPoint[];
}

export interface IModifierValue {
    key: string;
    value: number;
    valueStr: string | null;
}

/** A stage-wide modifier (CC/IS rune). */
export interface IModifier {
    key: string;
    difficulty: string;
    profession: string;
    blackboard: IModifierValue[];
}

/** Stage rules parsed from the level `Options` block. */
export interface IMapOptions {
    charLimit: number;
    maxLife: number;
    initialCost: number;
    maxCost: number;
    costIncreaseTime: number;
    moveMultiplier: number;
    isTraining: boolean;
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
    /** Grouped spawn schedule (one entry per action), for the spawn table. */
    schedule: IScheduleEntry[];
    roster: IRosterEntry[];
    /** Stage rules (character limit, cost, life points, …). */
    options: IMapOptions;
    /** Conditional / branch routes the board can reveal. */
    hiddenRoutes: IHiddenRoute[];
    /** Stage-wide modifiers (CC/IS runes); empty for normal play. */
    modifiers: IModifier[];
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
