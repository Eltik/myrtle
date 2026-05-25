import type { IMaterialItem } from "#/lib/api/materials";
import type { IInventoryItem } from "#/lib/api/user";

export type ItemCategory = "all" | "exp" | "lmd" | "mat" | "skill" | "module" | "chip" | "furniture" | "ticket" | "consume" | "other";

export type ItemSortKey = "rarity" | "qty" | "name" | "category";
export type SortOrder = "asc" | "desc";
export type ItemRarityFilter = "all" | "1" | "2" | "3" | "4" | "5";
export type ItemViewMode = "detailed" | "compact";

export interface IItemEntry extends IInventoryItem {
    meta: IMaterialItem | null;
    name: string;
    rarityNum: number;
    category: ItemCategory;
    iconId: string | null;
    expValue: number | null;
}

export interface IItemFilterState {
    search: string;
    category: ItemCategory;
    rarity: ItemRarityFilter;
    sortBy: ItemSortKey;
    sortOrder: SortOrder;
    viewMode: ItemViewMode;
}
