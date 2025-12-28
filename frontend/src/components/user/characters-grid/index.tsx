"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CharacterData, User } from "~/types/api/impl/user";
import { CharacterCard } from "../character-card";
import { CompactCharacterCard } from "../character-card/compact-card";
import { CharacterFilters } from "./impl/character-filters";
import { filterAndSortCharacters } from "./impl/helpers";
import type { RarityFilter, SortBy, SortOrder, ViewMode } from "./impl/types";

interface CharactersGridProps {
    data: User;
}

export function CharactersGrid({ data }: CharactersGridProps) {
    const [sortBy, setSortBy] = useState<SortBy>("level");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [filterRarity, setFilterRarity] = useState<RarityFilter>("all");
    const [searchTerm, setSearchTerm] = useState("");
    const [displayCount, setDisplayCount] = useState(40);
    const [viewMode, setViewMode] = useState<ViewMode>("detailed");

    const sortedAndFilteredCharacters = useMemo(() => {
        const chars = Object.values(data.troop.chars) as (CharacterData & { static?: { name?: string; rarity?: string } })[];
        return filterAndSortCharacters(chars, sortBy, sortOrder, filterRarity, searchTerm);
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
    }, []);

    // Increase initial display count for compact view since cards are smaller
    useEffect(() => {
        if (viewMode === "compact") {
            setDisplayCount(80);
        } else {
            setDisplayCount(40);
        }
    }, [viewMode]);

    return (
        <div className="flex w-full flex-col space-y-6">
            <CharacterFilters filterRarity={filterRarity} searchTerm={searchTerm} setFilterRarity={setFilterRarity} setSearchTerm={setSearchTerm} setSortBy={setSortBy} setViewMode={setViewMode} sortBy={sortBy} sortOrder={sortOrder} toggleSortOrder={toggleSortOrder} viewMode={viewMode} />

            {viewMode === "detailed" ? (
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                        <div className="flex" key={char.charId} ref={index === displayCount - 1 ? lastCharacterRef : null}>
                            <CharacterCard data={char} />
                        </div>
                    ))}
                </div>
            ) : (
                <div className="flex flex-wrap justify-center gap-3 sm:justify-start">
                    {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                        <div key={char.charId} ref={index === displayCount - 1 ? lastCharacterRef : null}>
                            <CompactCharacterCard data={char} />
                        </div>
                    ))}
                </div>
            )}

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
