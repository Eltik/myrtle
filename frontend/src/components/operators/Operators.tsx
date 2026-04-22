import { operatorsListQueryOptions } from "#/lib/api/operators";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { useOperatorFilters } from "./impl/useOperatorFilters";
import { voicesQueryOptions } from "#/lib/api/voices";
import { enrichOperators } from "./impl/enrich";
import type { SortOption, SortOrder, ViewMode } from "./impl/types";
import { CHIP_CONFIG, FILTERS_VISIBLE_KEY, ITEMS_PER_PAGE, SORT_OPTIONS, VIEW_MODE_KEY, VIEW_MODES } from "./impl/constants";
import { cn } from "#/lib/utils";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, LayoutGrid, LayoutList, Rows3, Search } from "lucide-react";
import { OperatorFilters } from "./impl/components/OperatorFilters";
import { Tooltip, TooltipPopup, TooltipTrigger } from "../ui/tooltip";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";

export function OperatorsList() {
    const { data: operators = [] } = useQuery(operatorsListQueryOptions());
    const { data: voices } = useQuery(voicesQueryOptions());
    const enriched = useMemo(() => enrichOperators(operators, voices), [operators, voices]);

    const { filters, filterOptions, filteredOperators, setSearchQuery, setClasses, setSubclasses, setRarities, setGenders, setNations, setFactions, setRaces, setBirthPlaces, setArtists, setVoiceActors, setSortBy, setSortOrder, removeFrom, clearFilters, hasActiveFilters, activeFilterCount } =
        useOperatorFilters(enriched);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
        serialize: (v) => v,
    });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>(FILTERS_VISIBLE_KEY, false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    const toggleFilters = () => setFiltersVisible((v) => !v);

    const [currentPage, setCurrentPage] = useState(1);
    useEffect(() => {
        setCurrentPage(1);
    }, [filters]);

    const totalPages = Math.max(1, Math.ceil(filteredOperators.length / ITEMS_PER_PAGE));
    const page = Math.min(currentPage, totalPages);

    const { paginated, fromIndex, toIndex } = useMemo(() => {
        const start = (page - 1) * ITEMS_PER_PAGE;
        const end = page * ITEMS_PER_PAGE;
        return {
            paginated: filteredOperators.slice(start, end),
            fromIndex: filteredOperators.length === 0 ? 0 : start + 1,
            toIndex: Math.min(end, filteredOperators.length),
        };
    }, [filteredOperators, page]);

    const activeChips = useMemo(
        () =>
            CHIP_CONFIG.flatMap(({ key, prefix, label }) =>
                (filters[key] as string[]).map((v) => ({
                    key: `${prefix}-${v}`,
                    label: label(v),
                    onRemove: () => removeFrom(key, v),
                })),
            ),
        [filters, removeFrom],
    );

    return (
        <div className="operators-page-wrap">
            <div className="page-head">
                <nav className="crumbs" aria-label="Breadcrumb">
                    <span>Collection</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="here">Operators</span>
                </nav>
                <h1>Operators</h1>
                <p className="meta">
                    View all the Operators in Arknights. <strong className="text-foreground">{operators.length}</strong> in your roster.
                </p>
            </div>
            <div className="operators-layout">
                <OperatorFilters
                    selectedClasses={filters.classes}
                    selectedSubclasses={filters.subclasses}
                    selectedRarities={filters.rarities}
                    selectedGenders={filters.genders}
                    selectedNations={filters.nations}
                    selectedFactions={filters.factions}
                    selectedRaces={filters.races}
                    selectedBirthPlaces={filters.birthPlaces}
                    selectedArtists={filters.artists}
                    selectedVoiceActors={filters.voiceActors}
                    options={filterOptions}
                    onClassesChange={setClasses}
                    onSubclassesChange={setSubclasses}
                    onRaritiesChange={setRarities}
                    onGendersChange={setGenders}
                    onNationsChange={setNations}
                    onFactionsChange={setFactions}
                    onRacesChange={setRaces}
                    onBirthPlacesChange={setBirthPlaces}
                    onArtistsChange={setArtists}
                    onVoiceActorsChange={setVoiceActors}
                    onClearAll={clearFilters}
                    hasActiveFilters={hasActiveFilters}
                    collapsed={!filtersVisible}
                />
                <Tooltip>
                    <TooltipTrigger render={<button type="button" className={cn("rail-toggle", !filtersVisible && "rail-toggle--collapsed")} onClick={toggleFilters} aria-label={filtersVisible ? "Hide filters" : "Show filters"} aria-expanded={filtersVisible} />}>
                        {filtersVisible ? <ChevronLeft className="h-3.5 w-3.5" aria-hidden="true" /> : <ChevronRight className="h-3.5 w-3.5" aria-hidden="true" />}
                        {!filtersVisible && activeFilterCount > 0 && <span className="rail-toggle-count">{activeFilterCount}</span>}
                    </TooltipTrigger>
                    <TooltipPopup side="right" sideOffset={8}>
                        Filters
                    </TooltipPopup>
                </Tooltip>

                <main className="results" aria-label="Operator results">
                    <div className="toolbar">
                        <div className="search">
                            <Search className="h-3.75 w-3.75" aria-hidden="true" />
                            <input type="text" value={filters.searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search operators..." aria-label="Search operators" />
                        </div>

                        <div className="seg" role="group" aria-label="View mode">
                            <button type="button" title="Grid" className={cn(viewMode === "grid" && "on")} onClick={() => setViewMode("grid")} aria-pressed={viewMode === "grid"}>
                                <LayoutGrid className="h-4 w-4" aria-hidden="true" />
                            </button>
                            <button type="button" title="Compact" className={cn(viewMode === "compact" && "on")} onClick={() => setViewMode("compact")} aria-pressed={viewMode === "compact"}>
                                <Rows3 className="h-4 w-4" aria-hidden="true" />
                            </button>
                            <button type="button" title="List" className={cn(viewMode === "list" && "on")} onClick={() => setViewMode("list")} aria-pressed={viewMode === "list"}>
                                <LayoutList className="h-4 w-4" aria-hidden="true" />
                            </button>
                        </div>

                        <div className="sortpick">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label="Sort operators">
                                <SelectTrigger size="sm" className="sortpick-trigger">
                                    <span className="sortpick-trigger-kicker">Sort</span>
                                    <SelectValue>{(value) => SORT_OPTIONS.find((o) => o.value === value)?.label ?? value}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {SORT_OPTIONS.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {opt.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <button type="button" className="dir" title={filters.sortOrder === "asc" ? "Ascending" : "Descending"} onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)} aria-label="Toggle sort direction">
                                {filters.sortOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" aria-hidden="true" /> : <ArrowDown className="h-3.5 w-3.5" aria-hidden="true" />}
                            </button>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    );
}
