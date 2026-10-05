import type { ITierEntity } from "#/lib/api/tier-entities";
import { compactForSearch } from "#/lib/search/fuzzy";
import type { IPoolFacet } from "./poolKinds";

// How a pool's search and facet selections narrow a catalogue, shared by the
// tier-list pool (`KindPool`) and the grid picker.

/** The options chosen in each facet row, by facet id. An absent or empty row filters nothing. */
export type FacetSelection = Record<string, string[]>;

/** Whether `entity` passes every facet row with a selection: its value is one of the chosen options. An entity with no value for a row never passes it. */
export function matchesFacets(entity: ITierEntity, facets: readonly IPoolFacet[], selected: FacetSelection): boolean {
    for (const facet of facets) {
        const want = selected[facet.id];
        if (want && want.length > 0 && !want.includes(facet.valueOf(entity) ?? "")) return false;
    }
    return true;
}

/** Whether any facet row has a selection. */
export function anyFacetSelected(selected: FacetSelection): boolean {
    return Object.values(selected).some((v) => v.length > 0);
}

/** Whether `q`, already through `compactForSearch`, is blank or found in one of `texts`. A `null` text never matches. */
export function matchesSearch(texts: readonly (string | null)[], q: string): boolean {
    return q.length === 0 || texts.some((text) => Boolean(text) && compactForSearch(text ?? "").includes(q));
}
