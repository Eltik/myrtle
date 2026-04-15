// Gacha types

// ============================================
// v3 Gacha Record Types (raw DB row — still used by community + settings pages)
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

/** v3 global gacha statistics (camelCase from service layer). Used by community page. */
export interface GachaGlobalStats {
    totalPulls: number;
    totalUsers: number;
    sixStarRate: number;
    fiveStarRate: number;
}

/** v3 gacha fetch result (camelCase from service layer). */
export interface GachaFetchResult {
    totalFetched: number;
    newRecords: number;
}

// ============================================
// Grouped GachaRecords (limited/regular/special) — used by history page
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

// ============================================
// History / Settings
// ============================================

/** Individual pull entry from stored history */
export interface GachaRecordEntry {
    id: string;
    charId: string;
    charName: string;
    rarity: number;
    poolId: string;
    poolName: string;
    gachaType: string;
    pullTimestamp: number;
    pullTimestampStr: string | null;
}

/** Pagination metadata for gacha history */
export interface GachaPaginationInfo {
    limit: number;
    offset: number;
    total: number;
    hasMore: boolean;
}

export interface DateRange {
    from: number | null;
    to: number | null;
}

export interface HistoryFilters {
    rarity: number | null;
    gachaType: string | null;
    charId: string | null;
    dateRange: DateRange | null;
}

export interface GachaHistoryResponse {
    records: GachaRecordEntry[];
    pagination: GachaPaginationInfo;
    filtersApplied: HistoryFilters;
}

export interface GachaHistoryParams {
    limit?: number;
    offset?: number;
    rarity?: number;
    gachaType?: GachaType;
    charId?: string;
    from?: number;
    to?: number;
    order?: "asc" | "desc";
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
