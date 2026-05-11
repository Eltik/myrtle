import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";

export const getStagesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/stages");
    if (!res.ok) throw new Error(`Failed to load stages: ${res.status}`);
    const raw = (await res.json()) as Record<string, IStage>;
    return Object.values(raw);
});

export function stagesQueryOptions() {
    return queryOptions({
        queryKey: ["static", "stages"],
        queryFn: () => getStagesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getZonesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/zones");
    if (!res.ok) throw new Error(`Failed to load zones: ${res.status}`);
    const raw = (await res.json()) as Record<string, IZone>;
    return Object.values(raw);
});

export function zonesQueryOptions() {
    return queryOptions({
        queryKey: ["static", "zones"],
        queryFn: () => getZonesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getUserStageClearsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const res = await backendFetch(`/stage-clears?uid=${encodeURIComponent(uid)}`, { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return {} as StageClearsMap;
            if (res.status === 403) return {} as StageClearsMap;
            throw new Error(`Failed to load stage clears: ${res.status}`);
        }
        return (await res.json()) as StageClearsMap;
    });

export function userStageClearsQueryOptions(uid: string | null, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "stage-clears", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => (uid ? getUserStageClearsFn({ data: { uid, bearerToken } }) : Promise.resolve({} as StageClearsMap)),
        enabled: !!uid,
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}
