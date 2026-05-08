import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";

/** Lightweight community summary. Rates are fractions in [0, 1]. */
export interface IGachaGlobalStats {
    totalPulls: number;
    totalUsers: number;
    sixStarRate: number;
    fiveStarRate: number;
}

export interface IGachaCollectiveStats {
    totalPulls: number;
    totalUsers: number;
    totalSixStars: number;
    totalFiveStars: number;
    totalFourStars: number;
    totalThreeStars: number;
    /** Unix seconds. Omitted by the server when the corpus is empty. */
    firstPullAt?: number;
}

export interface IGachaPullRates {
    sixStarRate: number;
    fiveStarRate: number;
}

export interface IOperatorPopularity {
    charId: string;
    /** Server returns an empty string; resolve via the operator index client-side. */
    charName: string;
    rarity: number;
    pullCount: number;
    /** Fraction of all community pulls in [0, 1]. */
    percentage: number;
}

export interface IHourlyPullData {
    hour: number;
    pullCount: number;
    percentage: number;
}

export interface IDayOfWeekPullData {
    /** 0 = Sunday … 6 = Saturday (Postgres DOW). */
    day: number;
    dayName: string;
    pullCount: number;
    percentage: number;
}

export interface IDatePullData {
    /** YYYY-MM-DD. */
    date: string;
    pullCount: number;
}

export interface IPullTimingData {
    byHour: IHourlyPullData[];
    byDayOfWeek: IDayOfWeekPullData[];
    byDate?: IDatePullData[];
}

export interface IGachaEnhancedStats {
    collectiveStats: IGachaCollectiveStats;
    pullRates: IGachaPullRates;
    /** Top-N is computed *per rarity* server-side (3-stars dominate the global tally otherwise). */
    mostCommonOperators: IOperatorPopularity[];
    averagePullsToSixStar: number;
    averagePullsToFiveStar: number;
    pullTiming?: PullTimingData | null;
    /** RFC3339 UTC timestamp. */
    computedAt: string;
    /** True when this payload was served from the backend cache. */
    cached: boolean;
}

// Alias kept for the optional field above (name matches the server type).
export type PullTimingData = IPullTimingData;

export const getGachaGlobalStatsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/gacha/global-stats");
    if (!res.ok) throw new Error(`Failed to load gacha global stats: ${res.status}`);
    return (await res.json()) as IGachaGlobalStats;
});

