export interface StatsResponse {
    users: {
        total: number;
        byServer: Record<string, number>;
        recentSignups7d: number;
        recentSignups30d: number;
        publicProfiles: number;
    };
    gacha: {
        totalPulls: number;
        contributingUsers: number;
        pullRates: {
            sixStarRate: number;
            fiveStarRate: number;
            fourStarRate: number;
            threeStarRate: number;
        };
    };
    gameData: {
        operators: number;
        skills: number;
        modules: number;
        skins: number;
        items: number;
        stages: number;
        enemies: number;
        gachaPools: number;
    };
    tierLists: {
        total: number;
        active: number;
        totalVersions: number;
        totalPlacements: number;
        communityCount: number;
    };
    computedAt: string;
    cached: boolean;
}
