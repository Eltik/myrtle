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
    /** Rank change vs. the snapshot baseline for the requested movement_interval.
     *  Positive = climbed. Null when movement wasn't requested or there's no baseline yet. */
    rank_delta: number | null;
}

export interface ILeaderboardPage {
    entries: ILeaderboardEntry[];
    total: number;
    updated_at: string | null;
}

export interface ILeaderboardInput {
    sort?: string;
    server?: string;
    /** "1 day" | "7 days" | "30 days" — when set, each entry receives `rank_delta`. */
    movement_interval?: string;
    /** When true (requires `movement_interval`), only users with non-zero movement are returned. */
    movement_only?: boolean;
    limit?: number;
    offset?: number;
}

export const getLeaderboardFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardInput) => data)
    .handler(async ({ data: { sort, server, movement_interval, movement_only, limit, offset } }) => {
        const params = new URLSearchParams();
        if (sort) params.set("sort", sort);
        if (server) params.set("server", server);
        if (movement_interval) params.set("movement_interval", movement_interval);
        if (movement_only) params.set("movement_only", "true");
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));

        const res = await backendFetch(`/leaderboard?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard: ${res.status}`);
        return (await res.json()) as ILeaderboardPage;
    });

export function leaderboardQueryOptions(input: ILeaderboardInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", input.sort ?? null, input.server ?? null, input.movement_interval ?? null, input.movement_only ?? false, input.limit ?? null, input.offset ?? null],
        queryFn: () => getLeaderboardFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type LeaderboardMoverDirection = "up" | "down";
export type LeaderboardMoverInterval = "1 day" | "7 days" | "30 days";

export interface ILeaderboardMover {
    uid: string;
    nickname: string | null;
    nick_number: string | null;
    avatar_id: string | null;
    server: string;
    current_rank: number;
    previous_rank: number;
    /** Positive = climbed (previous_rank - current_rank). */
    rank_delta: number;
    current_score: number | null;
    score_delta: number | null;
}

export interface ILeaderboardMoversInput {
    direction?: LeaderboardMoverDirection;
    interval?: LeaderboardMoverInterval;
    server?: string;
    limit?: number;
}

export const getLeaderboardMoversFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardMoversInput) => data)
    .handler(async ({ data: { direction, interval, server, limit } }) => {
        const params = new URLSearchParams();
        if (direction) params.set("direction", direction);
        if (interval) params.set("interval", interval);
        if (server) params.set("server", server);
        if (limit !== undefined) params.set("limit", String(limit));

        const res = await backendFetch(`/leaderboard/movers?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard movers: ${res.status}`);
        return (await res.json()) as ILeaderboardMover[];
    });

export function leaderboardMoversQueryOptions(input: ILeaderboardMoversInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "movers", input.direction ?? null, input.interval ?? null, input.server ?? null, input.limit ?? null],
        queryFn: () => getLeaderboardMoversFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface IServerShare {
    server: string;
    players: number;
}

export interface ILeaderboardDistributionInput {
    top?: number;
}

export const getLeaderboardDistributionFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardDistributionInput) => data)
    .handler(async ({ data: { top } }) => {
        const params = new URLSearchParams();
        if (top !== undefined) params.set("top", String(top));

        const res = await backendFetch(`/leaderboard/distribution?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard distribution: ${res.status}`);
        return (await res.json()) as IServerShare[];
    });

export function leaderboardDistributionQueryOptions(input: ILeaderboardDistributionInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "distribution", input.top ?? null],
        queryFn: () => getLeaderboardDistributionFn({ data: input }),
        staleTime: 5 * 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}

export interface IPlayerStanding {
    player: ILeaderboardEntry;
    neighbors: ILeaderboardEntry[];
    /** 0.0 = top, 1.0 = bottom. */
    percentile: number;
    /** Positive = climbed since the 7-day baseline. `null` if no baseline exists. */
    rank_delta_7d: number | null;
}

export interface IPlayerStandingInput {
    uid: string;
    server: string;
    window?: number;
}

export const getPlayerStandingFn = createServerFn({ method: "GET" })
    .inputValidator((data: IPlayerStandingInput) => data)
    .handler(async ({ data: { uid, server, window } }) => {
        const params = new URLSearchParams({ uid, server });
        if (window !== undefined) params.set("window", String(window));

        const res = await backendFetch(`/leaderboard/standing?${params.toString()}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load player standing: ${res.status}`);
        }
        return (await res.json()) as IPlayerStanding;
    });

export function playerStandingQueryOptions(input: IPlayerStandingInput) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "standing", input.uid, input.server, input.window ?? null],
        queryFn: () => getPlayerStandingFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
