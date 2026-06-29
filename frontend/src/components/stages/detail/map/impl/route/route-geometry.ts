import type { GameMap, IWaypoint } from "../map-model";
import type { ICheckpoint, IRouteDef } from "../types";
import { MOTION_MODE, MOVE_TYPE } from "../util/enums";
import { tileCenter } from "../util/geometry";

function normalizeCheckpoint(c: ICheckpoint): IWaypoint {
    return { row: c.position.row, col: c.position.col, type: c.type, time: c.time, reachOffset: c.reachOffset };
}

const WAIT_TYPES: (string | number)[] = [MOVE_TYPE.WAIT_FOR_SECONDS, MOVE_TYPE.WAIT_FOR_PLAY_TIME, MOVE_TYPE.WAIT_CURRENT_FRAGMENT_TIME, MOVE_TYPE.WAIT_CURRENT_WAVE_TIME, MOVE_TYPE.DISAPPEAR, 1, 3, 4, 5];
const TIMER_WAIT: (string | number)[] = [MOVE_TYPE.WAIT_FOR_SECONDS, MOVE_TYPE.WAIT_CURRENT_FRAGMENT_TIME, 1, 3];

export interface IRoutePath {
    d: string;
    length: number;
    start: { x: number; y: number } | null;
    dots: { x: number; y: number }[];
    /** `dist` is the distance along the path (px) where the pause happens. */
    waits: { x: number; y: number; time: number; dist: number }[];
    isAir: boolean;
}

export function buildRoutePath(route: IRouteDef, mapObject: GameMap, vbHeight: number): IRoutePath {
    const { motionMode, startPosition, checkpoints = [], endPosition, allowDiagonalMove, visitEveryCheckPoint } = route;
    const ground = motionMode === MOTION_MODE.WALK || motionMode === 0;
    const hasPatrol = (checkpoints || []).some((c) => c.type === MOVE_TYPE.PATROL_MOVE);
    const fallback = (): IWaypoint[] => [startPosition, ...(checkpoints || []).map(normalizeCheckpoint).filter(Boolean), endPosition];

    let waypoints: IWaypoint[];
    if (!mapObject.initialized || !ground) {
        waypoints = fallback();
    } else {
        try {
            waypoints = mapObject.findPath({ startPosition, checkpoints, endPosition: hasPatrol ? null : endPosition, motionMode, allowDiagonalMove, visitEveryCheckPoint });
        } catch {
            waypoints = fallback();
        }
    }
    waypoints = waypoints.filter(Boolean);

    const cmds: (string | number)[] = [];
    const dots: { x: number; y: number }[] = [];
    const waits: { x: number; y: number; time: number; dist: number }[] = [];
    let start: { x: number; y: number } | null = null;
    let prev = waypoints[0];
    let portal = false;
    let length = 0;
    let lastXY: { x: number; y: number } | null = null;
    for (const wp of waypoints) {
        if (wp.type === MOVE_TYPE.ALERT || wp.type === 7) continue;
        if (wp.type === MOVE_TYPE.DISAPPEAR || wp.type === 5) portal = true;
        if (wp.type === MOVE_TYPE.APPEAR_AT_POS || wp.type === 6) portal = false;
        const { x, y } = tileCenter(wp);
        const sy = vbHeight - y;
        if (wp.type !== undefined && WAIT_TYPES.includes(wp.type)) {
            const c = tileCenter(prev);
            if (TIMER_WAIT.includes(wp.type)) waits.push({ x: c.x, y: vbHeight - c.y, time: wp.time ?? 0, dist: length });
            continue;
        }
        const isMoveDot = (wp.type === MOVE_TYPE.MOVE || wp.type === 0) && !(wp.row === startPosition.row && wp.col === startPosition.col) && !(wp.row === endPosition.row && wp.col === endPosition.col);
        if (isMoveDot) dots.push({ x, y: sy });
        if (lastXY) length += Math.hypot(x - lastXY.x, sy - lastXY.y);
        lastXY = { x, y: sy };
        if (cmds.length === 0) start = { x, y: sy };
        cmds.push(cmds.length === 0 ? "M" : "L", x, sy);
        prev = wp;
    }
    void portal;
    return { d: cmds.join(" "), length: length || 1, start, dots, waits, isAir: motionMode === MOTION_MODE.FLY || motionMode === 1 };
}
