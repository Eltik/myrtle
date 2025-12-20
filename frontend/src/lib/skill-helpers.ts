import type { SkillLevel } from "~/types/api/impl/skill";

/**
 * Represents changes between two skill levels for comparison display.
 */
export interface SkillDiff {
    spCostChanged: boolean;
    initSpChanged: boolean;
    durationChanged: boolean;
    blackboardChanges: Map<string, { prev: number; curr: number }>;
}

/**
 * Computes the differences between two skill levels.
 * Used for skill comparison views to highlight what changed.
 */
export function computeSkillDiff(prevLevel: SkillLevel | null, currLevel: SkillLevel): SkillDiff {
    const blackboardChanges = new Map<string, { prev: number; curr: number }>();

    if (prevLevel) {
        // Compare blackboard values
        const prevBlackboard = new Map(prevLevel.blackboard?.map((b) => [b.key.toLowerCase(), b.value]) ?? []);
        for (const curr of currLevel.blackboard ?? []) {
            const prevValue = prevBlackboard.get(curr.key.toLowerCase());
            if (prevValue !== undefined && prevValue !== curr.value) {
                blackboardChanges.set(curr.key.toLowerCase(), { prev: prevValue, curr: curr.value });
            }
        }
    }

    return {
        spCostChanged: prevLevel !== null && prevLevel.spData?.spCost !== currLevel.spData?.spCost,
        initSpChanged: prevLevel !== null && prevLevel.spData?.initSp !== currLevel.spData?.initSp,
        durationChanged: prevLevel !== null && prevLevel.duration !== currLevel.duration,
        blackboardChanges,
    };
}

/**
 * Formats a blackboard value for display.
 * Automatically detects percentage-based keys and formats accordingly.
 */
export function formatBlackboardValue(key: string, value: number): string {
    // Common percentage-based keys
    const percentageKeys = ["atk", "attack_speed", "def", "max_hp", "hp_recovery_per_sec", "sp_recovery_per_sec", "damage_scale", "atk_scale", "def_scale"];
    const isPercentage = percentageKeys.some((k) => key.toLowerCase().includes(k));

    if (isPercentage && Math.abs(value) < 10) {
        return `${Math.round(value * 100)}%`;
    }
    if (Number.isInteger(value)) {
        return String(value);
    }
    return value.toFixed(1);
}

/**
 * Formats a skill level index for display.
 * Returns "Lv.X" for levels 1-7 and "MX" for mastery levels.
 */
export function formatSkillLevel(level: number): string {
    if (level < 7) return `Lv.${level + 1}`;
    return `M${level - 6}`;
}

/**
 * Gets the human-readable label for an SP recovery type.
 */
export function getSpTypeLabel(spType: string): string {
    switch (spType) {
        case "INCREASE_WITH_TIME":
            return "Auto Recovery";
        case "INCREASE_WHEN_ATTACK":
            return "Offensive Recovery";
        case "INCREASE_WHEN_TAKEN_DAMAGE":
            return "Defensive Recovery";
        case "UNKNOWN_8":
            return "N/A";
        default:
            return spType;
    }
}

/**
 * Gets the human-readable label for a skill activation type.
 */
export function getSkillTypeLabel(skillType: number | string): string {
    if (skillType === 0 || skillType === "PASSIVE") return "Passive";
    if (skillType === 1 || skillType === "MANUAL") return "Manual Trigger";
    if (skillType === 2 || skillType === "AUTO") return "Auto Trigger";
    return "Unknown";
}
