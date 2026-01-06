"use client";

import { ArrowDown, ArrowUp, Grid3X3, LayoutList, Search } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatedBackground } from "~/components/ui/motion-primitives/animated-background";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { cn } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { CLASSES, GENDERS, ITEMS_PER_PAGE, RARITIES, SORT_OPTIONS } from "../constants";
import { useOperatorFilters } from "../hooks";
import type { SortOption } from "../hooks/impl/use-operator-filters";
import { OperatorCard } from "../operator-card";
import { OperatorFilters } from "../operator-filters";
import { Pagination } from "../ui/impl/pagination";
import { ResponsiveFilterContainer } from "../ui/impl/responsive-filter-container";
import { CONTAINER_TRANSITION, TOGGLE_TRANSITION } from "./impl/constants";
import { getInitialListColumns, getInitialViewMode } from "./impl/helpers";

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
                    <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                        className="h-10 w-full rounded-lg border border-border bg-secondary/50 pr-4 pl-10 text-foreground text-sm transition-colors placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                        onChange={(e) => handleSearchChange(e.target.value)}
                        placeholder="Search operators..."
                        type="text"
                        value={filters.searchQuery}
                    />
                </div>

                <div className="flex items-center gap-2">
                    {/* View Toggle with animated sliding indicator */}
                    <motion.div className="flex items-center rounded-lg border border-border bg-secondary/50 p-1" layout transition={TOGGLE_TRANSITION}>
                        <AnimatedBackground
                            className="rounded-md bg-primary"
                            defaultValue={viewMode}
                            onValueChange={(value) => {
                                if (value === "grid" || value === "list") {
                                    setViewMode(value);
                                }
                            }}
                            transition={TOGGLE_TRANSITION}
                        >
                            <button className={cn("flex h-8 w-8 cursor-pointer items-center justify-center rounded-md transition-colors duration-150", viewMode === "grid" ? "text-primary-foreground" : "text-muted-foreground hover:text-foreground")} data-id="grid" type="button">
                                <Grid3X3 className="h-4 w-4" />
                            </button>
                            <button className={cn("flex h-8 w-8 cursor-pointer items-center justify-center rounded-md transition-colors duration-150", viewMode === "list" ? "text-primary-foreground" : "text-muted-foreground hover:text-foreground")} data-id="list" type="button">
                                <LayoutList className="h-4 w-4" />
                            </button>
                        </AnimatedBackground>
                    </motion.div>

                    {/* List Columns Selector (desktop only, when in list view) */}
                    <AnimatePresence mode="popLayout">
                        {viewMode === "list" && (
                            <motion.div animate={{ opacity: 1, scale: 1 }} className="hidden h-10 items-center rounded-lg border border-border bg-secondary/50 px-2.5 md:block" exit={{ opacity: 0, scale: 0.95 }} initial={{ opacity: 0, scale: 0.95 }} layout transition={TOGGLE_TRANSITION}>
                                <Select onValueChange={handleListColumnsChange} value={listColumns.toString()}>
                                    <SelectTrigger className="h-7 w-20 border-0 bg-transparent px-1.5 text-sm focus:ring-0">
                                        <SelectValue placeholder="Cols" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="1">1 col</SelectItem>
                                        <SelectItem value="2">2 cols</SelectItem>
                                        <SelectItem value="3">3 cols</SelectItem>
                                    </SelectContent>
                                </Select>
                            </motion.div>
                        )}
                    </AnimatePresence>

                    {/* Sort Controls - Always visible next to view toggle */}
                    <motion.div className="flex h-10 items-center gap-1 rounded-lg border border-border bg-secondary/50 px-1" layout transition={TOGGLE_TRANSITION}>
                        <Select onValueChange={(value) => setSortBy(value as SortOption)} value={filters.sortBy}>
                            <SelectTrigger className="h-8 w-22 border-0 bg-transparent px-2.5 text-sm shadow-none focus:ring-0 focus-visible:ring-0">
                                <SelectValue placeholder="Sort by" />
                            </SelectTrigger>
                            <SelectContent align="start" className="min-w-32 rounded-lg border-border bg-card/75 backdrop-blur-sm">
                                {SORT_OPTIONS.map((option) => (
                                    <SelectItem className="cursor-pointer rounded-md" key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <button className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground" onClick={() => setSortOrder(filters.sortOrder === "asc" ? "desc" : "asc")} type="button">
                            {filters.sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                        </button>
                    </motion.div>

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
            <AnimatePresence mode="wait">
                {viewMode === "list" && paginatedOperators.length > 0 && listColumns === 1 && (
                    <motion.div animate={{ opacity: 1, y: 0 }} className="hidden items-center gap-3 border-border/50 border-b px-3 pb-2 text-muted-foreground text-xs uppercase tracking-wider md:flex" exit={{ opacity: 0, y: -8 }} initial={{ opacity: 0, y: -8 }} transition={CONTAINER_TRANSITION}>
                        <div className="w-12 shrink-0" />
                        <div className="min-w-0 flex-1">Name</div>
                        <div className="w-24 shrink-0">Rarity</div>
                        <div className="w-32 shrink-0">Class</div>
                        <div className="hidden w-40 shrink-0 lg:block">Archetype</div>
                        <div className="hidden w-8 shrink-0 text-center xl:block">Faction</div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Operators Grid/List */}
            <AnimatePresence initial={false} mode="wait">
                {paginatedOperators.length > 0 ? (
                    <motion.div
                        animate={{ opacity: 1, scale: 1 }}
                        className={cn(
                            viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:gap-4" : cn("grid gap-1", listColumns === 1 ? "grid-cols-1" : listColumns === 2 ? "grid-cols-1 md:grid-cols-2" : "grid-cols-1 md:grid-cols-2 lg:grid-cols-3"),
                            "will-change-transform contain-layout",
                        )}
                        exit={{ opacity: 0, scale: 0.98 }}
                        initial={{ opacity: 0, scale: 0.98 }}
                        key={`${viewMode}-${listColumns}`}
                        transition={CONTAINER_TRANSITION}
                    >
                        {paginatedOperators.map((operator, index) => {
                            const operatorId = operator.id ?? "";
                            const isCurrentlyHovered = hoveredOperator === operatorId;
                            const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;

                            return (
                                <motion.div
                                    animate={{ opacity: 1, y: 0 }}
                                    className="contain-content"
                                    initial={{ opacity: 0, y: 8 }}
                                    key={operatorId}
                                    transition={{
                                        duration: 0.2,
                                        delay: Math.min(index * 0.015, 0.3),
                                        ease: [0.4, 0, 0.2, 1],
                                    }}
                                >
                                    <OperatorCard isHovered={isCurrentlyHovered} listColumns={listColumns} onHoverChange={hoverHandlers.get(operatorId)} operator={operator} shouldGrayscale={shouldGrayscale} viewMode={viewMode} />
                                </motion.div>
                            );
                        })}
                    </motion.div>
                ) : (
                    <motion.div animate={{ opacity: 1, scale: 1 }} className="flex flex-col items-center justify-center py-16 text-center" exit={{ opacity: 0, scale: 0.98 }} initial={{ opacity: 0, scale: 0.98 }} key="empty" transition={CONTAINER_TRANSITION}>
                        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                            <Search className="h-8 w-8 text-muted-foreground" />
                        </div>
                        <h3 className="mb-2 font-semibold text-foreground text-lg">No operators found</h3>
                        <p className="mb-4 max-w-sm text-muted-foreground text-sm">Try adjusting your search or filter criteria to find what you're looking for.</p>
                        <button className="text-primary text-sm hover:underline" onClick={handleClearFilters} type="button">
                            Clear all filters
                        </button>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Pagination */}
            <Pagination currentPage={currentPage} onPageChange={handlePageChange} totalPages={totalPages} />
        </div>
    );
}
