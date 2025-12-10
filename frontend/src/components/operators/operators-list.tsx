"use client";

import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Grid3X3, LayoutList, Search, SlidersHorizontal } from "lucide-react";
import { useMemo, useState } from "react";
import { cn, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/morphing-popover";
import { OperatorCard } from "./operator-card";
import { OperatorFilters } from "./operator-filters";

// Internal profession names as used in the data
const CLASSES = ["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER", "SPECIAL", "PIONEER"];
const RARITIES = [6, 5, 4, 3, 2, 1];
const GENDERS = ["Male", "Female", "Conviction"];

const ITEMS_PER_PAGE = 48;

export function OperatorsList({ data }: { data: OperatorFromList[] }) {
    const [searchQuery, setSearchQuery] = useState("");
    const [showFilters, setShowFilters] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [currentPage, setCurrentPage] = useState(1);
    const [pageInput, setPageInput] = useState("1");
    const [selectedClasses, setSelectedClasses] = useState<string[]>([]);
    const [selectedSubclasses, setSelectedSubclasses] = useState<string[]>([]);
    const [selectedRarities, setSelectedRarities] = useState<number[]>([]);
    const [selectedBirthPlaces, setSelectedBirthPlaces] = useState<string[]>([]);
    const [selectedNations, setSelectedNations] = useState<string[]>([]);
    const [selectedFactions, setSelectedFactions] = useState<string[]>([]);
    const [selectedGenders, setSelectedGenders] = useState<string[]>([]);
    const [selectedRaces, setSelectedRaces] = useState<string[]>([]);
    const [selectedArtists, setSelectedArtists] = useState<string[]>([]);
    const [sortBy, setSortBy] = useState<"name" | "rarity">("rarity");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

    // Compute available filter options from data
    const filterOptions = useMemo(() => {
        const subclasses = new Set<string>();
        const birthPlaces = new Set<string>();
        const nations = new Set<string>();
        const factions = new Set<string>();
        const races = new Set<string>();
        const artists = new Set<string>();

        data.forEach((op) => {
            if (op.subProfessionId) {
                subclasses.add(op.subProfessionId.toLowerCase());
            }
            if (op.profile?.basicInfo?.placeOfBirth && op.profile.basicInfo.placeOfBirth !== "Unknown" && op.profile.basicInfo.placeOfBirth !== "Undisclosed") {
                birthPlaces.add(op.profile.basicInfo.placeOfBirth);
            }
            if (op.nationId) {
                nations.add(op.nationId);
            }
            if (op.groupId) {
                factions.add(op.groupId);
            }
            if (op.teamId) {
                factions.add(op.teamId);
            }
            if (op.profile?.basicInfo?.race && op.profile.basicInfo.race !== "Unknown" && op.profile.basicInfo.race !== "Undisclosed") {
                races.add(op.profile.basicInfo.race);
            }
            op.artists?.forEach((artist) => {
                if (artist) artists.add(artist);
            });
        });

        return {
            subclasses: Array.from(subclasses).sort(),
            birthPlaces: Array.from(birthPlaces).sort(),
            nations: Array.from(nations).sort(),
            factions: Array.from(factions).sort(),
            races: Array.from(races).sort(),
            artists: Array.from(artists).sort(),
        };
    }, [data]);

    const filteredOperators = useMemo(() => {
        let result = [...data];

        // Search filter
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            result = result.filter((op) => op.name.toLowerCase().includes(query) || op.profession.toLowerCase().includes(query) || op.subProfessionId.toLowerCase().includes(query));
        }

        // Class filter
        if (selectedClasses.length > 0) {
            result = result.filter((op) => selectedClasses.includes(op.profession));
        }

        // Subclass filter
        if (selectedSubclasses.length > 0) {
            result = result.filter((op) => selectedSubclasses.includes(op.subProfessionId.toLowerCase()));
        }

        // Rarity filter
        if (selectedRarities.length > 0) {
            result = result.filter((op) => selectedRarities.includes(rarityToNumber(op.rarity)));
        }

        // Birth place filter
        if (selectedBirthPlaces.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.placeOfBirth && selectedBirthPlaces.includes(op.profile.basicInfo.placeOfBirth));
        }

        // Nation filter
        if (selectedNations.length > 0) {
            result = result.filter((op) => op.nationId && selectedNations.includes(op.nationId));
        }

        // Faction filter
        if (selectedFactions.length > 0) {
            result = result.filter((op) => (op.groupId && selectedFactions.includes(op.groupId)) || (op.teamId && selectedFactions.includes(op.teamId)));
        }

        // Gender filter
        if (selectedGenders.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.gender && selectedGenders.includes(op.profile.basicInfo.gender));
        }

        // Race filter
        if (selectedRaces.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.race && selectedRaces.includes(op.profile.basicInfo.race));
        }

        // Artists filter
        if (selectedArtists.length > 0) {
            result = result.filter((op) => op.artists?.some((artist) => selectedArtists.includes(artist)));
        }

        // Sorting
        result.sort((a, b) => {
            if (sortBy === "name") {
                return sortOrder === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
            }

            const aRarity = rarityToNumber(a.rarity);
            const bRarity = rarityToNumber(b.rarity);
            return sortOrder === "asc" ? aRarity - bRarity : bRarity - aRarity;
        });

        return result;
    }, [searchQuery, selectedClasses, selectedSubclasses, selectedRarities, selectedBirthPlaces, selectedNations, selectedFactions, selectedGenders, selectedRaces, selectedArtists, sortBy, sortOrder, data]);

    const totalPages = Math.ceil(filteredOperators.length / ITEMS_PER_PAGE);
    const paginatedOperators = filteredOperators.slice((currentPage - 1) * ITEMS_PER_PAGE, currentPage * ITEMS_PER_PAGE);

    const handlePageChange = (page: number) => {
        const newPage = Math.max(1, Math.min(page, totalPages));
        setCurrentPage(newPage);
        setPageInput(newPage.toString());
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const clearFilters = () => {
        setSelectedClasses([]);
        setSelectedSubclasses([]);
        setSelectedRarities([]);
        setSelectedBirthPlaces([]);
        setSelectedNations([]);
        setSelectedFactions([]);
        setSelectedGenders([]);
        setSelectedRaces([]);
        setSelectedArtists([]);
        setSearchQuery("");
        setCurrentPage(1);
        setPageInput("1");
    };

    const activeFilterCount = selectedClasses.length + selectedSubclasses.length + selectedRarities.length + selectedBirthPlaces.length + selectedNations.length + selectedFactions.length + selectedGenders.length + selectedRaces.length + selectedArtists.length + (searchQuery ? 1 : 0);
    const hasActiveFilters = activeFilterCount > 0;

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
                        onChange={(e) => {
                            setSearchQuery(e.target.value);
                            setCurrentPage(1);
                        }}
                        placeholder="Search operators..."
                        type="text"
                        value={searchQuery}
                    />
                </div>

                <div className="flex items-center gap-2">
                    {/* View Toggle */}
                    <div className="flex items-center rounded-lg border border-border bg-secondary/50 p-1">
                        <button className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "grid" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("grid")}>
                            <Grid3X3 className="h-4 w-4" />
                        </button>
                        <button className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "list" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("list")}>
                            <LayoutList className="h-4 w-4" />
                        </button>
                    </div>

                    {/* Filter Toggle with Morphing Popover */}
                    <MorphingPopover open={showFilters} onOpenChange={setShowFilters}>
                        <MorphingPopoverTrigger>
                            <button className={cn("flex h-10 items-center gap-2 rounded-lg border px-3 transition-colors", showFilters || hasActiveFilters ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} type="button">
                                <SlidersHorizontal className="h-4 w-4" />
                                <span className="font-medium text-sm">Filters</span>
                                {hasActiveFilters && <span className="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground text-xs">{activeFilterCount}</span>}
                            </button>
                        </MorphingPopoverTrigger>
                        <MorphingPopoverContent className="w-[calc(100vw-2rem)] max-w-4xl bg-card/95 p-0 drop-shadow-2xl backdrop-blur-sm sm:w-[600px] md:w-[700px] lg:w-[900px]">
                            <OperatorFilters
                                classes={CLASSES}
                                subclasses={filterOptions.subclasses}
                                genders={GENDERS}
                                birthPlaces={filterOptions.birthPlaces}
                                nations={filterOptions.nations}
                                factions={filterOptions.factions}
                                races={filterOptions.races}
                                artists={filterOptions.artists}
                                onClassChange={setSelectedClasses}
                                onSubclassChange={setSelectedSubclasses}
                                onClearFilters={clearFilters}
                                onGenderChange={setSelectedGenders}
                                onBirthPlaceChange={setSelectedBirthPlaces}
                                onNationChange={setSelectedNations}
                                onFactionChange={setSelectedFactions}
                                onRaceChange={setSelectedRaces}
                                onArtistChange={setSelectedArtists}
                                onRarityChange={setSelectedRarities}
                                onSortByChange={setSortBy}
                                onSortOrderChange={setSortOrder}
                                rarities={RARITIES}
                                selectedClasses={selectedClasses}
                                selectedSubclasses={selectedSubclasses}
                                selectedGenders={selectedGenders}
                                selectedBirthPlaces={selectedBirthPlaces}
                                selectedNations={selectedNations}
                                selectedFactions={selectedFactions}
                                selectedRaces={selectedRaces}
                                selectedArtists={selectedArtists}
                                selectedRarities={selectedRarities}
                                sortBy={sortBy}
                                sortOrder={sortOrder}
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

            {viewMode === "list" && paginatedOperators.length > 0 && (
                <div className="hidden items-center gap-3 border-border/50 border-b px-3 pb-2 text-muted-foreground text-xs uppercase tracking-wider md:flex">
                    {/* Portrait column */}
                    <div className="w-12 shrink-0" />
                    {/* Name column - flex-1 */}
                    <div className="min-w-0 flex-1">Name</div>
                    {/* Rarity column - w-24 */}
                    <div className="w-24 shrink-0">Rarity</div>
                    {/* Class column - w-32 */}
                    <div className="w-32 shrink-0">Class</div>
                    {/* Archetype column - w-40, hidden on smaller screens */}
                    <div className="hidden w-40 shrink-0 lg:block">Archetype</div>
                    {/* Faction column - w-8, hidden until xl */}
                    <div className="hidden w-8 shrink-0 text-center xl:block">Faction</div>
                </div>
            )}

            {/* Operators Grid/List */}
            {paginatedOperators.length > 0 ? (
                <div
                    className={cn(
                        viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:grid-cols-8 xl:gap-4" : "flex flex-col gap-1", // Reduced gap for tighter list rows
                    )}
                >
                    {paginatedOperators.map((operator) => {
                        const operatorId = operator.id!;
                        const isCurrentlyHovered = hoveredOperator === operatorId;
                        const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;

                        const handleHoverChange = (isOpen: boolean) => {
                            if (isOpen) {
                                setHoveredOperator(operatorId);
                                setIsGrayscaleActive(true);
                            } else {
                                if (hoveredOperator === operatorId) {
                                    setHoveredOperator(null);
                                    setIsGrayscaleActive(false);
                                }
                            }
                        };

                        return <OperatorCard isHovered={isCurrentlyHovered} key={operatorId} onHoverChange={handleHoverChange} operator={operator} shouldGrayscale={shouldGrayscale} viewMode={viewMode} />;
                    })}
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center py-16 text-center">
                    <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                        <Search className="h-8 w-8 text-muted-foreground" />
                    </div>
                    <h3 className="mb-2 font-semibold text-foreground text-lg">No operators found</h3>
                    <p className="mb-4 max-w-sm text-muted-foreground text-sm">Try adjusting your search or filter criteria to find what you're looking for.</p>
                    <button className="text-primary text-sm hover:underline" onClick={clearFilters}>
                        Clear all filters
                    </button>
                </div>
            )}

            {/* Pagination */}
            {totalPages > 1 && (
                <div className="flex flex-col items-center gap-4 pt-6 sm:flex-row sm:justify-between">
                    <div className="flex flex-wrap items-center justify-center gap-3">
                        <span className="text-muted-foreground text-sm">
                            Page {currentPage} of {totalPages}
                        </span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-muted-foreground text-sm">Go to:</span>
                            <input
                                className="h-8 w-14 rounded-md border border-border bg-secondary/50 px-2 text-center text-foreground text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                                type="number"
                                min={1}
                                max={totalPages}
                                value={pageInput}
                                onChange={(e) => setPageInput(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === "Enter") {
                                        const page = Number.parseInt(pageInput, 10);
                                        if (!Number.isNaN(page) && page >= 1 && page <= totalPages) {
                                            handlePageChange(page);
                                        } else {
                                            setPageInput(currentPage.toString());
                                        }
                                    }
                                }}
                                onBlur={() => {
                                    const page = Number.parseInt(pageInput, 10);
                                    if (!Number.isNaN(page) && page >= 1 && page <= totalPages) {
                                        handlePageChange(page);
                                    } else {
                                        setPageInput(currentPage.toString());
                                    }
                                }}
                            />
                        </div>
                    </div>
                    <div className="flex items-center gap-1">
                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === 1}
                            onClick={() => handlePageChange(1)}
                            title="First page"
                        >
                            <ChevronsLeft className="h-4 w-4" />
                        </button>
                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === 1}
                            onClick={() => handlePageChange(currentPage - 1)}
                            title="Previous page"
                        >
                            <ChevronLeft className="h-4 w-4" />
                        </button>

                        {/* Page numbers - responsive based on screen size */}
                        {(() => {
                            const getPages = (maxVisible: number): (number | "ellipsis-start" | "ellipsis-end")[] => {
                                const pages: (number | "ellipsis-start" | "ellipsis-end")[] = [];

                                if (totalPages <= maxVisible) {
                                    for (let i = 1; i <= totalPages; i++) pages.push(i);
                                    return pages;
                                }

                                pages.push(1);
                                const sideCount = Math.floor((maxVisible - 3) / 2);

                                if (currentPage <= sideCount + 2) {
                                    for (let i = 2; i <= Math.min(maxVisible - 2, totalPages - 1); i++) pages.push(i);
                                    if (totalPages > maxVisible - 1) pages.push("ellipsis-end");
                                } else if (currentPage >= totalPages - sideCount - 1) {
                                    pages.push("ellipsis-start");
                                    for (let i = Math.max(totalPages - maxVisible + 3, 2); i <= totalPages - 1; i++) pages.push(i);
                                } else {
                                    pages.push("ellipsis-start");
                                    for (let i = currentPage - sideCount; i <= currentPage + sideCount; i++) pages.push(i);
                                    pages.push("ellipsis-end");
                                }

                                pages.push(totalPages);
                                return pages;
                            };

                            return (
                                <>
                                    {/* Mobile: fewer pages */}
                                    <div className="flex items-center gap-1 sm:hidden">
                                        {getPages(5).map((page, idx) =>
                                            page === "ellipsis-start" || page === "ellipsis-end" ? (
                                                <span key={`${page}-${idx}`} className="px-1 text-muted-foreground">
                                                    ...
                                                </span>
                                            ) : (
                                                <button
                                                    key={page}
                                                    className={cn(
                                                        "flex h-9 w-9 items-center justify-center rounded-lg border text-sm transition-colors",
                                                        currentPage === page ? "border-primary bg-primary text-primary-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground",
                                                    )}
                                                    onClick={() => handlePageChange(page)}
                                                >
                                                    {page}
                                                </button>
                                            ),
                                        )}
                                    </div>
                                    {/* Desktop: more pages */}
                                    <div className="hidden items-center gap-1 sm:flex">
                                        {getPages(7).map((page, idx) =>
                                            page === "ellipsis-start" || page === "ellipsis-end" ? (
                                                <span key={`${page}-${idx}`} className="px-1 text-muted-foreground">
                                                    ...
                                                </span>
                                            ) : (
                                                <button
                                                    key={page}
                                                    className={cn(
                                                        "flex h-9 w-9 items-center justify-center rounded-lg border text-sm transition-colors",
                                                        currentPage === page ? "border-primary bg-primary text-primary-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground",
                                                    )}
                                                    onClick={() => handlePageChange(page)}
                                                >
                                                    {page}
                                                </button>
                                            ),
                                        )}
                                    </div>
                                </>
                            );
                        })()}

                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === totalPages}
                            onClick={() => handlePageChange(currentPage + 1)}
                            title="Next page"
                        >
                            <ChevronRight className="h-4 w-4" />
                        </button>
                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === totalPages}
                            onClick={() => handlePageChange(totalPages)}
                            title="Last page"
                        >
                            <ChevronsRight className="h-4 w-4" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
