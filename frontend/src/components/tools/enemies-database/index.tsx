"use client";

import { ArrowDown, ArrowUp, Grid3X3, LayoutList, Search, Skull } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useMemo, useState } from "react";
import { AnimatedBackground } from "~/components/ui/motion-primitives/animated-background";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { cn } from "~/lib/utils";
import type { Enemy, EnemyInfoList, RaceData } from "~/types/api/impl/enemy";
import { CONTAINER_TRANSITION, ENEMY_LEVELS, ITEMS_PER_PAGE, SORT_OPTIONS, TOGGLE_TRANSITION } from "./impl/constants";
import { EnemyCard } from "./impl/enemy-card";
import { EnemyFilters } from "./impl/enemy-filters";
import { Pagination } from "./impl/pagination";
import { ResponsiveFilterContainer } from "./impl/responsive-filter-container";
import type { EnemySortOption, ImmunityType, SortOrder, StatRange } from "./impl/types";
import { filterEnemies, sortEnemies } from "./impl/utils";

interface EnemiesDatabaseProps {
    enemies: Enemy[];
    races: Record<string, RaceData>;
    levelInfo: EnemyInfoList[];
    total: number;
}

export function EnemiesDatabase({ enemies, races, levelInfo: _levelInfo, total: _total }: EnemiesDatabaseProps) {
    // UI state
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [showFilters, setShowFilters] = useState(false);
    const [currentPage, setCurrentPage] = useState(1);
    const [hoveredEnemy, setHoveredEnemy] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

    // Filter state
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedLevels, setSelectedLevels] = useState<string[]>([]);
    const [selectedDamageTypes, setSelectedDamageTypes] = useState<string[]>([]);
    const [selectedRaces, setSelectedRaces] = useState<string[]>([]);
    const [selectedImmunities, setSelectedImmunities] = useState<ImmunityType[]>([]);
    const [statFilters, setStatFilters] = useState<{
        hp: StatRange;
        atk: StatRange;
        def: StatRange;
        res: StatRange;
    }>({
        hp: { min: null, max: null },
        atk: { min: null, max: null },
        def: { min: null, max: null },
        res: { min: null, max: null },
    });

    // Sort state
    const [sortBy, setSortBy] = useState<EnemySortOption>("sortId");
    const [sortOrder, setSortOrder] = useState<SortOrder>("asc");

    // Compute available race options from the races data
    const raceOptions = useMemo(() => {
        return Object.values(races)
            .sort((a, b) => a.sortId - b.sortId)
            .map((race) => ({
                id: race.id,
                name: race.raceName,
            }));
    }, [races]);

    // Filter and sort enemies
    const filteredEnemies = useMemo(() => {
        const filtered = filterEnemies(enemies, {
            searchQuery,
            selectedLevels,
            selectedDamageTypes,
            selectedRaces,
            selectedImmunities,
            statFilters,
        });
        return sortEnemies(filtered, sortBy, sortOrder);
    }, [enemies, searchQuery, selectedLevels, selectedDamageTypes, selectedRaces, selectedImmunities, statFilters, sortBy, sortOrder]);

    // Pagination
    const totalPages = Math.ceil(filteredEnemies.length / ITEMS_PER_PAGE);
    const paginatedEnemies = filteredEnemies.slice((currentPage - 1) * ITEMS_PER_PAGE, currentPage * ITEMS_PER_PAGE);

    // Compute active filter count
    const hasStatFilters = statFilters.hp.min !== null || statFilters.hp.max !== null || statFilters.atk.min !== null || statFilters.atk.max !== null || statFilters.def.min !== null || statFilters.def.max !== null || statFilters.res.min !== null || statFilters.res.max !== null;
    const activeFilterCount = selectedLevels.length + selectedDamageTypes.length + selectedRaces.length + selectedImmunities.length + (hasStatFilters ? 1 : 0);
    const hasActiveFilters = activeFilterCount > 0 || searchQuery.length > 0;

    // Handlers
    const handlePageChange = (page: number) => {
        setCurrentPage(page);
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const handleClearFilters = () => {
        setSearchQuery("");
        setSelectedLevels([]);
        setSelectedDamageTypes([]);
        setSelectedRaces([]);
        setSelectedImmunities([]);
        setStatFilters({
            hp: { min: null, max: null },
            atk: { min: null, max: null },
            def: { min: null, max: null },
            res: { min: null, max: null },
        });
        setCurrentPage(1);
    };

    const handleStatFilterChange = useCallback((stat: "hp" | "atk" | "def" | "res", range: StatRange) => {
        setStatFilters((prev) => ({ ...prev, [stat]: range }));
        setCurrentPage(1);
    }, []);

    const handleSearchChange = useCallback((query: string) => {
        setSearchQuery(query);
        setCurrentPage(1);
    }, []);

    // Pre-compute hover handlers for performance
    const hoverHandlers = useMemo(() => {
        const handlers = new Map<string, (isOpen: boolean) => void>();
        for (const enemy of paginatedEnemies) {
            const enemyId = enemy.enemyId;
            handlers.set(enemyId, (isOpen: boolean) => {
                if (isOpen) {
                    setHoveredEnemy(enemyId);
                    setIsGrayscaleActive(true);
                } else {
                    setHoveredEnemy((current) => (current === enemyId ? null : current));
                    setIsGrayscaleActive(false);
                }
            });
        }
        return handlers;
    }, [paginatedEnemies]);

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-4">
                <div className="flex items-center gap-4">
                    <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10">
                        <Skull className="h-6 w-6 text-primary" />
                    </div>
                    <div>
                        <h1 className="font-bold text-2xl text-foreground sm:text-3xl">Enemy Database</h1>
                        <p className="text-muted-foreground text-sm">Browse and search enemies from Arknights with detailed stats and abilities</p>
                    </div>
                </div>
            </div>

            {/* Search and Controls */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="relative max-w-md flex-1">
                    <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                        className="h-10 w-full rounded-lg border border-border bg-secondary/50 pr-4 pl-10 text-foreground text-sm transition-colors placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                        onChange={(e) => handleSearchChange(e.target.value)}
                        placeholder="Search enemies..."
                        type="text"
                        value={searchQuery}
                    />
                </div>

                <div className="flex items-center gap-2">
                    {/* View Toggle */}
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

                    {/* Sort Controls */}
                    <motion.div className="flex h-10 items-center gap-1 rounded-lg border border-border bg-secondary/50 px-1" layout transition={TOGGLE_TRANSITION}>
                        <Select onValueChange={(value) => setSortBy(value as EnemySortOption)} value={sortBy}>
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
                        <button className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground" onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")} type="button">
                            {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                        </button>
                    </motion.div>

                    {/* Filter Toggle */}
                    <ResponsiveFilterContainer activeFilterCount={activeFilterCount} hasActiveFilters={hasActiveFilters} onClearFilters={handleClearFilters} onOpenChange={setShowFilters} open={showFilters}>
                        <EnemyFilters
                            levels={[...ENEMY_LEVELS]}
                            onClearFilters={handleClearFilters}
                            onDamageTypeChange={setSelectedDamageTypes}
                            onImmunityChange={setSelectedImmunities}
                            onLevelChange={setSelectedLevels}
                            onRaceChange={setSelectedRaces}
                            onStatFilterChange={handleStatFilterChange}
                            raceOptions={raceOptions}
                            selectedDamageTypes={selectedDamageTypes}
                            selectedImmunities={selectedImmunities}
                            selectedLevels={selectedLevels}
                            selectedRaces={selectedRaces}
                            statFilters={statFilters}
                        />
                    </ResponsiveFilterContainer>
                </div>
            </div>

            {/* Results Count */}
            <div className="flex items-center justify-between text-muted-foreground text-sm">
                <span>
                    Showing {Math.min((currentPage - 1) * ITEMS_PER_PAGE + 1, filteredEnemies.length)} to {Math.min(currentPage * ITEMS_PER_PAGE, filteredEnemies.length)} of {filteredEnemies.length} enemies
                </span>
            </div>

            {/* List View Header */}
            <AnimatePresence mode="wait">
                {viewMode === "list" && paginatedEnemies.length > 0 && (
                    <motion.div animate={{ opacity: 1, y: 0 }} className="hidden items-center gap-3 border-border/50 border-b px-3 pb-2 text-muted-foreground text-xs uppercase tracking-wider md:flex" exit={{ opacity: 0, y: -8 }} initial={{ opacity: 0, y: -8 }} transition={CONTAINER_TRANSITION}>
                        <div className="w-12 shrink-0" />
                        <div className="min-w-0 flex-1">Name</div>
                        <div className="w-20 shrink-0">Level</div>
                        <div className="w-24 shrink-0">Damage</div>
                        <div className="hidden w-16 shrink-0 text-right lg:block">HP</div>
                        <div className="hidden w-16 shrink-0 text-right lg:block">ATK</div>
                        <div className="hidden w-16 shrink-0 text-right xl:block">DEF</div>
                        <div className="hidden w-16 shrink-0 text-right xl:block">RES</div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Enemies Grid/List */}
            <AnimatePresence initial={false} mode="wait">
                {paginatedEnemies.length > 0 ? (
                    <motion.div
                        animate={{ opacity: 1, scale: 1 }}
                        className={cn(viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:gap-4" : "grid grid-cols-1 gap-1", "will-change-transform contain-layout")}
                        exit={{ opacity: 0, scale: 0.98 }}
                        initial={{ opacity: 0, scale: 0.98 }}
                        key={viewMode}
                        transition={CONTAINER_TRANSITION}
                    >
                        {paginatedEnemies.map((enemy, index) => {
                            const isCurrentlyHovered = hoveredEnemy === enemy.enemyId;
                            const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;
                            const shouldAnimate = index < 6;

                            return (
                                <motion.div
                                    animate={{ opacity: 1, y: 0 }}
                                    className="contain-content"
                                    initial={shouldAnimate ? { opacity: 0, y: 8 } : false}
                                    key={enemy.enemyId}
                                    transition={
                                        shouldAnimate
                                            ? {
                                                  duration: 0.2,
                                                  delay: index * 0.015,
                                                  ease: [0.4, 0, 0.2, 1],
                                              }
                                            : { duration: 0 }
                                    }
                                >
                                    <EnemyCard enemy={enemy} isHovered={isCurrentlyHovered} onHoverChange={hoverHandlers.get(enemy.enemyId)} shouldGrayscale={shouldGrayscale} viewMode={viewMode} />
                                </motion.div>
                            );
                        })}
                    </motion.div>
                ) : (
                    <motion.div animate={{ opacity: 1, scale: 1 }} className="flex flex-col items-center justify-center py-16 text-center" exit={{ opacity: 0, scale: 0.98 }} initial={{ opacity: 0, scale: 0.98 }} key="empty" transition={CONTAINER_TRANSITION}>
                        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                            <Search className="h-8 w-8 text-muted-foreground" />
                        </div>
                        <h3 className="mb-2 font-semibold text-foreground text-lg">No enemies found</h3>
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
