"use client";

import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CharacterData, User } from "~/types/api/impl/user";
import { CharacterCard } from "../character-card";
import { CompactCharacterCard } from "../character-card/compact-card";
import { CharacterFilters } from "./impl/character-filters";
import { filterAndSortCharacters } from "./impl/helpers";
import type { RarityFilter, SortBy, SortOrder, ViewMode } from "./impl/types";

// Performance-optimized card wrapper with CSS containment
// This allows the browser to skip rendering cards that are off-screen
const DetailedCardWrapper = memo(function DetailedCardWrapper({ char, isLast, lastRef }: { char: CharacterData; isLast: boolean; lastRef: ((node: HTMLDivElement) => void) | null }) {
    return (
        <div
            className="flex"
            ref={isLast ? lastRef : null}
            style={{
                contentVisibility: "auto",
                containIntrinsicSize: "auto 420px", // Estimated card height for layout calculation
            }}
        >
            <CharacterCard data={char} />
        </div>
    );
});

const CompactCardWrapper = memo(function CompactCardWrapper({ char, isLast, lastRef }: { char: CharacterData; isLast: boolean; lastRef: ((node: HTMLDivElement) => void) | null }) {
    // Note: No CSS containment here as compact cards have badges that overflow their bounds
    return (
        <div ref={isLast ? lastRef : null}>
            <CompactCharacterCard data={char} />
        </div>
    );
});

interface CharactersGridProps {
    data: User;
}

export function CharactersGrid({ data }: CharactersGridProps) {
    const [sortBy, setSortBy] = useState<SortBy>("level");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [filterRarity, setFilterRarity] = useState<RarityFilter>("all");
    const [searchTerm, setSearchTerm] = useState("");
    const [displayCount, setDisplayCount] = useState(24); // Start smaller for faster initial render
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
            observer.current = new IntersectionObserver(
                (entries) => {
                    if (entries[0]?.isIntersecting && displayCount < sortedAndFilteredCharacters.length) {
                        // Load in smaller batches for smoother scrolling
                        setDisplayCount((prevCount) => Math.min(prevCount + 24, sortedAndFilteredCharacters.length));
                    }
                },
                { rootMargin: "200px" }, // Start loading before reaching the end
            );
            if (node) observer.current.observe(node);
        },
        [displayCount, sortedAndFilteredCharacters.length],
    );

    // Reset display count when filtered list length or view mode changes
    const totalCount = sortedAndFilteredCharacters.length;
    useEffect(() => {
        const initialCount = viewMode === "compact" ? 48 : 24;
        setDisplayCount(Math.min(initialCount, totalCount));
    }, [totalCount, viewMode]);

    return (
        <div className="flex w-full flex-col space-y-6">
            <CharacterFilters filterRarity={filterRarity} searchTerm={searchTerm} setFilterRarity={setFilterRarity} setSearchTerm={setSearchTerm} setSortBy={setSortBy} setViewMode={setViewMode} sortBy={sortBy} sortOrder={sortOrder} toggleSortOrder={toggleSortOrder} viewMode={viewMode} />

            {viewMode === "detailed" ? (
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                        <DetailedCardWrapper char={char} isLast={index === displayCount - 1} key={char.charId} lastRef={lastCharacterRef} />
                    ))}
                </div>
            ) : (
                <div className="grid 3xl:grid-cols-7 grid-cols-3 justify-center gap-3 sm:grid-cols-4 sm:justify-start lg:grid-cols-5 xl:grid-cols-6">
                    {sortedAndFilteredCharacters.slice(0, displayCount).map((char, index) => (
                        <CompactCardWrapper char={char} isLast={index === displayCount - 1} key={char.charId} lastRef={lastCharacterRef} />
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
