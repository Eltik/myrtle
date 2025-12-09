// Material types

export type ItemRarity = "TIER_1" | "TIER_2" | "TIER_3" | "TIER_4" | "TIER_5" | "TIER_6";

export type ItemClass = "MATERIAL" | "CONSUME" | "NORMAL" | "NONE";

export type ItemType =
    | "GOLD"
    | "CARD_EXP"
    | "MATERIAL"
    | "DIAMOND"
    | "DIAMOND_SHD"
    | "HGG_SHD"
    | "LGG_SHD"
    | "EXP_PLAYER"
    | "PLAYER_AVATAR"
    | "TKT_TRY"
    | "TKTRY_RECRUIT"
    | "TKT_INST_FIN"
    | "TKT_GACHA"
    | "TKT_GACHA_10"
    | "SOCIAL_PT"
    | "AP_GAMEPLAY"
    | "AP_BASE"
    | "TKT_GACHA_PRSV"
    | "LMTGS_COIN"
    | "EPGS_COIN"
    | "REP_COIN"
    | "CRS_SHOP_COIN"
    | "CRS_SHOP_COIN_V2"
    | "RETRO_COIN"
    | "RENAMING_CARD"
    | "AP_SUPPLY"
    | "EXTERMINATION_AGENT"
    | "LIMITED_TKT_GACHA_10"
    | "LINKAGE_TKT_GACHA_10"
    | "VOUCHER_PICK"
    | "VOUCHER_LEVELMAX_6"
    | "VOUCHER_LEVELMAX_5"
    | "VOUCHER_ELITE_II_6"
    | "VOUCHER_ELITE_II_5"
    | "VOUCHER_SKIN"
    | "VOUCHER_CGACHA"
    | "OPTIONAL_VOUCHER_PICK"
    | "ITEM_PACK"
    | "VOUCHER_MGACHA"
    | "VOUCHER_FULL_POTENTIAL"
    | "UNI_COLLECTION"
    | "AP_ITEM"
    | "CRS_RUNE_COIN"
    | "ACTIVITY_COIN"
    | "ACTIVITY_ITEM"
    | "ET_STAGE"
    | "RL_COIN"
    | "RETURN_CREDIT"
    | "MEDAL"
    | "ACTIVITY_POTENTIAL"
    | "FAVOR_ADD_ITEM"
    | "CLASSIC_SHD"
    | "CLASSIC_TKT_GACHA"
    | "CLASSIC_TKT_GACHA_10"
    | "LIMITED_BUFF"
    | "CLASSIC_FES_PICK_TIER_5"
    | "CLASSIC_FES_PICK_TIER_6"
    | "RETURN_PROGRESS"
    | "NEW_PROGRESS"
    | "MCARD_VOUCHER"
    | "MATERIAL_ISSUE_VOUCHER"
    | "SANDBOX_TOKEN"
    | "EXCLUSIVE_TKT_GACHA"
    | "EXCLUSIVE_TKT_GACHA_10"
    | string; // For unknown types

export type ItemOccPer = "USUAL" | "ALMOST" | "ALWAYS" | "SOMETIMES" | "OFTEN";

export type BuildingRoomType = "WORKSHOP" | "MANUFACTURE";

export type VoucherItemType = "OPTIONAL_VOUCHER_PICK" | "MATERIAL_ISSUE_VOUCHER";

export type VoucherDisplayType = "NONE" | "DIVIDE";

export interface StageDrop {
    stageId: string;
    occPer: ItemOccPer;
}

export interface BuildingProduct {
    roomType: BuildingRoomType;
    formulaId: string;
}

export interface VoucherRelate {
    voucherId: string;
    voucherItemType: VoucherItemType;
}

export interface UniqueItem {
    id: string;
    count: number;
    type: string;
}

export interface UniCollectionInfo {
    uniCollectionItemId: string;
    uniqueItem: UniqueItem[];
}

export interface ItemPackContent {
    id: string;
    count: number;
    type: ItemType;
}

export interface ItemPackInfo {
    packId: string;
    content: ItemPackContent[];
}

export interface FullPotentialCharacter {
    itemId: string;
    ts: number;
}

export interface ActivityPotentialCharacter {
    charId: string;
}

export interface FavorCharacter {
    itemId: string;
    charId: string;
    favorAddAmt: number;
}

export interface ExpItem {
    id: string;
    gainExp: number;
}

export interface ApSupply {
    id: string;
    ap: number;
    hasTs: boolean;
}

export interface CharVoucherItem {
    id: string;
    displayType: VoucherDisplayType;
}

export interface Item {
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
    stageDropList: StageDrop[];
    buildingProductList: BuildingProduct[];
    voucherRelateList: VoucherRelate[] | null;
}

export interface Materials {
    items: Record<string, Item>;
    expItems: Record<string, ExpItem>;
    potentialItems: Record<string, Record<string, string>>;
    apSupplies: Record<string, ApSupply>;
    charVoucherItems: Record<string, CharVoucherItem>;
}

export interface ItemTableFile {
    items: Record<string, Item>;
    expItems: Record<string, ExpItem>;
    potentialItems: unknown[];
    apSupplyOutOfDateDict: Record<string, ApSupply>;
    charVoucherItems: Record<string, CharVoucherItem>;
}
