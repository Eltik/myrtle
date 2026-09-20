import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { extractVoiceActors } from "#/components/operators/list/impl/enrich";
import { buildFilterOptions, countSharedFilters, EMPTY_SHARED_FILTERS, toFilterSets } from "#/components/operators/list/impl/shared-filters";
import type { ArrayFilterKey, ISharedFilters } from "#/components/operators/list/impl/types";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useMediaQuery } from "#/hooks/use-media-query";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import type { IVoices } from "#/types/voices";
import { filterEntries, sortEntries } from "./helpers";
import { type IDisplayEntry, type IRosterFilterState, OWNED_ONLY_SORTS, type ViewMode } from "./types";

const INITIAL: IRosterFilterState = {
    search: "",
    ownership: "owned",
    source: "any",
    ...EMPTY_SHARED_FILTERS,
    sortBy: "rarity",
    sortOrder: "desc",
    viewMode: "detailed",
};

const PAGE_SIZE: Record<ViewMode, number> = { detailed: 24, compact: 48 };

export function useRoster(roster: IRosterEntry[], operatorsIndex: IOperatorIndexEntry[], operatorsStatic: IOperatorListItem[], voices: IVoices | undefined) {
    // This key predates classes and subclasses, so the merge prevents missing arrays
    // from throwing when returning visitors use the new filters.
    // `hadStoredViewMode` records whether the visitor has ever chosen a view
    // mode, so the responsive default below can tell "never picked one" from
    // "picked compact". It is a ref rather than state: it must be readable by
    // the init effect on the same tick the parse ran, and it never re-renders.
    const hadStoredViewMode = useRef(false);
    const [filters, setFilters] = useLocalStorageState<IRosterFilterState>("user:roster:filters", INITIAL, {
        parse: (raw) => {
            const stored = JSON.parse(raw) as Partial<IRosterFilterState> & { rarity?: unknown };
            // Returning visitors carry this deprecated key, so it must not leak into the typed state.
            delete stored.rarity;
            if (stored.viewMode === "detailed" || stored.viewMode === "compact") hadStoredViewMode.current = true;
            return { ...INITIAL, ...stored };
        },
    });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>("user:roster:filters-visible", false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    const toggleFilters = () => setFiltersVisible((v) => !v);
    const isDesktop = useMediaQuery("(min-width: 768px)");

    // The responsive default is a FIRST-VISIT default, not a reset. This ran on
    // every mount and overwrote the stored `viewMode` unconditionally, so any
    // visitor who chose icons got full art back the moment they left the tab
    // and came back (roster -> stats -> roster remounts this hook). It now only
    // fires when localStorage carried no view mode at all.
    const didInit = useRef(false);
    useEffect(() => {
        if (didInit.current) return;
        didInit.current = true;
        if (hadStoredViewMode.current) return;
        setFilters((p) => ({ ...p, viewMode: isDesktop ? "detailed" : "compact" }));
    }, [isDesktop, setFilters]);

    useEffect(() => {
        if (filters.ownership === "unowned" && OWNED_ONLY_SORTS.has(filters.sortBy)) {
            setFilters((p) => ({ ...p, sortBy: "rarity" }));
        }
    }, [filters.ownership, filters.sortBy, setFilters]);

    const indexMap = useMemo(() => {
        const m = new Map<string, IOperatorIndexEntry>();
        for (const op of operatorsIndex) m.set(op.id, op);
        return m;
    }, [operatorsIndex]);

    const filterOptions = useMemo(() => buildFilterOptions(operatorsIndex.map((op) => ({ ...op, voiceActors: extractVoiceActors(op.id, voices) }))), [operatorsIndex, voices]);

    const ownedIds = useMemo(() => new Set(roster.map((r) => r.operator_id)), [roster]);

    const staticMap = useMemo(() => {
        const m = new Map<string, IOperatorListItem>();
        for (const op of operatorsStatic) if (op.id) m.set(op.id, op);
        return m;
    }, [operatorsStatic]);

    const allEntries = useMemo<IDisplayEntry[]>(() => {
        const owned: IDisplayEntry[] = roster.map((r) => {
            const meta = indexMap.get(r.operator_id) ?? null;
            return {
                ...r,
                isOwned: true,
                meta,
                static: staticMap.get(r.operator_id) ?? null,
                name: meta?.name ?? r.operator_id,
                rarity: meta?.rarity ?? 1,
                voiceActors: extractVoiceActors(r.operator_id, voices),
            };
        });
        if (filters.ownership === "owned") return owned;

        const unowned: IDisplayEntry[] = [];
        for (const op of operatorsIndex) {
            if (op.isNotObtainable || ownedIds.has(op.id)) continue;
            // Branch forms (e.g. Amiya Guard `char_1001_amiya2`, Medic
            // `char_1037_amiya3`) are alt forms of a base operator. Ownership
            // follows the base - never show a branch in the "unowned" list.
            const staticOp = staticMap.get(op.id);
            if (staticOp?.tmplDefault && staticOp.tmplDefault !== op.id) continue;
            unowned.push({ isOwned: false, operator_id: op.id, name: op.name, rarity: op.rarity, meta: op, static: staticOp ?? null, voiceActors: extractVoiceActors(op.id, voices) });
        }
        return filters.ownership === "unowned" ? unowned : [...owned, ...unowned];
    }, [roster, operatorsIndex, indexMap, staticMap, ownedIds, filters.ownership, voices]);

    const sets = useMemo(() => toFilterSets(filters), [filters]);
    const filtered = useMemo(() => filterEntries(allEntries, filters.search, sets, filters.source), [allEntries, filters.search, sets, filters.source]);

    const sorted = useMemo(() => sortEntries(filtered, filters.sortBy, filters.sortOrder), [filtered, filters.sortBy, filters.sortOrder]);

    const [displayCount, setDisplayCount] = useState(PAGE_SIZE.detailed);
    useEffect(() => {
        setDisplayCount(Math.min(PAGE_SIZE[filters.viewMode], sorted.length));
    }, [sorted.length, filters.viewMode]);

    const observer = useRef<IntersectionObserver | null>(null);
    const lastRef = useCallback(
        (node: HTMLElement | null) => {
            observer.current?.disconnect();
            if (!node) return;
            observer.current = new IntersectionObserver(
                (entries) => {
                    if (entries[0]?.isIntersecting) {
                        setDisplayCount((n) => (n < sorted.length ? Math.min(n + 24, sorted.length) : n));
                    }
                },
                { rootMargin: "200px" },
            );
            observer.current.observe(node);
        },
        [sorted.length],
    );

    const visible = useMemo(() => sorted.slice(0, displayCount), [sorted, displayCount]);

    const set = useCallback(<K extends keyof IRosterFilterState>(key: K, value: IRosterFilterState[K]) => setFilters((p) => ({ ...p, [key]: value })), [setFilters]);
    const removeFrom = useCallback((key: ArrayFilterKey, value: string) => setFilters((p) => ({ ...p, [key]: p[key].filter((v) => v !== value) })), [setFilters]);
    const setShared = useCallback(<K extends ArrayFilterKey>(key: K, value: ISharedFilters[K]) => setFilters((p) => ({ ...p, [key]: value })), [setFilters]);
    // Ownership, sorting, and view mode define scope or presentation and remain intact.
    // Source is a filter like the notes select on /operators: it counts, chips, and clears.
    const clearFilters = useCallback(() => setFilters((p) => ({ ...p, search: "", source: "any", ...EMPTY_SHARED_FILTERS })), [setFilters]);
    const activeFilterCount = countSharedFilters(filters) + (filters.search ? 1 : 0) + (filters.source !== "any" ? 1 : 0);

    return {
        filters,
        set,
        filtersVisible,
        toggleFilters,
        filterOptions,
        removeFrom,
        setShared,
        clearFilters,
        activeFilterCount,
        hasActiveFilters: activeFilterCount > 0,
        toggleSortOrder: () => set("sortOrder", filters.sortOrder === "asc" ? "desc" : "asc"),
        visible,
        totalCount: sorted.length,
        displayCount,
        lastRef,
    };
}
