import type { IEnemy } from "#/lib/api/enemies";
import type { IDisplayDetailReward } from "#/types/stages";

export interface IEnemyTally {
    enemy: IEnemy | null;
    id: string;
    count: number;
}

export interface ISpawnRow {
    idx: number;
    id: string;
    enemy: IEnemy | null;
    count: number;
    interval: number;
    preDelay: number;
    time: number;
    wave: number;
    hiddenGroup: string | null;
    actionType: string;
}

export interface IResolvedDrop {
    reward: IDisplayDetailReward;
    name: string;
    iconUrl: string;
    rarity: number;
    isChar: boolean;
    occ: { label: string; level: number; tone: string };
}

export interface IDropGroup {
    type: string;
    label: string;
    order: number;
    drops: IResolvedDrop[];
}
