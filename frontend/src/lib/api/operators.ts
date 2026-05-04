import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry, IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";

// The /static/operators endpoint serves some nested shapes (phases,
// attributesKeyFrames, skills[].levelUpCostCond, talents, trait,
// potentialRanks, drone subtree, etc.) with PascalCase + trailing-underscore
// keys (e.g. `AttributesKeyFrames`, `MaxHp`, `Type_`). The rest of the
// response is camelCase. Normalizing here means every consumer can rely on
// the camelCase shape declared in `#/types/operators`.
function normalizeKey(key: string): string {
    const trimmed = key.endsWith("_") ? key.slice(0, -1) : key;
    if (trimmed.length === 0) return trimmed;
    const first = trimmed.charCodeAt(0);
    if (first >= 65 && first <= 90) return trimmed[0].toLowerCase() + trimmed.slice(1);
    return trimmed;
}

export function deepCamelize<T>(value: T): T {
    if (Array.isArray(value)) return value.map(deepCamelize) as unknown as T;
    if (value !== null && typeof value === "object") {
        const out: Record<string, unknown> = {};
        for (const [k, v] of Object.entries(value)) out[normalizeKey(k)] = deepCamelize(v);
        return out as T;
    }
    return value;
}

export const getOperatorsIndexFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/operators/index");
    if (!res.ok) throw new Error(`Failed to load operators index: ${res.status}`);
    return (await res.json()) as IOperatorIndexEntry[];
});

export function operatorsIndexQueryOptions() {
    return queryOptions({
        queryKey: ["operators", "index"],
        queryFn: () => getOperatorsIndexFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getOperatorsListFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/operators");
    if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
    const raw = (await res.json()) as IOperatorsStaticMap;
    const normalized = deepCamelize(raw);
    return Object.values(normalized) as IOperatorListItem[];
});

export function operatorsListQueryOptions() {
    return queryOptions({
        queryKey: ["operators", "list"],
        queryFn: () => getOperatorsListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((id: string) => id)
    .handler(async ({ data: id }) => {
        const res = await backendFetch("/static/operators");
        if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
        const raw = (await res.json()) as IOperatorsStaticMap;
        const normalized = deepCamelize(raw);
        const operators = Object.values(normalized) as IOperatorListItem[];
        return operators.find((op) => op.id === id);
    });

export function operatorQueryOptions(id: string) {
    return queryOptions({
        queryKey: ["operators", "detail", id],
        queryFn: () => getOperatorFn({ data: id }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
