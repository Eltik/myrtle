import type { IEnemy } from "#/lib/api/enemies";
import type { StageDifficulty, StageType } from "#/types/stages";
import type { messages as detailConstantsMessages } from "./constants.messages";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type StageDetailMessageKey = keyof typeof detailConstantsMessages & string;

export const STAGE_TYPE_LABEL_KEY: Record<StageType, StageDetailMessageKey> = {
    MAIN: "stageType.MAIN",
    SUB: "stageType.SUB",
    ACTIVITY: "stageType.ACTIVITY",
    DAILY: "stageType.DAILY",
    CAMPAIGN: "stageType.CAMPAIGN",
    CLIMB_TOWER: "stageType.CLIMB_TOWER",
    GUIDE: "stageType.GUIDE",
    SPECIAL_STORY: "stageType.SPECIAL_STORY",
    // `#[serde(other)]` catch-all: game data can ship a stageType we don't name.
    UNKNOWN: "stageType.UNKNOWN",
};

export const DIFFICULTY_LABEL_KEY: Record<StageDifficulty, StageDetailMessageKey> = {
    NORMAL: "difficulty.NORMAL",
    FOUR_STAR: "difficulty.FOUR_STAR",
    SIX_STAR: "difficulty.SIX_STAR",
    UNKNOWN: "difficulty.UNKNOWN",
};

export const ENEMY_LEVEL_ACCENT: Record<IEnemy["enemyLevel"], string> = {
    NORMAL: "var(--muted-foreground)",
    ELITE: "var(--warning)",
    BOSS: "var(--destructive)",
};

export const DROP_TYPE_META: Record<string, { labelKey: StageDetailMessageKey; order: number }> = {
    ONCE: { labelKey: "dropType.ONCE", order: 0 },
    COMPLETE: { labelKey: "dropType.COMPLETE", order: 1 },
    NORMAL: { labelKey: "dropType.NORMAL", order: 2 },
    SPECIAL: { labelKey: "dropType.SPECIAL", order: 3 },
    ADDITIONAL: { labelKey: "dropType.ADDITIONAL", order: 4 },
    CONDITION_DROP: { labelKey: "dropType.CONDITION_DROP", order: 5 },
};

export const OCC_META: Record<string, { labelKey: StageDetailMessageKey; level: number; tone: string }> = {
    ALWAYS: { labelKey: "occ.ALWAYS", level: 5, tone: "var(--success)" },
    ALMOST: { labelKey: "occ.ALMOST", level: 4, tone: "var(--success)" },
    USUAL: { labelKey: "occ.USUAL", level: 3, tone: "var(--info)" },
    OFTEN: { labelKey: "occ.OFTEN", level: 2, tone: "var(--info)" },
    SOMETIMES: { labelKey: "occ.SOMETIMES", level: 1, tone: "var(--warning)" },
    RARELY: { labelKey: "occ.RARELY", level: 0, tone: "var(--destructive)" },
};

export const OCC_FALLBACK: { labelKey: StageDetailMessageKey; level: number; tone: string } = { labelKey: "occ.FALLBACK", level: 2, tone: "var(--muted-foreground)" };
