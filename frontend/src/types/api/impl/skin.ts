// Skin types

export interface TokenSkinMapEntry {
    tokenId: string;
    tokenSkinId: string;
}

export interface BattleSkin {
    overwritePrefab: boolean;
    skinOrPrefabId: string;
}

export interface DisplaySkin {
    skinName: string | null;
    colorList: string[];
    titleList: string[];
    modelName: string;
    drawerList: string[];
    designerList: string[] | null;
    skinGroupId: string;
    skinGroupName: string;
    skinGroupSortIndex: number;
    content: string;
    dialog: string | null;
    usage: string | null;
    description: string | null;
    obtainApproach: string | null;
    sortId: number;
    displayTagId: string | null;
    getTime: number;
    onYear: number;
    onPeriod: number;
}

export interface BrandGroup {
    skinGroupId: string;
    publishTime: number;
}

export interface BrandKvImg {
    kvImgId: string;
    linkedSkinGroupId: string;
}

export interface Brand {
    brandId: string;
    groupList: BrandGroup[];
    kvImgIdList: BrandKvImg[];
    brandName: string;
    brandCapitalName: string;
    description: string;
    publishTime: number;
    sortId: number;
}

export interface SpecialSkinInfo {
    skinId: string;
    startTime: number;
    endTime: number;
}

export interface Skin {
    skinId: string;
    charId: string;
    tokenSkinMap: TokenSkinMapEntry[] | null;
    illustId: string;
    dynIllustId: string | null;
    avatarId: string;
    portraitId: string;
    dynPortraitId: string | null;
    dynEntranceId: string | null;
    buildingId: string | null;
    battleSkin: BattleSkin;
    isBuySkin: boolean;
    tmplId: string | null;
    voiceId: string | null;
    voiceType: string;
    displaySkin: DisplaySkin;
}

export interface SkinData {
    charSkins: Record<string, Skin>;
    buildinEvolveMap: Record<string, Record<string, string>>;
    buildinPatchMap: Record<string, Record<string, string>>;
    brandList: Record<string, Brand>;
    specialSkinInfoList: SpecialSkinInfo[];
    enrichedSkins?: Record<string, EnrichedSkin>;
}

export interface SkinImages {
    avatar: string;
    portrait: string;
    skin: string;
}

export interface EnrichedSkin extends Skin {
    id: string;
    images: SkinImages;
}

export interface SkinTableFile {
    charSkins: Record<string, Skin>;
    buildinEvolveMap: Record<string, unknown>;
    buildinPatchMap: Record<string, unknown>;
    brandList: Record<string, Brand>;
    specialSkinInfoList: SpecialSkinInfo[];
}
