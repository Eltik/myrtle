import { PropertiesSection } from "frontend";

/** `stage_table` rows and their `Options` blocks, straight from the backend. */
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
const OPTIONS_4_10 = {
    characterLimit: 10,
    maxLifePoint: 3,
    initialCost: 10,
    maxCost: 99,
    costIncreaseTime: 1,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
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
const OPTIONS_S4_1 = {
    characterLimit: 8,
    maxLifePoint: 3,
    initialCost: 10,
    maxCost: 99,
    costIncreaseTime: 1,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
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
const OPTIONS_CE6 = {
    characterLimit: 8,
    maxLifePoint: 3,
    initialCost: 50,
    maxCost: 99,
    costIncreaseTime: 3,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
};

export const MainStoryStage = () => (
    <div className="flex max-w-md flex-col gap-6">
        <PropertiesSection stage={STAGE_4_10} level={{ options: OPTIONS_4_10 }} />
    </div>
);

/** A weekly resource run: no challenge variant, so the Challenge ID tile is dropped. */
export const WeeklyResourceRun = () => (
    <div className="flex max-w-md flex-col gap-6">
        <PropertiesSection stage={STAGE_CE6} level={{ options: OPTIONS_CE6 }} />
    </div>
);

export const SubStage = () => (
    <div className="flex max-w-md flex-col gap-6">
        <PropertiesSection stage={STAGE_S4_1} level={{ options: OPTIONS_S4_1 }} />
    </div>
);

/** Without a level file the option-backed flags disappear and only `stage_table` remains. */
export const NoLevelData = () => (
    <div className="flex max-w-md flex-col gap-6">
        <PropertiesSection stage={STAGE_4_10} level={null} />
    </div>
);
