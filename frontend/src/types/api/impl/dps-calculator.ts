/**
 * DPS Calculator Types
 * Based on v3 backend Rust structures
 */

// ============================================================================
// Request Types
// ============================================================================

/**
 * Buff parameters that can be applied to operators
 */
export interface DpsBuffs {
    atk?: number;
    flatAtk?: number;
    aspd?: number;
    fragile?: number;
}

/**
 * Defense/Resistance shred parameters
 */
export interface DpsShred {
    def?: number;
    defFlat?: number;
    res?: number;
    resFlat?: number;
}

/**
 * Conditional damage toggles for trait/talent/skill/module bonuses
 */
export interface DpsConditionals {
    traitDamage?: boolean;
    talentDamage?: boolean;
    talent2Damage?: boolean;
    skillDamage?: boolean;
    moduleDamage?: boolean;
}

/**
 * Request body for DPS calculation endpoint
 */
export interface DpsCalculateRequest {
    operatorId: string;
    params?: DpsOperatorParams;
    enemy?: DpsEnemyStats;
    range?: DpsRangeParams;
}

// ============================================================================
// Response Types
// ============================================================================

/**
 * Response from DPS calculation endpoint (single point)
 */
export interface DpsSingleResult {
    skill_dps: number;
    total_damage: number;
    average_dps: number;
}

/**
 * Range data point for DPS curve
 */
export interface DpsRangeDataPoint {
    value: number;
    dps: number;
}

/**
 * Response from DPS range calculation
 */
export interface DpsRangeResult {
    byDefense: DpsRangeDataPoint[];
    byResistance: DpsRangeDataPoint[];
}

/**
 * DPS calculation response - either single or range result
 */
export interface DpsCalculateResponse {
    operator: DpsOperatorInfo;
    dps: DpsSingleResult | DpsRangeResult;
}

/**
 * Type guard for range result
 */
export function isDpsRangeResult(dps: DpsSingleResult | DpsRangeResult): dps is DpsRangeResult {
    return "byDefense" in dps;
}

/**
 * Type guard for single result
 */
export function isDpsSingleResult(dps: DpsSingleResult | DpsRangeResult): dps is DpsSingleResult {
    return "skill_dps" in dps;
}

/**
 * Operator info returned in DPS response
 */
export interface DpsOperatorInfo {
    id: string;
    name: string;
    rarity: number;
    profession: string;
}

// ============================================================================
// Operator List Types
// ============================================================================

/**
 * Conditional type enum values
 */
export type DpsConditionalType = "trait" | "talent" | "talent2" | "skill" | "module";

/**
 * Conditional info from backend - describes operator-specific conditionals
 */
export interface DpsConditionalInfo {
    conditionalType: DpsConditionalType;
    name: string;
    default: boolean;
    inverted?: boolean;
    applicableSkills: number[];
    applicableModules: number[];
    minElite: number;
    minModuleLevel: number;
    /** @deprecated v3 snake_case alias */
    conditional_type?: string;
    /** @deprecated v3 snake_case alias */
    skills?: number[];
    /** @deprecated v3 snake_case alias */
    modules?: number[];
}

/**
 * Operator entry in the list response
 */
export interface DpsOperatorListEntry {
    id: string;
    name: string;
    calculatorName: string;
    rarity: number;
    profession: string;
    availableSkills: number[];
    availableModules: number[];
    defaultSkillIndex: number;
    defaultModuleIndex: number;
    defaultPotential: number;
    maxPromotion: number;
    phaseLevels?: number[];
    skillData?: DpsSkillData[];
    moduleData?: DpsModuleData[];
    potentialRanks?: DpsPotentialRank[];
    conditionals: DpsConditionalInfo[];
    /** @deprecated v3 snake_case alias */
    available_skills?: number[];
    /** @deprecated v3 snake_case alias */
    available_modules?: number[];
    /** @deprecated v3 snake_case alias */
    default_skill?: number;
    /** @deprecated v3 snake_case alias */
    default_module?: number;
}

/**
 * Response from list operators endpoint - v3 returns array directly
 */
export type DpsListOperatorsResponse = DpsOperatorListEntry[];

// ============================================================================
// Client-side Operator Parameter Types
// ============================================================================

/**
 * Enemy stats for DPS calculation
 */
export interface DpsEnemyStats {
    defense?: number;
    res?: number;
}

/**
 * Client-side operator params (camelCase, sent to /api/dps-calculator)
 */
export interface DpsOperatorParams {
    potential?: number;
    trust?: number;
    promotion?: number;
    level?: number;
    skillIndex?: number;
    masteryLevel?: number;
    moduleIndex?: number;
    moduleLevel?: number;
    targets?: number;
    spBoost?: number;
    allCond?: boolean;
    conditionals?: DpsConditionals;
    buffs?: DpsBuffs;
    shred?: DpsShred;
}

/**
 * Range params for DPS curve generation
 */
export interface DpsRangeParams {
    minDef?: number;
    maxDef?: number;
    defStep?: number;
    minRes?: number;
    maxRes?: number;
    resStep?: number;
}

// ============================================================================
// Operator Metadata Types (returned in operator list)
// ============================================================================

/**
 * Skill metadata for display
 */
export interface DpsSkillData {
    index: number;
    name: string;
    iconId?: string;
}

/**
 * Module metadata for display
 */
export interface DpsModuleData {
    index: number;
    name: string;
    typeName1?: string;
    uniEquipName?: string;
    iconId?: string;
}

/**
 * Potential rank description
 */
export interface DpsPotentialRank {
    rank: number;
    description: string;
    /** Backend returns Description (capital D) - game data format */
    Description?: string;
}
