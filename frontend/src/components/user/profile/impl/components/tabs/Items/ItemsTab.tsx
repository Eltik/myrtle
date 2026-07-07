import { useQuery } from "@tanstack/react-query";
import { ArrowDown, ArrowUp, Grid3x3, LayoutGrid, Search } from "lucide-react";
import { Button } from "#/components/ui/button";
import { FilterChip } from "#/components/ui/filter-chip";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useWindowVirtualRows } from "#/hooks/use-window-virtual-rows";
import { materialsQueryOptions } from "#/lib/api/materials";
import type { IInventoryItem } from "#/lib/api/user";
import { capitalize } from "#/lib/utils";
import { CompactCard } from "./CompactCard";
import { DetailedCard } from "./DetailedCard";
import { CATEGORY_LABELS, CATEGORY_ORDER } from "./helpers";
import type { IItemEntry, ItemRarityFilter, ItemSortKey, ItemViewMode } from "./types";
import { useItems } from "./useItems";

interface IItemsTabProps {
    inventory: IInventoryItem[];
}

const SORT_LABELS: Record<ItemSortKey, string> = {
    rarity: "Sort by Rarity",
    qty: "Sort by Quantity",
    name: "Sort by Name",
    category: "Sort by Category",
};

const RARITY_OPTIONS: { value: ItemRarityFilter; label: string }[] = [
    { value: "all", label: "All Rarities" },
    { value: "5", label: "5 Star" },
    { value: "4", label: "4 Star" },
    { value: "3", label: "3 Star" },
    { value: "2", label: "2 Star" },
    { value: "1", label: "1 Star" },
];

const DETAILED_MIN_CARD_WIDTH_PX = 300;
const COMPACT_MIN_CARD_WIDTH_PX = 96;
const DETAILED_GRID_GAP_PX = 16; // gap-4
const COMPACT_GRID_GAP_PX = 10; // gap-2.5
const DETAILED_ROW_ESTIMATE_PX = 200;
const COMPACT_ROW_ESTIMATE_PX = 130;
const COMPACT_CARD_LABEL_PX = 30; // quantity label + padding below a square icon
const ITEMS_VIRTUAL_OVERSCAN = 4;

function getRarityLabel(value: ItemRarityFilter): string {
    return RARITY_OPTIONS.find((o) => o.value === value)?.label ?? value;
}

