import { StageHeader } from "frontend";

/** `stage_table` / `zone_table` rows exactly as `GET /stages/{id}/detail` returns them. */
const STAGE_4_10 = {
    stageId: "main_04-10",
    levelId: "Obt/Main/level_main_04-10",
    zoneId: "main_4",
    code: "4-10",
    name: "Extinguished Flames",
    description: "FrostNova and her elite team have the area completely surrounded.\nAll we can do now is to engage and hold them at our perimeter.",
    stageType: "MAIN",
    difficulty: "NORMAL",
    apCost: 21,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "Elite 1 Lv. 70",
    dangerPoint: -1,
    expGain: 210,
    goldGain: 210,
    appearanceStyle: "MAIN_NORMAL",
    hardStagedId: "main_04-10#f#",
    mainStageId: "main_04-10",
    unlockCondition: [{stageId: "main_04-09", completeState: "PASS"}],
    loadingPicId: "loading4",
    bossMark: true,
    stageDropInfo: {
        displayDetailRewards: [
            {dropType: "NORMAL", id: "30063", occPercent: "OFTEN", itemType: "MATERIAL"},
            {dropType: "SPECIAL", id: "30064", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30011", occPercent: "OFTEN", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30061", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30012", occPercent: "OFTEN", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30062", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30013", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30063", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30073", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "31013", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "31073", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "3003", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "COMPLETE", id: "4002", occPercent: "ALWAYS", itemType: "DIAMOND"},
        ],
    },
};
const STAGE_4_10_CM = {
    stageId: "main_04-10#f#",
    levelId: "Obt/Main/level_main_04-10",
    zoneId: "main_4",
    code: "4-10",
    name: "Extinguished Flames",
    description: "<@lv.fs>Condition:</>\nThe ATK and Skill range of FrostNova are increased significantly.",
    stageType: "MAIN",
    difficulty: "FOUR_STAR",
    apCost: 21,
    canPractice: true,
    canBattleReplay: false,
    canMultipleBattle: false,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "-",
    dangerPoint: -1,
    expGain: 210,
    goldGain: 210,
    appearanceStyle: "MAIN_NORMAL",
    hardStagedId: null,
    mainStageId: "main_04-10#f#",
    unlockCondition: [{stageId: "main_02-08", completeState: "PASS"}, {stageId: "main_04-10", completeState: "COMPLETE"}],
    loadingPicId: "loading4",
    bossMark: false,
    stageDropInfo: {
        displayDetailRewards: [
            {dropType: "ONCE", id: "furni_dropS001_trinity_01", occPercent: "ALWAYS", itemType: "FURN"},
            {dropType: "COMPLETE", id: "4002", occPercent: "ALWAYS", itemType: "DIAMOND"},
        ],
    },
};
const STAGE_S4_1 = {
    stageId: "sub_04-1-1",
    levelId: "Obt/Main/level_sub_04-1-1",
    zoneId: "main_4",
    code: "S4-1",
    name: "Cluster-1",
    description: "Make good use of the Active Originium in the middle of the battlefield and things will be much easier.\n<@lv.item><Active Originium></>Operators deployed on it and enemies who have passed it will take constant damage, but their ATK and Attack Speed will be increased significantly",
    stageType: "SUB",
    difficulty: "NORMAL",
    apCost: 18,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "Elite 1 Lv. 35",
    dangerPoint: -1,
    expGain: 180,
    goldGain: 180,
    appearanceStyle: "SUB",
    hardStagedId: null,
    mainStageId: "main_04-03",
    unlockCondition: [{stageId: "main_04-03", completeState: "PASS"}],
    loadingPicId: "loading4",
    bossMark: false,
    stageDropInfo: {
        displayDetailRewards: [
            {dropType: "NORMAL", id: "30043", occPercent: "OFTEN", itemType: "MATERIAL"},
            {dropType: "SPECIAL", id: "30044", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30021", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30041", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "2001", occPercent: "SOMETIMES", itemType: "CARD_EXP"},
            {dropType: "ADDITIONAL", id: "30022", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30042", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "2002", occPercent: "SOMETIMES", itemType: "CARD_EXP"},
            {dropType: "ADDITIONAL", id: "30023", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30043", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "30083", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "31023", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "31063", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "31093", occPercent: "SOMETIMES", itemType: "MATERIAL"},
            {dropType: "ADDITIONAL", id: "2003", occPercent: "SOMETIMES", itemType: "CARD_EXP"},
            {dropType: "COMPLETE", id: "4002", occPercent: "ALWAYS", itemType: "DIAMOND"},
        ],
    },
};
const STAGE_CE6 = {
    stageId: "wk_melee_6",
    levelId: "Obt/Weekly/level_weekly_melee_6",
    zoneId: "weekly_9",
    code: "CE-6",
    name: "Experimental Material Escort",
    description: "Defend against enemies in a flexible way.\n<@lv.rem>The recovery speed of Deployment Points is slow in this operation, but deployment is not limited to melee and ranged cells.</>",
    stageType: "DAILY",
    difficulty: "NORMAL",
    apCost: 36,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "Elite 2 Lv. 20",
    dangerPoint: -1,
    expGain: 360,
    goldGain: 8325,
    appearanceStyle: "MAIN_NORMAL",
    hardStagedId: null,
    mainStageId: null,
    unlockCondition: [{stageId: "wk_melee_5", completeState: "PASS"}],
    loadingPicId: "loading2",
    bossMark: false,
    stageDropInfo: {
        displayDetailRewards: [
            {dropType: "NORMAL", id: "4001", occPercent: "ALWAYS", itemType: "GOLD"},
            {dropType: "COMPLETE", id: "4002", occPercent: "ALWAYS", itemType: "DIAMOND"},
        ],
    },
};
const ZONE_MAIN_4 = {
    zoneId: "main_4",
    zoneIndex: 0,
    type: "MAINLINE",
    zoneNameFirst: "Episode 4",
    zoneNameSecond: "Burning Run",
    zoneNameTitleCurrent: "04",
    zoneNameTitleUnCurrent: "04",
    zoneNameTitleEx: "EPISODE",
    zoneNameThird: "EPISODE 04",
    lockedText: "Clear the previous chapter to unlock",
    canPreview: true,
    hasAdditionalPanel: false,
};
const ZONE_WEEKLY_9 = {zoneId: "weekly_9", zoneIndex: 5, type: "WEEKLY", zoneNameSecond: "Cargo Escort", lockedText: "Currently locked", canPreview: false, hasAdditionalPanel: false};

/** A chapter finale: the boss pill sits beside the stage code. */
export const BossStage = () => <StageHeader stage={STAGE_4_10} zone={ZONE_MAIN_4} />;

/** The Challenge Mode variant adds a warning pill and swaps in the condition text. */
export const ChallengeMode = () => <StageHeader stage={STAGE_4_10_CM} zone={ZONE_MAIN_4} />;

/** Side operations carry the mechanic blurb; `<@lv.item>` markup renders as emphasis. */
export const SubStage = () => <StageHeader stage={STAGE_S4_1} zone={ZONE_MAIN_4} />;

/** A weekly resource run: no episode subtitle, and the stage type reads "Daily / Resource". */
export const WeeklyResourceRun = () => <StageHeader stage={STAGE_CE6} zone={ZONE_WEEKLY_9} />;

/** Without a zone row the breadcrumb falls back to the raw `zoneId`. */
export const NoZone = () => <StageHeader stage={STAGE_4_10} zone={undefined} />;
