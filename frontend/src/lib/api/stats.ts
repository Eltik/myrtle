import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
// Generated from `backend/src/app/services/stats.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { GameDataStats } from "#/types/generated/GameDataStats";
import type { RostersStats } from "#/types/generated/RostersStats";
import type { StatsResponse } from "#/types/generated/StatsResponse";
import type { TierListSiteStats } from "#/types/generated/TierListSiteStats";
import { backendFetch } from "../fetch";

export type IGameDataStats = GameDataStats;

export type ITierListStats = TierListSiteStats;

export type IRostersStats = RostersStats;

export type IStatsResponse = StatsResponse;

export const getStatsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/stats");
    if (!res.ok) throw new Error(`Failed to load stats: ${res.status}`);
    return (await res.json()) as IStatsResponse;
});

export function statsQueryOptions() {
    return queryOptions({
        queryKey: ["stats"],
        queryFn: () => getStatsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}
