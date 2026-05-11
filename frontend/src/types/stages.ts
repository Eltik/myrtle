export type StageType = "MAIN" | "SUB" | "ACTIVITY" | "DAILY" | "CAMPAIGN" | "CLIMB_TOWER" | "GUIDE" | "SPECIAL_STORY";

export type StageDifficulty = "NORMAL" | "FOUR_STAR" | "SIX_STAR";

export type AppearanceStyle = "MAIN_NORMAL" | "SUB" | "TRAINING" | "SPECIAL_STORY" | "HIGH_DIFFICULTY" | "MAIN_PREDEFINED" | "MIST_OPS";

export interface IUnlockCondition {
    stageId: string;
    completeState: string;
}

export interface IStage {
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
    unlockCondition: IUnlockCondition[];
    loadingPicId?: string;
    bossMark: boolean;
}

export type ZoneType = "MAINLINE" | "SIDESTORY" | "BRANCHLINE" | "ACTIVITY" | "WEEKLY" | "CAMPAIGN" | "CLIMB_TOWER" | "ROGUELIKE" | "GUIDE" | "EVOLVE" | "MAINLINE_ACTIVITY" | "MAINLINE_RETRO" | "SPECIAL";

export interface IZone {
    zoneId: string;
    zoneIndex: number;
    type: ZoneType;
    zoneNameFirst?: string;
    zoneNameSecond?: string;
    zoneNameTitleCurrent?: string;
    zoneNameTitleUnCurrent?: string;
    zoneNameTitleEx?: string;
    zoneNameThird?: string;
    lockedText?: string;
    canPreview: boolean;
    hasAdditionalPanel: boolean;
}

export interface IStageClear {
    state: number;
    completeTimes: number;
    practiceTimes: number;
}

export type StageClearsMap = Record<string, IStageClear>;
