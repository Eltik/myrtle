export type Materials = {
    items: Record<string, Item>;
    expItems: Record<
        string,
        {
            id: string;
            gainExp: number;
        }
    >;
    potentialItems: Record<string, Record<string, string>>;
    apSupplies: Record<
        string,
        {
            id: string;
            ap: number;
            hasTs: boolean;
        }
    >;
    charVoucherItems: Record<
        string,
        {
            id: string;
            displayType: VoucherDisplayType;
        }
    >;
};

export type Item = {
    itemId: string;
    name: string;
    description: string;
    rarity: ItemRarity;
    iconId: string;
    overrideBkg: string | null;
    stackIconId: string | null;
    sortId: number;
    usage: string;
    obtainApproach: string | null;
    hideInItemGet: boolean;
    classifyType: ItemClass;
    itemType: ItemType;
    stageDropList: {
        stageId: string;
        occPer: ItemOccPer;
    }[];
    buildingProductList: {
        roomType: BuildingRoomType;
        formulaId: string;
    }[];
    voucherRelateList:
        | {
              voucherId: string;
              voucherItemType: VoucherItemType;
          }[]
        | null;
    uniqueInfo: Record<string, number>;
    itemTimeLimit: Record<string, number>;
    uniCollectionInfo: Record<
        string,
        {
            uniCollectionItemId: string;
            uniqueItem: {
                id: string;
                count: number;
                type: string; // Almost always just "FURN"
            }[];
        }
    >;
    itemPackInfos: Record<
        string,
        {
            packId: string;
            content: {
                id: string;
                count: number;
                type: ItemType;
            }[];
        }
    >;
    fullPotentialCharacters: Record<
        string,
        {
            itemId: string;
            ts: number;
        }
    >;
    activityPotentialCharacters: Record<
        string,
        {
            chardId: string;
        }
    >;
    favorCharacters: Record<
        string,
        {
            itemId: string;
            charId: string;
            favorAddAmt: number;
        }
    >;
};

export enum ItemRarity {
    TIER_1 = 1,
    TIER_2 = 2,
    TIER_3 = 3,
    TIER_4 = 4,
    TIER_5 = 5,
    TIER_6 = 6,
}

export enum ItemClass {
    MATERIAL = "MATERIAL",
    CONSUMABLE = "CONSUME",
    NORMAL = "NORMAL",
    NONE = "NONE",
}

export enum ItemType {
    CARD_EXP = "CARD_EXP",
    MATERIAL = "MATERIAL",
    DIAMOND = "DIAMOND",
    DIAMOND_SHD = "DIAMOND_SHD",
    HGG_SHD = "HGG_SHD",
    LGG_SHD = "LGG_SHD",
    EXP_PLAYER = "EXP_PLAYER",
    PLAYER_AVATAR = "PLAYER_AVATAR",
    TKT_TRY = "TKT_TRY",
    TKTRY_RECRUIT = "TKTRY_RECRUIT",
    TKT_INST_FIN = "TKT_INST_FIN",
    TKT_GACHA = "TKT_GACHA",
    TKT_GACHA_10 = "TKT_GACHA_10",
    SOCIAL_PT = "SOCIAL_PT",
    AP_GAMEPLAY = "AP_GAMEPLAY",
    AP_BASE = "AP_BASE",
    TKT_GACHA_PRSV = "TKT_GACHA_PRSV",
    LMTGS_COIN = "LMTGS_COIN",
    EPGS_COIN = "EPGS_COIN",
    REP_COIN = "REP_COIN",
    CRS_SHOP_COIN = "CRS_SHOP_COIN",
    CRS_SHOP_COIN_V2 = "CRS_SHOP_COIN_V2",
    RETRO_COIN = "RETRO_COIN",
    RENAMING_CARD = "RENAMING_CARD",
    AP_SUPPLY = "AP_SUPPLY",
    EXTERMINATION_AGENT = "EXTERMINATION_AGENT",
    LIMITED_TKT_GACHA_10 = "LIMITED_TKT_GACHA_10",
    LINKAGE_TKT_GACHA_10 = "LINKAGE_TKT_GACHA_10",
    VOUCHER_PICK = "VOUCHER_PICK",
    VOUCHER_LEVELMAX_6 = "VOUCHER_LEVELMAX_6",
    VOUCHER_LEVELMAX_5 = "VOUCHER_LEVELMAX_5",
    VOUCHER_ELITE_II_6 = "VOUCHER_ELITE_II_6",
    VOUCHER_ELITE_II_5 = "VOUCHER_ELITE_II_5",
    VOUCHER_SKIN = "VOUCHER_SKIN",
    VOUCHER_CGACHA = "VOUCHER_CGACHA",
    OPTIONAL_VOUCHER_PICK = "OPTIONAL_VOUCHER_PICK",
    ITEM_PACK = "ITEM_PACK",
    VOUCHER_MGACHA = "VOUCHER_MGACHA",
    VOUCHER_FULL_POTENTIAL = "VOUCHER_FULL_POTENTIAL",
    UNI_COLLECTION = "UNI_COLLECTION",
    AP_ITEM = "AP_ITEM",
    CRS_RUNE_COIN = "CRS_RUNE_COIN",
    ACTIVITY_COIN = "ACTIVITY_COIN",
    ACTIVITY_ITEM = "ACTIVITY_ITEM",
    ET_STAGE = "ET_STAGE",
    RL_COIN = "RL_COIN",
    RETURN_CREDIT = "RETURN_CREDIT",
    MEDAL = "MEDAL",
    ACTIVITY_POTENTIAL = "ACTIVITY_POTENTIAL",
    FAVOR_ADD_ITEM = "FAVOR_ADD_ITEM",
    CLASSIC_SHD = "CLASSIC_SHD",
    CLASSIC_TKT_GACHA = "CLASSIC_TKT_GACHA",
    CLASSIC_TKT_GACHA_10 = "CLASSIC_TKT_GACHA_10",
    LIMITED_BUFF = "LIMITED_BUFF",
    CLASSIC_FES_PICK_TIER_5 = "CLASSIC_FES_PICK_TIER_5",
    CLASSIC_FES_PICK_TIER_6 = "CLASSIC_FES_PICK_TIER_6",
    RETURN_PROGRESS = "RETURN_PROGRESS",
    NEW_PROGRESS = "NEW_PROGRESS",
    MCARD_VOUCHER = "MCARD_VOUCHER",
    MATERIAL_ISSUE_VOUCHER = "MATERIAL_ISSUE_VOUCHER",
    SANDBOX_TOKEN = "SANDBOX_TOKEN",
    EXCLUSIVE_TKT_GACHA = "EXCLUSIVE_TKT_GACHA",
    EXCLUSIVE_TKT_GACHA_10 = "EXCLUSIVE_TKT_GACHA_10",
}

export enum ItemOccPer {
    USUAL = "USUAL",
    ALMOST = "ALMOST",
    ALWAYS = "ALWAYS",
    SOMETIMES = "SOMETIMES",
}

export enum BuildingRoomType {
    WORKSHOP = "WORKSHOP",
    MANUFACTURE = "MANUFACTURE",
}

export enum VoucherItemType {
    OPTIONAL_VOUCHER_PICK = "OPTIONAL_VOUCHER_PICK",
    MATERIAL_ISSUE_VOUCHER = "MATERIAL_ISSUE_VOUCHER",
}

export enum VoucherDisplayType {
    NONE = "NONE",
    DIVIDE = "DIVIDE",
}
