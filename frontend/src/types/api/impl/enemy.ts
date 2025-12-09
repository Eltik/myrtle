// Enemy types

export type EnemyLevel = "NORMAL" | "ELITE" | "BOSS";

export type DamageType = "PHYSIC" | "MAGIC" | "NO_DAMAGE";

export interface StatRange {
    min: number;
    max: number;
}

export interface EnemyInfoList {
    classLevel: string;
    attack: StatRange;
    def: StatRange;
    magicRes: StatRange;
    maxHP: StatRange;
    moveSpeed: StatRange;
    attackSpeed: StatRange;
    enemyDamageRes: StatRange;
    enemyRes: StatRange;
}

export interface RaceData {
    id: string;
    raceName: string;
    sortId: number;
}

export interface AbilityInfo {
    text: string;
    textFormat: string;
}

export interface Enemy {
    enemyId: string;
    enemyIndex: string;
    enemyTags: string[] | null;
    sortId: number;
    name: string;
    enemyLevel: EnemyLevel;
    description: string;
    attackType: string | null;
    ability: string | null;
    isInvalidKilled: boolean;
    overrideKillCntInfos: unknown | null;
    hideInHandbook: boolean;
    hideInStage: boolean;
    abilityList: AbilityInfo[];
    linkEnemies: string[];
    damageType: DamageType[];
    invisibleDetail: boolean;
}

export interface EnemyHandbook {
    levelInfoList: EnemyInfoList[];
    enemyData: Record<string, Enemy>;
    raceData: Record<string, RaceData>;
}
