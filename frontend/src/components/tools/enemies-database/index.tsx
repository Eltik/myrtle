"use client";

import type { Enemy, EnemyInfoList, RaceData } from "~/types/api";

interface EnemiesDatabaseProps {
    enemies: Enemy[];
    races: Record<string, RaceData>;
    levelInfo: EnemyInfoList[];
    total: number;
}

export function EnemiesDatabase({ enemies: _enemies, races: _races, levelInfo: _levelInfo, total: _total }: EnemiesDatabaseProps) {
    return <div></div>;
}
