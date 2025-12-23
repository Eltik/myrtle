"use client";

import { ArrowDown, ArrowUp } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { capitalize } from "~/lib/utils";
import type { RarityFilter, SortBy, SortOrder } from "./types";

interface CharacterFiltersProps {
    searchTerm: string;
    setSearchTerm: (value: string) => void;
    sortBy: SortBy;
    setSortBy: (value: SortBy) => void;
    filterRarity: RarityFilter;
    setFilterRarity: (value: RarityFilter) => void;
    sortOrder: SortOrder;
    toggleSortOrder: () => void;
}

export function CharacterFilters({ searchTerm, setSearchTerm, sortBy, setSortBy, filterRarity, setFilterRarity, sortOrder, toggleSortOrder }: CharacterFiltersProps) {
    return (
        <div className="flex w-full flex-col gap-4 sm:flex-row sm:flex-wrap sm:items-center">
            <Input className="w-full sm:w-[280px]" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search operators..." value={searchTerm} />
            <Select onValueChange={(value: SortBy) => setSortBy(value)} value={sortBy}>
                <SelectTrigger className="w-full sm:w-[180px]">
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
    );
}
