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
// Gacha History Types (User Pull History)
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

/** Date range filter */
export interface DateRange {
    from: number | null;
    to: number | null;
}

/** Filters applied to history query */
export interface HistoryFilters {
    rarity: number | null;
    gachaType: string | null;
    charId: string | null;
    dateRange: DateRange | null;
}

/** Response for user pull history endpoint */
export interface GachaHistoryResponse {
    records: GachaRecordEntry[];
    pagination: GachaPaginationInfo;
    filtersApplied: HistoryFilters;
}

/** Query parameters for history endpoint */
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

// ============================================
// Enhanced Statistics Types
// ============================================

/** Collective statistics for all consenting users */
export interface CollectiveStats {
    totalPulls: number;
    totalUsers: number;
    totalSixStars: number;
    totalFiveStars: number;
    totalFourStars: number;
    totalThreeStars: number;
}

/** Pull rate percentages */
export interface PullRates {
    sixStarRate: number;
    fiveStarRate: number;
}

/** Operator popularity statistics */
export interface OperatorPopularity {
    charId: string;
    charName: string;
    rarity: number;
    pullCount: number;
    percentage: number;
}

/** Hourly pull distribution */
export interface HourlyPullData {
    hour: number;
    pullCount: number;
    percentage: number;
}

/** Day of week pull distribution */
export interface DayOfWeekPullData {
    day: number;
    dayName: string;
    pullCount: number;
    percentage: number;
}

/** Pull history by date (time series) */
export interface DatePullData {
    date: string;
    pullCount: number;
}

/** Pull timing data for graphs */
export interface PullTimingData {
    byHour: HourlyPullData[];
    byDayOfWeek: DayOfWeekPullData[];
    byDate?: DatePullData[];
}

/** Full enhanced statistics response */
export interface GachaEnhancedStats {
    collectiveStats: CollectiveStats;
    pullRates: PullRates;
    mostCommonOperators: OperatorPopularity[];
    averagePullsToSixStar: number;
    averagePullsToFiveStar: number;
    pullTiming?: PullTimingData;
    computedAt: string;
    cached: boolean;
}

/** Query parameters for enhanced stats endpoint */
export interface GachaEnhancedStatsParams {
    topN?: number;
    includeTiming?: boolean;
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
