import { useQuery } from "@tanstack/react-query";
import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Download, LayoutGrid, LayoutList, Rows3, Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { ExportDialog } from "#/components/export/ExportDialog";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { noteHasContent, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { voicesQueryOptions } from "#/lib/api/voices";
import { operatorsExportSchema } from "#/lib/export";
import { compactForSearch } from "#/lib/search/fuzzy";
import type { OperatorRarityTier } from "#/types/operators";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../ui/select";
import { Tooltip, TooltipPopup, TooltipTrigger } from "../../ui/tooltip";
import { OperatorCardCompact } from "./impl/components/OperatorCardCompact";
import { OperatorCardGrid } from "./impl/components/OperatorCardGrid";
import { OperatorCardList } from "./impl/components/OperatorCardList";
import { OperatorCardUpcoming } from "./impl/components/OperatorCardUpcoming";
import { OperatorFilters } from "./impl/components/OperatorFilters";
import { Pagination } from "./impl/components/Pagination";
import { CHIP_CONFIG, FILTERS_VISIBLE_KEY, HAS_NOTES_LABELS, ITEMS_PER_PAGE, ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE_OPTIONS, type ItemsPerPage, LIST_GRID_COLS, SORT_OPTIONS, VIEW_MODE_KEY, VIEW_MODES } from "./impl/constants";
import { enrichOperators } from "./impl/enrich";
import type { SortOption, SortOrder, ViewMode } from "./impl/types";
import { useOperatorFilters } from "./impl/useOperatorFilters";

export function OperatorsList() {
    const { data: operators = [] } = useQuery(operatorsListQueryOptions());
    const { data: voices } = useQuery(voicesQueryOptions());
    const { data: notes } = useQuery(operatorNotesListQueryOptions());
    const notedIds = useMemo(() => {
        if (!notes) return undefined;
        return new Set(notes.filter(noteHasContent).map((n) => n.operator_id));
    }, [notes]);
    const enriched = useMemo(() => enrichOperators(operators, voices, notedIds), [operators, voices, notedIds]);

    const {
        filters,
        filterOptions,
        filteredOperators,
        setSearchQuery,
        setClasses,
        setSubclasses,
        setRarities,
        setGenders,
        setNations,
        setFactions,
        setRaces,
        setBirthPlaces,
        setArtists,
        setVoiceActors,
        setHasNotes,
        setAvailability,
        setSortBy,
        setSortOrder,
        removeFrom,
        clearFilters,
        hasActiveFilters,
        activeFilterCount,
    } = useOperatorFilters(enriched);

    const { data: upcoming = [] } = useQuery(upcomingQueryOptions());
    const isUpcoming = filters.availability === "upcoming";

    const upcomingFiltered = useMemo(() => {
        const query = compactForSearch(filters.searchQuery.trim());
        const classes = new Set(filters.classes);
        const subclasses = new Set(filters.subclasses);
        const rarities = new Set(filters.rarities);
        const nations = new Set(filters.nations);
        return upcoming
            .filter((op) => {
                if (query && !compactForSearch(`${op.name} ${op.appellation ?? ""} ${op.subProfessionId}`).includes(query)) return false;
                if (classes.size && !classes.has(op.profession)) return false;
                if (subclasses.size && !subclasses.has(op.subProfessionId)) return false;
                if (rarities.size && !rarities.has(`TIER_${op.rarity}` as OperatorRarityTier)) return false;
                if (nations.size && !nations.has(op.nationId)) return false;
                return true;
            })
            .sort((a, b) => b.rarity - a.rarity || a.name.localeCompare(b.name));
    }, [upcoming, filters.searchQuery, filters.classes, filters.subclasses, filters.rarities, filters.nations]);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
        serialize: (v) => v,
    });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>(FILTERS_VISIBLE_KEY, false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    const toggleFilters = () => setFiltersVisible((v) => !v);

    const [itemsPerPage, setItemsPerPage] = useLocalStorageState<ItemsPerPage>(ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE, {
        parse: (raw) => {
            const num = Number(raw);
            return ITEMS_PER_PAGE_OPTIONS.includes(num as ItemsPerPage) ? (num as ItemsPerPage) : undefined;
        },
        serialize: (v) => String(v),
    });

    const [currentPage, setCurrentPage] = useState(1);
    useEffect(() => {
        setCurrentPage(1);
    }, []);

    const [exportOpen, setExportOpen] = useState(false);

    const totalCount = isUpcoming ? upcomingFiltered.length : filteredOperators.length;
    const totalPages = Math.max(1, Math.ceil(totalCount / itemsPerPage));
    const page = Math.min(currentPage, totalPages);

    const { paginated, upcomingPaginated, fromIndex, toIndex } = useMemo(() => {
        const start = (page - 1) * itemsPerPage;
        const end = page * itemsPerPage;
        return {
            paginated: filteredOperators.slice(start, end),
            upcomingPaginated: upcomingFiltered.slice(start, end),
            fromIndex: totalCount === 0 ? 0 : start + 1,
            toIndex: Math.min(end, totalCount),
        };
    }, [filteredOperators, upcomingFiltered, page, itemsPerPage, totalCount]);

    const activeChips = useMemo(() => {
        const chips = CHIP_CONFIG.flatMap(({ key, prefix, label }) =>
            (filters[key] as string[]).map((v) => ({
                key: `${prefix}-${v}`,
                label: label(v),
                onRemove: () => removeFrom(key, v),
            })),
        );
        if (filters.hasNotes !== "any") {
            chips.push({
                key: `notes-${filters.hasNotes}`,
                label: HAS_NOTES_LABELS[filters.hasNotes],
                onRemove: () => setHasNotes("any"),
            });
        }
        return chips;
    }, [filters, removeFrom, setHasNotes]);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-1.5">
                <nav className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label="Breadcrumb">
                    <span>Collection</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">Operators</span>
                </nav>
                <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">Operators</h1>
                <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">
                    View all <strong className="text-foreground">{operators.length}</strong> operators.
                </p>
            </div>
            <div className="relative flex items-start pt-5">
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
                    selectedHasNotes={filters.hasNotes}
                    selectedAvailability={filters.availability}
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
                    onHasNotesChange={setHasNotes}
                    onAvailabilityChange={(v) => {
                        setAvailability(v);
                        setCurrentPage(1);
                    }}
                    onClearAll={clearFilters}
                    hasActiveFilters={hasActiveFilters}
                    collapsed={!filtersVisible}
                    onToggle={toggleFilters}
                    activeFilterCount={activeFilterCount}
                />

                <main className="flex min-w-0 flex-1 flex-col gap-3.5" aria-label="Operator results">
                    <div className="flex flex-wrap items-center gap-2.5">
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <button
                                        type="button"
                                        className="relative box-border inline-flex h-6 w-6 shrink-0 cursor-pointer appearance-none items-center justify-center rounded-md border border-border bg-[color-mix(in_oklch,var(--secondary)_70%,var(--card))] p-0 font-[inherit] text-muted-foreground shadow-[0_1px_2px_color-mix(in_oklch,var(--foreground)_6%,transparent)] transition-[background-color,border-color,color,box-shadow] duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card hover:text-foreground hover:shadow-[0_2px_8px_color-mix(in_oklch,var(--foreground)_10%,transparent)] focus-visible:border-primary focus-visible:shadow-[0_0_0_3px_color-mix(in_oklch,var(--primary)_24%,transparent)] focus-visible:outline-none motion-reduce:transition-none"
                                        onClick={toggleFilters}
                                        aria-label={filtersVisible ? "Hide filters" : "Show filters"}
                                        aria-expanded={filtersVisible}
                                    />
                                }
                            >
                                {filtersVisible ? <ChevronLeft className="block h-3.5 w-3.5" aria-hidden="true" /> : <ChevronRight className="block h-3.5 w-3.5" aria-hidden="true" />}
                                {!filtersVisible && activeFilterCount > 0 && (
                                    <span className="absolute -top-1.25 -right-1.25 inline-flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-primary px-0.75 font-mono font-semibold text-[9px] text-primary-foreground leading-none shadow-[0_0_0_2px_var(--background)]">{activeFilterCount}</span>
                                )}
                            </TooltipTrigger>
                            <TooltipPopup side="top" sideOffset={8}>
                                Filters
                            </TooltipPopup>
                        </Tooltip>

                        <div className="flex h-10 min-w-60 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] sm:max-w-115 [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                            <Search className="h-3.75 w-3.75" aria-hidden="true" />
                            <input
                                type="text"
                                value={filters.searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                placeholder="Search operators..."
                                aria-label="Search operators"
                                className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground"
                            />
                        </div>

                        {/* biome-ignore lint/a11y/useSemanticElements: role="group" is appropriate for this toggle button group */}
                        <div
                            className="inline-flex h-10 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1 [&>button:not([data-on]):hover]:text-foreground [&>button[data-on]]:bg-primary [&>button[data-on]]:text-primary-foreground [&>button]:inline-flex [&>button]:h-8 [&>button]:w-8 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:border-0 [&>button]:bg-transparent [&>button]:p-0 [&>button]:text-muted-foreground [&>button]:transition-[background-color,color] [&>button]:duration-150"
                            role="group"
                            aria-label="View mode"
                        >
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title="Grid" data-on={viewMode === "grid" || undefined} onClick={() => setViewMode("grid")} aria-pressed={viewMode === "grid"}>
                                            <LayoutGrid className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    Grid
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title="Compact" data-on={viewMode === "compact" || undefined} onClick={() => setViewMode("compact")} aria-pressed={viewMode === "compact"}>
                                            <Rows3 className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    Compact
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title="List" data-on={viewMode === "list" || undefined} onClick={() => setViewMode("list")} aria-pressed={viewMode === "list"}>
                                            <LayoutList className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    List
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        <div className="inline-flex h-10 items-center gap-1 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label="Sort operators">
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Sort</span>
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
                            <button
                                type="button"
                                className="inline-flex h-8 w-8 cursor-pointer appearance-none items-center justify-center rounded-md border-0 bg-transparent p-0 text-muted-foreground transition-[background-color,color] duration-150 hover:bg-secondary hover:text-foreground"
                                title={filters.sortOrder === "asc" ? "Ascending" : "Descending"}
                                onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)}
                                aria-label="Toggle sort direction"
                            >
                                {filters.sortOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" aria-hidden="true" /> : <ArrowDown className="h-3.5 w-3.5" aria-hidden="true" />}
                            </button>
                        </div>

                        <div className="inline-flex h-10 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1">
                            <Select
                                value={String(itemsPerPage)}
                                onValueChange={(v) => {
                                    setItemsPerPage(Number(v) as ItemsPerPage);
                                    setCurrentPage(1);
                                }}
                                aria-label="Items per page"
                            >
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Show</span>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {ITEMS_PER_PAGE_OPTIONS.map((opt) => (
                                        <SelectItem key={opt} value={String(opt)}>
                                            {opt}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <button
                                        type="button"
                                        className="inline-flex h-10 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 font-medium font-sans text-[13px] text-foreground transition-colors hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card"
                                        onClick={() => setExportOpen(true)}
                                        aria-label="Export operators"
                                    >
                                        <Download className="h-3.5 w-3.5" aria-hidden="true" />
                                        <span className="hidden sm:inline">Export</span>
                                    </button>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={8}>
                                Export operators
                            </TooltipPopup>
                        </Tooltip>
                    </div>

                    {activeChips.length > 0 && (
                        <div className="flex flex-wrap items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                            <span className="mr-0.5">Active:</span>
                            {activeChips.map((chip) => (
                                <span
                                    className="inline-flex items-center gap-1.5 rounded-full bg-[color-mix(in_oklch,var(--primary)_10%,transparent)] py-1 pr-1 pl-2.25 font-medium font-sans text-[11.5px] text-primary leading-none [&>button:hover]:bg-[color-mix(in_oklch,var(--primary)_32%,transparent)] [&>button]:inline-flex [&>button]:h-3.75 [&>button]:w-3.75 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-full [&>button]:border-0 [&>button]:bg-[color-mix(in_oklch,var(--primary)_22%,transparent)] [&>button]:p-0"
                                    key={chip.key}
                                >
                                    {chip.label}
                                    <button type="button" onClick={chip.onRemove} aria-label={`Remove ${chip.label}`}>
                                        <X className="h-2 w-2" aria-hidden="true" />
                                    </button>
                                </span>
                            ))}
                            <button type="button" className="cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium font-sans text-[12px] text-muted-foreground leading-none underline underline-offset-[3px] hover:text-foreground" onClick={clearFilters}>
                                Clear all
                            </button>
                        </div>
                    )}

                    <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
                        <span>
                            Showing <strong className="text-foreground">{fromIndex}</strong> to <strong className="text-foreground">{toIndex}</strong> of <strong className="text-foreground">{totalCount}</strong> operators
                        </span>
                        <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] md:inline">Hover for preview · Click to open</span>
                    </div>

                    {totalCount === 0 ? (
                        <div className="rounded-xl border border-border border-dashed bg-card/50 py-16 text-center">
                            <p className="font-sans text-muted-foreground text-sm">{isUpcoming ? "No upcoming operators match your filters." : "No operators match your filters."}</p>
                            <button type="button" onClick={clearFilters} className="mt-3 inline-flex items-center gap-1 font-medium text-[12px] text-primary hover:underline">
                                Clear all filters
                            </button>
                        </div>
                    ) : isUpcoming ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {upcomingPaginated.map((op) => (
                                <OperatorCardUpcoming key={op.id} operator={op} />
                            ))}
                        </div>
                    ) : viewMode === "grid" ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {paginated.map((op) => (
                                <OperatorCardGrid key={op.id} operator={op} />
                            ))}
                        </div>
                    ) : viewMode === "compact" ? (
                        <div className="grid grid-cols-3 gap-1 min-[1080px]:grid-cols-6 min-[1280px]:grid-cols-7 min-[1536px]:grid-cols-8 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5">
                            {paginated.map((op) => (
                                <OperatorCardCompact key={op.id} operator={op} />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col gap-1 pt-1">
                            <div className="grid items-center gap-3 rounded-lg border border-transparent border-b-border/60 px-3 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] max-[900px]:hidden" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                                <span />
                                <span>Name</span>
                                <span className="text-center">Rarity</span>
                                <span className="text-center">Class</span>
                                <span className="text-center">Archetype</span>
                                <span />
                            </div>
                            <div className="flex flex-col gap-1">
                                {paginated.map((op) => (
                                    <OperatorCardList key={op.id} operator={op} />
                                ))}
                            </div>
                        </div>
                    )}

                    <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={setCurrentPage} />
                </main>
            </div>
            <ExportDialog open={exportOpen} onOpenChange={setExportOpen} schema={operatorsExportSchema} allRows={enriched} filteredRows={filteredOperators} pageRows={paginated} title="Operators" />
        </div>
    );
}
