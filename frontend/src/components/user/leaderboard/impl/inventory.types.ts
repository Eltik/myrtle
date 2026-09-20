import type { IMaterialItem } from "#/lib/api/materials";
import type { IItemHoldingSummary } from "#/lib/api/user";

/** A catalog row joined with its game-data record, or `null` meta when the
 * roster holds an id the current item table does not name. */
export interface ICatalogItem extends IItemHoldingSummary {
    meta: IMaterialItem | null;
    name: string;
    rarityNum: number;
    iconId: string | null;
}
