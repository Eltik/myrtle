import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

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
