import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useMediaQuery } from "#/hooks/use-media-query";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry } from "#/types/operators";
import { filterEntries, sortEntries } from "./helpers";
import { type IDisplayEntry, type IRosterFilterState, OWNED_ONLY_SORTS, type ViewMode } from "./types";

const INITIAL: IRosterFilterState = {
    search: "",
    ownership: "owned",
    rarity: "all",
    sortBy: "rarity",
    sortOrder: "desc",
    viewMode: "detailed",
};

const PAGE_SIZE: Record<ViewMode, number> = { detailed: 24, compact: 48 };

export function useRoster(roster: IRosterEntry[], operatorsIndex: IOperatorIndexEntry[]) {
    const [filters, setFilters] = useLocalStorageState<IRosterFilterState>("user:roster:filters", INITIAL);
    const isDesktop = useMediaQuery("(min-width: 768px)");

    const didInit = useRef(false);
    useEffect(() => {
        if (didInit.current) return;
        didInit.current = true;
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

    const ownedIds = useMemo(() => new Set(roster.map((r) => r.operator_id)), [roster]);

    const allEntries = useMemo<IDisplayEntry[]>(() => {
        const owned: IDisplayEntry[] = roster.map((r) => {
            const meta = indexMap.get(r.operator_id) ?? null;
            return { ...r, isOwned: true, meta, name: meta?.name ?? r.operator_id, rarity: meta?.rarity ?? 1 };
        });
        if (filters.ownership === "owned") return owned;

        const unowned: IDisplayEntry[] = [];
        for (const op of operatorsIndex) {
            if (op.isNotObtainable || ownedIds.has(op.id)) continue;
            unowned.push({ isOwned: false, operator_id: op.id, name: op.name, rarity: op.rarity, meta: op });
        }
        return filters.ownership === "unowned" ? unowned : [...owned, ...unowned];
    }, [roster, operatorsIndex, indexMap, ownedIds, filters.ownership]);

    const filtered = useMemo(() => filterEntries(allEntries, filters.rarity, filters.search), [allEntries, filters.rarity, filters.search]);

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

    return {
        filters,
        set,
        toggleSortOrder: () => set("sortOrder", filters.sortOrder === "asc" ? "desc" : "asc"),
        visible,
        totalCount: sorted.length,
        displayCount,
        lastRef,
    };
}
