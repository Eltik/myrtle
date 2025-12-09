"use client";

import { useState, useMemo } from "react";
import { Search, SlidersHorizontal, Grid3X3, LayoutList, ChevronLeft, ChevronRight } from "lucide-react";
import { OperatorCard } from "./operator-card";
import { OperatorFilters } from "./operator-filters";
import { cn } from "~/lib/utils";

// Mock operator data - in production this would come from an API
const OPERATORS = [
    {
        id: "surtr",
        name: "Surtr",
        rarity: 6,
        class: "Guard",
        subclass: "Arts Fighter",
        element: "Arts",
        image: "/surtr-arknights-operator-anime-girl-red-hair.jpg",
    },
    {
        id: "silverash",
        name: "SilverAsh",
        rarity: 6,
        class: "Guard",
        subclass: "Ranged Guard",
        element: "Physical",
        image: "/silverash-arknights-operator-white-hair-male.jpg",
    },
    {
        id: "eyjafjalla",
        name: "Eyjafjalla",
        rarity: 6,
        class: "Caster",
        subclass: "Core Caster",
        element: "Arts",
        image: "/eyjafjalla-arknights-operator-white-hair-sheep-hor.jpg",
    },
    {
        id: "saria",
        name: "Saria",
        rarity: 6,
        class: "Defender",
        subclass: "Healing Defender",
        element: "Physical",
        image: "/saria-arknights-operator-white-hair-stern.jpg",
    },
    {
        id: "blaze",
        name: "Blaze",
        rarity: 6,
        class: "Guard",
        subclass: "AoE Guard",
        element: "Physical",
        image: "/blaze-arknights-operator-orange-hair-chainsaw.jpg",
    },
    {
        id: "chen",
        name: "Ch'en",
        rarity: 6,
        class: "Guard",
        subclass: "Dualstrike Guard",
        element: "Physical",
        image: "/chen-arknights-operator-blue-hair-dragon-horns-swo.jpg",
    },
    {
        id: "mudrock",
        name: "Mudrock",
        rarity: 6,
        class: "Defender",
        subclass: "Enmity Defender",
        element: "Physical",
        image: "/mudrock-arknights-operator-dark-armor-knight.jpg",
    },
    {
        id: "thorns",
        name: "Thorns",
        rarity: 6,
        class: "Guard",
        subclass: "Ranged Guard",
        element: "Physical",
        image: "/thorns-arknights-operator-white-hair-male-spear.jpg",
    },
    {
        id: "mountain",
        name: "Mountain",
        rarity: 6,
        class: "Guard",
        subclass: "Brawler",
        element: "Physical",
        image: "/mountain-arknights-operator-white-hair-feline.jpg",
    },
    {
        id: "exusiai",
        name: "Exusiai",
        rarity: 6,
        class: "Sniper",
        subclass: "Marksman",
        element: "Physical",
        image: "/exusiai-arknights-operator-red-hair-angel-halo.jpg",
    },
    {
        id: "schwarz",
        name: "Schwarz",
        rarity: 6,
        class: "Sniper",
        subclass: "Heavyshooter",
        element: "Physical",
        image: "/schwarz-arknights-operator-black-hair-crossbow.jpg",
    },
    {
        id: "w",
        name: "W",
        rarity: 6,
        class: "Sniper",
        subclass: "Bombardier",
        element: "Physical",
        image: "/w-arknights-operator-dark-hair-sarkaz-demon-girl.jpg",
    },
    {
        id: "ifrit",
        name: "Ifrit",
        rarity: 6,
        class: "Caster",
        subclass: "Blast Caster",
        element: "Arts",
        image: "/ifrit-arknights-operator-orange-hair-fire.jpg",
    },
    {
        id: "ceobe",
        name: "Ceobe",
        rarity: 6,
        class: "Caster",
        subclass: "Core Caster",
        element: "Arts",
        image: "/ceobe-arknights-operator-blonde-dog-ears-staff.jpg",
    },
    {
        id: "saga",
        name: "Saga",
        rarity: 6,
        class: "Vanguard",
        subclass: "Pioneer",
        element: "Physical",
        image: "/saga-arknights-operator-orange-hair-dog-ears-nagin.jpg",
    },
    {
        id: "bagpipe",
        name: "Bagpipe",
        rarity: 6,
        class: "Vanguard",
        subclass: "Charger",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "texas",
        name: "Texas",
        rarity: 5,
        class: "Vanguard",
        subclass: "Pioneer",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "lappland",
        name: "Lappland",
        rarity: 5,
        class: "Guard",
        subclass: "Ranged Guard",
        element: "Arts",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "specter",
        name: "Specter",
        rarity: 5,
        class: "Guard",
        subclass: "AoE Guard",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "ptilopsis",
        name: "Ptilopsis",
        rarity: 5,
        class: "Medic",
        subclass: "AoE Medic",
        element: "Healing",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "warfarin",
        name: "Warfarin",
        rarity: 5,
        class: "Medic",
        subclass: "ST Medic",
        element: "Healing",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "blue-poison",
        name: "Blue Poison",
        rarity: 5,
        class: "Sniper",
        subclass: "Marksman",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "meteorite",
        name: "Meteorite",
        rarity: 5,
        class: "Sniper",
        subclass: "Artilleryman",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "nearl",
        name: "Nearl",
        rarity: 5,
        class: "Defender",
        subclass: "Healing Defender",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "projekt-red",
        name: "Projekt Red",
        rarity: 5,
        class: "Specialist",
        subclass: "Executor",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "kroos",
        name: "Kroos",
        rarity: 3,
        class: "Sniper",
        subclass: "Marksman",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "melantha",
        name: "Melantha",
        rarity: 3,
        class: "Guard",
        subclass: "Dreadnought",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "ansel",
        name: "Ansel",
        rarity: 3,
        class: "Medic",
        subclass: "ST Medic",
        element: "Healing",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "fang",
        name: "Fang",
        rarity: 3,
        class: "Vanguard",
        subclass: "Pioneer",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "beagle",
        name: "Beagle",
        rarity: 3,
        class: "Defender",
        subclass: "Protector",
        element: "Physical",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "hibiscus",
        name: "Hibiscus",
        rarity: 3,
        class: "Medic",
        subclass: "ST Medic",
        element: "Healing",
        image: "/placeholder.svg?height=160&width=160",
    },
    {
        id: "lava",
        name: "Lava",
        rarity: 3,
        class: "Caster",
        subclass: "Splash Caster",
        element: "Arts",
        image: "/placeholder.svg?height=160&width=160",
    },
];

