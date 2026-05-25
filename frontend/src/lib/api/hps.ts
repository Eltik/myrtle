import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import type { ICalcRequestBase, IConditionalInfo, IOperatorListEntry } from "#/components/tools/shared/types";
import { backendFetch } from "#/lib/fetch";

// Operator/conditional/buff types come from `components/tools/shared/types`,
// re-exported here under HPS names.
export type { ICalcBuffs as IHpsCalculateBuffs, ICalcConditionals as IHpsCalculateConditionals } from "#/components/tools/shared/types";
export type IHpsConditionalInfo = IConditionalInfo;
export type IHpsOperatorListEntry = IOperatorListEntry;

/** Body for `POST /hps/calculate`. The HPS engine ignores enemy DEF/RES, so there are no enemy fields. */
export type IHpsCalculateRequest = ICalcRequestBase;

/**
 * Result from `POST /hps/calculate` (snake_case, like {@link import("./dps").IDpsCalculateResponse}).
 */
export interface IHpsCalculateResponse {
    /** Healing per second while the skill is active. */
    skill_hps: number;
    /** Always-on healing during SP recharge; 0 for burst healers. */
    base_hps: number;
    /** Cycle-averaged HPS over a full skill + recharge cycle. */
    avg_hps: number;
    /** Allows reading any result value by metric key. */
    [metric: string]: number;
}

export const getHpsOperatorsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/hps/operators");
    if (!res.ok) throw new Error(`Failed to load HPS operators: ${res.status}`);
    return (await res.json()) as IHpsOperatorListEntry[];
});

export function hpsOperatorsQueryOptions() {
    return queryOptions({
        queryKey: ["hps", "operators"],
        queryFn: () => getHpsOperatorsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const calculateHpsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IHpsCalculateRequest) => data)
    .handler(async ({ data }) => {
        const res = await backendFetch("/hps/calculate", {
            method: "POST",
            body: JSON.stringify(data),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to calculate HPS: ${res.status}`);
        }
        return (await res.json()) as IHpsCalculateResponse;
    });

function stableStringify(value: unknown): string {
    if (value === null || typeof value !== "object") return JSON.stringify(value) ?? "null";
    if (Array.isArray(value)) return `[${value.map(stableStringify).join(",")}]`;
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${stableStringify(obj[k])}`).join(",")}}`;
}

export function hpsCalculateQueryOptions(req: IHpsCalculateRequest) {
    return queryOptions({
        queryKey: ["hps", "calculate", req.operatorId, stableStringify(req)],
        queryFn: () => calculateHpsFn({ data: req }),
        enabled: Boolean(req.operatorId),
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}
