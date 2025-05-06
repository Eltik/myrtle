import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { type Operator, OperatorProfession } from "~/types/impl/api/static/operator";
import { formatProfession } from "~/helper";
import { type Dispatch, type SetStateAction, useCallback } from "react";

export const DisplayFilters = ({ filterRarityNumeric, filterProfession, selectedTags, filteredOperators, excludedOperators, setFilterRarityNumeric, setFilterProfession, setSelectedTags, setExcludedOperators }: { filterRarityNumeric: Set<number>; filterProfession: Set<OperatorProfession>; selectedTags: Set<string>; filteredOperators: Operator[]; excludedOperators: Set<string>; setFilterRarityNumeric: Dispatch<SetStateAction<Set<number>>>; setFilterProfession: Dispatch<SetStateAction<Set<OperatorProfession>>>; setSelectedTags: Dispatch<SetStateAction<Set<string>>>; setExcludedOperators: Dispatch<SetStateAction<Set<string>>> }) => {
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
        <>
            <div className="mb-2 flex flex-shrink-0 flex-wrap items-center gap-1">
                {/* Selected Rarity */}
                {Array.from(filterRarityNumeric)
                    .sort()
                    .filter((num) => num >= 1 && num <= 6)
                    .map((num) => (
                        <Badge key={`rarity-sel-${num}`} variant="secondary" className="cursor-pointer" onClick={() => handleRarityChange(num)}>
                            {num}★ &times;
                        </Badge>
                    ))}
                {/* Selected Profession */}
                {Array.from(filterProfession)
                    .sort((a, b) => formatProfession(a).localeCompare(formatProfession(b)))
                    .map((prof) => (
                        <Badge key={`prof-sel-${prof}`} variant="secondary" className="cursor-pointer" onClick={() => handleProfessionChange(prof)}>
                            {formatProfession(prof)} &times;
                        </Badge>
                    ))}
                {/* Selected Tags */}
                {Array.from(selectedTags)
                    .sort()
                    .map((tag) => (
                        <Badge key={`tag-sel-${tag}`} variant="secondary" className="cursor-pointer" onClick={() => handleTagSelect(tag)}>
                            {tag} &times;
                        </Badge>
                    ))}
                {/* Clear Buttons Container - Removed ml-auto for testing */}
                <div className="flex items-center gap-2">
                    {" "}
                    {/* Container for clear buttons */}
                    {/* Exclude All Visible Button - Moved outside the conditional below */}
                    <Button variant="link" size="sm" className="h-auto p-0 px-2 text-xs text-destructive hover:text-destructive/80 disabled:text-muted-foreground/50 disabled:no-underline" onClick={handleExcludeAllVisible} disabled={filteredOperators.length === 0}>
                        Exclude all visible ({filteredOperators.length})
                    </Button>
                    {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 || excludedOperators.size > 0 ? (
                        <>
                            {/* Clear Excluded Button */}
                            {excludedOperators.size > 0 && (
                                <Button variant="link" size="sm" className="h-auto p-0 px-2 text-xs text-destructive hover:text-destructive/80" onClick={() => setExcludedOperators(new Set())}>
                                    Clear excluded ({excludedOperators.size})
                                </Button>
                            )}
                            {/* Clear Filters Button */}
                            {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 ? (
                                <Button
                                    variant="link"
                                    size="sm"
                                    className="h-auto p-0 px-2 text-xs text-muted-foreground"
                                    onClick={() => {
                                        setFilterRarityNumeric(new Set([1, 2, 3, 4, 5, 6]));
                                        setFilterProfession(new Set(Object.values(OperatorProfession)));
                                        setSelectedTags(new Set());
                                    }}
                                >
                                    Clear filters
                                </Button>
                            ) : null}
                        </>
                    ) : null}
                </div>
            </div>
        </>
    );
};
