import { compactForSearch } from "#/lib/search/fuzzy";
import { matchesSharedFilters, toFilterSets } from "./shared-filters";
import type { IFilterState, IOperatorView } from "./types";

export type OperatorFilterInput = Omit<IFilterState, "availability" | "sortBy" | "sortOrder">;

/**
 * The /operators page's whole filter predicate: search, every shared array filter, and notes.
 * Both the Global list and the CN-only ("upcoming") list run through it, so a filter added to the
 * panel cannot apply to one list and silently pass everything on the other.
 */
export function operatorFilterPredicate(filters: OperatorFilterInput): (op: IOperatorView) => boolean {
    const query = compactForSearch(filters.searchQuery.trim());
    const sets = toFilterSets(filters);
    return (op) => {
        if (query && !compactForSearch(`${op.name} ${op.appellation ?? ""} ${op.subProfessionId}`).includes(query)) return false;
        if (!matchesSharedFilters(op, sets)) return false;
        if (filters.hasNotes === "yes" && !op.hasNotes) return false;
        if (filters.hasNotes === "no" && op.hasNotes) return false;
        return true;
    };
}
