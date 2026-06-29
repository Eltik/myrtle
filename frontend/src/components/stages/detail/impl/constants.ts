import type { IEnemy } from "#/lib/api/enemies";
import type { StageDifficulty, StageType } from "#/types/stages";

export const STAGE_TYPE_LABEL: Record<StageType, string> = {
    MAIN: "Main Theme",
    SUB: "Sub Stage",
    ACTIVITY: "Event",
    DAILY: "Daily / Resource",
    CAMPAIGN: "Campaign",
    CLIMB_TOWER: "Stationary Security",
    GUIDE: "Tutorial",
    SPECIAL_STORY: "Special Story",
};

export const DIFFICULTY_LABEL: Record<StageDifficulty, string> = {
    NORMAL: "Normal",
    FOUR_STAR: "Challenge Mode",
    SIX_STAR: "Extreme",
};

export const ENEMY_LEVEL_ACCENT: Record<IEnemy["enemyLevel"], string> = {
    NORMAL: "var(--muted-foreground)",
    ELITE: "var(--warning)",
    BOSS: "var(--destructive)",
};

export const DROP_TYPE_META: Record<string, { label: string; order: number }> = {
    ONCE: { label: "First Clear", order: 0 },
    COMPLETE: { label: "On Completion", order: 1 },
    NORMAL: { label: "Regular Drops", order: 2 },
    SPECIAL: { label: "Special Drops", order: 3 },
    ADDITIONAL: { label: "Bonus Drops", order: 4 },
    CONDITION_DROP: { label: "Conditional Drops", order: 5 },
};

export const OCC_META: Record<string, { label: string; level: number; tone: string }> = {
    ALWAYS: { label: "Guaranteed", level: 5, tone: "var(--success)" },
    ALMOST: { label: "Common", level: 4, tone: "var(--success)" },
    USUAL: { label: "Uncommon", level: 3, tone: "var(--info)" },
    OFTEN: { label: "Rare", level: 2, tone: "var(--info)" },
    SOMETIMES: { label: "Very Rare", level: 1, tone: "var(--warning)" },
    RARELY: { label: "Very Rare", level: 0, tone: "var(--destructive)" },
};

export const OCC_FALLBACK = { label: "Drops", level: 2, tone: "var(--muted-foreground)" };