export function gachaGlobalStatsQueryOptions() {
    return queryOptions({
        queryKey: ["gacha", "global-stats"],
        queryFn: () => getGachaGlobalStatsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export interface IGachaEnhancedStatsInput {
    /** Operators returned per rarity (1-50, default 20). */
    topN?: number;
    /** Include hourly / day-of-week / per-date breakdowns. */
    includeTiming?: boolean;
}

export const getGachaEnhancedStatsFn = createServerFn({ method: "GET" })
    .inputValidator((data: IGachaEnhancedStatsInput) => data)
    .handler(async ({ data: { topN, includeTiming } }) => {
        const params = new URLSearchParams();
        if (topN !== undefined) params.set("topN", String(topN));
        if (includeTiming !== undefined) params.set("includeTiming", String(includeTiming));

        const qs = params.toString();
        const res = await backendFetch(`/gacha/stats/enhanced${qs ? `?${qs}` : ""}`);
        if (!res.ok) throw new Error(`Failed to load gacha enhanced stats: ${res.status}`);
        return (await res.json()) as IGachaEnhancedStats;
    });

export function gachaEnhancedStatsQueryOptions(input: IGachaEnhancedStatsInput = {}) {
    return queryOptions({
        queryKey: ["gacha", "enhanced-stats", input.topN ?? null, input.includeTiming ?? null],
        queryFn: () => getGachaEnhancedStatsFn({ data: input }),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

/** Per-user stats from the v_gacha_stats view. snake_case from the backend model. */
export interface IGachaStats {
    user_id: string;
    total_pulls: number | null;
    six_star_count: number | null;
    five_star_count: number | null;
    four_star_count: number | null;
    /** Unix milliseconds (sourced from Yostar's `at` field, which is ms). */
    first_pull: number | null;
    /** Unix milliseconds (sourced from Yostar's `at` field, which is ms). */
    last_pull: number | null;
}

export interface IGachaRecordEntry {
    id: string;
    charId: string;
    /** Empty from the server; resolve via the operator index. */
    charName: string;
    rarity: number;
    poolId: string;
    poolName: string;
    gachaType: string;
    /** Unix milliseconds (Yostar's `at` field, persisted as-is). Multiply by `* 1` (no-op) for `new Date(...)`. */
    pullTimestamp: number;
    pullTimestampStr: string | null;
}

export interface IGachaItem {
    charId: string;
    charName: string;
    star: string;
    color: string;
    poolId: string;
    poolName: string;
    typeName: string;
    /** Unix milliseconds (Yostar's `at` field, persisted as-is). Pass directly to `new Date(...)`. */
    at: number;
    atStr: string;
}

export type GachaGroup = "limited" | "regular" | "special";

export interface IGachaTypeRecords {
    gacha_type: GachaGroup;
    records: IGachaItem[];
    total: number;
}

export interface IGachaRecords {
    limited: IGachaTypeRecords;
    regular: IGachaTypeRecords;
    special: IGachaTypeRecords;
}

/**
 * Client-side 4-bucket grouping derived from `IGachaItem.typeName`.
 *
 * Backend merges `limited`+`linkage` into the wire-level "limited" bucket and
 * `single`+`boot` into "special", but they have meaningfully different pity rules
 * and rate-up semantics (e.g. linkage has a 120-pull hard guarantee), so the UI
 * separates them.
 */
export type ClientGachaGroup = "limited" | "linkage" | "regular" | "special";

export interface IClientGachaTypeRecords {
    gacha_type: ClientGachaGroup;
    records: IGachaItem[];
    total: number;
}

export interface IClientGachaRecords {
    limited: IClientGachaTypeRecords;
    linkage: IClientGachaTypeRecords;
    regular: IClientGachaTypeRecords;
    special: IClientGachaTypeRecords;
}

/**
 * Classify a record by its `pool_id` prefix (the same logic the backend uses
 * during ingestion). `typeName` is consulted only as a fallback for older rows
 * whose pool_id may not match a known prefix.
 */
export function classifyClientGachaGroup(item: { poolId: string; typeName: string }): ClientGachaGroup {
    const id = item.poolId ?? "";
    if (id.startsWith("LIMITED_")) return "limited";
    if (id.startsWith("LINKAGE_")) return "linkage";
    // FESCLASSIC_ is the festival/anniversary kernel banner — same pool as CLASSIC_/BOOT_.
    if (id.startsWith("FESCLASSIC_") || id.startsWith("CLASSIC_") || id.startsWith("BOOT_")) return "special";
    if (id.startsWith("SINGLE_") || id.startsWith("NORM_")) return "regular";

    switch (item.typeName) {
        case "limited":
            return "limited";
        case "linkage":
            return "linkage";
        case "classic":
        case "boot":
            return "special";
        default:
            // "single" (debut/rerun rate-up) and "normal" (standard headhunting) both belong here.
            return "regular";
    }
}

export function deriveClientGachaRecords(records: IGachaRecords): IClientGachaRecords {
    const buckets: Record<ClientGachaGroup, IGachaItem[]> = {
        limited: [],
        linkage: [],
        regular: [],
        special: [],
    };

    for (const item of [...records.limited.records, ...records.regular.records, ...records.special.records]) {
        buckets[classifyClientGachaGroup(item)].push(item);
    }

    return {
        limited: { gacha_type: "limited", records: buckets.limited, total: buckets.limited.length },
        linkage: { gacha_type: "linkage", records: buckets.linkage, total: buckets.linkage.length },
        regular: { gacha_type: "regular", records: buckets.regular, total: buckets.regular.length },
        special: { gacha_type: "special", records: buckets.special, total: buckets.special.length },
    };
}

export interface IGachaPaginationInfo {
    limit: number;
    offset: number;
    total: number;
    hasMore: boolean;
}

export interface IGachaHistoryFiltersApplied {
    rarity: number | null;
    gachaType: string | null;
    charId: string | null;
    dateRange: { from: number | null; to: number | null } | null;
}

export interface IGachaHistoryEnvelope {
    records: IGachaRecordEntry[];
    pagination: IGachaPaginationInfo;
    filtersApplied: IGachaHistoryFiltersApplied;
}

export interface IGachaHistoryInput {
    bearerToken?: string;
    rarity?: number;
    gachaType?: string;
    charId?: string;
    /** Unix milliseconds, inclusive. Matched directly against `gacha_records.pull_timestamp` (also ms). */
    from?: number;
    /** Unix milliseconds, inclusive. Matched directly against `gacha_records.pull_timestamp` (also ms). */
    to?: number;
    order?: "asc" | "desc";
    limit?: number;
    offset?: number;
}

export const getGachaHistoryFn = createServerFn({ method: "GET" })
    .inputValidator((data: IGachaHistoryInput) => data)
    .handler(async ({ data }) => {
        const { bearerToken, ...rest } = data;
        const params = new URLSearchParams();
        if (rest.rarity !== undefined) params.set("rarity", String(rest.rarity));
        if (rest.gachaType) params.set("gachaType", rest.gachaType);
        if (rest.charId) params.set("charId", rest.charId);
        if (rest.from !== undefined) params.set("from", String(rest.from));
        if (rest.to !== undefined) params.set("to", String(rest.to));
        if (rest.order) params.set("order", rest.order);
        if (rest.limit !== undefined) params.set("limit", String(rest.limit));
        if (rest.offset !== undefined) params.set("offset", String(rest.offset));

        const qs = params.toString();
        const res = await backendFetch(`/gacha/history${qs ? `?${qs}` : ""}`, { bearerToken });
        if (!res.ok) throw new Error(`Failed to load gacha history: ${res.status}`);
        return (await res.json()) as IGachaHistoryEnvelope;
    });

export function gachaHistoryQueryOptions(input: IGachaHistoryInput) {
    return queryOptions({
        queryKey: ["gacha", "history", input.rarity ?? null, input.gachaType ?? null, input.charId ?? null, input.from ?? null, input.to ?? null, input.order ?? null, input.limit ?? null, input.offset ?? null, input.bearerToken ? "auth" : "anon"],
        queryFn: () => getGachaHistoryFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getGachaHistoryByCharFn = createServerFn({ method: "GET" })
    .inputValidator((data: { charId: string; bearerToken?: string }) => data)
    .handler(async ({ data: { charId, bearerToken } }) => {
        const res = await backendFetch(`/gacha/history/${encodeURIComponent(charId)}`, { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return [];
            throw new Error(`Failed to load gacha history for ${charId}: ${res.status}`);
        }
        return (await res.json()) as IGachaRecordEntry[];
    });

export function gachaHistoryByCharQueryOptions(charId: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["gacha", "history", "by-char", charId, bearerToken ? "auth" : "anon"],
        queryFn: () => getGachaHistoryByCharFn({ data: { charId, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getGachaStoredRecordsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { bearerToken?: string }) => data)
    .handler(async ({ data: { bearerToken } }) => {
        const res = await backendFetch("/gacha/stored-records", { bearerToken });
        if (!res.ok) throw new Error(`Failed to load stored gacha records: ${res.status}`);
        return (await res.json()) as IGachaRecords;
    });

export function gachaStoredRecordsQueryOptions(bearerToken?: string) {
    return queryOptions({
        queryKey: ["gacha", "stored-records", bearerToken ? "auth" : "anon"],
        queryFn: () => getGachaStoredRecordsFn({ data: { bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

/** Server fn variant that pulls the auth cookie automatically. Returns null when unauthenticated. */
export const getMyGachaStoredRecordsFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = getCookie("site_token");
    if (!token) return null;
    const res = await backendFetch("/gacha/stored-records", { bearerToken: token });
    if (!res.ok) {
        if (res.status === 401) return null;
        throw new Error(`Failed to load stored gacha records: ${res.status}`);
    }
    return (await res.json()) as IGachaRecords;
});

export function myGachaStoredRecordsQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["gacha", "my-stored-records", authed ? "auth" : "anon"],
        queryFn: () => (authed ? getMyGachaStoredRecordsFn() : Promise.resolve(null)),
        enabled: authed,
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getGachaStatsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { bearerToken?: string }) => data)
    .handler(async ({ data: { bearerToken } }) => {
        const res = await backendFetch("/gacha/stats", { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load gacha stats: ${res.status}`);
        }
        return (await res.json()) as IGachaStats;
    });

export function gachaStatsQueryOptions(bearerToken?: string) {
    return queryOptions({
        queryKey: ["gacha", "stats", bearerToken ? "auth" : "anon"],
        queryFn: () => getGachaStatsFn({ data: { bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

/** Server fn variant that pulls the auth cookie automatically. Returns null when unauthenticated. */
export const getMyGachaStatsFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = getCookie("site_token");
    if (!token) return null;
    const res = await backendFetch("/gacha/stats", { bearerToken: token });
    if (!res.ok) {
        if (res.status === 404 || res.status === 401) return null;
        throw new Error(`Failed to load gacha stats: ${res.status}`);
    }
    return (await res.json()) as IGachaStats;
});

export function myGachaStatsQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["gacha", "my-stats", authed ? "auth" : "anon"],
        queryFn: () => (authed ? getMyGachaStatsFn() : Promise.resolve(null)),
        enabled: authed,
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface IGachaSettings {
    user_id: string;
    store_records: boolean;
    share_anonymous_stats: boolean;
    total_pulls: number;
    six_star_count: number;
    five_star_count: number;
    last_sync_at: string | null;
}

export const getGachaSettingsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { bearerToken?: string }) => data)
    .handler(async ({ data: { bearerToken } }) => {
        const res = await backendFetch("/gacha/settings", { bearerToken });
        if (!res.ok) throw new Error(`Failed to load gacha settings: ${res.status}`);
        return (await res.json()) as IGachaSettings;
    });

export function gachaSettingsQueryOptions(bearerToken?: string) {
    return queryOptions({
        queryKey: ["gacha", "settings", bearerToken ? "auth" : "anon"],
        queryFn: () => getGachaSettingsFn({ data: { bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface IUpdateGachaSettingsInput {
    bearerToken?: string;
    store_records?: boolean;
    share_anonymous_stats?: boolean;
}

export const updateGachaSettingsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpdateGachaSettingsInput) => data)
    .handler(async ({ data }) => {
        const { bearerToken, ...body } = data;
        const res = await backendFetch("/gacha/settings", {
            method: "POST",
            bearerToken,
            body: JSON.stringify(body),
        });
        if (!res.ok) throw new Error(`Failed to update gacha settings: ${res.status}`);
        return (await res.json()) as IGachaSettings;
    });

export interface IGachaFetchResult {
    totalFetched: number;
    newRecords: number;
}

export const fetchGachaRecordsFn = createServerFn({ method: "POST" })
    .inputValidator((data: { bearerToken?: string }) => data)
    .handler(async ({ data: { bearerToken } }) => {
        const res = await backendFetch("/gacha/fetch", {
            method: "POST",
            bearerToken,
            body: "{}",
        });
        if (!res.ok) throw new Error(`Failed to fetch gacha records: ${res.status}`);
        return (await res.json()) as IGachaFetchResult;
    });

/**
 * Cookie-driven variant. Triggers `/gacha/fetch` (POST) which pulls the latest
 * pulls from Yostar and persists any new ones. Requires a live portal session
 * cached server-side from a recent login.
 */
export const fetchMyGachaRecordsFn = createServerFn({ method: "POST" }).handler(async () => {
    const token = getCookie("site_token");
    if (!token) throw new Error("Not signed in.");
    const res = await backendFetch("/gacha/fetch", {
        method: "POST",
        bearerToken: token,
        body: "{}",
    });
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || `Failed to refresh gacha records: ${res.status}`);
    }
    return (await res.json()) as IGachaFetchResult;
});
