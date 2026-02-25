"use client";

import { Loader2 } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useCDNPrefetch } from "~/hooks/use-cdn-prefetch";
import { useUnownedOperators } from "~/hooks/use-unowned-operators";
import { useUserCharacters } from "~/hooks/use-user-characters";
import type { CharacterData, DisplayCharacter } from "~/types/api/impl/user";
import type { OwnershipFilter, RarityFilter, SortBy, SortOrder, ViewMode } from "~/types/frontend/impl/user";
import { CharacterCard } from "../character-card";
import { CompactCharacterCard } from "../character-card/compact-card";
import { checkIsMaxed } from "../character-card/impl/helpers";
import { UnownedCharacterCard } from "../character-card/unowned-card";
import { CharacterFilters } from "./impl/character-filters";
import { filterAndSortCharacters, filterAndSortDisplayCharacters } from "./impl/helpers";

const STATIC_ICONS = [
    ...Array.from({ length: 6 }, (_, i) => `/api/cdn/upk/arts/rarity_hub/rarity_yellow_${i}.png`),
    ...Array.from({ length: 3 }, (_, i) => `/api/cdn/upk/arts/elite_hub/elite_${i}.png`),
    ...Array.from({ length: 6 }, (_, i) => `/api/cdn/upk/arts/potential_hub/potential_${i}.png`),
    ...Array.from({ length: 3 }, (_, i) => `/api/cdn/upk/arts/specialized_hub/specialized_${i + 1}.png`),
];

const OWNED_ONLY_SORTS = new Set<SortBy>(["level", "obtained", "potential"]);

