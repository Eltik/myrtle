"use client";

import { Grid3X3, LayoutList, Search, Settings2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { cn } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { CLASSES, GENDERS, ITEMS_PER_PAGE, RARITIES } from "./constants";
import { useOperatorFilters } from "./hooks";
import { OperatorCard } from "./operator-card";
import { OperatorFilters } from "./operator-filters";
import { Pagination } from "./ui/impl/pagination";
import { ResponsiveFilterContainer } from "./ui/impl/responsive-filter-container";

// Check if we're on mobile (matches Tailwind's md breakpoint)
function getInitialViewMode(): "grid" | "list" {
    if (typeof window === "undefined") return "grid";
    return window.innerWidth < 768 ? "list" : "grid";
}

// Get initial list columns from localStorage
function getInitialListColumns(): number {
    if (typeof window === "undefined") return 2;
    const saved = localStorage.getItem("operatorListColumns");
    return saved ? Number.parseInt(saved, 10) : 2;
}

export function OperatorsList({ data }: { data: OperatorFromList[] }) {
    // UI state - default to list view on mobile
    const [showFilters, setShowFilters] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [listColumns, setListColumns] = useState(2);

    // Set initial view mode and list columns based on screen size (runs once on mount)
    useEffect(() => {
        setViewMode(getInitialViewMode());
        setListColumns(getInitialListColumns());
    }, []);

    // Persist list columns to localStorage
    const handleListColumnsChange = useCallback((value: string) => {
        const cols = Number.parseInt(value, 10);
        setListColumns(cols);
        localStorage.setItem("operatorListColumns", value);
    }, []);
    const [currentPage, setCurrentPage] = useState(1);
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

    // Filter state from custom hook
    const {
        filters,
        filterOptions,
        filteredOperators,
        setSearchQuery,
        setSelectedClasses,
        setSelectedSubclasses,
        setSelectedRarities,
        setSelectedBirthPlaces,
        setSelectedNations,
        setSelectedFactions,
        setSelectedGenders,
        setSelectedRaces,
        setSelectedArtists,
        setSortBy,
        setSortOrder,
        clearFilters,
        activeFilterCount,
        hasActiveFilters,
    } = useOperatorFilters(data);

    // Pagination
    const totalPages = Math.ceil(filteredOperators.length / ITEMS_PER_PAGE);
    const paginatedOperators = filteredOperators.slice((currentPage - 1) * ITEMS_PER_PAGE, currentPage * ITEMS_PER_PAGE);

    const handlePageChange = (page: number) => {
        setCurrentPage(page);
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const handleClearFilters = () => {
        clearFilters();
        setCurrentPage(1);
    };

    const handleSearchChange = useCallback(
        (query: string) => {
            setSearchQuery(query);
            setCurrentPage(1);
        },
        [setSearchQuery],
    );

    const hoverHandlers = useMemo(() => {
        const handlers = new Map<string, (isOpen: boolean) => void>();
        for (const operator of paginatedOperators) {
            const operatorId = operator.id ?? "";
            handlers.set(operatorId, (isOpen: boolean) => {
                if (isOpen) {
                    setHoveredOperator(operatorId);
                    setIsGrayscaleActive(true);
                } else {
                    setHoveredOperator((current) => (current === operatorId ? null : current));
                    setIsGrayscaleActive(false);
                }
            });
        }
        return handlers;
    }, [paginatedOperators]);

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <h1 className="font-bold text-3xl text-foreground md:text-4xl">Operators</h1>
                <p className="text-muted-foreground">View all the Operators in Arknights</p>
            </div>

            {/* Search and Controls */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="relative max-w-md flex-1">
                    <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                    <input
                        className="h-10 w-full rounded-lg border border-border bg-secondary/50 pr-4 pl-10 text-foreground text-sm transition-colors placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                        onChange={(e) => handleSearchChange(e.target.value)}
                        placeholder="Search operators..."
                        type="text"
                        value={filters.searchQuery}
                    />
                </div>

                <div className="flex items-center gap-2">
                    {/* View Toggle */}
                    <div className="flex items-center rounded-lg border border-border bg-secondary/50 p-1">
                        <button className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "grid" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("grid")} type="button">
                            <Grid3X3 className="h-4 w-4" />
                        </button>
                        <button className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "list" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("list")} type="button">
                            <LayoutList className="h-4 w-4" />
                        </button>
                    </div>

                    {/* List Columns Selector (desktop only, when in list view) */}
                    {viewMode === "list" && (
                        <div className="hidden items-center gap-1.5 md:flex">
                            <Settings2 className="h-4 w-4 text-muted-foreground" />
                            <Select onValueChange={handleListColumnsChange} value={listColumns.toString()}>
                                <SelectTrigger className="h-8 w-20">
                                    <SelectValue placeholder="Cols" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="1">1 col</SelectItem>
                                    <SelectItem value="2">2 cols</SelectItem>
                                    <SelectItem value="3">3 cols</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    )}

                    {/* Filter Toggle - Responsive: Dialog on mobile, Popover on desktop */}
                    <ResponsiveFilterContainer activeFilterCount={activeFilterCount} hasActiveFilters={hasActiveFilters} onOpenChange={setShowFilters} open={showFilters}>
                        <OperatorFilters
                            artists={filterOptions.artists}
                            birthPlaces={filterOptions.birthPlaces}
                            classes={[...CLASSES]}
                            factions={filterOptions.factions}
                            genders={[...GENDERS]}
                            nations={filterOptions.nations}
                            onArtistChange={setSelectedArtists}
                            onBirthPlaceChange={setSelectedBirthPlaces}
                            onClassChange={setSelectedClasses}
                            onClearFilters={handleClearFilters}
                            onFactionChange={setSelectedFactions}
                            onGenderChange={setSelectedGenders}
                            onNationChange={setSelectedNations}
                            onRaceChange={setSelectedRaces}
                            onRarityChange={setSelectedRarities}
                            onSortByChange={setSortBy}
                            onSortOrderChange={setSortOrder}
                            onSubclassChange={setSelectedSubclasses}
                            races={filterOptions.races}
                            rarities={[...RARITIES]}
                            selectedArtists={filters.selectedArtists}
                            selectedBirthPlaces={filters.selectedBirthPlaces}
                            selectedClasses={filters.selectedClasses}
                            selectedFactions={filters.selectedFactions}
                            selectedGenders={filters.selectedGenders}
                            selectedNations={filters.selectedNations}
                            selectedRaces={filters.selectedRaces}
                            selectedRarities={filters.selectedRarities}
                            selectedSubclasses={filters.selectedSubclasses}
                            sortBy={filters.sortBy}
                            sortOrder={filters.sortOrder}
                            subclasses={filterOptions.subclasses}
                        />
                    </ResponsiveFilterContainer>
                </div>
            </div>

            {/* Results Count */}
            <div className="flex items-center justify-between text-muted-foreground text-sm">
                <span>
                    Showing {(currentPage - 1) * ITEMS_PER_PAGE + 1} to {Math.min(currentPage * ITEMS_PER_PAGE, filteredOperators.length)} of {filteredOperators.length} operators
                </span>
            </div>

            {/* List View Header - only show for single column layout */}
            {viewMode === "list" && paginatedOperators.length > 0 && listColumns === 1 && (
                <div className="hidden items-center gap-3 border-border/50 border-b px-3 pb-2 text-muted-foreground text-xs uppercase tracking-wider md:flex">
                    <div className="w-12 shrink-0" />
                    <div className="min-w-0 flex-1">Name</div>
                    <div className="w-24 shrink-0">Rarity</div>
                    <div className="w-32 shrink-0">Class</div>
                    <div className="hidden w-40 shrink-0 lg:block">Archetype</div>
                    <div className="hidden w-8 shrink-0 text-center xl:block">Faction</div>
                </div>
            )}

            {/* Operators Grid/List */}
            {paginatedOperators.length > 0 ? (
                <div className={cn(viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:gap-4" : cn("grid gap-1", listColumns === 1 ? "grid-cols-1" : listColumns === 2 ? "grid-cols-1 md:grid-cols-2" : "grid-cols-1 md:grid-cols-2 lg:grid-cols-3"), "contain-layout")}>
                    {paginatedOperators.map((operator) => {
                        const operatorId = operator.id ?? "";
                        const isCurrentlyHovered = hoveredOperator === operatorId;
                        const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;

                        return <OperatorCard isHovered={isCurrentlyHovered} key={operatorId} listColumns={listColumns} onHoverChange={hoverHandlers.get(operatorId)} operator={operator} shouldGrayscale={shouldGrayscale} viewMode={viewMode} />;
                    })}
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center py-16 text-center">
                    <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                        <Search className="h-8 w-8 text-muted-foreground" />
                    </div>
                    <h3 className="mb-2 font-semibold text-foreground text-lg">No operators found</h3>
                    <p className="mb-4 max-w-sm text-muted-foreground text-sm">Try adjusting your search or filter criteria to find what you're looking for.</p>
                    <button className="text-primary text-sm hover:underline" onClick={handleClearFilters} type="button">
                        Clear all filters
                    </button>
                </div>
            )}

            {/* Pagination */}
            <Pagination currentPage={currentPage} onPageChange={handlePageChange} totalPages={totalPages} />
        </div>
    );
}
