import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import type { ICalcRequestBase, IConditionalInfo, IOperatorListEntry } from "#/components/tools/shared/types";
import { backendFetch } from "#/lib/fetch";

// Operator/conditional/buff types come from `components/tools/shared/types`,
// re-exported here under DPS names.
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
}

/**
 * Result from `POST /dps/calculate`. The backend returns snake_case (the
 * `DpsResult` struct has no rename attribute).
 */
export interface IDpsCalculateResponse {
    skill_dps: number;
    total_damage: number;
    average_dps: number;
    /** Allows reading any result value by metric key. */
    [metric: string]: number;
}

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

function stableStringify(value: unknown): string {
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
