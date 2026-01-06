/**
 * DPS Calculator Types
 * Based on backend Rust structures in /backend/src/app/routes/dps_calculator/
 */

// ============================================================================
// Request Types
// ============================================================================

/**
 * Buff parameters that can be applied to operators
 */
export interface DpsBuffs {
    /** ATK buff as decimal (e.g., 0.4 = 40% ATK increase) */
    atk?: number;
    /** Flat ATK bonus added after percentage buffs */
    flatAtk?: number;
    /** Attack speed buff (e.g., 30 = +30 ASPD) */
    aspd?: number;
    /** Fragile debuff on enemy - increases damage taken (e.g., 0.3 = +30% damage) */
    fragile?: number;
}

/**
 * Defense/Resistance shred parameters
 */
export interface DpsShred {
    /** Percentage DEF reduction (e.g., 40 = -40% DEF) */
    def?: number;
    /** Flat DEF reduction */
    defFlat?: number;
    /** Percentage RES reduction (e.g., 20 = -20 RES) */
    res?: number;
    /** Flat RES reduction */
    resFlat?: number;
}

/**
 * Conditional damage toggles for trait/talent/skill/module bonuses
 */
export interface DpsConditionals {
    /** Enable trait damage bonus */
    traitDamage?: boolean;
    /** Enable talent 1 damage bonus */
    talentDamage?: boolean;
    /** Enable talent 2 damage bonus */
    talent2Damage?: boolean;
    /** Enable skill damage bonus */
    skillDamage?: boolean;
    /** Enable module damage bonus */
    moduleDamage?: boolean;
}

/**
 * Operator configuration parameters for DPS calculation
 */
export interface DpsOperatorParams {
    /** Operator potential (1-6), defaults to 6 */
    potential?: number;
    /** Elite/promotion level (0-2), defaults to max */
    promotion?: number;
    /** Operator level, defaults to max for elite */
    level?: number;
    /** Trust level (0-100), defaults to 100 */
    trust?: number;
    /** Skill index (0=basic attack, 1=S1, 2=S2, 3=S3) */
    skillIndex?: number;
    /** Mastery level (0-3), defaults to 3 */
    masteryLevel?: number;
    /** Module index (0=none, 1-3 for X/Y/Delta modules) */
    moduleIndex?: number;
    /** Module level (1-3), defaults to 3 */
    moduleLevel?: number;
    /** External buffs applied to operator */
    buffs?: DpsBuffs;
    /** Base buffs applied before skill multipliers */
    baseBuffs?: DpsBuffs;
    /** SP regeneration boost per second */
    spBoost?: number;
    /** Number of targets hit */
    targets?: number;
    /** Conditional damage toggles */
    conditionals?: DpsConditionals;
    /** Enable/disable all conditionals at once */
    allCond?: boolean;
    /** DEF/RES shred parameters */
    shred?: DpsShred;
}

/**
 * Enemy stats for DPS calculation
 */
export interface DpsEnemyStats {
    /** Enemy DEF value */
    defense: number;
    /** Enemy RES value (0-100+) */
    res: number;
}

/**
 * Range parameters for generating DPS curves
 */
export interface DpsRangeParams {
    /** Minimum DEF value for range calculation */
    minDef?: number;
    /** Maximum DEF value (default: 3000) */
    maxDef?: number;
    /** Step size for DEF iteration (default: 100) */
    defStep?: number;
    /** Minimum RES value for range calculation */
    minRes?: number;
    /** Maximum RES value (default: 120) */
    maxRes?: number;
    /** Step size for RES iteration (default: 10) */
    resStep?: number;
}

/**
 * Request body for DPS calculation endpoint
 */
export interface DpsCalculateRequest {
    /** Operator ID (e.g., "char_017_huang" for Blaze) */
    operatorId: string;
    /** Operator configuration parameters */
    params?: DpsOperatorParams;
    /** Enemy stats to calculate against */
    enemy?: DpsEnemyStats;
    /** Range parameters for generating DPS curves */
    range?: DpsRangeParams;
}

