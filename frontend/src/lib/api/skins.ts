import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { optionalSiteToken } from "./_shared.server";

export interface IDisplaySkin {
    skinName: string | null;
    colorList: string[];
    titleList: string[];
    modelName: string;
    drawerList: string[];
    designerList: string[] | null;
    skinGroupId: string;
    skinGroupName: string;
    skinGroupSortIndex: number;
    content: string;
    dialog: string | null;
    usage: string | null;
    description: string | null;
    obtainApproach: string | null;
    sortId: number;
    displayTagId: string | null;
    getTime: number;
    onYear: number;
    onPeriod: number;
}

export interface ISkin {
    skinId: string;
    charId: string;
    /** Template id (form) the skin belongs to. For Amiya, `charId` is always
     *  the base `char_002_amiya` and `tmplId` identifies the specific form. */
    tmplId: string | null;
    illustId: string;
    portraitId: string;
    avatarId: string;
    buildingId: string | null;
    isBuySkin: boolean;
    voiceId: string | null;
    voiceType: string;
    displaySkin: IDisplaySkin;
}

export interface ISkinDataResponse {
    charSkins: Record<string, ISkin>;
}

/** The slim per-skin projection served by `GET /skins/index` - exactly the field
 *  set the profile Stats tab and its skin-collection dialog read, over ALL skins. */
export interface ISkinIndexDisplay {
    skinName: string | null;
    skinGroupId: string;
    skinGroupName: string;
    skinGroupSortIndex: number;
    displayTagId: string | null;
    getTime: number;
    sortId: number;
    description: string | null;
    content: string;
    dialog: string | null;
    usage: string | null;
    obtainApproach: string | null;
    designerList: string[] | null;
    drawerList: string[];
}

export interface ISkinIndexEntry {
    skinId: string;
    charId: string;
    displaySkin: ISkinIndexDisplay;
}

export type ISkinIndex = Record<string, ISkinIndexEntry>;

export const getSkinsIndexFn = createServerFn({ method: "GET" })
    .inputValidator((server: "en" | "cn") => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(server === "cn" ? "/cn/skins/index" : "/skins/index");
        if (!res.ok) throw new Error(`Failed to load skins index: ${res.status}`);
        return (await res.json()) as ISkinIndex;
    });

export function skinsIndexQueryOptions(server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["skins", "index", server],
        queryFn: () => getSkinsIndexFn({ data: server }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** One operator's skins (a few KB) instead of the entire skins table. */
export const getOperatorSkinsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server: "en" | "cn" }) => data)
    .handler(async ({ data: { id, server } }) => {
        const prefix = server === "cn" ? "/cn" : "";
        const res = await backendFetch(`${prefix}/skins/${encodeURIComponent(id)}`);
        if (!res.ok) throw new Error(`Failed to load operator skins: ${res.status}`);
        return (await res.json()) as ISkinDataResponse;
    });

export function operatorSkinsQueryOptions(id: string, server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["skins", "operator", server, id],
        queryFn: () => getOperatorSkinsFn({ data: { id, server } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export interface IOwnedSkin {
    skin_id: string;
    obtained_at: number | null;
}

export const getUserSkinsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/user-skins?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            // Absent (404) or private (403) rosters are an empty state, not an error.
            if (res.status === 404 || res.status === 403) return null;
            throw new Error(`Failed to load owned skins: ${res.status}`);
        }
        return (await res.json()) as IOwnedSkin[];
    });

export function userSkinsQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "skins", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserSkinsFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface ISkinPopularity {
    totalUsers: number;
    counts: Record<string, number>;
    computedAt: string;
}

export const getSkinPopularityFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/skins/popularity");
    if (!res.ok) throw new Error(`Failed to load skin popularity: ${res.status}`);
    return (await res.json()) as ISkinPopularity;
});

export function skinPopularityQueryOptions() {
    return queryOptions({
        queryKey: ["skins", "popularity"],
        queryFn: () => getSkinPopularityFn(),
        staleTime: 30 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}
