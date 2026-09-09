import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/app/services/gacha.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { BannerPullStat } from "#/types/generated/BannerPullStat";
import type { CollectiveStats } from "#/types/generated/CollectiveStats";
import type { DatePullData } from "#/types/generated/DatePullData";
import type { DayOfWeekPullData } from "#/types/generated/DayOfWeekPullData";
import type { FetchResult } from "#/types/generated/FetchResult";
import type { GachaEnhancedStats } from "#/types/generated/GachaEnhancedStats";
import type { GachaHistoryEnvelopeDto } from "#/types/generated/GachaHistoryEnvelopeDto";
import type { GachaItemDto } from "#/types/generated/GachaItemDto";
import type { GachaPaginationInfoDto } from "#/types/generated/GachaPaginationInfoDto";
/**
 * Banner metadata sourced from the static `gacha_table.json` → `GachaPoolClient`.
 *
 * Generated from `backend/src/core/gamedata/types/gacha.rs`. Times are **unix
 * seconds** (not ms); pull records, by contrast, use unix ms.
 */
import type { GachaPoolClient } from "#/types/generated/GachaPoolClient";
import type { GachaRecordEntryDto } from "#/types/generated/GachaRecordEntryDto";
import type { GachaRecordsDto } from "#/types/generated/GachaRecordsDto";
import type { GachaSettingsDto } from "#/types/generated/GachaSettingsDto";
import type { GachaTypeRecordsDto } from "#/types/generated/GachaTypeRecordsDto";
import type { GlobalGachaStats } from "#/types/generated/GlobalGachaStats";
import type { HistoryFiltersAppliedDto } from "#/types/generated/HistoryFiltersAppliedDto";
import type { HourlyPullData } from "#/types/generated/HourlyPullData";
import type { OperatorPopularity } from "#/types/generated/OperatorPopularity";
import type { PullRates } from "#/types/generated/PullRates";
import type { PullTimingData } from "#/types/generated/PullTimingData";
import type { RarityRate } from "#/types/generated/RarityRate";
import type { WeightUpChar } from "#/types/generated/WeightUpChar";

export type IBanner = GachaPoolClient;
export type IWeightUpChar = WeightUpChar;
export type IRarityRate = RarityRate;

export const getBannersFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/banners");
    if (!res.ok) throw new Error(`Failed to load banners: ${res.status}`);
    return (await res.json()) as IBanner[];
});

export function bannersQueryOptions() {
    return queryOptions({
        queryKey: ["banners"],
        queryFn: () => getBannersFn(),
        // Banners are static-data; refresh roughly once per session.
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** Lightweight community summary. Rates are fractions in [0, 1]. */
export type IGachaGlobalStats = GlobalGachaStats;

export type IGachaCollectiveStats = CollectiveStats;

export type IGachaPullRates = PullRates;

export type IOperatorPopularity = OperatorPopularity;

export type IHourlyPullData = HourlyPullData;

export type IDayOfWeekPullData = DayOfWeekPullData;

export type IDatePullData = DatePullData;

export type IPullTimingData = PullTimingData;

/**
 * Community pull totals grouped by `pool_id`. Served by
 * `GET /gacha/stats/per-banner`. Only `share_stats=true` users contribute.
 * Sorted by `pullCount` desc; pools with zero community pulls are absent.
 */
export type IBannerPullStat = BannerPullStat;

export const getPerBannerStatsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/gacha/stats/per-banner");
    if (!res.ok) throw new Error(`Failed to load per-banner stats: ${res.status}`);
    return (await res.json()) as IBannerPullStat[];
});

export function perBannerStatsQueryOptions() {
    return queryOptions({
        queryKey: ["gacha", "per-banner-stats"],
        queryFn: () => getPerBannerStatsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export type IGachaEnhancedStats = GachaEnhancedStats;

// Alias kept for the optional field above (name matches the server type).

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

export type IGachaRecordEntry = GachaRecordEntryDto;

export type IGachaItem = GachaItemDto;

export type GachaGroup = "limited" | "regular" | "special";

export type IGachaTypeRecords = GachaTypeRecordsDto;

export type IGachaRecords = GachaRecordsDto;

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
    // FESCLASSIC_ is the festival/anniversary kernel banner - same pool as CLASSIC_/BOOT_.
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

/**
 * Classify a banner-pool entry (from `/static/banners`) into the same 4-bucket
 * UI grouping used for pull records. Mirrors {@link classifyClientGachaGroup}
 * but keyed off the banner's `gachaRuleType` (and `gachaPoolId` prefix as a
 * fallback) since static banner metadata doesn't carry a `typeName`.
 */
export function classifyBannerGroup(banner: { gachaRuleType: string; gachaPoolId: string }): ClientGachaGroup {
    switch (banner.gachaRuleType) {
        case "LIMITED":
            return "limited";
        case "LINKAGE":
            return "linkage";
        case "CLASSIC":
        case "CLASSIC_ATTAIN":
        case "CLASSIC_DOUBLE":
        case "FESCLASSIC":
            return "special";
        case "SINGLE":
        case "NORMAL":
        case "ATTAIN":
        case "DOUBLE":
        case "SPECIAL":
            return "regular";
        default: {
            const id = banner.gachaPoolId ?? "";
            if (id.startsWith("LIMITED_")) return "limited";
            if (id.startsWith("LINKAGE_")) return "linkage";
            if (id.startsWith("FESCLASSIC_") || id.startsWith("CLASSIC_") || id.startsWith("BOOT_")) return "special";
            return "regular";
        }
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

export type IGachaPaginationInfo = GachaPaginationInfoDto;

export type IGachaHistoryFiltersApplied = HistoryFiltersAppliedDto;

export type IGachaHistoryEnvelope = GachaHistoryEnvelopeDto;

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

export type IGachaSettings = GachaSettingsDto;

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

export type IGachaFetchResult = FetchResult;

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
