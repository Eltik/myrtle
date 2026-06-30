import type { IEnemy } from "#/lib/api/enemies";
import type { IDisplayDetailReward } from "#/types/stages";

export interface IEnemyTally {
    enemy: IEnemy | null;
    id: string;
    count: number;
}

/** An enemy's stats as they apply in a specific stage (phase + stage overrides + move multiplier). */
export interface IStageEnemyStats {
    /** Index into the enemy's `stats.levels[]` this stage uses. */
    levelIndex: number;
    /** Total number of stat blocks the enemy ships (>1 ⇒ multi-phase/form). */
    phaseCount: number;
    maxHp: number;
    atk: number;
    def: number;
    /** Magic resistance, shown as a percentage. */
    res: number;
    /** Effective tiles/sec after the stage's move multiplier. */
    moveSpeed: number;
    /** Seconds between attacks, after attack-speed scaling. */
    attackInterval: number;
    /** Whether the stage overrides any base attribute. */
    hasOverride: boolean;
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
