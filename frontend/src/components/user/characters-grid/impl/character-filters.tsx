"use client";

import { ArrowDown, ArrowUp, Grid3X3, LayoutGrid } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { capitalize } from "~/lib/utils";
import type { RarityFilter, SortBy, SortOrder, ViewMode } from "~/types/frontend/impl/user";

interface CharacterFiltersProps {
    searchTerm: string;
    setSearchTerm: (value: string) => void;
    sortBy: SortBy;
    setSortBy: (value: SortBy) => void;
    filterRarity: RarityFilter;
    setFilterRarity: (value: RarityFilter) => void;
    sortOrder: SortOrder;
    toggleSortOrder: () => void;
    viewMode: ViewMode;
    setViewMode: (value: ViewMode) => void;
}

export function CharacterFilters({ searchTerm, setSearchTerm, sortBy, setSortBy, filterRarity, setFilterRarity, sortOrder, toggleSortOrder, viewMode, setViewMode }: CharacterFiltersProps) {
    return (
        <div className="flex w-full flex-col gap-4 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between">
            <div className="flex w-full flex-col gap-4 sm:w-auto sm:flex-row sm:flex-wrap sm:items-center">
                <Input className="w-full sm:w-70" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search operators..." value={searchTerm} />
                <Select onValueChange={(value: SortBy) => setSortBy(value)} value={sortBy}>
                    <SelectTrigger className="w-full sm:w-45">
                        <SelectValue placeholder="Sort by" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="level">Sort by Level</SelectItem>
                        <SelectItem value="rarity">Sort by Rarity</SelectItem>
                        <SelectItem value="obtained">Sort by Obtained</SelectItem>
                        <SelectItem value="potential">Sort by Potential</SelectItem>
                    </SelectContent>
                </Select>
                <Select onValueChange={(value: RarityFilter) => setFilterRarity(value)} value={filterRarity}>
                    <SelectTrigger className="w-full sm:w-40">
                        <SelectValue placeholder="Filter by Rarity" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">All Rarities</SelectItem>
                        <SelectItem value="TIER_6">6 Star</SelectItem>
                        <SelectItem value="TIER_5">5 Star</SelectItem>
                        <SelectItem value="TIER_4">4 Star</SelectItem>
                        <SelectItem value="TIER_3">3 Star</SelectItem>
                        <SelectItem value="TIER_2">2 Star</SelectItem>
                        <SelectItem value="TIER_1">1 Star</SelectItem>
                    </SelectContent>
                </Select>
                <Button className="flex w-full items-center justify-center gap-2 bg-transparent sm:w-auto" onClick={toggleSortOrder} variant="outline">
                    <span>{capitalize(sortOrder)}</span>
                    {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                </Button>
            </div>

            {/* View Mode Toggle */}
            <TooltipProvider>
                <div className="flex items-center gap-1 rounded-lg p-1 md:bg-secondary/50">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.button
                                animate={{ scale: 1 }}
                                className={`relative flex size-8.5 cursor-pointer items-center justify-center rounded-md border transition-colors duration-150 hover:bg-secondary ${viewMode === "detailed" ? "bg-secondary" : ""}`}
                                onClick={() => setViewMode("detailed")}
                                onKeyDown={(e) => e.key === "Enter" && setViewMode("detailed")}
                                tabIndex={0}
                                transition={{ duration: 0.15, ease: "easeOut" }}
                                type="button"
                                whileTap={{ scale: 0.85 }}
                            >
                                <AnimatePresence mode="wait">
                                    <motion.div animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }} exit={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} initial={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} key="detailed" transition={{ duration: 0.12, ease: "easeOut" }}>
                                        <LayoutGrid size={15} />
                                    </motion.div>
                                </AnimatePresence>
                            </motion.button>
                        </TooltipTrigger>
                        <TooltipContent>Detailed View</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.button
                                animate={{ scale: 1 }}
                                className={`relative flex size-8.5 cursor-pointer items-center justify-center rounded-md border transition-colors duration-150 hover:bg-secondary ${viewMode === "compact" ? "bg-secondary" : ""}`}
                                onClick={() => setViewMode("compact")}
                                onKeyDown={(e) => e.key === "Enter" && setViewMode("compact")}
                                tabIndex={0}
                                transition={{ duration: 0.15, ease: "easeOut" }}
                                type="button"
                                whileTap={{ scale: 0.85 }}
                            >
                                <AnimatePresence mode="wait">
                                    <motion.div animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }} exit={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} initial={{ opacity: 0, scale: 0.6, filter: "blur(3px)" }} key="compact" transition={{ duration: 0.12, ease: "easeOut" }}>
                                        <Grid3X3 size={15} />
                                    </motion.div>
                                </AnimatePresence>
                            </motion.button>
                        </TooltipTrigger>
                        <TooltipContent>Compact View</TooltipContent>
                    </Tooltip>
                </div>
            </TooltipProvider>
        </div>
    );
}
