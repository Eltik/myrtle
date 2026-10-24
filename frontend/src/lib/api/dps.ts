import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import type { ICalcRequestBase, IConditionalInfo, IOperatorListEntry } from "#/components/tools/shared/types";
import { backendFetch } from "#/lib/fetch";
import type { BatchRequest } from "#/types/generated/BatchRequest";
import type { BatchResponse } from "#/types/generated/BatchResponse";
import type { BuffZones } from "#/types/generated/BuffZones";
import type { DpsResult } from "#/types/generated/DpsResult";
import type { RankRequest } from "#/types/generated/RankRequest";
import type { RankResponse } from "#/types/generated/RankResponse";
import type { RankRow } from "#/types/generated/RankRow";
import type { RequestCycleModel } from "#/types/generated/RequestCycleModel";
import type { RequestSecondary } from "#/types/generated/RequestSecondary";
import type { SupporterInvestment } from "#/types/generated/SupporterInvestment";
import type { SupportersResponse } from "#/types/generated/SupportersResponse";

export type { ICalcBuffs as IDpsCalculateBuffs, ICalcConditionals as IDpsCalculateConditionals } from "#/components/tools/shared/types";
export type IDpsConditionalInfo = IConditionalInfo;
export type IDpsOperatorListEntry = IOperatorListEntry;

/**
 * DEF/RES shred applied to the enemy. Percent fields are integers (e.g.
 * `def: 40` = -40% DEF), not decimals. DPS-specific (HPS ignores enemy stats).
 */
export interface IDpsCalculateShred {
    def?: number;
    defFlat?: number;
    res?: number;
    resFlat?: number;
}

/** Body for `POST /dps/calculate` - the shared build fields plus enemy stats. */
export interface IDpsCalculateRequest extends ICalcRequestBase {
    /** Enemy DEF before shreds. Defaults to 0. */
    defense?: number;
    /** Enemy RES (0-100) before shreds. Defaults to 0. */
    res?: number;
    shred?: IDpsCalculateShred;
    /** Buff zones with stacking rules; the legacy `buffs` / `shred` / `spBoost` join their zone server-side. */
    zones?: BuffZones;
    /** Exclusive with `targets` above 1: `count` sets the extra targets. */
    secondary?: RequestSecondary;
    /** Absent = `legacy`, the HEAD average, bit for bit. */
    cycleModel?: RequestCycleModel;
}

/**
 * Result from `POST /dps/calculate`: the three snake_case metrics, plus the camelCase
 * extras (`primary`, `category`, ...) only when the request asked for them.
 */
export type IDpsCalculateResponse = DpsResult;

/**
 * A ranked row. `retention` is the roster score, computed by the backend at its own
 * REFERENCE context (p1Max, no buffs, one target, legacy cycle), never at the request's;
 * null until `retentionStatus` is `ready`.
 */
export type IDpsRankRow = RankRow;
export type IDpsRankResponse = RankResponse;

export const getDpsOperatorsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/dps/operators");
    if (!res.ok) throw new Error(`Failed to load DPS operators: ${res.status}`);
    return (await res.json()) as IDpsOperatorListEntry[];
});

export function dpsOperatorsQueryOptions() {
    return queryOptions({
        queryKey: ["dps", "operators"],
        queryFn: () => getDpsOperatorsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const calculateDpsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IDpsCalculateRequest) => data)
    .handler(async ({ data }) => {
        const res = await backendFetch("/dps/calculate", {
            method: "POST",
            body: JSON.stringify(data),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to calculate DPS: ${res.status}`);
        }
        return (await res.json()) as IDpsCalculateResponse;
    });

export function stableStringify(value: unknown): string {
    if (value === null || typeof value !== "object") return JSON.stringify(value) ?? "null";
    if (Array.isArray(value)) return `[${value.map(stableStringify).join(",")}]`;
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${stableStringify(obj[k])}`).join(",")}}`;
}

export function dpsCalculateQueryOptions(req: IDpsCalculateRequest) {
    return queryOptions({
        queryKey: ["dps", "calculate", req.operatorId, stableStringify(req)],
        queryFn: () => calculateDpsFn({ data: req }),
        enabled: Boolean(req.operatorId),
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

/** `POST /dps/batch`: at most 64 requests per call (`MAX_BATCH`); one failed request gets `error` and the rest still compute. */
export const batchDpsFn = createServerFn({ method: "POST" })
    .inputValidator((data: BatchRequest) => data)
    .handler(async ({ data }) => {
        const res = await backendFetch("/dps/batch", { method: "POST", body: JSON.stringify(data) });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to calculate DPS: ${res.status}`);
        }
        return (await res.json()) as BatchResponse;
    });

export function dpsBatchQueryOptions(body: BatchRequest) {
    return queryOptions({
        queryKey: ["dps", "batch", stableStringify(body)],
        queryFn: () => batchDpsFn({ data: body }),
        enabled: body.requests.length > 0,
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

/** `POST /dps/rank`. The backend caches the full row set per context; filters and sort here are client-side. */
export const rankDpsFn = createServerFn({ method: "POST" })
    .inputValidator((data: RankRequest) => data)
    .handler(async ({ data }) => {
        const res = await backendFetch("/dps/rank", { method: "POST", body: JSON.stringify(data) });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to rank operators: ${res.status}`);
        }
        return (await res.json()) as IDpsRankResponse;
    });

export function dpsRankQueryOptions(body: RankRequest) {
    return queryOptions({
        queryKey: ["dps", "rank", stableStringify(body)],
        queryFn: () => rankDpsFn({ data: body }),
        staleTime: 10 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

/** `GET /dps/supporters` at one investment; every value is read from game data server-side. */
export const getDpsSupportersFn = createServerFn({ method: "GET" })
    .inputValidator((data: SupporterInvestment) => data)
    .handler(async ({ data }) => {
        const qs = new URLSearchParams({ promotion: String(data.promotion), skillLevel: String(data.skillLevel), potential: String(data.potential), moduleLevel: String(data.moduleLevel) });
        const res = await backendFetch(`/dps/supporters?${qs.toString()}`);
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to load supporters: ${res.status}`);
        }
        return (await res.json()) as SupportersResponse;
    });

export function dpsSupportersQueryOptions(investment: SupporterInvestment) {
    return queryOptions({
        queryKey: ["dps", "supporters", investment.promotion, investment.skillLevel, investment.potential, investment.moduleLevel],
        queryFn: () => getDpsSupportersFn({ data: investment }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
