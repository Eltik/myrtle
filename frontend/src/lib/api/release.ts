import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
import type { BannersResponse } from "#/types/generated/BannersResponse";
import type { EventsResponse } from "#/types/generated/EventsResponse";
import type { LagResponse } from "#/types/generated/LagResponse";
import type { PutReleasePlan } from "#/types/generated/PutReleasePlan";
import type { ReleasePlan } from "#/types/generated/ReleasePlan";
import type { SkinsResponse } from "#/types/generated/SkinsResponse";

const STALE_MS = 60 * 60 * 1000;
const GC_MS = 24 * 60 * 60 * 1000;

export const getReleaseEventsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/release/events");
    if (!res.ok) throw new Error(`Failed to load release events: ${res.status}`);
    return (await res.json()) as EventsResponse;
});

export const getReleaseBannersFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/release/banners");
    if (!res.ok) throw new Error(`Failed to load release banners: ${res.status}`);
    return (await res.json()) as BannersResponse;
});

export const getReleaseSkinsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/release/skins");
    if (!res.ok) throw new Error(`Failed to load release skins: ${res.status}`);
    return (await res.json()) as SkinsResponse;
});

export const getReleaseLagFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/release/lag");
    if (!res.ok) throw new Error(`Failed to load release lag model: ${res.status}`);
    return (await res.json()) as LagResponse;
});

export function releaseEventsQueryOptions() {
    return queryOptions({
        queryKey: ["release", "events"],
        queryFn: () => getReleaseEventsFn(),
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export function releaseBannersQueryOptions() {
    return queryOptions({
        queryKey: ["release", "banners"],
        queryFn: () => getReleaseBannersFn(),
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export function releaseSkinsQueryOptions() {
    return queryOptions({
        queryKey: ["release", "skins"],
        queryFn: () => getReleaseSkinsFn(),
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export function releaseLagQueryOptions() {
    return queryOptions({
        queryKey: ["release", "lag"],
        queryFn: () => getReleaseLagFn(),
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export const getReleasePlanFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = getCookie("site_token");
    if (!token) return null;
    const res = await backendFetch("/release/plan", { bearerToken: token });
    if (!res.ok) {
        if (res.status === 401 || res.status === 403) return null;
        throw new Error(`Failed to load the release plan: ${res.status}`);
    }
    return (await res.json()) as ReleasePlan | null;
});

export const putReleasePlanFn = createServerFn({ method: "POST" })
    .inputValidator((data: PutReleasePlan) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch("/release/plan", { method: "PUT", bearerToken: token, body: JSON.stringify(data) });
        if (!res.ok) throw new Error(`Failed to save the release plan: ${res.status}`);
        return (await res.json()) as ReleasePlan;
    });

export function releasePlanQueryOptions(uid: string | null) {
    return queryOptions({
        queryKey: ["user", "release-plan", uid],
        queryFn: () => getReleasePlanFn(),
        enabled: !!uid,
        staleTime: 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}
