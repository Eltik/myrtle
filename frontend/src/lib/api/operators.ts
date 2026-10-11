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

/**
 * Read by the browser straight from the backend (`backendFetch`), as
 * `fetchOperatorsRoster` is. Through a server function the 645,037-byte index
 * reached the browser as 1,543,898 bytes of uncompressed seroval; the backend
 * answers 45,034 bytes of brotli.
 */
async function fetchOperatorsIndex(server: string): Promise<IOperatorIndexEntry[]> {
    const res = await backendFetch(gamedataPath(server, "/operators/index"));
    if (!res.ok) throw new Error(`Failed to load operators index: ${res.status}`);
    return (await res.json()) as IOperatorIndexEntry[];
}

export function operatorsIndexQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "index", ...gamedataKey(server)],
        queryFn: () => fetchOperatorsIndex(resolveGamedataServer(server)),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

const OPERATORS_LIST_TTL_MS = 30 * 60 * 1000;

/**
 * SSR-process cache of the whole operator table, per game server.
 *
 * Keyed by server. Unkeyed, one locale pinned to `jp` is a cross-locale data leak: the first
 * render to warm the entry would decide what language every other locale's operator names came
 * out in for 30 minutes.
 */
const operatorsListCache = new Map<string, { promise: Promise<IOperatorListItem[]>; expiresAt: number }>();

async function loadOperatorsList(server: string | undefined): Promise<IOperatorListItem[]> {
    const res = await backendFetch(gamedataPath(server, "/static/operators"));
    if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
    const raw = (await res.json()) as IOperatorsStaticMap;
    const normalized = deepCamelize(raw);
    return Object.values(normalized) as IOperatorListItem[];
}

/** The whole operator table for one game server, read through `operatorsListCache`. */
function cachedOperatorsList(server: string | undefined): Promise<IOperatorListItem[]> {
    const resolved = resolveGamedataServer(server);
    const now = Date.now();
    const hit = operatorsListCache.get(resolved);
    if (hit && now < hit.expiresAt) return hit.promise;
    const promise = loadOperatorsList(resolved);
    operatorsListCache.set(resolved, { promise, expiresAt: now + OPERATORS_LIST_TTL_MS });
    promise.catch(() => {
        if (operatorsListCache.get(resolved)?.promise === promise) operatorsListCache.delete(resolved);
    });
    return promise;
}

function pickFields<K extends keyof IOperatorListItem>(list: IOperatorListItem[], fields: readonly K[]): Pick<IOperatorListItem, K>[] {
    return list.map((op) => {
        const out = {} as Pick<IOperatorListItem, K>;
        for (const field of fields) out[field] = op[field];
        return out;
    });
}

/**
 * The whole table, for server-side callers only (the OG renderer calls it in process).
 *
 * Do not query it from the browser: its serialized result is one ~40 MB record, above the
 * 32 MiB a server function may return, so a browser call fails with a 500. A browser caller
 * gets a projection that carries only the fields it reads, as the export does below, or reads
 * the backend directly, as the profile tabs do.
 */
export const getOperatorsListFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => cachedOperatorsList(server));

/**
 * The full-table fields the operators export adds to each index row, keyed by `id`. The
 * export dialog's columns come from these and the index; nothing else of the table is read.
 */
const OPERATOR_EXPORT_FIELDS = ["id", "displayNumber", "description", "itemUsage", "itemDesc", "itemObtainApproach", "isSpChar", "maxPotentialLevel", "skin"] as const;

export type IOperatorExportFields = Pick<IOperatorListItem, (typeof OPERATOR_EXPORT_FIELDS)[number]>;

export const getOperatorsExportFieldsFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => pickFields(await cachedOperatorsList(server), OPERATOR_EXPORT_FIELDS));

export function operatorsExportFieldsQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "export-fields", ...gamedataKey(server)],
        queryFn: () => getOperatorsExportFieldsFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * The full-table fields the profile Roster and Optimizer tabs read: phases, skills, modules,
 * potential and trust for the cards and stats, base skills for the optimizer, art, the source
 * filter and the alternate-form template.
 */
const OPERATOR_ROSTER_FIELDS = ["id", "name", "profession", "professionName", "phases", "skills", "modules", "potentialRanks", "favorKeyFrames", "baseSkills", "skin", "portrait", "itemObtainApproach", "obtainChannel", "tmplDefault"] as const;

export type IOperatorRosterItem = Pick<IOperatorListItem, (typeof OPERATOR_ROSTER_FIELDS)[number]>;

const ROSTER_FIELD_SET: ReadonlySet<string> = new Set(OPERATOR_ROSTER_FIELDS);

/**
 * Keeps the roster fields of each raw row and camelizes only those. For `/static/operators`
 * rows that gives the same result as `deepCamelize` of the whole table followed by a pick;
 * the backend's `/operators/roster` rows already carry just these fields, so there it only
 * camelizes.
 */
export function toRosterItems(rows: Iterable<unknown>): IOperatorRosterItem[] {
    return Array.from(rows, (op) => {
        const out: Record<string, unknown> = {};
        for (const [key, value] of Object.entries(op as Record<string, unknown>)) {
            const name = normalizeKey(key);
            if (ROSTER_FIELD_SET.has(name)) out[name] = deepCamelize(value);
        }
        return out as IOperatorRosterItem;
    });
}

/**
 * The roster rows, read by the browser straight from the backend's `/operators/roster`,
 * which serves the `/static/operators` table reduced to these fields (about two thirds of
 * it), compressed, with an ETag and `Cache-Control: public, max-age=300`. The tabs only query
 * this on the client, after a tab opens. It used to go through a server function, whose
 * result serialized to about 28 MB of seroval, 16 % under the 32 MiB a server function may
 * return, and reached the browser uncompressed.
 */
async function fetchOperatorsRoster(server: string): Promise<IOperatorRosterItem[]> {
    const res = await backendFetch(gamedataPath(server, "/operators/roster"));
    if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
    return toRosterItems((await res.json()) as unknown[]);
}

export function operatorsRosterQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "roster-fields", ...gamedataKey(server)],
        queryFn: () => fetchOperatorsRoster(resolveGamedataServer(server)),
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

/**
 * One operator, read by the browser straight from the backend (cached there
 * with an `ETag` and `Cache-Control: public, max-age=300`). Through a server
 * function Amiya's 56,828-byte record arrived as 94,213 bytes of uncompressed
 * seroval; the backend answers 10,270 bytes of brotli. The Squad Builder reads
 * one per seat.
 */
async function fetchOperator(id: string, server: string): Promise<IOperatorListItem | null> {
    const res = await backendFetch(gamedataPath(server, `/operators/${encodeURIComponent(id)}`));
    // `null`, not `undefined`: TanStack Query rejects a query whose data is
    // undefined, which turned an unknown id into a 500 instead of a 404.
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`Failed to load operator: ${res.status}`);
    return deepCamelize(await res.json()) as IOperatorListItem;
}

export function operatorQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["operators", "detail", id, ...gamedataKey(server)],
        queryFn: () => fetchOperator(id, resolveGamedataServer(server)),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
