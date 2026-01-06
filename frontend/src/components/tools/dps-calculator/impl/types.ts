import type { DpsModuleData, DpsOperatorParams, DpsPotentialRank, DpsSkillData } from "~/types/api/impl/dps-calculator";

/**
 * Operator configuration for the DPS calculator
 */
export interface OperatorConfiguration {
    /** Unique ID for this configuration instance */
    id: string;
    /** Backend operator ID */
    operatorId: string;
    /** Display name */
    operatorName: string;
    /** Rarity (1-6) */
    rarity: number;
    /** Chart line color */
    color: string;
    /** DPS calculation parameters (includes skillIndex, masteryLevel, etc.) */
    params: DpsOperatorParams;
    /** Available skill indices (e.g., [1, 2] or [1, 2, 3]) */
    availableSkills: number[];
    /** Available module indices (e.g., [1, 2] or []) */
    availableModules: number[];
    /** Maximum promotion/elite level (0, 1, or 2) based on rarity */
    maxPromotion: number;
    /** Skill metadata for display (icons, names) */
    skillData?: DpsSkillData[];
    /** Module metadata for display (icons, names, types) */
    moduleData?: DpsModuleData[];
    /** Max level for each phase [E0, E1, E2] */
    phaseLevels?: number[];
    /** Potential rank descriptions (for tooltip display) */
    potentialRanks?: DpsPotentialRank[];
}

/**
 * Chart data point with multiple operator DPS values
 */
export interface ChartDataPoint {
    /** Defense or Resistance value */
    value: number;
    /** DPS values keyed by operator configuration ID */
    [operatorId: string]: number;
}
