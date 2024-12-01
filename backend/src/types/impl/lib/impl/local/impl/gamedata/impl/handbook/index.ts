import type { ItemType } from "../materials";

export type Handbook = {
    handbookDict: Record<string, HandbookItem>;
    npcDict: Record<string, HandbookNPCItem>;
    teamMissionList: Record<
        string,
        {
            id: string;
            sort: number;
            powerId: string;
            powerName: string;
            item: {
                id: string;
                count: number;
                type: ItemType;
            };
            favorPoint: number;
        }
    >;
    handbookDisplayConditionList: Record<
        string,
        {
            charId: string;
            conditionCharId: string;
            type: string;
        }
    >;
    handbookStageData: Record<string, HandbookStageData>;
    handbookStageTime: {
        timestamp: number;
        charSet: string[];
    }[];
};

export type HandbookItem = {
    charID: string;
    infoName: string;
    isLimited: boolean;
    storyTextAudio: HandbookStoryTextAudio[];
    handbookAvgList: HandbookAvgList[];
};

export type HandbookStoryTextAudio = {
    stories: {
        storyText: string;
        unlockType: string;
        unLockParam: string;
        unLockString: string;
        patchIdList: string[] | null;
    }[];
    storyTitle: string;
    unLockorNot: boolean;
};

export type HandbookAvgList = {
    storySetId: string;
    storySetName: string;
    sortId: number;
    storyGetTime: number;
    rewardItem: {
        id: string;
        count: number;
        type: ItemType;
    }[];
    unlockParam: {
        unlockType: string;
        unlockParam1: string | null;
        unlockParam2: string | null;
        unlockParam3: string | null;
    }[];
    avgList: {
        storyId: string;
        storySetId: string;
        storySort: number;
        storyCanShow: boolean;
        storyIntro: string;
        storyInfo: string;
        storyTxt: string;
    }[];
    charId: string;
};

export type HandbookNPCItem = {
    npcId: string;
    name: string;
    appellation: string;
    profession: string;
    illustList: string[] | null;
    designerList: string[] | null;
    cv: string;
    displayNumber: string;
    nationId: string | null;
    groupId: string | null;
    teamId: string | null;
    resType: string;
    npcShowAudioInfoFlag: boolean;
    unlockDict: Record<
        string,
        {
            unLockType: string;
            unLockParam: string;
            unLockString: string | null;
        }
    >;
};

export type HandbookStageData = {
    charId: string;
    stageId: string;
    levelId: string;
    zoneId: string;
    code: string;
    name: string;
    loadingPicId: string;
    description: string;
    unlockParam: {
        unlockType: string;
        unlockParam1: string | null;
        unlockParam2: string | null;
        unlockParam3: string | null;
    }[];
    rewardItem: {
        id: string;
        count: number;
        type: ItemType;
    }[];
    stageNameForShow: string;
    zoneNameForShow: string;
    picId: string;
    stageGetTime: number;
};
