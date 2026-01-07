// Stage types for stage randomization

export type StageType = "MAIN" | "SUB" | "ACTIVITY" | "DAILY" | "CAMPAIGN" | "CLIMB_TOWER" | "GUIDE" | "SPECIAL_STORY";

export type StageDifficulty = "NORMAL" | "FOUR_STAR" | "SIX_STAR";

export type AppearanceStyle = "MAIN_NORMAL" | "SUB" | "TRAINING" | "SPECIAL_STORY" | "HIGH_DIFFICULTY" | "MAIN_PREDEFINED" | "MIST_OPS";

export interface UnlockCondition {
    stageId: string;
    completeState: string;
}

export interface Stage {
    stageId: string;
    levelId?: string;
    zoneId: string;
    code: string;
    name?: string;
    description?: string;
    stageType: StageType;
    difficulty: StageDifficulty;
    apCost: number;
    canPractice: boolean;
    canBattleReplay: boolean;
    canMultipleBattle: boolean;
    isStoryOnly: boolean;
    isPredefined: boolean;
    dangerLevel?: string;
    dangerPoint: number;
    expGain: number;
    goldGain: number;
    appearanceStyle?: AppearanceStyle;
    hardStagedId?: string;
    mainStageId?: string;
    unlockCondition: UnlockCondition[];
    loadingPicId?: string;
    bossMark: boolean;
}

export interface StagesResponse {
    stages: Stage[];
    nextCursor: string | null;
    hasMore: boolean;
    total: number;
}

export interface StageResponse {
    stage: Stage;
}
