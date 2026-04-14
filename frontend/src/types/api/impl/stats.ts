export interface StatsResponse {
    users: {
        total: number;
        byServer: {
            en: number;
            jp: number;
            kr: number;
            cn: number;
            bili: number;
            tw: number;
        };
        signups7d: number;
        signups30d: number;
        publicProfiles: number;
    };
    gacha: {
        totalPulls: number;
        contributingUsers: number;
        sixStarCount: number;
        fiveStarCount: number;
        fourStarCount: number;
    };
    gameData: {
        operators: number;
        skills: number;
        modules: number;
        skins: number;
        stages: number;
        zones: number;
        enemies: number;
    };
    tierLists: {
        total: number;
        active: number;
        totalVersions: number;
        totalPlacements: number;
    };
    computedAt: string;
}

// ==================== User Stats Types ====================

/** Per-sub-profession completion data */
export interface SubProfessionStat {
    /** Internal sub-profession ID (e.g., "fastshot", "blastcaster") */
    subProfessionId: string;
    /** Display name (e.g., "Marksman Sniper", "Blast Caster") */
    displayName: string;
    /** Number of operators the user owns in this sub-profession */
    owned: number;
    /** Total obtainable operators in this sub-profession */
    total: number;
    /** Completion percentage (0-100) */
    percentage: number;
}

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
    /** Per-sub-profession breakdown, sorted alphabetically */
    subProfessions: SubProfessionStat[];
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
    /** Skin collection stats */
    skins: {
        /** Total non-default skins owned by the user */
        totalOwned: number;
        /** Total non-default skins available in the game */
        totalAvailable: number;
        /** Completion percentage (0-100) */
        percentage: number;
    };
    /** Total operators owned */
    totalOwned: number;
    /** Total obtainable operators in the game */
    totalAvailable: number;
    /** Overall collection completion percentage (0-100) */
    collectionPercentage: number;
}
