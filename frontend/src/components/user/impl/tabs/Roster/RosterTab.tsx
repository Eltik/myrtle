import { ArrowDown, ArrowUp, Grid3x3, LayoutGrid } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import type { IRosterEntry } from "#/lib/api/user";
import { capitalize, rarityToNumber } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem, OperatorRarityTier } from "#/types/operators";
import { CompactCard } from "./CompactCard";
import { DetailedCard } from "./DetailedCard";
import type { OwnershipFilter, RarityFilter, SortKey, ViewMode } from "./types";
import { UnownedCard } from "./UnownedCard";
import { useRoster } from "./useRoster";

interface IRosterTabProps {
    roster: IRosterEntry[];
    operatorsIndex: IOperatorIndexEntry[];
    operatorsStatic: IOperatorListItem[];
}

const OWNERSHIP_LABELS: Record<OwnershipFilter, string> = {
    owned: "Owned",
    unowned: "Unowned",
    all: "All Operators",
};

const SORT_LABELS: Record<SortKey, string> = {
    level: "Sort by Level",
    rarity: "Sort by Rarity",
    obtained: "Sort by Obtained",
    potential: "Sort by Potential",
    trust: "Sort by Trust",
};

const RARITY_TIERS: OperatorRarityTier[] = ["TIER_6", "TIER_5", "TIER_4", "TIER_3", "TIER_2", "TIER_1"];

function getRarityLabel(value: RarityFilter): string {
    if (value === "all") return "All Rarities";
    return `${rarityToNumber(value)} Star`;
}

export function RosterTab({ roster, operatorsIndex, operatorsStatic }: IRosterTabProps) {
    const { filters, set, toggleSortOrder, visible, totalCount, displayCount, lastRef } = useRoster(roster, operatorsIndex, operatorsStatic);
    const { search, ownership, sortBy, sortOrder, rarity, viewMode } = filters;

    return (
        <section className="flex flex-col gap-4" aria-label="Operator roster">
            <div className="flex flex-col gap-3">
                <div className="flex w-full flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                    <Input className="w-full sm:w-64 sm:flex-1 sm:min-w-48 sm:max-w-80" onChange={(e) => set("search", e.target.value)} placeholder="Search operators..." value={search} />
                    <Select onValueChange={(value) => value && set("ownership", value as OwnershipFilter)} value={ownership}>
                        <SelectTrigger className="w-full sm:w-40">
                            <SelectValue placeholder="Ownership">{(value) => OWNERSHIP_LABELS[value as OwnershipFilter] ?? value}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="owned">Owned</SelectItem>
                            <SelectItem value="unowned">Unowned</SelectItem>
                            <SelectItem value="all">All Operators</SelectItem>
                        </SelectContent>
                    </Select>
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
                        </SelectContent>
                    </Select>
                    <Select onValueChange={(value) => value && set("rarity", value as RarityFilter)} value={rarity}>
                        <SelectTrigger className="w-full sm:w-36">
                            <SelectValue placeholder="Filter by Rarity">{(value) => getRarityLabel(value as RarityFilter)}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Rarities</SelectItem>
                            {RARITY_TIERS.map((tier) => (
                                <SelectItem key={tier} value={tier}>
                                    {getRarityLabel(tier)}
                                </SelectItem>
                            ))}
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
                {filters.viewMode === "detailed" ? (
                    <div className="grid gap-6 grid-cols-[repeat(auto-fill,minmax(min(100%,280px),1fr))]">
                        {visible.map((entry, i) => {
                            const isLast = i === visible.length - 1;
                            const ref = isLast ? lastRef : null;
                            const key = entry.isOwned ? entry.operator_id : `unowned-${entry.operator_id}`;
                            return entry.isOwned ? <DetailedCard key={key} entry={entry} lastRef={ref} /> : <UnownedCard key={key} entry={entry} viewMode={filters.viewMode} lastRef={ref} />;
                        })}
                    </div>
                ) : (
                    <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 2xl:grid-cols-9">
                        {visible.map((entry, i) => {
                            const isLast = i === visible.length - 1;
                            const ref = isLast ? lastRef : null;
                            const key = entry.isOwned ? entry.operator_id : `unowned-${entry.operator_id}`;

                            return entry.isOwned ? <CompactCard key={key} entry={entry} lastRef={ref} /> : <UnownedCard key={key} entry={entry} viewMode={filters.viewMode} lastRef={ref} />;
                        })}
                    </div>
                )}
                {displayCount < totalCount && (
                    <p className="py-4 text-center text-sm text-muted-foreground">
                        Showing {displayCount} of {totalCount} operators. Scroll to load more.
                    </p>
                )}
            </div>
        </section>
    );
}
