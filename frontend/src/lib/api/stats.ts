import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "../fetch";

export interface GameDataStats {
    operators: number;
    skills: number;
    modules: number;
    skins: number;
    stages: number;
    zones: number;
    enemies: number;
}

export interface TierListStats {
    total: number;
    active: number;
    totalVersions: number;
    totalPlacements: number;
}

export interface RostersStats {
    total: number;
}

export interface StatsResponse {
    gameData: GameDataStats;
    tierLists: TierListStats;
    rosters: RostersStats;
    computedAt: string;
}

export const getStatsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/stats");
    if (!res.ok) throw new Error(`Failed to load stats: ${res.status}`);
    return (await res.json()) as StatsResponse;
});

export function statsQueryOptions() {
    return queryOptions({
        queryKey: ["stats"],
        queryFn: () => getStatsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}
