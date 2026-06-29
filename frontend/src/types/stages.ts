export type StageType = "MAIN" | "SUB" | "ACTIVITY" | "DAILY" | "CAMPAIGN" | "CLIMB_TOWER" | "GUIDE" | "SPECIAL_STORY";

export type StageDifficulty = "NORMAL" | "FOUR_STAR" | "SIX_STAR";

export type AppearanceStyle = "MAIN_NORMAL" | "SUB" | "TRAINING" | "SPECIAL_STORY" | "HIGH_DIFFICULTY" | "MAIN_PREDEFINED" | "MIST_OPS";

export interface IUnlockCondition {
    stageId: string;
    completeState: string;
}

/** Drop bucket on the stage results screen. */
export type IStageDropType = "ONCE" | "NORMAL" | "SPECIAL" | "ADDITIONAL" | "COMPLETE" | "CONDITION_DROP" | (string & {});

/** Qualitative drop-rate band the game shows instead of an exact percentage. */
export type IStageDropOcc = "ALWAYS" | "ALMOST" | "USUAL" | "OFTEN" | "SOMETIMES" | "RARELY" | (string & {});

export interface IDisplayDetailReward {
    dropType: IStageDropType;
    id: string;
    occPercent: IStageDropOcc;
    itemType: string;
}

export interface IStageDropInfo {
    displayDetailRewards: IDisplayDetailReward[];
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
    stageDropInfo?: IStageDropInfo;
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

export interface IActivity {
    id: string;
    name: string;
    startTime: number;
    endTime: number;
    hasStage: boolean;
    isReplicate: boolean;
}

export type RetroActType = "SIDESTORY" | "BRANCHLINE" | string;

export interface IRetroAct {
    retroId: string;
    name: string;
    index: number;
    startTime: number;
    type: RetroActType;
    linkedActId: string[];
}
