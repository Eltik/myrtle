import { rarityTierToNumber } from "#/components/user/profile/impl/components/tabs/Items/helpers";
import type { IItemEntry } from "#/components/user/profile/impl/components/tabs/Items/types";
import type { IMaterials } from "#/lib/api/materials";
import type { IItemHoldingSummary } from "#/lib/api/user";
import type { ICatalogItem } from "./inventory.types";

/** Join one catalog row with the item table. An id the table does not name
 * still ranks and is shown by its raw id, so a roster ahead of the item
 * table is never hidden. */
export function toCatalogItem(row: IItemHoldingSummary, materials: IMaterials | undefined): ICatalogItem {
    const meta = materials?.items[row.item_id] ?? null;
    return {
        ...row,
        meta,
        name: meta?.name ?? row.item_id,
        rarityNum: rarityTierToNumber(meta?.rarity),
        iconId: meta?.iconId ?? null,
    };
}

/**
 * The catalog row for `itemId`, or a stand-in named from the item table when
 * the catalog has no row for it: nobody visible holds it, or the catalog is
 * still loading. `counts` lets a caller fill the stand-in from another query.
 */
export function resolveCatalogItem(catalog: ICatalogItem[], itemId: string, materials: IMaterials | undefined, counts: { holders: number; top: number } = { holders: 0, top: 0 }): ICatalogItem {
    return catalog.find((c) => c.item_id === itemId) ?? toCatalogItem({ item_id: itemId, ...counts }, materials);
}

/** The shape `ItemIcon` renders; the quantity is not shown there so it is 0. */
export function toIconEntry(item: Pick<ICatalogItem, "item_id" | "name" | "rarityNum" | "iconId" | "meta">): IItemEntry {
    return {
        item_id: item.item_id,
        quantity: 0,
        meta: item.meta,
        name: item.name,
        rarityNum: item.rarityNum,
        category: "other",
        iconId: item.iconId,
        expValue: null,
    };
}
