import { useCallback, useEffect, useMemo, useRef } from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useMediaQuery } from "#/hooks/use-media-query";
import type { IMaterials } from "#/lib/api/materials";
import type { IInventoryItem } from "#/lib/api/user";
import { compactForSearch } from "#/lib/search/fuzzy";
import { categorizeItem, rarityTierToNumber } from "./helpers";
import type { IItemEntry, IItemFilterState } from "./types";

const INITIAL: IItemFilterState = {
    search: "",
    category: "all",
    rarity: "all",
    sortBy: "rarity",
    sortOrder: "desc",
    viewMode: "detailed",
};

export function useItems(inventory: IInventoryItem[], materials: IMaterials | undefined) {
    const [filters, setFilters] = useLocalStorageState<IItemFilterState>("user:items:filters", INITIAL);
    const isDesktop = useMediaQuery("(min-width: 768px)");

    const didInit = useRef(false);
    useEffect(() => {
        if (didInit.current) return;
        didInit.current = true;
        setFilters((p) => ({ ...p, viewMode: isDesktop ? "detailed" : "compact" }));
    }, [isDesktop, setFilters]);

    const expIds = useMemo(() => {
        const s = new Set<string>();
        if (materials) for (const id of Object.keys(materials.expItems)) s.add(id);
        return s;
    }, [materials]);

    const entries = useMemo<IItemEntry[]>(() => {
        return inventory.map((item) => {
            const meta = materials?.items[item.item_id] ?? null;
            return {
                ...item,
                meta,
                name: meta?.name ?? item.item_id,
                rarityNum: rarityTierToNumber(meta?.rarity),
                category: categorizeItem(meta, expIds),
                iconId: meta?.iconId ?? null,
                expValue: materials?.expItems[item.item_id]?.gainExp ?? null,
            };
        });
    }, [inventory, materials, expIds]);

    const categoryCounts = useMemo(() => {
        const map = new Map<string, number>();
        map.set("all", entries.length);
        for (const e of entries) map.set(e.category, (map.get(e.category) ?? 0) + 1);
        return map;
    }, [entries]);

    const filtered = useMemo(() => {
        let list = entries;
        if (filters.category !== "all") list = list.filter((e) => e.category === filters.category);
        if (filters.rarity !== "all") list = list.filter((e) => String(e.rarityNum) === filters.rarity);
        const q = compactForSearch(filters.search);
        if (q) list = list.filter((e) => compactForSearch(e.name).includes(q) || compactForSearch(e.item_id).includes(q));
        return list;
    }, [entries, filters.category, filters.rarity, filters.search]);

    const sorted = useMemo(() => {
        const arr = filtered.slice();
        const dir = filters.sortOrder === "asc" ? 1 : -1;
        arr.sort((a, b) => {
            let cmp = 0;
            switch (filters.sortBy) {
                case "rarity":
                    cmp = a.rarityNum - b.rarityNum;
                    if (cmp === 0) cmp = a.quantity - b.quantity;
                    break;
                case "qty":
                    cmp = a.quantity - b.quantity;
                    break;
                case "name":
                    cmp = a.name.localeCompare(b.name);
                    break;
                case "category":
                    cmp = a.category.localeCompare(b.category);
                    if (cmp === 0) cmp = b.rarityNum - a.rarityNum;
                    break;
            }
            return cmp === 0 ? a.item_id.localeCompare(b.item_id) : cmp * dir;
        });
        return arr;
    }, [filtered, filters.sortBy, filters.sortOrder]);

    const set = useCallback(<K extends keyof IItemFilterState>(key: K, value: IItemFilterState[K]) => setFilters((p) => ({ ...p, [key]: value })), [setFilters]);
    const toggleSortOrder = useCallback(() => setFilters((p) => ({ ...p, sortOrder: p.sortOrder === "asc" ? "desc" : "asc" })), [setFilters]);

    return { filters, set, toggleSortOrder, entries, sorted, categoryCounts, totalQty: useMemo(() => sorted.reduce((s, i) => s + i.quantity, 0), [sorted]) };
}
