import { useQuery } from "@tanstack/react-query";
import { ArrowDown, ArrowUp, Grid3x3, LayoutGrid, Search } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Toggle } from "#/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { materialsQueryOptions } from "#/lib/api/materials";
import type { IInventoryItem } from "#/lib/api/user";
import { capitalize, cn } from "#/lib/utils";
import { CompactCard } from "./CompactCard";
import { DetailedCard } from "./DetailedCard";
import { CATEGORY_LABELS, CATEGORY_ORDER } from "./helpers";
import type { ItemCategory, ItemRarityFilter, ItemSortKey, ItemViewMode } from "./types";
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

function getRarityLabel(value: ItemRarityFilter): string {
    if (value === "all") return "All Rarities";
    return `${value} Star`;
}

export function ItemsTab({ inventory }: IItemsTabProps) {
    const { data: materials } = useQuery(materialsQueryOptions());
    const { filters, set, toggleSortOrder, sorted, categoryCounts, totalQty } = useItems(inventory, materials);

    const visibleCategories = CATEGORY_ORDER.filter((c) => c === "all" || (categoryCounts.get(c) ?? 0) > 0);

    return (
        <section aria-label="Inventory items" className="flex flex-col gap-4">
            <div className="flex flex-wrap items-center gap-2">
                {visibleCategories.map((c) => (
                    <CategoryChip key={c} category={c} active={c === filters.category} count={categoryCounts.get(c) ?? 0} onSelect={() => set("category", c)} />
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
                        <SelectItem value="rarity">Sort by Rarity</SelectItem>
                        <SelectItem value="qty">Sort by Quantity</SelectItem>
                        <SelectItem value="name">Sort by Name</SelectItem>
                        <SelectItem value="category">Sort by Category</SelectItem>
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

            {sorted.length === 0 ? (
                <EmptyItems hasInventory={inventory.length > 0} />
            ) : filters.viewMode === "detailed" ? (
                <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,300px),1fr))] gap-4">
                    {sorted.map((item) => (
                        <DetailedCard key={item.item_id} item={item} />
                    ))}
                </div>
            ) : (
                <div className="grid grid-cols-3 gap-2.5 sm:grid-cols-5 md:grid-cols-7 lg:grid-cols-9 xl:grid-cols-11">
                    {sorted.map((item) => (
                        <CompactCard key={item.item_id} item={item} />
                    ))}
                </div>
            )}
        </section>
    );
}

interface ICategoryChipProps {
    category: ItemCategory;
    active: boolean;
    count: number;
    onSelect: () => void;
}

function CategoryChip({ category, active, count, onSelect }: ICategoryChipProps) {
    return (
        <Toggle
            pressed={active}
            onPressedChange={() => onSelect()}
            variant="outline"
            size="sm"
            className={cn("h-8 gap-2 rounded-full px-3 font-medium text-[13px]", active && "border-primary/50 bg-primary/10 text-foreground shadow-[0_0_0_1px_color-mix(in_srgb,var(--primary)_25%,transparent),0_0_12px_color-mix(in_srgb,var(--primary)_20%,transparent)] hover:bg-primary/15")}
        >
            <span>{CATEGORY_LABELS[category]}</span>
            <Badge size="sm" variant={active ? "default" : "secondary"} className="font-mono tabular-nums">
                {count}
            </Badge>
        </Toggle>
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
