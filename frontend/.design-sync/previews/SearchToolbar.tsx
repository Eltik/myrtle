import { SearchToolbar } from "frontend";
import { useState } from "react";

// SearchToolbar is the user search's control block: the "RANK BY" sort picker
// and a direction toggle, a "FILTERS · n" line with Clear, then three filters
// on one row: "Owns operators" (multi picker), "Support unit has" (single
// picker) and "Owns every" (a class/archetype scope). It reads the operator
// index itself (a query the design bundle cannot answer), so the pickers show
// their loading placeholder here: the honest state of a cold page. The
// controls (URL-backed in the product) are local state.

type Scope = { kind: "class"; profession: string } | { kind: "sub"; subProfessionId: string };

function Toolbar({ sort = "score", dir = "desc" as "asc" | "desc", all = null as Scope | null, has = [] as string[], support = "" }) {
    const [state, setState] = useState({ sort, dir, all, has, support });
    const activeFilters = (state.has.length > 0 ? 1 : 0) + (state.support ? 1 : 0) + (state.all ? 1 : 0);
    const controls = {
        ...state,
        activeFilters,
        setSort: (next: string) => setState((s) => ({ ...s, sort: next })),
        toggleDir: () => setState((s) => ({ ...s, dir: s.dir === "desc" ? "asc" : "desc" })),
        setHas: (ids: string[]) => setState((s) => ({ ...s, has: ids })),
        setSupport: (id: string) => setState((s) => ({ ...s, support: id })),
        setAll: (scope: Scope | null) => setState((s) => ({ ...s, all: scope })),
        clearFilters: () => setState((s) => ({ ...s, has: [], support: "", all: null })),
    };
    return (
        <div style={{ width: 820 }}>
            <SearchToolbar controls={controls} />
        </div>
    );
}

/** A cold page: ranked by score, descending, no filters. */
export const Default = () => <Toolbar />;

/** Ranked by how many Guards each player owns, ascending. */
export const RankedByClass = () => <Toolbar sort="class:WARRIOR" dir="asc" />;

/** "Owns every" set to the Bard archetype: one filter active, Clear shown. */
export const OwnsEveryBard = () => <Toolbar all={{ kind: "sub", subProfessionId: "bard" }} />;
