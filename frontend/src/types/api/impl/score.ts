// Score types - Converted from backend Rust types

/** Completion status of an operator based on investment */
export type CompletionStatus = "not_started" | "in_progress" | "partially_completed" | "highly_invested" | "absolutely_completed";

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
    // === Operator Stats ===
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
    /** Total operators available in the game (excluding tokens/traps) */
    totalOperatorsAvailable: number;
    /** Operator collection completion percentage (0-100) */
    operatorCollectionPercentage: number;

    // === Stage Stats ===
    /** Main story completion percentage (0-100) */
    mainlineCompletion: number;
    /** Side story completion percentage (0-100) */
    sidestoryCompletion: number;
    /** Event/activity completion percentage (0-100) */
    activityCompletion: number;
    /** Permanent stages completed (mainline + sidestory only) */
    permanentStagesCompleted: number;
    /** Permanent stages available (mainline + sidestory only) */
    permanentStagesAvailable: number;
    /** Total stages completed across all zones (including time-limited) */
    totalStagesCompleted: number;
    /** Total stages available in the game (including time-limited) */
    totalStagesAvailable: number;
    /** Total perfect/3-star clears */
    totalPerfectClears: number;
    /** Overall stage completion percentage (0-100) - based on permanent stages only */
    overallStageCompletionPercentage: number;

    // === Roguelike (Integrated Strategies) Stats ===
    /** Number of roguelike themes played */
    roguelikeThemesPlayed: number;
    /** Total unique endings unlocked across all themes */
    roguelikeTotalEndings: number;
    /** Total battle pass levels achieved */
    roguelikeTotalBpLevels: number;
    /** Total buffs unlocked */
    roguelikeTotalBuffs: number;
    /** Total collectibles (bands + relics + capsules) */
    roguelikeTotalCollectibles: number;
    /** Total runs completed */
    roguelikeTotalRuns: number;
    /** Total challenges cleared at max difficulty (grade 2) */
    roguelikeGrade2Challenges: number;
    /** Number of themes with at least one grade 2 clear */
    roguelikeThemesAtMaxDifficulty: number;
    /** Total themes available in the game */
    roguelikeTotalThemesAvailable: number;
    /** Total endings available across all themes */
    roguelikeTotalMaxEndings: number;
    /** Total collectibles available across all themes */
    roguelikeTotalMaxCollectibles: number;
    /** Total challenges available across all themes */
    roguelikeTotalMaxChallenges: number;
    /** Roguelike themes completion percentage (0-100) */
    roguelikeThemesCompletionPercentage: number;
    /** Roguelike endings completion percentage (0-100) */
    roguelikeEndingsCompletionPercentage: number;
    /** Roguelike collectibles completion percentage (0-100) */
    roguelikeCollectiblesCompletionPercentage: number;
    /** Roguelike challenges (grade 2) completion percentage (0-100) */
    roguelikeChallengesCompletionPercentage: number;
    /** Overall roguelike completion percentage (0-100) */
    roguelikeOverallCompletionPercentage: number;

    // === Sandbox (Reclamation Algorithm) Stats ===
    /** Places completed (state = 2) */
    sandboxPlacesCompleted: number;
    /** Places discovered (state = 1) */
    sandboxPlacesDiscovered: number;
    /** Total places available */
    sandboxPlacesTotal: number;
    /** Completion percentage for places */
    sandboxCompletionPercentage: number;
    /** Total nodes completed */
    sandboxNodesCompleted: number;
    /** Landmark nodes completed */
    sandboxLandmarkNodes: number;
    /** Special nodes completed */
    sandboxSpecialNodes: number;
    /** Tech trees completed */
    sandboxTechTreesCompleted: number;
    /** Stories unlocked */
    sandboxStoriesUnlocked: number;
    /** Events triggered */
    sandboxEventsCompleted: number;
    /** Log entries collected */
    sandboxLogEntries: number;
    /** Chapters with at least one log */
    sandboxChaptersWithLogs: number;

    // === Medal Stats ===
    /** Total medals earned */
    medalTotalEarned: number;
    /** Total medals available */
    medalTotalAvailable: number;
    /** Medal completion percentage */
    medalCompletionPercentage: number;
    /** T1 (Common) medals earned */
    medalT1Earned: number;
    /** T2 (Uncommon) medals earned */
    medalT2Earned: number;
    /** T3 (Rare) medals earned */
    medalT3Earned: number;
    /** T2D5 (Special) medals earned */
    medalT2d5Earned: number;
    /** Number of medal groups fully completed */
    medalGroupsComplete: number;

    // === Base (RIIC) Efficiency Stats ===
    /** Number of trading posts */
    baseTradingPostCount: number;
    /** Number of factories */
    baseFactoryCount: number;
    /** Number of power plants */
    basePowerPlantCount: number;
    /** Number of dormitories */
    baseDormitoryCount: number;
    /** Average trading post efficiency (percentage) */
    baseAvgTradingEfficiency: number;
    /** Average factory efficiency (percentage) */
    baseAvgFactoryEfficiency: number;
    /** Total comfort across all dormitories */
    baseTotalComfort: number;
    /** Electricity balance (output - consumption) */
    baseElectricityBalance: number;
    /** Number of buildings at max level */
    baseMaxLevelBuildings: number;

    // === Check-In Stats ===
    /** Days checked in during current cycle (count of 1s in check_in_history) */
    checkInCurrentCycle: number;
    /** Total days in current check-in cycle window (typically 15-16) */
    checkInCycleLength: number;
    /** Check-in completion percentage for current cycle (0-100) */
    checkInCompletionPercentage: number;
}

