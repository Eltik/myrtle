import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export type IItemRarity = "TIER_1" | "TIER_2" | "TIER_3" | "TIER_4" | "TIER_5" | "TIER_6";

export type IItemClass = "MATERIAL" | "CONSUME" | "NORMAL" | "NONE";

export type IItemOccPer = "USUAL" | "ALMOST" | "ALWAYS" | "SOMETIMES" | "OFTEN";

export type IBuildingRoomType = "WORKSHOP" | "MANUFACTURE";

export type IVoucherItemType = "OPTIONAL_VOUCHER_PICK" | "MATERIAL_ISSUE_VOUCHER";

export type IVoucherDisplayType = "NONE" | "DIVIDE";

export type IItemType = string;

export interface IStageDrop {
    stageId: string;
    occPer: IItemOccPer;
}

export interface IBuildingProduct {
    roomType: IBuildingRoomType;
    formulaId: string;
}

export interface IVoucherRelate {
    voucherId: string;
    voucherItemType: IVoucherItemType;
}

export interface IExpItem {
    id: string;
    gainExp: number;
}

export interface IApSupply {
    id: string;
    ap: number;
    hasTs: boolean;
}

export interface ICharVoucherItem {
    id: string;
    displayType: IVoucherDisplayType;
}

export interface IMaterialItem {
    itemId: string;
    name: string;
    description: string;
    rarity: IItemRarity;
    iconId: string;
    overrideBkg: string | null;
    stackIconId: string | null;
    sortId: number;
    usage: string;
    obtainApproach: string | null;
    hideInItemGet: boolean;
    classifyType: IItemClass;
    itemType: IItemType;
    stageDropList: IStageDrop[];
    buildingProductList: IBuildingProduct[];
    voucherRelateList: IVoucherRelate[] | null;
}

export interface IMaterials {
    items: Record<string, IMaterialItem>;
    expItems: Record<string, IExpItem>;
    potentialItems: Record<string, Record<string, string>>;
    apSupplies: Record<string, IApSupply>;
    charVoucherItems: Record<string, ICharVoucherItem>;
}

export const getMaterialsFn = createServerFn({ method: "GET" })
    .inputValidator((server: "en" | "cn") => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(server === "cn" ? "/cn/static/materials" : "/static/materials");
        if (!res.ok) throw new Error(`Failed to load materials: ${res.status}`);
        return (await res.json()) as IMaterials;
    });

export function materialsQueryOptions(server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["materials", server],
        queryFn: () => getMaterialsFn({ data: server }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getItemsListFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/materials");
    if (!res.ok) throw new Error(`Failed to load materials: ${res.status}`);
    const materials = (await res.json()) as IMaterials;
    return Object.values(materials.items).sort((a, b) => a.sortId - b.sortId);
});

export function itemsListQueryOptions() {
    return queryOptions({
        queryKey: ["materials", "list"],
        queryFn: () => getItemsListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
