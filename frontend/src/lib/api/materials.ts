import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { checkSample, firstValue, materialItemContract } from "#/lib/api/contract";
import { backendFetch } from "#/lib/fetch";

export type { ApSupply as IApSupply } from "#/types/generated/ApSupply";
export type { BuildingProduct as IBuildingProduct } from "#/types/generated/BuildingProduct";
/**
 * Material types - re-exported from the ts-rs bindings generated out of
 * `backend/src/core/gamedata/types/material.rs`.
 *
 * Hand-written until an `ItemClass` variant the enum didn't name (`MEMENTO`, CN
 * only) failed the whole `item_table` and blanked every item name and icon.
 * Generating them means a rename or a new enum variant is a compile error here
 * rather than a silent `undefined` at runtime.
 *
 * Do not add fields here - change the Rust struct and re-run:
 *   cd backend && cargo test export_bindings
 */
export type { BuildingRoomType as IBuildingRoomType } from "#/types/generated/BuildingRoomType";
export type { CharVoucherItem as ICharVoucherItem } from "#/types/generated/CharVoucherItem";
export type { ExpItem as IExpItem } from "#/types/generated/ExpItem";
export type { Item as IMaterialItem } from "#/types/generated/Item";
export type { ItemClass as IItemClass } from "#/types/generated/ItemClass";
export type { ItemOccPer as IItemOccPer } from "#/types/generated/ItemOccPer";
export type { ItemRarity as IItemRarity } from "#/types/generated/ItemRarity";
export type { ItemType as IItemType } from "#/types/generated/ItemType";
export type { StageDrop as IStageDrop } from "#/types/generated/StageDrop";
export type { VoucherDisplayType as IVoucherDisplayType } from "#/types/generated/VoucherDisplayType";
export type { VoucherItemType as IVoucherItemType } from "#/types/generated/VoucherItemType";
export type { VoucherRelate as IVoucherRelate } from "#/types/generated/VoucherRelate";

import type { Item as IMaterialItem } from "#/types/generated/Item";
import type { Materials as IMaterials } from "#/types/generated/Materials";

export type { Materials as IMaterials } from "#/types/generated/Materials";

export const getMaterialsFn = createServerFn({ method: "GET" })
    .inputValidator((server: "en" | "cn") => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(server === "cn" ? "/cn/static/materials" : "/static/materials");
        if (!res.ok) throw new Error(`Failed to load materials: ${res.status}`);
        const payload = (await res.json()) as IMaterials;
        checkSample("static/materials", materialItemContract, firstValue(payload.items));
        return payload;
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
    checkSample("static/materials", materialItemContract, firstValue(materials.items));
    return Object.values(materials.items)
        .filter((i): i is IMaterialItem => i !== undefined)
        .sort((a, b) => a.sortId - b.sortId);
});

export function itemsListQueryOptions() {
    return queryOptions({
        queryKey: ["materials", "list"],
        queryFn: () => getItemsListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
