"use client";

import { Grid3X3, LayoutList, Search, SlidersHorizontal } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/motion-primitives/morphing-popover";
import { cn } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { CLASSES, GENDERS, ITEMS_PER_PAGE, RARITIES } from "./constants";
import { useOperatorFilters } from "./hooks";
import { OperatorCard } from "./operator-card";
import { OperatorFilters } from "./operator-filters";
import { Pagination } from "./ui/impl/pagination";

export function OperatorsList({ data }: { data: OperatorFromList[] }) {
    // UI state
    const [showFilters, setShowFilters] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
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

                    {/* Filter Toggle with Morphing Popover */}
                    <MorphingPopover onOpenChange={setShowFilters} open={showFilters}>
                        <MorphingPopoverTrigger>
                            <button className={cn("flex h-10 items-center gap-2 rounded-lg border px-3 transition-colors", showFilters || hasActiveFilters ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} type="button">
                                <SlidersHorizontal className="h-4 w-4" />
                                <span className="font-medium text-sm">Filters</span>
                                {hasActiveFilters && <span className="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground text-xs">{activeFilterCount}</span>}
                            </button>
                        </MorphingPopoverTrigger>
                        <MorphingPopoverContent className="w-[calc(100vw-2rem)] max-w-4xl bg-card/95 p-0 drop-shadow-2xl backdrop-blur-sm sm:w-[600px] md:w-[700px] lg:w-[900px]">
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
                        </MorphingPopoverContent>
                    </MorphingPopover>
                </div>
            </div>

            {/* Results Count */}
            <div className="flex items-center justify-between text-muted-foreground text-sm">
                <span>
                    Showing {(currentPage - 1) * ITEMS_PER_PAGE + 1} to {Math.min(currentPage * ITEMS_PER_PAGE, filteredOperators.length)} of {filteredOperators.length} operators
                </span>
            </div>

            {/* List View Header */}
            {viewMode === "list" && paginatedOperators.length > 0 && (
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
                <div className={cn(viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:grid-cols-8 xl:gap-4" : "flex flex-col gap-1", "contain-layout")}>
                    {paginatedOperators.map((operator) => {
                        const operatorId = operator.id ?? "";
                        const isCurrentlyHovered = hoveredOperator === operatorId;
                        const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;

                        return <OperatorCard isHovered={isCurrentlyHovered} key={operatorId} onHoverChange={hoverHandlers.get(operatorId)} operator={operator} shouldGrayscale={shouldGrayscale} viewMode={viewMode} />;
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
