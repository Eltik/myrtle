// Gacha types

// ============================================
// Gacha Record Types (Pull History)
// ============================================

export type GachaType = "limited" | "regular" | "special";

export interface GachaItem {
    charId: string;
    charName: string;
    star: string;
    color: string;
    poolId: string;
    poolName: string;
    typeName: string;
    at: number;
    atStr: string;
}

export interface GachaTypeRecords {
    gacha_type: GachaType;
    records: GachaItem[];
    total: number;
}

export interface GachaRecords {
    limited: GachaTypeRecords;
    regular: GachaTypeRecords;
    special: GachaTypeRecords;
}

export interface GachaSettings {
    user_id: string;
    store_records: boolean;
    share_anonymous_stats: boolean;
    total_pulls: number;
    six_star_count: number;
    five_star_count: number;
    last_sync_at: string | null;
}

export interface GachaGlobalStats {
    total_pulls: number;
    six_star_count: number;
    five_star_count: number;
    six_star_rate: number;
    five_star_rate: number;
    contributor_count: number;
}

// ============================================
// Gacha Pool Types (Game Client Data)
// ============================================

export interface GachaPoolClient {
    gachaPoolId: string;
    gachaIndex: number;
    openTime: number;
    endTime: number;
    gachaPoolName: string;
    gachaPoolSummary: string;
    gachaPoolDetail: string | null;
    guarantee5Avail: number;
    guarantee5Count: number;
    gachaRuleType: string;
    lmtgsid: string | null;
    cdPrimColor: string | null;
    cdSecColor: string | null;
    limitParam: unknown | null;
    linkageParam: unknown | null;
    linkageRuleId: string | null;
    dynMeta: unknown | null;
    freeBackColor: string | null;
    guaranteeName: string | null;
}

export interface NewbeeGachaPoolClient {
    gachaPoolId: string;
    gachaIndex: number;
    gachaPoolName: string;
    gachaPoolDetail: string | null;
    gachaPrice: number;
    gachaTimes: number;
    gachaOffset: string | null;
}

export interface GachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

export interface RecruitTimeEntry {
    recruitPrice: number;
}

export interface RecruitPool {
    recruitTimeTable: RecruitTimeEntry[];
}

export interface RecruitRarityEntry {
    rarityStart: number;
    rarityEnd: number;
}

export interface FesGachaPoolRelateEntry {
    rarityRank5ItemId: string;
    rarityRank6ItemId: string;
}

export interface GachaData {
    gachaPoolClient: GachaPoolClient[];
    newbeeGachaPoolClient: NewbeeGachaPoolClient[];
    specialRecruitPool: unknown[];
    gachaTags: GachaTag[];
    recruitPool: RecruitPool;
    potentialMaterialConverter: unknown;
    classicPotentialMaterialConverter: unknown;
    recruitRarityTable: Record<number, RecruitRarityEntry>;
    specialTagRarityTable: Record<number, number[]>;
    recruitDetail: string;
    showGachaLogEntry: boolean;
    carousel: unknown[];
    freeGacha: unknown[];
    limitTenGachaItem: unknown[];
    linkageTenGachaItem: unknown[];
    normalGachaItem: unknown[];
    fesGachaPoolRelateItem: Record<string, FesGachaPoolRelateEntry>;
    dicRecruit6StarHint: Record<string, string>;
    specialGachaPercentDict: Record<number, number>;
}

export interface GachaTableFile {
    gachaPoolClient: GachaPoolClient[];
    newbeeGachaPoolClient: NewbeeGachaPoolClient[];
    specialRecruitPool: unknown[];
    gachaTags: GachaTag[];
    recruitPool: RecruitPool;
    potentialMaterialConverter: unknown;
    classicPotentialMaterialConverter: unknown;
    recruitRarityTable: Record<number, RecruitRarityEntry>;
    specialTagRarityTable: Record<number, number[]>;
    recruitDetail: string;
    showGachaLogEntry: boolean;
    carousel: unknown[];
    freeGacha: unknown[];
    limitTenGachaItem: unknown[];
    linkageTenGachaItem: unknown[];
    normalGachaItem: unknown[];
    fesGachaPoolRelateItem: Record<string, FesGachaPoolRelateEntry>;
    dicRecruit6StarHint: Record<string, string>;
    specialGachaPercentDict: Record<number, number>;
}
