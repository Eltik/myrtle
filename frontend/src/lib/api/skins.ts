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

export const getSkinsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/skins");
    if (!res.ok) throw new Error(`Failed to load skins: ${res.status}`);
    return (await res.json()) as ISkinDataResponse;
});

export function skinsQueryOptions() {
    return queryOptions({
        queryKey: ["skins"],
        queryFn: () => getSkinsFn(),
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
            if (res.status === 404) return null;
            if (res.status === 403) return null;
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
