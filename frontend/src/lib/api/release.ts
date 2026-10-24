import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { bannersDueNow, eventsDueNow, skinsDueNow } from "#/components/tools/release/impl/resolution";
import { backendFetch } from "#/lib/fetch";
import type { BannersResponse } from "#/types/generated/BannersResponse";
import type { EventsResponse } from "#/types/generated/EventsResponse";
import type { LagResponse } from "#/types/generated/LagResponse";
import type { PutOverride } from "#/types/generated/PutOverride";
import type { PutReleasePlan } from "#/types/generated/PutReleasePlan";
import type { ReleaseOverride } from "#/types/generated/ReleaseOverride";
import type { ReleasePlan } from "#/types/generated/ReleasePlan";
import type { SkinsResponse } from "#/types/generated/SkinsResponse";
import { type IBackendStatus, parseError } from "./_shared";
import { requireSiteToken } from "./_shared.server";

const STALE_MS = 60 * 60 * 1000;
const GC_MS = 24 * 60 * 60 * 1000;

// Every page reads release data through these, so an estimate whose day has
// passed turns due-now here, once, against the browser's today. Module-level
// functions keep the select stable, so it reruns only when the data changes.
const selectEvents = (res: EventsResponse): EventsResponse => eventsDueNow(res);
const selectBanners = (res: BannersResponse): BannersResponse => bannersDueNow(res);
const selectSkins = (res: SkinsResponse): SkinsResponse => skinsDueNow(res);

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
        select: selectEvents,
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export function releaseBannersQueryOptions() {
    return queryOptions({
        queryKey: ["release", "banners"],
        queryFn: () => getReleaseBannersFn(),
        select: selectBanners,
        staleTime: STALE_MS,
        gcTime: GC_MS,
    });
}

export function releaseSkinsQueryOptions() {
    return queryOptions({
        queryKey: ["release", "skins"],
        queryFn: () => getReleaseSkinsFn(),
        select: selectSkins,
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

/** Names one override row; the backend keys them on (kind, cnId). */
export interface IReleaseOverrideKey {
    kind: string;
    cnId: string;
}

/** Every manual date override. Super admins only: anyone else gets a 403. */
export const listReleaseOverridesFn = createServerFn({ method: "POST" }).handler(async (): Promise<ReleaseOverride[]> => {
    const token = requireSiteToken();
    const res = await backendFetch("/release/overrides", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as ReleaseOverride[];
});

/** Upsert on (kind, cnId). A full replace: a field left null is cleared, not kept. */
export const putReleaseOverrideFn = createServerFn({ method: "POST" })
    .inputValidator((data: PutOverride) => data)
    .handler(async ({ data }): Promise<ReleaseOverride> => {
        const token = requireSiteToken();
        const res = await backendFetch("/release/overrides", { method: "PUT", bearerToken: token, body: JSON.stringify(data) });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as ReleaseOverride;
    });

export const deleteReleaseOverrideFn = createServerFn({ method: "POST" })
    .inputValidator((data: IReleaseOverrideKey) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/release/overrides/${encodeURIComponent(data.kind)}/${encodeURIComponent(data.cnId)}`, { method: "DELETE", bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });

/**
 * Under the `["release"]` prefix, so a write's one invalidation refetches this list with the rows
 * it moved. The caller gates it on the role: a non-admin fetch is a guaranteed 403.
 */
export function releaseOverridesQueryOptions() {
    return queryOptions({
        queryKey: ["release", "overrides"],
        queryFn: () => listReleaseOverridesFn(),
        staleTime: 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}
