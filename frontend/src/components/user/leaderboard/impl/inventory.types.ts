import type { IMaterialItem } from "#/lib/api/materials";
import type { IItemHoldingSummary } from "#/lib/api/user";

/** An item's numbers without its id: holders, top holding, total held. */
export type IHoldingCounts = Omit<IItemHoldingSummary, "item_id">;

/** The counts of an item nobody visible holds. */
export const NO_HOLDINGS: IHoldingCounts = { holders: 0, top: 0, total_quantity: 0 };

/** A catalog row joined with its game-data record, or `null` meta when the
 * roster holds an id the current item table does not name. */
export interface ICatalogItem extends IItemHoldingSummary {
    meta: IMaterialItem | null;
    name: string;
    rarityNum: number;
    iconId: string | null;
}

/** The item catalog as the page holds it: every held item, joined with the
 * item table, and the visible population the holder counts are taken over.
 * `population` is `null` until the catalog has loaded. */
export interface ICatalog {
    items: ICatalogItem[];
    population: number | null;
}

/** The catalog before anything has loaded. */
export const EMPTY_CATALOG: ICatalog = { items: [], population: null };
