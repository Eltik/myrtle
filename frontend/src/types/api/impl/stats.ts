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

// ==================== User Stats Types ====================

/** Per-profession completion data */
export interface ProfessionStat {
    /** Internal profession name (e.g., "WARRIOR", "SNIPER") */
    profession: string;
    /** Display name (e.g., "Guard", "Sniper") */
    displayName: string;
    /** Number of operators the user owns in this profession */
    owned: number;
    /** Total obtainable operators in this profession */
    total: number;
    /** Completion percentage (0-100) */
    percentage: number;
}

/** Complete stats response for the Stats tab */
export interface UserStatsResponse {
    /** Per-profession owned/total breakdown, sorted by CLASS_SORT_ORDER */
    professions: ProfessionStat[];
    /** E0/E1/E2 breakdown */
    eliteBreakdown: {
        e0: number;
        e1: number;
        e2: number;
        total: number;
    };
    /** Mastery investment stats */
    masteries: {
        /** Operators with at least one skill at M3 */
        m3Count: number;
        /** Operators with exactly 2 skills at M3 */
        m6Count: number;
        /** Operators with exactly 3 skills at M3 */
        m9Count: number;
        /** Sum of all specializeLevel values across all operators */
        totalMasteryLevels: number;
        /** Maximum possible mastery levels (E2 operators * skills count * 3) */
        maxPossibleMasteryLevels: number;
    };
    /** Module investment stats */
    modules: {
        /** Non-INITIAL modules unlocked (level > 0) */
        unlocked: number;
        /** Non-INITIAL modules at max level (level === 3) */
        atMax: number;
        /** Total non-INITIAL modules available across all owned operators */
        totalAvailable: number;
    };
    /** Skin collection stats (from score breakdown) */
    skins: {
        /** Total skins owned */
        totalOwned: number;
        /** Operators with complete skin collection */
        fullCollectionCount: number;
    };
    /** Total operators owned */
    totalOwned: number;
    /** Total obtainable operators in the game */
    totalAvailable: number;
    /** Overall collection completion percentage (0-100) */
    collectionPercentage: number;
}
