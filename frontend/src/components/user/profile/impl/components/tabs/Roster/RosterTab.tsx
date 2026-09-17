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
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import type { IVoices } from "#/types/voices";
import { CompactCard } from "./CompactCard";
import { DetailedCard } from "./DetailedCard";
import { RosterFilters } from "./RosterFilters";
import type { messages } from "./RosterTab.messages";
import type { SortKey, ViewMode } from "./types";
import { UnownedCard } from "./UnownedCard";
import { useRoster } from "./useRoster";

interface IRosterTabProps {
    roster: IRosterEntry[];
    operatorsIndex: IOperatorIndexEntry[];
    operatorsStatic: IOperatorListItem[];
    voices?: IVoices;
}

/** A key in `RosterTab.messages.ts`; resolved by the toolbar below. */
type MessageKey = keyof typeof messages & string;

const SORT_LABELS: Record<SortKey, MessageKey> = {
    investment: "profile.roster.sort.investment",
    level: "profile.roster.sort.level",
    rarity: "profile.roster.sort.rarity",
    obtained: "profile.roster.sort.obtained",
    potential: "profile.roster.sort.potential",
    trust: "profile.roster.sort.trust",
    maxed: "profile.roster.sort.maxed",
};

export function RosterTab({ roster, operatorsIndex, operatorsStatic, voices }: IRosterTabProps) {
    const t: TypedT<typeof messages> = useT("user");
    const { filters, set, toggleSortOrder, visible, totalCount, displayCount, lastRef, filtersVisible, toggleFilters, filterOptions, removeFrom, setShared, clearFilters, activeFilterCount, hasActiveFilters } = useRoster(roster, operatorsIndex, operatorsStatic, voices);
    const { search, ownership, sortBy, sortOrder, viewMode } = filters;
    const activeChips = useMemo(() => buildSharedChips(filters, removeFrom), [filters, removeFrom]);

    return (
        <section className="flex flex-col gap-4" aria-label={t("profile.roster.aria")}>
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
                            <Input className="w-full sm:w-64 sm:min-w-48 sm:max-w-80 sm:flex-1" onChange={(e) => set("search", e.target.value)} placeholder={t("profile.roster.search.placeholder")} value={search} />
                        </div>
                        <Select onValueChange={(value) => value && set("sortBy", value as SortKey)} value={sortBy}>
                            <SelectTrigger className="w-full sm:w-40">
                                <SelectValue placeholder={t("profile.roster.sort.placeholder")}>{(value) => (SORT_LABELS[value as SortKey] ? t(SORT_LABELS[value as SortKey]) : value)}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem disabled={ownership === "unowned"} value="investment">
                                    {t("profile.roster.sort.investment")}
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="level">
                                    {t("profile.roster.sort.level")}
                                </SelectItem>
                                <SelectItem value="rarity">{t("profile.roster.sort.rarity")}</SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="obtained">
                                    {t("profile.roster.sort.obtained")}
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="potential">
                                    {t("profile.roster.sort.potential")}
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="trust">
                                    {t("profile.roster.sort.trust")}
                                </SelectItem>
                                <SelectItem disabled={ownership === "unowned"} value="maxed">
                                    {t("profile.roster.sort.maxed")}
                                </SelectItem>
                            </SelectContent>
                        </Select>
                        <Button className="w-full sm:w-auto" onClick={toggleSortOrder} variant="outline">
                            <span>{sortOrder === "asc" ? t("profile.roster.sort.asc") : t("profile.roster.sort.desc")}</span>
                            {sortOrder === "asc" ? <ArrowUp /> : <ArrowDown />}
                        </Button>
                        <ToggleGroup
                            aria-label={t("profile.roster.viewMode.aria")}
                            className="sm:ml-auto md:bg-secondary/50"
                            onValueChange={(value) => {
                                const next = value[0] as ViewMode | undefined;
                                if (next) set("viewMode", next);
                            }}
                            value={[viewMode]}
                            variant="outline"
                        >
                            <ToggleGroupItem aria-label={t("profile.roster.viewMode.detailed")} value="detailed">
                                <LayoutGrid />
                            </ToggleGroupItem>
                            <ToggleGroupItem aria-label={t("profile.roster.viewMode.compact")} value="compact">
                                <Grid3x3 />
                            </ToggleGroupItem>
                        </ToggleGroup>
                    </div>
                    <ActiveFilterChips chips={activeChips} onClearAll={clearFilters} />
                    {/* Compact below `sm` is THREE fixed columns, not auto-fill. `minmax(6.5rem, 1fr)`
                        needed ~336px of content width for its third track, so a 360px phone (328px of
                        content) got two columns while a 412px one got three: the same page, two
                        different densities, reported in #ui-ux from both ends.

                        Past 440px the row takes as many as fit, because three fixed columns on a
                        600px tablet is the same wasted space in a different shape. The minimum is
                        7.5rem from `sm` up, rather than the old 9rem, because the card FILLS its
                        track now: at 9rem the track stretched to ~162px around a card that stayed
                        134px wide whatever happened, so every row carried a column's worth of dead
                        space. A 700px roster holds five cards where it held four. */}
                    <div className={viewMode === "detailed" ? "grid grid-cols-[repeat(auto-fill,minmax(min(100%,280px),1fr))] gap-6" : "grid grid-cols-3 gap-2 sm:grid-cols-[repeat(auto-fill,minmax(7.5rem,1fr))] sm:gap-3 min-[440px]:grid-cols-[repeat(auto-fill,minmax(6.5rem,1fr))]"}>
                        {visible.map((entry, i) => {
                            const ref = i === visible.length - 1 ? lastRef : null;
                            const key = entry.isOwned ? entry.operator_id : `unowned-${entry.operator_id}`;
                            if (!entry.isOwned) return <UnownedCard key={key} entry={entry} viewMode={viewMode} lastRef={ref} />;
                            return viewMode === "detailed" ? <DetailedCard key={key} entry={entry} lastRef={ref} /> : <CompactCard key={key} entry={entry} lastRef={ref} />;
                        })}
                    </div>
                    {displayCount < totalCount && <p className="py-4 text-center text-muted-foreground text-sm">{t("profile.roster.showing", { shown: displayCount, total: totalCount })}</p>}
                </div>
            </div>
        </section>
    );
}
