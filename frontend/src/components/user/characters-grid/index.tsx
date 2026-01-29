"use client";

import { Loader2 } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useCDNPrefetch } from "~/hooks/use-cdn-prefetch";
import { useUserCharacters } from "~/hooks/use-user-characters";
import type { CharacterData } from "~/types/api/impl/user";
import type { RarityFilter, SortBy, SortOrder, ViewMode } from "~/types/frontend/impl/user";
import { CharacterCard } from "../character-card";
import { CompactCharacterCard } from "../character-card/compact-card";
import { checkIsMaxed } from "../character-card/impl/helpers";
import { CharacterFilters } from "./impl/character-filters";
import { filterAndSortCharacters } from "./impl/helpers";

// Static UI icons that appear in every dialog
const STATIC_ICONS = [
    // Rarity stars (0-5)
    ...Array.from({ length: 6 }, (_, i) => `/api/cdn/upk/arts/rarity_hub/rarity_yellow_${i}.png`),
    // Elite badges (0-2)
    ...Array.from({ length: 3 }, (_, i) => `/api/cdn/upk/arts/elite_hub/elite_${i}.png`),
    // Potential icons (0-5)
    ...Array.from({ length: 6 }, (_, i) => `/api/cdn/upk/arts/potential_hub/potential_${i}.png`),
    // Mastery icons (1-3)
    ...Array.from({ length: 3 }, (_, i) => `/api/cdn/upk/arts/specialized_hub/specialized_${i + 1}.png`),
];

const DetailedCardWrapper = memo(function DetailedCardWrapper({ char, isLast, lastRef }: { char: CharacterData; isLast: boolean; lastRef: ((node: HTMLDivElement) => void) | null }) {
    const isMaxed = checkIsMaxed(char);

    return (
        <div
            className="flex"
            ref={isLast ? lastRef : null}
            style={
                isMaxed
                    ? {
                          // For maxed cards: disable contentVisibility to allow glow to escape containment
                          position: "relative",
                          zIndex: 10,
                      }
                    : {
                          // For non-maxed cards: use contentVisibility for performance
                          contentVisibility: "auto",
                          containIntrinsicSize: "auto 420px",
                      }
            }
        >
            <CharacterCard data={char} />
        </div>
    );
});

const CompactCardWrapper = memo(function CompactCardWrapper({ char, isLast, lastRef }: { char: CharacterData; isLast: boolean; lastRef: ((node: HTMLDivElement) => void) | null }) {
    const isMaxed = checkIsMaxed(char);

    return (
        <div
            ref={isLast ? lastRef : null}
            style={{
                // Lift maxed cards above neighbors so glow is visible
                position: isMaxed ? "relative" : undefined,
                zIndex: isMaxed ? 10 : undefined,
            }}
        >
            <CompactCharacterCard data={char} />
        </div>
    );
});

interface CharactersGridProps {
    userId: string;
}

export function CharactersGrid({ userId }: CharactersGridProps) {
    const { characters, isLoading, error } = useUserCharacters(userId);
    const [sortBy, setSortBy] = useState<SortBy>("level");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [filterRarity, setFilterRarity] = useState<RarityFilter>("all");
    const [searchTerm, setSearchTerm] = useState("");
    const [displayCount, setDisplayCount] = useState(24); // Start smaller for faster initial render
    const [viewMode, setViewMode] = useState<ViewMode>("detailed");
    const { prefetch } = useCDNPrefetch();
    const hasPreloadedStaticIcons = useRef(false);
    const observer = useRef<IntersectionObserver | null>(null);

    // Preload static UI icons after initial render
    useEffect(() => {
        if (hasPreloadedStaticIcons.current) return;
        hasPreloadedStaticIcons.current = true;

        // Delay prefetch to not compete with initial render
        const timeoutId = setTimeout(() => {
            prefetch(STATIC_ICONS, "low");
        }, 1500);

        return () => clearTimeout(timeoutId);
    }, [prefetch]);

    const sortedAndFilteredCharacters = useMemo(() => {
        if (!characters) return [];
        const chars = Object.values(characters) as (CharacterData & { static?: { name?: string; rarity?: string } })[];
        return filterAndSortCharacters(chars, sortBy, sortOrder, filterRarity, searchTerm);
    }, [characters, sortBy, sortOrder, filterRarity, searchTerm]);

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

    const toggleSortOrder = () => {
        setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
    };

    if (isLoading) {
        return (
            <div className="flex min-h-[400px] items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex min-h-[400px] flex-col items-center justify-center text-center">
                <p className="text-destructive">Failed to load characters</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">{error}</p>
            </div>
        );
    }

    if (!characters || Object.keys(characters).length === 0) {
        return (
            <div className="flex min-h-[400px] flex-col items-center justify-center text-center">
                <p className="text-muted-foreground">No characters found</p>
            </div>
        );
    }

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