// ============================================================================
// Response Types
// ============================================================================

/**
 * Single DPS result (when no range is specified)
 */
export interface DpsSingleResult {
    /** DPS while skill is active */
    skillDps: number;
    /** Total damage during skill duration */
    totalDamage: number;
    /** Average DPS including skill downtime */
    averageDps: number;
}

/**
 * Data point for DPS range calculations
 */
export interface DpsRangeDataPoint {
    /** DEF or RES value */
    value: number;
    /** DPS at that value */
    dps: number;
}

/**
 * Range DPS result (when range params are specified)
 */
export interface DpsRangeResult {
    /** DPS values by enemy defense */
    byDefense: DpsRangeDataPoint[];
    /** DPS values by enemy resistance */
    byResistance: DpsRangeDataPoint[];
}

/**
 * Operator metadata returned with DPS calculation
 */
export interface DpsOperatorInfo {
    /** Operator ID */
    id: string;
    /** Operator name */
    name: string;
    /** Rarity (1-6) */
    rarity: number;
    /** Elite level used in calculation */
    elite: number;
    /** Level used in calculation */
    level: number;
    /** Potential used in calculation */
    potential: number;
    /** Trust used in calculation */
    trust: number;
    /** Skill index used */
    skillIndex: number;
    /** Effective skill level (1-10) */
    skillLevel: number;
    /** Module index used */
    moduleIndex: number;
    /** Module level used */
    moduleLevel: number;
    /** Final calculated ATK */
    atk: number;
    /** Base attack interval */
    attackInterval: number;
    /** Attack speed (100 = base) */
    attackSpeed: number;
    /** Whether damage is physical (true) or magical (false) */
    isPhysical: boolean;
    /** Skill duration in seconds */
    skillDuration: number;
    /** Skill SP cost */
    skillCost: number;
    /** Skill blackboard parameters */
    skillParameters: number[];
    /** Talent 1 parameters */
    talent1Parameters: number[];
    /** Talent 2 parameters */
    talent2Parameters: number[];
}

/**
 * Response from DPS calculation endpoint
 */
export interface DpsCalculateResponse {
    /** DPS calculation results (single or range) */
    dps: DpsSingleResult | DpsRangeResult;
    /** Operator metadata and calculated stats */
    operator: DpsOperatorInfo;
}

/**
 * Operator entry in the list response
 */
export interface DpsOperatorListEntry {
    /** Operator ID (e.g., "char_017_huang") */
    id: string;
    /** Operator display name */
    name: string;
    /** Calculator implementation name */
    calculatorName: string;
    /** Rarity (1-6) */
    rarity: number;
    /** Profession/class */
    profession: string;
    /** Available skill indices (e.g., [1, 2] or [1, 2, 3]) */
    availableSkills: number[];
    /** Available module indices (e.g., [1, 2] or []) */
    availableModules: number[];
    /** Default skill index for this operator */
    defaultSkillIndex: number;
    /** Default potential for this operator */
    defaultPotential: number;
    /** Default module index for this operator */
    defaultModuleIndex: number;
    /** Maximum promotion/elite level (0, 1, or 2) based on rarity */
    maxPromotion: number;
}

/**
 * Response from list operators endpoint
 */
export interface DpsListOperatorsResponse {
    /** Total count of operators with DPS calculators */
    count: number;
    /** List of operators */
    operators: DpsOperatorListEntry[];
}

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Check if the DPS result is a single calculation (not a range)
 */
export function isDpsSingleResult(result: DpsSingleResult | DpsRangeResult): result is DpsSingleResult {
    return "skillDps" in result && "totalDamage" in result && "averageDps" in result;
}

/**
 * Check if the DPS result is a range calculation
 */
export function isDpsRangeResult(result: DpsSingleResult | DpsRangeResult): result is DpsRangeResult {
    return "byDefense" in result && "byResistance" in result;
}
