import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry, IOperatorListItem, OperatorRarityTier } from "#/types/operators";

export type SortKey = "level" | "rarity" | "obtained" | "potential" | "trust" | "maxed";
export type SortOrder = "asc" | "desc";
export type RarityFilter = "all" | OperatorRarityTier;
export type OwnershipFilter = "all" | "owned" | "unowned";
export type ViewMode = "detailed" | "compact";

export interface IOwnedEntry extends IRosterEntry {
    isOwned: true;
    meta: IOperatorIndexEntry | null;
    static: IOperatorListItem | null;
    name: string;
    rarity: number;
}

export interface IUnownedEntry {
    isOwned: false;
    operator_id: string;
    name: string;
    rarity: number;
    meta: IOperatorIndexEntry;
    static: IOperatorListItem | null;
}

export type IDisplayEntry = IOwnedEntry | IUnownedEntry;

export interface IRosterFilterState {
    search: string;
    ownership: OwnershipFilter;
    rarity: RarityFilter;
    sortBy: SortKey;
    sortOrder: SortOrder;
    viewMode: ViewMode;
}

export const OWNED_ONLY_SORTS: ReadonlySet<SortKey> = new Set(["level", "obtained", "potential", "trust", "maxed"]);
