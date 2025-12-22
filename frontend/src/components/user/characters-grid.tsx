"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ArrowDown, ArrowUp } from "lucide-react";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Button } from "~/components/ui/shadcn/button";
import { CharacterCard } from "./character-card";
import type { User, CharacterData } from "~/types/api/impl/user";

type SortBy = "level" | "rarity" | "obtained" | "potential";
type SortOrder = "asc" | "desc";
type RarityFilter = "all" | "TIER_6" | "TIER_5" | "TIER_4" | "TIER_3" | "TIER_2" | "TIER_1";

function getRarityNumber(rarity: string): number {
    const match = rarity?.match(/TIER_(\d)/);
    return match ? Number.parseInt(match[1] ?? "", 10) : 0;
}

function capitalize(s: string): string {
    return s ? s.charAt(0).toUpperCase() + s.slice(1).toLowerCase() : "";
}

interface CharactersGridProps {
    data: User;
}

export function CharactersGrid({ data }: CharactersGridProps) {
    const [sortBy, setSortBy] = useState<SortBy>("level");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [filterRarity, setFilterRarity] = useState<RarityFilter>("all");
    const [searchTerm, setSearchTerm] = useState("");
    const [displayCount, setDisplayCount] = useState(40);

    const sortedAndFilteredCharacters = useMemo(() => {
        const chars = Object.values(data.troop.chars) as (CharacterData & { static?: { name?: string; rarity?: string } })[];

        return chars
            .filter((char) => {
                const charRarity = (char.static?.rarity ?? "TIER_1") as string;
                const charName = (char.static?.name ?? "").toLowerCase();
                const matchesRarity = filterRarity === "all" || charRarity === filterRarity;
                const matchesSearch = charName.includes(searchTerm.toLowerCase());
                return matchesRarity && matchesSearch;
            })
            .sort((a, b) => {
                let comparison = 0;
                switch (sortBy) {
                    case "level":
                        if (b.evolvePhase !== a.evolvePhase) {
                            comparison = b.evolvePhase - a.evolvePhase;
                        } else {
                            comparison = b.level - a.level;
                        }
                        break;
                    case "rarity":
                        comparison = getRarityNumber(b.static?.rarity ?? "TIER_1") - getRarityNumber(a.static?.rarity ?? "TIER_1");
                        break;
                    case "obtained":
                        comparison = b.gainTime - a.gainTime;
                        break;
                    case "potential":
                        comparison = (b.potentialRank ?? 0) - (a.potentialRank ?? 0);
                        break;
                }
                return sortOrder === "asc" ? -comparison : comparison;
            });
    }, [data.troop.chars, sortBy, sortOrder, filterRarity, searchTerm]);

    const toggleSortOrder = () => {
        setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
    };

    const observer = useRef<IntersectionObserver | null>(null);
    const lastCharacterRef = useCallback(
        (node: HTMLDivElement) => {
            if (observer.current) observer.current.disconnect();
            observer.current = new IntersectionObserver((entries) => {
                if (entries[0]?.isIntersecting && displayCount < sortedAndFilteredCharacters.length) {
                    setDisplayCount((prevCount) => Math.min(prevCount + 40, sortedAndFilteredCharacters.length));
                }
            });
            if (node) observer.current.observe(node);
        },
        [displayCount, sortedAndFilteredCharacters.length],
    );

    useEffect(() => {
        setDisplayCount(40);
    }, [sortBy, sortOrder, filterRarity, searchTerm]);

    return (
        <div className="flex w-full flex-col space-y-6">
            {/* Filter Controls */}
            <div className="flex w-full flex-col gap-4 sm:flex-row sm:flex-wrap sm:items-center">
                <Input placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="w-full sm:w-[280px]" />
                <Select value={sortBy} onValueChange={(value: SortBy) => setSortBy(value)}>
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
                <Select value={filterRarity} onValueChange={(value: RarityFilter) => setFilterRarity(value)}>
                    <SelectTrigger className="w-full sm:w-[160px]">
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
                <Button onClick={toggleSortOrder} variant="outline" className="flex w-full items-center justify-center gap-2 bg-transparent sm:w-auto">
                    <span>{capitalize(sortOrder)}</span>
                    {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                </Button>
            </div>

            {/* Operator Grid */}
            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                    <div key={char.charId} ref={index === displayCount - 1 ? lastCharacterRef : null} className="flex">
                        <CharacterCard data={char} />
                    </div>
                ))}
            </div>

            {/* Loading indicator */}
            {displayCount < sortedAndFilteredCharacters.length && (
                <div className="flex justify-center py-4">
                    <p className="text-muted-foreground text-sm">
                        Showing {displayCount} of {sortedAndFilteredCharacters.length} operators. Scroll to load more.
                    </p>
                </div>
            )}
        </div>
    );
}