export function ItemsTab({ inventory }: IItemsTabProps) {
    const { data: materials } = useQuery(materialsQueryOptions());
    const { filters, set, toggleSortOrder, sorted, categoryCounts, totalQty } = useItems(inventory, materials);

    const visibleCategories = CATEGORY_ORDER.filter((c) => c === "all" || (categoryCounts.get(c) ?? 0) > 0);

    return (
        <section aria-label="Inventory items" className="flex flex-col gap-4">
            <div className="flex flex-wrap items-center gap-2">
                {visibleCategories.map((c) => (
                    <FilterChip key={c} label={CATEGORY_LABELS[c]} active={c === filters.category} count={categoryCounts.get(c) ?? 0} onSelect={() => set("category", c)} />
                ))}
            </div>

            <div className="flex w-full flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                <InputGroup className="w-full sm:w-64 sm:min-w-48 sm:max-w-80 sm:flex-1">
                    <InputGroupAddon>
                        <Search />
                    </InputGroupAddon>
                    <InputGroupInput onChange={(e) => set("search", e.target.value)} placeholder="Search items..." value={filters.search} />
                </InputGroup>
                <Select onValueChange={(v) => v && set("sortBy", v as ItemSortKey)} value={filters.sortBy}>
                    <SelectTrigger className="w-full sm:w-48">
                        <SelectValue placeholder="Sort by">{(value) => SORT_LABELS[value as ItemSortKey] ?? value}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {(Object.entries(SORT_LABELS) as [ItemSortKey, string][]).map(([value, label]) => (
                            <SelectItem key={value} value={value}>
                                {label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Select onValueChange={(v) => v && set("rarity", v as ItemRarityFilter)} value={filters.rarity}>
                    <SelectTrigger className="w-full sm:w-36">
                        <SelectValue placeholder="Filter by Rarity">{(value) => getRarityLabel(value as ItemRarityFilter)}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {RARITY_OPTIONS.map((o) => (
                            <SelectItem key={o.value} value={o.value}>
                                {o.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Button className="w-full sm:w-auto" onClick={toggleSortOrder} variant="outline">
                    <span>{capitalize(filters.sortOrder)}</span>
                    {filters.sortOrder === "asc" ? <ArrowUp /> : <ArrowDown />}
                </Button>
                <ToggleGroup
                    aria-label="View mode"
                    className="md:bg-secondary/50"
                    onValueChange={(value) => {
                        const next = value[0] as ItemViewMode | undefined;
                        if (next) set("viewMode", next);
                    }}
                    value={[filters.viewMode]}
                    variant="outline"
                >
                    <ToggleGroupItem aria-label="Detailed view" value="detailed">
                        <LayoutGrid />
                    </ToggleGroupItem>
                    <ToggleGroupItem aria-label="Compact view" value="compact">
                        <Grid3x3 />
                    </ToggleGroupItem>
                </ToggleGroup>
                <div className="hidden items-center gap-3 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em] sm:ml-auto md:flex">
                    <span>
                        <span className="text-foreground tabular-nums">{sorted.length}</span> items
                    </span>
                    <span aria-hidden className="text-border">
                        ·
                    </span>
                    <span>
                        <span className="text-foreground tabular-nums">{totalQty.toLocaleString()}</span> total
                    </span>
                </div>
            </div>

            {sorted.length === 0 ? <EmptyItems hasInventory={inventory.length > 0} /> : <VirtualizedItemsGrid items={sorted} viewMode={filters.viewMode} />}
        </section>
    );
}

function getDetailedColumnCount(width: number): number {
    if (width <= 0) return 1;
    return Math.max(1, Math.floor((width + DETAILED_GRID_GAP_PX) / (DETAILED_MIN_CARD_WIDTH_PX + DETAILED_GRID_GAP_PX)));
}

function getCompactColumnCount(width: number): number {
    if (width <= 0) return 3;
    return Math.max(2, Math.floor((width + COMPACT_GRID_GAP_PX) / (COMPACT_MIN_CARD_WIDTH_PX + COMPACT_GRID_GAP_PX)));
}

function VirtualizedItemsGrid({ items, viewMode }: { items: IItemEntry[]; viewMode: ItemViewMode }) {
    const columnsFor = (width: number) => (viewMode === "detailed" ? getDetailedColumnCount(width) : getCompactColumnCount(width));
    const rowHeightFor = (width: number) => {
        if (viewMode === "detailed") return DETAILED_ROW_ESTIMATE_PX;
        if (width <= 0) return COMPACT_ROW_ESTIMATE_PX;
        const cols = getCompactColumnCount(width);
        const cardWidth = (width - (cols - 1) * COMPACT_GRID_GAP_PX) / cols;
        return Math.round(cardWidth + COMPACT_CARD_LABEL_PX);
    };

    const {
        parentRef,
        width,
        scrollMargin,
        rows: chunks,
        virtualizer,
    } = useWindowVirtualRows<IItemEntry[]>({
        buildRows: (w) => {
            const cols = columnsFor(w);
            const out: IItemEntry[][] = [];
            for (let i = 0; i < items.length; i += cols) out.push(items.slice(i, i + cols));
            return out;
        },
        rowDeps: [items, viewMode],
        estimateSize: (_row, _index, w) => rowHeightFor(w),
        overscan: ITEMS_VIRTUAL_OVERSCAN,
        measureDeps: [viewMode],
        scrollMarginDeps: [viewMode],
    });

    const gridClassName = viewMode === "detailed" ? "grid gap-4" : "grid gap-2.5";
    const gridStyle = { gridTemplateColumns: `repeat(${columnsFor(width)}, minmax(0, 1fr))` };

    return (
        <div ref={parentRef} style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
            {virtualizer.getVirtualItems().map((vi) => {
                const row = chunks[vi.index];
                if (!row) return null;
                return (
                    <div
                        data-index={vi.index}
                        key={vi.key}
                        ref={virtualizer.measureElement}
                        style={{
                            contain: "content",
                            left: 0,
                            position: "absolute",
                            top: 0,
                            transform: `translateY(${vi.start - scrollMargin}px)`,
                            width: "100%",
                            willChange: "transform",
                        }}
                    >
                        <div className={gridClassName} style={gridStyle}>
                            {row.map((item) => (viewMode === "detailed" ? <DetailedCard key={item.item_id} item={item} /> : <CompactCard key={item.item_id} item={item} />))}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

function EmptyItems({ hasInventory }: { hasInventory: boolean }) {
    return (
        <div className="flex flex-col items-center justify-center gap-3 rounded-2xl border border-border bg-card px-8 py-16 text-center">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Inventory</span>
            <h3 className="font-semibold text-lg tracking-tight">{hasInventory ? "No items match" : "No items yet"}</h3>
            <p className="max-w-sm text-muted-foreground text-sm">{hasInventory ? "Try clearing filters or a different category." : "This Doctor's inventory is empty."}</p>
        </div>
    );
}