const CLASSES = ["Guard", "Sniper", "Defender", "Medic", "Supporter", "Caster", "Specialist", "Vanguard"];
const RARITIES = [6, 5, 4, 3, 2, 1];
const ELEMENTS = ["Physical", "Arts", "Healing"];

const ITEMS_PER_PAGE = 24;

export function OperatorsList() {
    const [searchQuery, setSearchQuery] = useState("");
    const [showFilters, setShowFilters] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [currentPage, setCurrentPage] = useState(1);
    const [selectedClasses, setSelectedClasses] = useState<string[]>([]);
    const [selectedRarities, setSelectedRarities] = useState<number[]>([]);
    const [selectedElements, setSelectedElements] = useState<string[]>([]);
    const [sortBy, setSortBy] = useState<"name" | "rarity">("rarity");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

    const filteredOperators = useMemo(() => {
        let result = [...OPERATORS];

        // Search filter
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            result = result.filter((op) => op.name.toLowerCase().includes(query) || op.class.toLowerCase().includes(query) || op.subclass.toLowerCase().includes(query));
        }

        // Class filter
        if (selectedClasses.length > 0) {
            result = result.filter((op) => selectedClasses.includes(op.class));
        }

        // Rarity filter
        if (selectedRarities.length > 0) {
            result = result.filter((op) => selectedRarities.includes(op.rarity));
        }

        // Element filter
        if (selectedElements.length > 0) {
            result = result.filter((op) => selectedElements.includes(op.element));
        }

        // Sorting
        result.sort((a, b) => {
            if (sortBy === "name") {
                return sortOrder === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
            }
            return sortOrder === "asc" ? a.rarity - b.rarity : b.rarity - a.rarity;
        });

        return result;
    }, [searchQuery, selectedClasses, selectedRarities, selectedElements, sortBy, sortOrder]);

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
                <div className="relative flex-1 max-w-md">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search operators..."
                        value={searchQuery}
                        onChange={(e) => {
                            setSearchQuery(e.target.value);
                            setCurrentPage(1);
                        }}
                        className="h-10 w-full rounded-lg border border-border bg-secondary/50 pl-10 pr-4 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary transition-colors"
                    />
                </div>

                <div className="flex items-center gap-2">
                    {/* View Toggle */}
                    <div className="flex items-center rounded-lg border border-border bg-secondary/50 p-1">
                        <button onClick={() => setViewMode("grid")} className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "grid" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}>
                            <Grid3X3 className="h-4 w-4" />
                        </button>
                        <button onClick={() => setViewMode("list")} className={cn("flex h-8 w-8 items-center justify-center rounded-md transition-colors", viewMode === "list" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}>
                            <LayoutList className="h-4 w-4" />
                        </button>
                    </div>

                    {/* Filter Toggle */}
                    <button onClick={() => setShowFilters(!showFilters)} className={cn("flex h-10 items-center gap-2 rounded-lg border px-3 transition-colors", showFilters || hasActiveFilters ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")}>
                        <SlidersHorizontal className="h-4 w-4" />
                        <span className="text-sm font-medium">Filters</span>
                        {hasActiveFilters && <span className="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">{selectedClasses.length + selectedRarities.length + selectedElements.length + (searchQuery ? 1 : 0)}</span>}
                    </button>
                </div>
            </div>

            {/* Filters Panel */}
            <OperatorFilters
                isOpen={showFilters}
                classes={CLASSES}
                rarities={RARITIES}
                elements={ELEMENTS}
                selectedClasses={selectedClasses}
                selectedRarities={selectedRarities}
                selectedElements={selectedElements}
                sortBy={sortBy}
                sortOrder={sortOrder}
                onClassChange={setSelectedClasses}
                onRarityChange={setSelectedRarities}
                onElementChange={setSelectedElements}
                onSortByChange={setSortBy}
                onSortOrderChange={setSortOrder}
                onClearFilters={clearFilters}
            />

            {/* Results Count */}
            <div className="flex items-center justify-between text-sm text-muted-foreground">
                <span>
                    Showing {(currentPage - 1) * ITEMS_PER_PAGE + 1} to {Math.min(currentPage * ITEMS_PER_PAGE, filteredOperators.length)} of {filteredOperators.length} operators
                </span>
            </div>

            {/* Operators Grid */}
            {paginatedOperators.length > 0 ? (
                <div className={cn(viewMode === "grid" ? "grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6" : "flex flex-col gap-2")}>
                    {paginatedOperators.map((operator) => (
                        <OperatorCard key={operator.id} operator={operator} viewMode={viewMode} />
                    ))}
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center py-16 text-center">
                    <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                        <Search className="h-8 w-8 text-muted-foreground" />
                    </div>
                    <h3 className="mb-2 font-semibold text-foreground text-lg">No operators found</h3>
                    <p className="mb-4 max-w-sm text-muted-foreground text-sm">Try adjusting your search or filter criteria to find what you're looking for.</p>
                    <button onClick={clearFilters} className="text-primary text-sm hover:underline">
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
                            onClick={() => handlePageChange(currentPage - 1)}
                            disabled={currentPage === 1}
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
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
                                        key={pageNum}
                                        onClick={() => handlePageChange(pageNum)}
                                        className={cn("flex h-9 w-9 items-center justify-center rounded-lg text-sm font-medium transition-colors", currentPage === pageNum ? "bg-primary text-primary-foreground" : "border border-border bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground")}
                                    >
                                        {pageNum}
                                    </button>
                                );
                            })}
                        </div>

                        <button
                            onClick={() => handlePageChange(currentPage + 1)}
                            disabled={currentPage === totalPages}
                            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            <ChevronRight className="h-4 w-4" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
