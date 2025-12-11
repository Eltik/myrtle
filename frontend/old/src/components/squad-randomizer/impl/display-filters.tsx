import { type Dispatch, type SetStateAction, useCallback } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { formatProfession } from "~/helper";
import { type Operator, OperatorProfession } from "~/types/impl/api/static/operator";

export const DisplayFilters = ({
    filterRarityNumeric,
    filterProfession,
    selectedTags,
    filteredOperators,
    excludedOperators,
    setFilterRarityNumeric,
    setFilterProfession,
    setSelectedTags,
    setExcludedOperators,
}: {
    filterRarityNumeric: Set<number>;
    filterProfession: Set<OperatorProfession>;
    selectedTags: Set<string>;
    filteredOperators: Operator[];
    excludedOperators: Set<string>;
    setFilterRarityNumeric: Dispatch<SetStateAction<Set<number>>>;
    setFilterProfession: Dispatch<SetStateAction<Set<OperatorProfession>>>;
    setSelectedTags: Dispatch<SetStateAction<Set<string>>>;
    setExcludedOperators: Dispatch<SetStateAction<Set<string>>>;
}) => {
    // Use numeric rarity (0-5) for handler
    const handleRarityChange = (numericRarity: number) => {
        setFilterRarityNumeric((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(numericRarity)) {
                newSet.delete(numericRarity);
            } else {
                newSet.add(numericRarity);
            }
            return newSet.size === 0 ? new Set([1, 2, 3, 4, 5, 6]) : newSet;
        });
    };

    const handleProfessionChange = (profession: OperatorProfession) => {
        setFilterProfession((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(profession)) {
                newSet.delete(profession);
            } else {
                newSet.add(profession);
            }
            return newSet.size === 0 ? new Set(Object.values(OperatorProfession)) : newSet;
        });
    };

    // Handler for tag selection/deselection
    const handleTagSelect = (tag: string) => {
        setSelectedTags((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(tag)) {
                newSet.delete(tag);
            } else {
                newSet.add(tag);
            }
            return newSet;
        });
    };

    // Handler to exclude all currently filtered operators
    const handleExcludeAllVisible = useCallback(() => {
        setExcludedOperators((prev) => {
            const newSet = new Set(prev);
            filteredOperators.forEach((op) => {
                if (op.id) {
                    newSet.add(op.id);
                }
            });
            return newSet;
        });
    }, [filteredOperators, setExcludedOperators]); // Depend on filteredOperators

    return (
        <div className="mb-2 flex flex-shrink-0 flex-wrap items-center gap-1">
            {/* Selected Rarity */}
            {Array.from(filterRarityNumeric)
                .sort()
                .filter((num) => num >= 1 && num <= 6)
                .map((num) => (
                    <Badge className="cursor-pointer" key={`rarity-sel-${num}`} onClick={() => handleRarityChange(num)} variant="secondary">
                        {num}★ &times;
                    </Badge>
                ))}
            {/* Selected Profession */}
            {Array.from(filterProfession)
                .sort((a, b) => formatProfession(a).localeCompare(formatProfession(b)))
                .map((prof) => (
                    <Badge className="cursor-pointer" key={`prof-sel-${prof}`} onClick={() => handleProfessionChange(prof)} variant="secondary">
                        {formatProfession(prof)} &times;
                    </Badge>
                ))}
            {/* Selected Tags */}
            {Array.from(selectedTags)
                .sort()
                .map((tag) => (
                    <Badge className="cursor-pointer" key={`tag-sel-${tag}`} onClick={() => handleTagSelect(tag)} variant="secondary">
                        {tag} &times;
                    </Badge>
                ))}
            {/* Clear Buttons Container - Removed ml-auto for testing */}
            <div className="flex items-center gap-2">
                {" "}
                {/* Container for clear buttons */}
                {/* Exclude All Visible Button - Moved outside the conditional below */}
                <Button className="h-auto p-0 px-2 text-destructive text-xs hover:text-destructive/80 disabled:text-muted-foreground/50 disabled:no-underline" disabled={filteredOperators.length === 0} onClick={handleExcludeAllVisible} size="sm" variant="link">
                    Exclude all visible ({filteredOperators.length})
                </Button>
                {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 || excludedOperators.size > 0 ? (
                    <>
                        {/* Clear Excluded Button */}
                        {excludedOperators.size > 0 && (
                            <Button className="h-auto p-0 px-2 text-destructive text-xs hover:text-destructive/80" onClick={() => setExcludedOperators(new Set())} size="sm" variant="link">
                                Clear excluded ({excludedOperators.size})
                            </Button>
                        )}
                        {/* Clear Filters Button */}
                        {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 ? (
                            <Button
                                className="h-auto p-0 px-2 text-muted-foreground text-xs"
                                onClick={() => {
                                    setFilterRarityNumeric(new Set([1, 2, 3, 4, 5, 6]));
                                    setFilterProfession(new Set(Object.values(OperatorProfession)));
                                    setSelectedTags(new Set());
                                }}
                                size="sm"
                                variant="link"
                            >
                                Clear filters
                            </Button>
                        ) : null}
                    </>
                ) : null}
            </div>
        </div>
    );
};
