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
    locked?: boolean;
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
    masteries: IRosterMastery[] | null;
    modules: IRosterModule[] | null;
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
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/roster?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load roster: ${res.status}`);
        }
        return (await res.json()) as IRosterEntry[];
    });

export function userRosterQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "roster", uid],
        queryFn: () => getUserRosterFn({ data: uid }),
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
