// Gacha types

// ============================================
// v3 Gacha Record Types
// ============================================

/** v3 gacha record from database */
export interface GachaRecord {
    id: number;
    user_id: string;
    char_id: string;
    pool_id: string;
    rarity: number;
    pull_timestamp: number;
    pool_name: string | null;
    gacha_type: string | null;
    created_at: string;
}

/** v3 user gacha statistics */
export interface GachaStats {
    user_id: string;
    total_pulls: number | null;
    six_star_count: number | null;
    five_star_count: number | null;
    four_star_count: number | null;
    first_pull: number | null;
    last_pull: number | null;
}

/** v3 global gacha statistics (camelCase from service layer) */
export interface GachaGlobalStats {
    totalPulls: number;
    totalUsers: number;
    sixStarRate: number;
    fiveStarRate: number;
}

/** v3 gacha fetch result (camelCase from service layer) */
export interface GachaFetchResult {
    totalFetched: number;
    newRecords: number;
}

/** v3 gacha history query params */
export interface GachaHistoryParams {
    rarity?: number;
    limit?: number;
    offset?: number;
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

/** Response from /static/gacha/pools endpoint */
export interface GachaPoolsResponse {
    pools: GachaPoolClient[];
    newbeePools: NewbeeGachaPoolClient[];
    total: number;
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
