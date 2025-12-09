"use client";

import { ChevronLeft, ChevronRight, Grid3X3, LayoutList, Search, SlidersHorizontal } from "lucide-react";
import { useMemo, useState } from "react";
import { cn, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { OperatorCard } from "./operator-card";
import { OperatorFilters } from "./operator-filters";

const CLASSES = ["Guard", "Sniper", "Defender", "Medic", "Supporter", "Caster", "Specialist", "Vanguard"];
const RARITIES = [6, 5, 4, 3, 2, 1];
const ELEMENTS = ["Physical", "Arts", "Healing"];

const ITEMS_PER_PAGE = 48;

export function OperatorsList({ data }: { data: OperatorFromList[] }) {
    const [searchQuery, setSearchQuery] = useState("");
    const [showFilters, setShowFilters] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [currentPage, setCurrentPage] = useState(1);
    const [selectedClasses, setSelectedClasses] = useState<string[]>([]);
    const [selectedRarities, setSelectedRarities] = useState<number[]>([]);
    const [selectedElements, setSelectedElements] = useState<string[]>([]);
    const [sortBy, setSortBy] = useState<"name" | "rarity">("rarity");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

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

        // Rarity filter
        if (selectedRarities.length > 0) {
            result = result.filter((op) => selectedRarities.includes(rarityToNumber(op.rarity)));
        }

        // Element filter
        if (selectedElements.length > 0) {
            result = result.filter((_op) => selectedElements.includes(""));
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
    }, [searchQuery, selectedClasses, selectedRarities, selectedElements, sortBy, sortOrder, data]);

    const totalPages = Math.ceil(filteredOperators.length / ITEMS_PER_PAGE);
    const paginatedOperators = filteredOperators.slice((currentPage - 1) * ITEMS_PER_PAGE, currentPage * ITEMS_PER_PAGE);

    const handlePageChange = (page: number) => {
        setCurrentPage(Math.max(1, Math.min(page, totalPages)));
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const clearFilters = () => {
        setSelectedClasses([]);
        setSelectedRarities([]);
        setSelectedElements([]);
        setSearchQuery("");
        setCurrentPage(1);
    };

    const hasActiveFilters = selectedClasses.length > 0 || selectedRarities.length > 0 || selectedElements.length > 0 || searchQuery.length > 0;

    return (
        <div className="space-y-6">
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

                    {/* Filter Toggle */}
                    <button className={cn("flex h-10 items-center gap-2 rounded-lg border px-3 transition-colors", showFilters || hasActiveFilters ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} onClick={() => setShowFilters(!showFilters)}>
                        <SlidersHorizontal className="h-4 w-4" />
                        <span className="font-medium text-sm">Filters</span>
                        {hasActiveFilters && <span className="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground text-xs">{selectedClasses.length + selectedRarities.length + selectedElements.length + (searchQuery ? 1 : 0)}</span>}
                    </button>
                </div>
            </div>

            {/* Filters Panel */}
            <OperatorFilters
                classes={CLASSES}
                elements={ELEMENTS}
                isOpen={showFilters}
                onClassChange={setSelectedClasses}
                onClearFilters={clearFilters}
                onElementChange={setSelectedElements}
                onRarityChange={setSelectedRarities}
                onSortByChange={setSortBy}
                onSortOrderChange={setSortOrder}
                rarities={RARITIES}
                selectedClasses={selectedClasses}
                selectedElements={selectedElements}
                selectedRarities={selectedRarities}
                sortBy={sortBy}
                sortOrder={sortOrder}
            />

            {/* Results Count */}
            <div className="flex items-center justify-between text-muted-foreground text-sm">
                <span>
                    Showing {(currentPage - 1) * ITEMS_PER_PAGE + 1} to {Math.min(currentPage * ITEMS_PER_PAGE, filteredOperators.length)} of {filteredOperators.length} operators
                </span>
            </div>

            {/* Operators Grid */}
            {paginatedOperators.length > 0 ? (
                <div className={cn(viewMode === "grid" ? "grid grid-cols-3 gap-2 sm:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:grid-cols-8 xl:gap-4" : "flex flex-col gap-2")}>
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
                    <div className="text-muted-foreground text-sm">
                        Page {currentPage} of {totalPages}
                    </div>
                    <div className="flex items-center gap-1">
                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === 1}
                            onClick={() => handlePageChange(currentPage - 1)}
                        >
                            <ChevronLeft className="h-4 w-4" />
                        </button>

                        {/* Page numbers */}
                        <div className="flex items-center gap-1">
                            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                                let pageNum: number;
                                if (totalPages <= 5) {
                                    pageNum = i + 1;
                                } else if (currentPage <= 3) {
                                    pageNum = i + 1;
                                } else if (currentPage >= totalPages - 2) {
                                    pageNum = totalPages - 4 + i;
                                } else {
                                    pageNum = currentPage - 2 + i;
                                }

                                return (
                                    <button
                                        className={cn("flex h-9 w-9 items-center justify-center rounded-lg font-medium text-sm transition-colors", currentPage === pageNum ? "bg-primary text-primary-foreground" : "border border-border bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground")}
                                        key={pageNum}
                                        onClick={() => handlePageChange(pageNum)}
                                    >
                                        {pageNum}
                                    </button>
                                );
                            })}
                        </div>

                        <button
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={currentPage === totalPages}
                            onClick={() => handlePageChange(currentPage + 1)}
                        >
                            <ChevronRight className="h-4 w-4" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
