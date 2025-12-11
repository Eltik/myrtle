export type SkinData = {
    charSkins: Record<string, Skin>;
    buildinEvolveMap: Record<string, Record<string, string>>;
    buildinPatchMap: Record<string, Record<string, string>>; // Amiya is a special child
    brandList: Record<string, Brand>;
    specialSkinInfoList: {
        skinId: string;
        startTime: number;
        endTime: number;
    }[];
};

export type Skin = {
    skinId: string;
    charId: string;
    tokenSkinMap:
        | {
              tokenId: string;
              tokenSkinId: string;
          }[]
        | null;
    illustId: string;
    dynIllustId: string | null;
    avatarId: string;
    portraitId: string;
    dynPortraitId: string | null;
    dynEntranceId: string | null;
    buildingId: string | null;
    battleSkin: {
        overwritePrefab: boolean;
        skinOrPrefabId: string;
    };
    isBuySkin: boolean;
    tmplId: string | null;
    voiceId: string | null;
    voiceType: string;
    displaySkin: {
        skinName: string;
        colorList: string[];
        titleList: string[];
        modelName: string;
        drawerList: string[];
        designerList: string[] | null;
        skinGroupId: string;
        skinGroupName: string;
        skinGroupSortIndex: number;
        content: string;
        dialog: string;
        usage: string;
        description: string;
        obtainApproach: string;
        sortId: number;
        displayTagId: string | null;
        getTime: number;
        onYear: number;
        onPeriod: number;
    };
    images: {
        avatar: string;
        portrait: string;
        skin: string;
    };
};

export type Brand = {
    brandId: string;
    groupList: {
        skinGroupId: string;
        publishTime: number;
    }[];
    kvImgIdList: {
        kvImgId: string;
        linkedSkinGroupId: string;
    }[];
    brandName: string;
    brandCapitalName: string;
    description: string;
    publishTime: number;
    sortId: number;
};
