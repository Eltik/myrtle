// Score types - Converted from backend Rust types

/** Completion status of an operator based on investment */
export type CompletionStatus =
    | "not_started"
    | "in_progress"
    | "partially_completed"
    | "highly_invested"
    | "absolutely_completed";

/** Details about mastery investment */
export interface MasteryDetails {
    /** Number of skills at M3 */
    m3Count: number;
    /** Total mastery levels across all skills (0-9) */
    totalMasteryLevels: number;
}

/** Details about module investment */
export interface ModuleDetails {
    /** Number of modules unlocked */
    modulesUnlocked: number;
    /** Number of modules at level 3+ */
    modulesAtMax: number;
    /** Highest module level */
    highestLevel: number;
}

/** Details about skin collection for an operator */
export interface SkinDetails {
    /** Total number of skins owned (excluding default E0/E1/E2) */
    ownedCount: number;
    /** Total number of skins available for this operator */
    totalAvailable: number;
    /** Number of L2D (animated) skins owned */
    ownedL2d: number;
    /** Number of store-purchased skins owned */
    ownedStore: number;
    /** Number of event reward skins owned */
    ownedEvent: number;
    /** Total L2D skins available */
    totalL2d: number;
    /** Total store skins available */
    totalStore: number;
    /** Total event skins available */
    totalEvent: number;
    /** Collection completion percentage (0-100) */
    completionPercentage: number;
}

/** Score breakdown for a single operator */
export interface OperatorScore {
    charId: string;
    name: string;
    rarity: number;
    baseScore: number;
    levelScore: number;
    trustScore: number;
    potentialScore: number;
    masteryScore: number;
    moduleScore: number;
    skinScore: number;
    totalScore: number;
    completionStatus: CompletionStatus;
    masteryDetails: MasteryDetails;
    moduleDetails: ModuleDetails;
    skinDetails: SkinDetails;
}

/** Summary statistics for the account */
export interface ScoreBreakdown {
    totalOperators: number;
    sixStarCount: number;
    fiveStarCount: number;
    fourStarCount: number;
    threeStarAndBelowCount: number;
    m9Count: number;
    m3Count: number;
    e2Count: number;
    averageScorePerOperator: number;
    /** Total skins owned across all operators */
    totalSkinsOwned: number;
    /** Operators with full skin collection (100%) */
    fullSkinCollectionCount: number;
}

/** Total account score with detailed breakdown */
export interface UserScore {
    totalScore: number;
    operatorScores: OperatorScore[];
    breakdown: ScoreBreakdown;
}
