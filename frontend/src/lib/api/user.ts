import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IUserProfile } from "#/types/user";

export interface IRosterMastery {
    index: number;
    mastery: number;
}

export interface IRosterModule {
    id: string;
    level: number;
    locked: boolean;
}

export interface IRosterEntry {
    user_id: string;
    operator_id: string;
    elite: number;
    level: number;
    exp: number;
    potential: number;
    skill_level: number;
    favor_point: number;
    skin_id: string | null;
    default_skill: number | null;
    voice_lan: string | null;
    current_equip: string | null;
    current_tmpl: string | null;
    obtained_at: number | null;
    masteries: IRosterMastery[];
    modules: IRosterModule[];
}

export interface IUserScore {
    user_id: string;
    total_score: number;
    operator_score: number;
    stage_score: number;
    roguelike_score: number;
    sandbox_score: number;
    medal_score: number;
    base_score: number;
    skin_score: number;
    grade: string | null;
    calculated_at: string;
}

export const getUserFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load user: ${res.status}`);
        }
        return (await res.json()) as IUserProfile;
    });

export function userQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "profile", uid],
        queryFn: () => getUserFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserRosterFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const res = await backendFetch(`/roster?uid=${encodeURIComponent(uid)}`, { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load roster: ${res.status}`);
        }
        return (await res.json()) as IRosterEntry[];
    });

export function userRosterQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "roster", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserRosterFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserRosterOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; operatorId: string }) => data)
    .handler(async ({ data: { uid, operatorId } }) => {
        const res = await backendFetch(`/roster/${encodeURIComponent(operatorId)}?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load roster operator: ${res.status}`);
        }
        return (await res.json()) as IRosterEntry;
    });

export function userRosterOperatorQueryOptions(uid: string, operatorId: string) {
    return queryOptions({
        queryKey: ["user", "roster", uid, operatorId],
        queryFn: () => getUserRosterOperatorFn({ data: { uid, operatorId } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface IInventoryItem {
    item_id: string;
    quantity: number;
}

export const getUserInventoryFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const res = await backendFetch(`/inventory?uid=${encodeURIComponent(uid)}`, { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load inventory: ${res.status}`);
        }
        return (await res.json()) as IInventoryItem[];
    });

export function userInventoryQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "inventory", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserInventoryFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserScoreFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/get-user-score?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load user score: ${res.status}`);
        }
        return (await res.json()) as IUserScore | null;
    });

export function userScoreQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "score", uid],
        queryFn: () => getUserScoreFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface ISearchUsersInput {
    q?: string;
    limit?: number;
    offset?: number;
}

export interface ISearchPage {
    entries: IUserProfile[];
    total: number;
}

export const searchUsersFn = createServerFn({ method: "GET" })
    .inputValidator((data: ISearchUsersInput) => data)
    .handler(async ({ data: { q, limit, offset } }) => {
        const params = new URLSearchParams();
        if (q) params.set("q", q);
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));

        const res = await backendFetch(`/search?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to search users: ${res.status}`);
        return (await res.json()) as ISearchPage;
    });

export function searchUsersQueryOptions(input: ISearchUsersInput) {
    return queryOptions({
        queryKey: ["user", "search", input.q ?? null, input.limit ?? null, input.offset ?? null],
        queryFn: () => searchUsersFn({ data: input }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface ILeaderboardEntry {
    id: string;
    uid: string;
    nickname: string | null;
    nick_number: string | null;
    level: number | null;
    avatar_id: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
    server: string;
    total_score: number | null;
    grade: string | null;
    operator_score: number | null;
    stage_score: number | null;
    roguelike_score: number | null;
    sandbox_score: number | null;
    medal_score: number | null;
    base_score: number | null;
    skin_score: number | null;
    rank_global: number | null;
    rank_server: number | null;
}

export interface ILeaderboardPage {
    entries: ILeaderboardEntry[];
    total: number;
}

export interface ILeaderboardInput {
    sort?: string;
    server?: string;
    limit?: number;
    offset?: number;
}

export const getLeaderboardFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardInput) => data)
    .handler(async ({ data: { sort, server, limit, offset } }) => {
        const params = new URLSearchParams();
        if (sort) params.set("sort", sort);
        if (server) params.set("server", server);
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));

        const res = await backendFetch(`/leaderboard?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard: ${res.status}`);
        return (await res.json()) as ILeaderboardPage;
    });

export function leaderboardQueryOptions(input: ILeaderboardInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", input.sort ?? null, input.server ?? null, input.limit ?? null, input.offset ?? null],
        queryFn: () => getLeaderboardFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
