import type { ILevel } from "./types";

const SKIP_TYPES = new Set<string | number>(["PREVIEW_CURSOR", "STORY", "TUTORIAL", "PLAY_BGM", "DISPLAY_ENEMY_INFO", "PLAY_OPERA", "DIALOG", 1, 2, 4, 5, 7]);

export interface IEnemyAction {
    routeIndex: number;
    timestamp: number;
    count: number;
    enemyIndexRange: string;
}

export interface IEnemyActions {
    actions: IEnemyAction[];
    totalCount: number;
}

export function buildEnemyActions(level: ILevel, waveIntervalFallback = 5): IEnemyActions {
    const enemyIds = new Set((level.enemyDbRefs || []).map((r) => r.id));
    const actions: IEnemyAction[] = [];
    let waveOffset = 0;
    let enemyCount = 0;

    for (const wave of level.waves || []) {
        let t = wave.preDelay;
        for (const frag of wave.fragments || []) {
            t += frag.preDelay;
            const fragStart = t;
            let fragEnd = t;
            const ordered = [...(frag.actions || [])].sort((a, b) => a.preDelay - b.preDelay);
            for (const a of ordered) {
                const count = a.count || 1;
                const interval = a.interval || 0;
                fragEnd = Math.max(fragEnd, fragStart + a.preDelay + (count - 1) * interval);
                if (SKIP_TYPES.has(a.actionType)) continue;
                t = fragStart + a.preDelay;
                const isEnemy = a.actionType === "SPAWN" || (a.key !== undefined && enemyIds.has(a.key));
                if (isEnemy && a.routeIndex != null && a.routeIndex >= 0) {
                    actions.push({
                        routeIndex: a.routeIndex,
                        timestamp: t + waveOffset,
                        count,
                        enemyIndexRange: count === 1 ? `${enemyCount + 1}` : `${enemyCount + 1} ~ ${enemyCount + count}`,
                    });
                    enemyCount += count;
                }
                t += (count - 1) * interval;
            }
            t = fragEnd;
        }
        t += wave.postDelay;
        waveOffset += t + waveIntervalFallback;
    }
    return { actions, totalCount: enemyCount };
}
