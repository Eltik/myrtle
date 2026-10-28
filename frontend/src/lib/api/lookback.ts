import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/database/models/lookback.rs`. To change a field,
// edit the Rust struct and run `bun run gen:types`.
import type { LookbackBuild } from "#/types/generated/LookbackBuild";
import type { LookbackOverview } from "#/types/generated/LookbackOverview";
import { optionalSiteToken } from "./_shared.server";

export type ILookbackOverview = LookbackOverview;
export type ILookbackBuild = LookbackBuild;

/** A build to read: a snapshot by its `id`, or the live roster. */
export type LookbackBuildRef = number | "current";

export const getLookbackFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string }) => data)
    .handler(async ({ data: { uid } }) => {
        const res = await backendFetch(`/lookback?uid=${encodeURIComponent(uid)}`, { bearerToken: optionalSiteToken() });
        if (!res.ok) {
            // 403 = private profile, Lookback or Roster tab; 404 = no such user.
            if (res.status === 404 || res.status === 403) return null;
            throw new Error(`Failed to load lookback: ${res.status}`);
        }
        return (await res.json()) as ILookbackOverview;
    });

/** The timeline: every snapshot's counts and the reconstructed ownership line. `null` when unavailable to this reader. */
export function lookbackQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "lookback", uid],
        queryFn: () => getLookbackFn({ data: { uid } }),
        staleTime: 5 * 60 * 1000,
        gcTime: 15 * 60 * 1000,
    });
}

export const getLookbackBuildFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; build: LookbackBuildRef }) => data)
    .handler(async ({ data: { uid, build } }) => {
        const path = build === "current" ? "/lookback/current" : `/lookback/snapshots/${build}`;
        const res = await backendFetch(`${path}?uid=${encodeURIComponent(uid)}`, { bearerToken: optionalSiteToken() });
        if (!res.ok) {
            if (res.status === 404 || res.status === 403) return null;
            throw new Error(`Failed to load lookback build: ${res.status}`);
        }
        return (await res.json()) as ILookbackBuild;
    });

/** One snapshot's operators, or the live roster's in the same shape. */
export function lookbackBuildQueryOptions(uid: string, build: LookbackBuildRef) {
    return queryOptions({
        queryKey: ["user", "lookback", uid, "build", build],
        queryFn: () => getLookbackBuildFn({ data: { uid, build } }),
        // A stored snapshot never changes; the live build follows syncs.
        staleTime: build === "current" ? 60 * 1000 : Number.POSITIVE_INFINITY,
        gcTime: 15 * 60 * 1000,
    });
}
