export type EnemyHandbook = {
    levelInfoList: EnemyInfoList[];
    enemyData: Record<string, Enemy>;
    raceData: Record<
        string,
        {
            id: string;
            raceName: string;
            sortId: number;
        }
    >;
};

export type EnemyInfoList = {
    classLevel: string;
    attack: {
        min: number;
        max: number;
    };
    def: {
        min: number;
        max: number;
    };
    magicRes: {
        min: number;
        max: number;
    };
    maxHP: {
        min: number;
        max: number;
    };
    moveSpeed: {
        min: number;
        max: number;
    };
    attackSpeed: {
        min: number;
        max: number;
    };
    enemyDamageRes: {
        min: number;
        max: number;
    };
    enemyRes: {
        min: number;
        max: number;
    };
};

export type Enemy = {
    enemyId: string;
    enemyIndex: string;
    enemyTags: string[] | null;
    sortId: number;
    name: string;
    enemyLevel: "NORMAL" | "ELITE" | "BOSS";
    description: string;
    attackType: string | null;
    ability: string | null;
    isInvalidKilled: boolean;
    overrideKillCntInfos: unknown;
    hideInHandbook: boolean;
    hideInStage: boolean;
    abilityList: {
        text: string;
        textFormat: string;
    }[];
    linkEnemies: string[];
    damageType: ("PHYSIC" | "MAGIC" | "NO_DAMAGE")[];
    invisibleDetail: boolean;
};
