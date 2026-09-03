import { ArrowDown, ArrowUp, Grid3x3, LayoutGrid } from "lucide-react";
import { useMemo } from "react";
import { ActiveFilterChips } from "#/components/operators/list/impl/components/ActiveFilterChips";
import { FilterToggleButton } from "#/components/operators/list/impl/components/FilterToggleButton";
import { buildSharedChips } from "#/components/operators/list/impl/shared-filters";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import type { IRosterEntry } from "#/lib/api/user";
import { capitalize } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import type { IVoices } from "#/types/voices";
import { CompactCard } from "./CompactCard";
import { DetailedCard } from "./DetailedCard";
import { RosterFilters } from "./RosterFilters";
import type { SortKey, ViewMode } from "./types";
import { UnownedCard } from "./UnownedCard";
import { useRoster } from "./useRoster";

interface IRosterTabProps {
    roster: IRosterEntry[];
    operatorsIndex: IOperatorIndexEntry[];
    operatorsStatic: IOperatorListItem[];
    voices?: IVoices;
}

const SORT_LABELS: Record<SortKey, string> = {
    level: "Sort by Level",
    rarity: "Sort by Rarity",
    obtained: "Sort by Obtained",
    potential: "Sort by Potential",
    trust: "Sort by Trust",
    maxed: "Sort by Maxed",
};

export function RosterTab({ roster, operatorsIndex, operatorsStatic, voices }: IRosterTabProps) {
    const { filters, set, toggleSortOrder, visible, totalCount, displayCount, lastRef, filtersVisible, toggleFilters, filterOptions, removeFrom, setShared, clearFilters, activeFilterCount, hasActiveFilters } = useRoster(roster, operatorsIndex, operatorsStatic, voices);
    const { search, ownership, sortBy, sortOrder, viewMode } = filters;
    const activeChips = useMemo(() => buildSharedChips(filters, removeFrom), [filters, removeFrom]);

    return (
        <section className="flex flex-col gap-4" aria-label="Operator roster">
            <div className="relative flex items-start">
                <RosterFilters
                    filters={filters}
                    options={filterOptions}
                    onChange={setShared}
                    ownership={filters.ownership}
                    onOwnershipChange={(v) => set("ownership", v)}
                    onClearAll={clearFilters}
                    hasActiveFilters={hasActiveFilters}
                    collapsed={!filtersVisible}
                    onToggle={toggleFilters}
                    activeFilterCount={activeFilterCount}
                />
                <div className="flex min-w-0 flex-1 flex-col gap-3">
                    <div className="flex w-full flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                        {/* Below sm the toolbar stacks; this row keeps the toggle beside the search box the way /operators does. At sm+ it dissolves so both stay direct toolbar children. */}
                        <div className="flex w-full items-center gap-3 sm:contents">
                            <FilterToggleButton visible={filtersVisible} onToggle={toggleFilters} activeCount={activeFilterCount} />
                            <Input className="w-full sm:w-64 sm:min-w-48 sm:max-w-80 sm:flex-1" onChange={(e) => set("search", e.target.value)} placeholder="Search operators..." value={search} />
                        </div>
                        <Select onValueChange={(value) => value && set("sortBy", value as SortKey)} value={sortBy}>
                            <SelectTrigger className="w-full sm:w-40">
                                <SelectValue placeholder="Sort by">{(value) => SORT_LABELS[value as SortKey] ?? value}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem disabled={ownership === "unowned"} value="level">
                                    Sort by Level
                                </SelectItem>
                                <SelectItem value="rarity">Sort by Rarity</SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="obtained">
                                    Sort by Obtained
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="potential">
                                    Sort by Potential
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="trust">
                                    Sort by Trust
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="maxed">
                                    Sort by Maxed
                                </SelectItem>
                            </SelectContent>
                        </Select>
                        <Button className="w-full sm:w-auto" onClick={toggleSortOrder} variant="outline">
                            <span>{capitalize(sortOrder)}</span>
                            {sortOrder === "asc" ? <ArrowUp /> : <ArrowDown />}
                        </Button>
                        <ToggleGroup
                            aria-label="View mode"
                            className="sm:ml-auto md:bg-secondary/50"
                            onValueChange={(value) => {
                                const next = value[0] as ViewMode | undefined;
                                if (next) set("viewMode", next);
                            }}
                            value={[viewMode]}
                            variant="outline"
                        >
                            <ToggleGroupItem aria-label="Detailed view" value="detailed">
                                <LayoutGrid />
                            </ToggleGroupItem>
                            <ToggleGroupItem aria-label="Compact view" value="compact">
                                <Grid3x3 />
                            </ToggleGroupItem>
                        </ToggleGroup>
                    </div>
                    <ActiveFilterChips chips={activeChips} onClearAll={clearFilters} />
                    <div className={viewMode === "detailed" ? "grid grid-cols-[repeat(auto-fill,minmax(min(100%,280px),1fr))] gap-6" : "grid grid-cols-[repeat(auto-fill,minmax(6.5rem,1fr))] gap-3 sm:grid-cols-[repeat(auto-fill,minmax(9rem,1fr))]"}>
                        {visible.map((entry, i) => {
                            const ref = i === visible.length - 1 ? lastRef : null;
                            const key = entry.isOwned ? entry.operator_id : `unowned-${entry.operator_id}`;
                            if (!entry.isOwned) return <UnownedCard key={key} entry={entry} viewMode={viewMode} lastRef={ref} />;
                            return viewMode === "detailed" ? <DetailedCard key={key} entry={entry} lastRef={ref} /> : <CompactCard key={key} entry={entry} lastRef={ref} />;
                        })}
                    </div>
                    {displayCount < totalCount && (
                        <p className="py-4 text-center text-muted-foreground text-sm">
                            Showing {displayCount} of {totalCount} operators. Scroll to load more.
                        </p>
                    )}
                </div>
            </div>
        </section>
    );
}