const DetailedCardWrapper = memo(function DetailedCardWrapper({ char, isLast, lastRef }: { char: CharacterData; isLast: boolean; lastRef: ((node: HTMLDivElement) => void) | null }) {
    const isMaxed = checkIsMaxed(char);

    return (
        <div className="flex" ref={isLast ? lastRef : null} style={isMaxed ? { position: "relative", zIndex: 10 } : { contentVisibility: "auto", containIntrinsicSize: "auto 420px" }}>
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
    const [displayCount, setDisplayCount] = useState(24);
    const [viewMode, setViewMode] = useState<ViewMode>("detailed");
    const [ownershipFilter, setOwnershipFilter] = useState<OwnershipFilter>("owned");
    const { prefetch } = useCDNPrefetch();
    const hasPreloadedStaticIcons = useRef(false);

    const { unownedOperators, isLoading: unownedLoading } = useUnownedOperators(userId, ownershipFilter !== "owned");

    // Auto-switch sort to "rarity" if current sort is inapplicable for unowned
    const handleOwnershipChange = useCallback(
        (value: OwnershipFilter) => {
            setOwnershipFilter(value);
            if (value === "unowned" && OWNED_ONLY_SORTS.has(sortBy)) {
                setSortBy("rarity");
            }
        },
        [sortBy],
    );

    // Set responsive default: compact on mobile, detailed on desktop
    const hasSetInitialViewMode = useRef(false);
    useEffect(() => {
        if (hasSetInitialViewMode.current) return;
        hasSetInitialViewMode.current = true;
        setViewMode(window.innerWidth < 768 ? "compact" : "detailed");
    }, []);

    const observer = useRef<IntersectionObserver | null>(null);

    useEffect(() => {
        if (hasPreloadedStaticIcons.current) return;
        hasPreloadedStaticIcons.current = true;
        const timeoutId = setTimeout(() => {
            prefetch(STATIC_ICONS, "low");
        }, 1500);
        return () => clearTimeout(timeoutId);
    }, [prefetch]);

    // Build combined DisplayCharacter list when using ownership filter modes
    const displayCharacters = useMemo((): DisplayCharacter[] | null => {
        if (ownershipFilter === "owned") return null; // Use the original path

        const owned: DisplayCharacter[] = characters ? (Object.values(characters) as CharacterData[]).map((c) => ({ ...c, isOwned: true as const })) : [];
        const unowned: DisplayCharacter[] = unownedOperators ?? [];

        if (ownershipFilter === "unowned") return unowned;
        return [...owned, ...unowned];
    }, [ownershipFilter, characters, unownedOperators]);

    const sortedAndFilteredCharacters = useMemo(() => {
        if (!characters) return [];
        const chars = Object.values(characters) as (CharacterData & { static?: { name?: string; rarity?: string } })[];
        return filterAndSortCharacters(chars, sortBy, sortOrder, filterRarity, searchTerm);
    }, [characters, sortBy, sortOrder, filterRarity, searchTerm]);

    const sortedAndFilteredDisplay = useMemo(() => {
        if (!displayCharacters) return [];
        return filterAndSortDisplayCharacters(displayCharacters, sortBy, sortOrder, filterRarity, searchTerm);
    }, [displayCharacters, sortBy, sortOrder, filterRarity, searchTerm]);

    // Choose which list to render based on ownership filter
    const activeList = ownershipFilter === "owned" ? sortedAndFilteredCharacters : sortedAndFilteredDisplay;

    const lastCharacterRef = useCallback(
        (node: HTMLDivElement) => {
            if (observer.current) observer.current.disconnect();
            observer.current = new IntersectionObserver(
                (entries) => {
                    if (entries[0]?.isIntersecting && displayCount < activeList.length) {
                        setDisplayCount((prevCount) => Math.min(prevCount + 24, activeList.length));
                    }
                },
                { rootMargin: "200px" },
            );
            if (node) observer.current.observe(node);
        },
        [displayCount, activeList.length],
    );

    const totalCount = activeList.length;
    useEffect(() => {
        const initialCount = viewMode === "compact" ? 48 : 24;
        setDisplayCount(Math.min(initialCount, totalCount));
    }, [totalCount, viewMode]);

    const toggleSortOrder = () => {
        setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
    };

    if (isLoading) {
        return (
            <div className="flex min-h-100 items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex min-h-100 flex-col items-center justify-center text-center">
                <p className="text-destructive">Failed to load characters</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">{error}</p>
            </div>
        );
    }

    if (!characters || Object.keys(characters).length === 0) {
        return (
            <div className="flex min-h-100 flex-col items-center justify-center text-center">
                <p className="text-muted-foreground">No characters found</p>
            </div>
        );
    }

    const renderCard = (char: DisplayCharacter | CharacterData, index: number) => {
        const isLast = index === displayCount - 1;
        const key = "isOwned" in char && !char.isOwned ? `unowned-${char.charId}` : char.charId;

        // Unowned character
        if ("isOwned" in char && !char.isOwned) {
            return (
                <div key={key} ref={isLast ? lastCharacterRef : null}>
                    <UnownedCharacterCard data={char} viewMode={viewMode} />
                </div>
            );
        }

        // Owned character
        const ownedChar = char as CharacterData;
        if (viewMode === "detailed") {
            return <DetailedCardWrapper char={ownedChar} isLast={isLast} key={key} lastRef={lastCharacterRef} />;
        }
        return <CompactCardWrapper char={ownedChar} isLast={isLast} key={key} lastRef={lastCharacterRef} />;
    };

    const showUnownedLoading = ownershipFilter !== "owned" && unownedLoading;

    return (
        <div className="flex w-full flex-col space-y-6">
            <CharacterFilters
                filterRarity={filterRarity}
                ownershipFilter={ownershipFilter}
                searchTerm={searchTerm}
                setFilterRarity={setFilterRarity}
                setOwnershipFilter={handleOwnershipChange}
                setSearchTerm={setSearchTerm}
                setSortBy={setSortBy}
                setViewMode={setViewMode}
                sortBy={sortBy}
                sortOrder={sortOrder}
                toggleSortOrder={toggleSortOrder}
                viewMode={viewMode}
            />

            {showUnownedLoading ? (
                <div className="flex min-h-40 items-center justify-center">
                    <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
            ) : ownershipFilter === "owned" ? (
                // Original owned-only rendering path (unchanged)
                viewMode === "detailed" ? (
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
                )
            ) : viewMode === "detailed" ? (
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">{sortedAndFilteredDisplay.slice(0, displayCount).map((char, index) => renderCard(char, index))}</div>
            ) : (
                <div className="grid 3xl:grid-cols-7 grid-cols-3 justify-center gap-3 sm:grid-cols-4 sm:justify-start lg:grid-cols-5 xl:grid-cols-6">{sortedAndFilteredDisplay.slice(0, displayCount).map((char, index) => renderCard(char, index))}</div>
            )}

            {displayCount < activeList.length && (
                <div className="flex justify-center py-4">
                    <p className="text-muted-foreground text-sm">
                        Showing {displayCount} of {activeList.length} operators. Scroll to load more.
                    </p>
                </div>
            )}
        </div>
    );
}