/** A single completion metric with current, maximum, and percentage */
export interface CompletionMetric {
    /** Current count achieved */
    current: number;
    /** Maximum possible count */
    maximum: number;
    /** Completion percentage (0-100) */
    percentage: number;
}

/** Summary of all finite completion metrics for easy access */
export interface CompletionSummary {
    /** Operator collection: owned / available */
    operators: CompletionMetric;
    /** Stage completion: completed / available */
    stages: CompletionMetric;
    /** Medal completion: earned / available */
    medals: CompletionMetric;
    /** Roguelike overall completion (collectibles-focused) */
    roguelike: CompletionMetric;
    /** Sandbox places completion: completed / total */
    sandbox: CompletionMetric;
    /** Check-in completion: days checked in / cycle length */
    checkIn: CompletionMetric;
}

/** Detailed stats for a roguelike theme */
export interface RoguelikeThemeDetails {
    /** Total unique endings unlocked */
    endingsUnlocked: number;
    /** Total runs by mode */
    normalRuns: number;
    challengeRuns: number;
    monthTeamRuns: number;
    /** Battle pass level achieved (based on rewards claimed) */
    bpLevel: number;
    /** Accumulated score from buff.score */
    totalAccumulatedScore: number;
    /** Outbuffs unlocked count */
    buffsUnlocked: number;
    /** Collectibles */
    bandsUnlocked: number;
    relicsUnlocked: number;
    capsulesUnlocked: number;
    /** Challenge mode grades achieved */
    challengeGradesAchieved: number;
    highestChallengeGrade: number;
    /** Challenges cleared at max difficulty (grade 2) */
    grade2Challenges: number;

    // === Max Values (from game data) ===
    /** Max endings available for this theme */
    maxEndings: number;
    /** Max relics available for this theme */
    maxRelics: number;
    /** Max capsules available for this theme */
    maxCapsules: number;
    /** Max bands available for this theme */
    maxBands: number;
    /** Max challenges available for this theme */
    maxChallenges: number;
    /** Max monthly squads available for this theme */
    maxMonthlySquads: number;
    /** Highest difficulty grade available for this theme */
    maxDifficultyGrade: number;

    // === Completion Percentages ===
    /** Endings completion percentage (0-100) */
    endingsCompletionPercentage: number;
    /** Relics completion percentage (0-100) */
    relicsCompletionPercentage: number;
    /** Capsules completion percentage (0-100) */
    capsulesCompletionPercentage: number;
    /** Bands completion percentage (0-100) */
    bandsCompletionPercentage: number;
    /** Total collectibles (relics + capsules + bands) completion percentage (0-100) */
    collectiblesCompletionPercentage: number;
    /** Challenges at max grade (grade 2) completion percentage (0-100) */
    challengesAtMaxGradePercentage: number;
    /** Overall theme completion percentage (0-100) */
    overallCompletionPercentage: number;
}

/** Score for a single roguelike theme */
export interface RoguelikeThemeScore {
    themeId: string;
    totalScore: number;
    endingsScore: number;
    bpScore: number;
    buffsScore: number;
    collectiblesScore: number;
    challengeScore: number;
    difficultyScore: number;
    details: RoguelikeThemeDetails;
}

/** Summary breakdown for roguelike progress */
export interface RoguelikeBreakdown {
    themesPlayed: number;
    totalEndings: number;
    totalBpLevels: number;
    totalBuffs: number;
    totalCollectibles: number;
    totalRuns: number;
    /** Total challenges cleared at max difficulty (grade 2) across all themes */
    totalGrade2Challenges: number;
    /** Number of themes with at least one grade 2 clear */
    themesAtMaxDifficulty: number;

    // === Max Totals (from game data) ===
    /** Total themes available in the game */
    totalThemesAvailable: number;
    /** Total endings available across all themes */
    totalMaxEndings: number;
    /** Total collectibles (relics + capsules + bands) available across all themes */
    totalMaxCollectibles: number;
    /** Total challenges available across all themes */
    totalMaxChallenges: number;
    /** Total monthly squads available across all themes */
    totalMaxMonthlySquads: number;

    // === Completion Percentages ===
    /** Themes played completion percentage (0-100) */
    themesCompletionPercentage: number;
    /** Endings completion percentage (0-100) */
    endingsCompletionPercentage: number;
    /** Collectibles (relics + capsules + bands) completion percentage (0-100) */
    collectiblesCompletionPercentage: number;
    /** Challenges at max grade (grade 2) completion percentage (0-100) */
    challengesCompletionPercentage: number;
    /** Overall roguelike completion percentage (0-100) */
    overallCompletionPercentage: number;
}

/** Overall roguelike score aggregating all themes */
export interface RoguelikeScore {
    totalScore: number;
    themeScores: RoguelikeThemeScore[];
    breakdown: RoguelikeBreakdown;
}

/** Total account score with detailed breakdown */
export interface UserScore {
    totalScore: number;
    operatorScores: OperatorScore[];
    breakdown: ScoreBreakdown;
}
