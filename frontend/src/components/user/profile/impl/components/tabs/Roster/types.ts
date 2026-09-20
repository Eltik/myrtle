import type { ISharedFilters } from "#/components/operators/list/impl/types";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";

export type SortKey = "investment" | "level" | "rarity" | "obtained" | "potential" | "trust" | "maxed";
export type SortOrder = "asc" | "desc";
export type OwnershipFilter = "all" | "owned" | "unowned";
/**
 * Where an operator comes from. `headhunting` is the gacha pool (recruitment
 * counts, it draws from the same pool); `welfare` is everything the game hands
 * out instead: event rewards, voucher exchange, IS rewards, the story Amiya.
 * Welfare 5 and 6 stars all reach P6 for free, so a potential review of the
 * paid pool needs them out of the way.
 */
export type SourceFilter = "any" | "headhunting" | "welfare";
export type ViewMode = "detailed" | "compact";

export interface IOwnedEntry extends IRosterEntry {
    isOwned: true;
    meta: IOperatorIndexEntry | null;
    static: IOperatorListItem | null;
    name: string;
    rarity: number;
    voiceActors: string[];
}

export interface IUnownedEntry {
    isOwned: false;
    operator_id: string;
    name: string;
    rarity: number;
    meta: IOperatorIndexEntry;
    static: IOperatorListItem | null;
    voiceActors: string[];
}

export type IDisplayEntry = IOwnedEntry | IUnownedEntry;

export interface IRosterFilterState extends ISharedFilters {
    search: string;
    ownership: OwnershipFilter;
    source: SourceFilter;
    sortBy: SortKey;
    sortOrder: SortOrder;
    viewMode: ViewMode;
}

export const OWNED_ONLY_SORTS: ReadonlySet<SortKey> = new Set(["investment", "level", "obtained", "potential", "trust", "maxed"]);
