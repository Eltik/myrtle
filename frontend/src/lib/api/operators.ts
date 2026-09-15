import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { OperatorBuildStatsResponse } from "#/types/generated/OperatorBuildStatsResponse";
import type { OperatorOwnershipResponse } from "#/types/generated/OperatorOwnershipResponse";
import type { IOperatorIndexEntry, IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

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

export const getOperatorsIndexFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/operators/index"));
        if (!res.ok) throw new Error(`Failed to load operators index: ${res.status}`);
        return (await res.json()) as IOperatorIndexEntry[];
    });

export function operatorsIndexQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "index", ...gamedataKey(server)],
        queryFn: () => getOperatorsIndexFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

const OPERATORS_LIST_TTL_MS = 30 * 60 * 1000;

/**
 * SSR-process cache of the whole operator table, per game server.
 *
 * Keyed by server: it used to be keyed by nothing at all, which was fine while
 * every render read the same client, and is a cross-locale data leak the moment
 * one locale is pinned to `jp` - the first render to warm the entry would decide
 * what language every other locale's operator names came out in for 30 minutes.
 */
const operatorsListCache = new Map<string, { promise: Promise<IOperatorListItem[]>; expiresAt: number }>();

async function loadOperatorsList(server: string | undefined): Promise<IOperatorListItem[]> {
    const res = await backendFetch(gamedataPath(server, "/static/operators"));
    if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
    const raw = (await res.json()) as IOperatorsStaticMap;
    const normalized = deepCamelize(raw);
    return Object.values(normalized) as IOperatorListItem[];
}

export const getOperatorsListFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => {
        const resolved = resolveGamedataServer(server);
        const now = Date.now();
        const hit = operatorsListCache.get(resolved);
        if (!hit || now >= hit.expiresAt) {
            const promise = loadOperatorsList(resolved);
            operatorsListCache.set(resolved, { promise, expiresAt: now + OPERATORS_LIST_TTL_MS });
            promise.catch(() => {
                if (operatorsListCache.get(resolved)?.promise === promise) operatorsListCache.delete(resolved);
            });
            return promise;
        }
        return hit.promise;
    });

export function operatorsListQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "list", ...gamedataKey(server)],
        queryFn: () => getOperatorsListFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** Population-level ownership: how many sharing players own each operator, and
 *  how many took them to E2. A missing id in `counts` means zero owners;
 *  `totalUsers` is the denominator. Generated from the Rust response type. */
export type IOperatorOwnership = OperatorOwnershipResponse;

/** What the community leaves selected on an operator: default skill and default
 *  module, each as a whole distribution ordered most-picked first. */
export type IOperatorBuildStats = OperatorBuildStatsResponse;

export const getOperatorBuildStatsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server?: string }) => data)
    .handler(async ({ data: { id, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/operators/${encodeURIComponent(id)}/build-stats`));
        // An operator nobody has built is not an error, and neither is a
        // backend that predates this route. The detail page falls back to its
        // own default, so `undefined` is a usable answer.
        if (res.status === 404) return undefined;
        if (!res.ok) throw new Error(`Failed to load operator build stats: ${res.status}`);
        return (await res.json()) as IOperatorBuildStats;
    });

export function operatorBuildStatsQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "build-stats", id, ...gamedataKey(server)],
        queryFn: () => getOperatorBuildStatsFn({ data: { id, server: resolveGamedataServer(server) } }),
        staleTime: 30 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export const getOperatorOwnershipFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/operators/ownership"));
        if (!res.ok) throw new Error(`Failed to load operator ownership: ${res.status}`);
        return (await res.json()) as IOperatorOwnership;
    });

export function operatorOwnershipQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "ownership", ...gamedataKey(server)],
        queryFn: () => getOperatorOwnershipFn({ data: resolveGamedataServer(server) }),
        staleTime: 30 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export const getOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server?: string }) => data)
    .handler(async ({ data: { id, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/operators/${encodeURIComponent(id)}`));
        if (res.status === 404) return undefined;
        if (!res.ok) throw new Error(`Failed to load operator: ${res.status}`);
        return deepCamelize(await res.json()) as IOperatorListItem;
    });

export function operatorQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "detail", id, ...gamedataKey(server)],
        queryFn: () => getOperatorFn({ data: { id, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
