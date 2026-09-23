import { useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { Route } from "#/routes/user.search";
import { DEFAULT_SORT, defaultDir, parseScope, type Scope, type SortDir, scopeToken, splitOperatorIds } from "./searchControls";

export interface ISearchControls {
    sort: string;
    /** The direction in force: the explicit `dir`, else the sort's default. */
    dir: SortDir;
    /** Operator ids every result must own. */
    has: string[];
    /** Operator id that must sit in the support unit, or the empty string. */
    support: string;
    /** Scope every obtainable operator of which must be owned, or null. */
    all: Scope | null;
    /** How many of the three filters are set. */
    activeFilters: number;
    setSort: (sort: string) => void;
    toggleDir: () => void;
    setHas: (ids: string[]) => void;
    setSupport: (id: string) => void;
    setAll: (scope: Scope | null) => void;
    clearFilters: () => void;
}

/**
 * The sort and filter state of the search page lives in the URL so a ranking
 * is a link. Every write resets `page` to 1 and calls `onChange` so the
 * caller's page state follows; a sort change also drops the explicit `dir`,
 * because the new sort has its own default and a flipped direction rarely
 * carries over meaningfully.
 */
export function useSearchControls(onChange: () => void): ISearchControls {
    const search = Route.useSearch();
    const navigate = useNavigate({ from: "/user/search" });

    const patch = useCallback(
        (next: Partial<{ sort: string; dir: SortDir | undefined; has: string; support: string; all: string }>) => {
            onChange();
            navigate({ search: (prev) => ({ ...prev, ...next, page: 1 }), replace: true, resetScroll: false });
        },
        [navigate, onChange],
    );

    const sort = search.sort ?? DEFAULT_SORT;
    const dir = search.dir ?? defaultDir(sort);
    const has = useMemo(() => splitOperatorIds(search.has), [search.has]);
    const all = useMemo(() => parseScope(search.all), [search.all]);
    const support = search.support ?? "";

    const setSort = useCallback((next: string) => patch({ sort: next, dir: undefined }), [patch]);
    const toggleDir = useCallback(() => {
        const flipped: SortDir = dir === "asc" ? "desc" : "asc";
        // The default direction is written as an absence so the URL stays as short as the sort alone.
        patch({ dir: flipped === defaultDir(sort) ? undefined : flipped });
    }, [dir, sort, patch]);
    const setHas = useCallback((ids: string[]) => patch({ has: ids.join(",") }), [patch]);
    const setSupport = useCallback((id: string) => patch({ support: id }), [patch]);
    const setAll = useCallback((scope: Scope | null) => patch({ all: scope ? scopeToken(scope) : "" }), [patch]);
    const clearFilters = useCallback(() => patch({ has: "", support: "", all: "" }), [patch]);

    const activeFilters = (has.length > 0 ? 1 : 0) + (support ? 1 : 0) + (all ? 1 : 0);

    return { sort, dir, has, support, all, activeFilters, setSort, toggleDir, setHas, setSupport, setAll, clearFilters };
}
