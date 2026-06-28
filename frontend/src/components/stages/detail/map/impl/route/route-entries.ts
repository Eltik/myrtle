import type { GameMap } from "../map-model";
import type { ILevel } from "../types";
import { viewBox } from "../util/geometry";
import { buildEnemyActions } from "../waves";
import { buildRoutePath } from "./route-geometry";

export interface RouteEntry {
    position: number;
    timestamp: number;
    enemyIndexRange: string;
    d: string;
    length: number;
    dots: { x: number; y: number }[];
    waits: { x: number; y: number; time: number }[];
    isAir: boolean;
}

export interface RouteEntries {
    entries: RouteEntry[];
    total: number;
    dims: { width: number; height: number };
}

export function buildRouteEntries(level: ILevel, mapObject: GameMap): RouteEntries {
    const dims = viewBox(level.mapData.map[0].length, level.mapData.map.length);
    const { actions, totalCount } = buildEnemyActions(level);
    const entries: RouteEntry[] = actions.map((a, position) => {
        const path = buildRoutePath(level.routes[a.routeIndex], mapObject, dims.height);
        return { position, timestamp: a.timestamp, enemyIndexRange: a.enemyIndexRange, ...path };
    });
    return { entries, total: totalCount, dims };
}
