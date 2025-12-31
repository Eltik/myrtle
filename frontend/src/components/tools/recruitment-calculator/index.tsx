"use client";

import { Calculator, RotateCcw } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { calculateResults } from "./impl/client-calculator";
import { MAX_SELECTED_TAGS } from "./impl/constants";
import { FilterOptions } from "./impl/filter-options";
import { groupTagsByType, transformTags } from "./impl/helpers";
import { ResultsList } from "./impl/results-list";
import { TagSelector } from "./impl/tag-selector";
import type { GachaTag, OperatorSortMode, RecruitableOperatorWithTags } from "./impl/types";

const STORAGE_KEY_SORT_MODE = "recruitment-operator-sort-mode";

interface RecruitmentCalculatorProps {
    tags: GachaTag[];
    recruitableOperators: RecruitableOperatorWithTags[];
}

export function RecruitmentCalculator({ tags, recruitableOperators }: RecruitmentCalculatorProps) {
    const [selectedTags, setSelectedTags] = useState<number[]>([]);
    const [includeRobots, setIncludeRobots] = useState(true);
    const [operatorSortMode, setOperatorSortMode] = useState<OperatorSortMode>("rarity-desc");

    // Load saved sort mode from localStorage
    useEffect(() => {
        const saved = localStorage.getItem(STORAGE_KEY_SORT_MODE);
        if (saved === "common-first" || saved === "rarity-desc") {
            setOperatorSortMode(saved);
        }
    }, []);

    // Persist sort mode to localStorage
    const handleSortModeChange = useCallback((mode: OperatorSortMode) => {
        setOperatorSortMode(mode);
        localStorage.setItem(STORAGE_KEY_SORT_MODE, mode);
    }, []);

    // Transform and group tags
    const transformedTags = useMemo(() => transformTags(tags), [tags]);
    const groupedTags = useMemo(() => groupTagsByType(transformedTags), [transformedTags]);

    const handleTagToggle = useCallback((tagId: number) => {
        setSelectedTags((prev) => {
            if (prev.includes(tagId)) {
                return prev.filter((t) => t !== tagId);
            }
            if (prev.length >= MAX_SELECTED_TAGS) {
                return prev;
            }
            return [...prev, tagId];
        });
    }, []);

    const handleClearAll = useCallback(() => {
        setSelectedTags([]);
    }, []);

    // Convert selected tag IDs to tag objects for calculation
    const selectedTagObjects = useMemo(() => {
        return selectedTags
            .map((id) => {
                const tag = transformedTags.find((t) => t.id === id);
                return tag ? { id: tag.id, name: tag.name } : null;
            })
            .filter(Boolean) as { id: number; name: string }[];
    }, [selectedTags, transformedTags]);

    // Calculate results
    const results = useMemo(() => {
        if (selectedTagObjects.length === 0) {
            return [];
        }

        return calculateResults(selectedTagObjects, recruitableOperators, {
            showLowRarity: true, // Always show all rarity groups
            includeRobots,
            operatorSortMode,
        });
    }, [selectedTagObjects, recruitableOperators, includeRobots, operatorSortMode]);

    const highValueCount = results.filter((r) => r.guaranteedRarity >= 5).length;

    return (
        <div className="space-y-8">
            {/* Header */}
            <div className="space-y-2">
                <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                        <Calculator className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                        <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl">Recruitment Calculator</h1>
                    </div>
                </div>
                <p className="max-w-2xl text-muted-foreground">Select up to 5 tags to calculate possible operator outcomes. Combinations are sorted by guaranteed rarity.</p>
            </div>

            {/* Tag Selection Section */}
            <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <h2 className="font-semibold text-foreground">Select Tags</h2>
                        <span className="rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground text-xs">
                            {selectedTags.length}/{MAX_SELECTED_TAGS}
                        </span>
                    </div>
                    {selectedTags.length > 0 && (
                        <Button className="gap-2" onClick={handleClearAll} size="sm" variant="ghost">
                            <RotateCcw className="h-3.5 w-3.5" />
                            Reset
                        </Button>
                    )}
                </div>

                <TagSelector maxTags={MAX_SELECTED_TAGS} onTagToggle={handleTagToggle} selectedTags={selectedTags} tags={groupedTags} />
            </div>

            {/* Options & Results Section */}
            <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
                <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
                    <div className="flex items-center gap-2">
                        <h2 className="font-semibold text-foreground">Results</h2>
                        {results.length > 0 && (
                            <>
                                <span className="rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground text-xs">{results.length} combinations</span>
                                {highValueCount > 0 && <span className="rounded-full bg-amber-500/15 px-2 py-0.5 font-medium text-amber-700 text-xs dark:bg-amber-500/20 dark:text-amber-400">{highValueCount} high-value</span>}
                            </>
                        )}
                    </div>
                    <FilterOptions includeRobots={includeRobots} onIncludeRobotsChange={setIncludeRobots} onOperatorSortModeChange={handleSortModeChange} operatorSortMode={operatorSortMode} />
                </div>

                <ResultsList results={results} />
            </div>

            {/* Footer note */}
            <p className="text-center text-muted-foreground text-xs">Note: This calculator shows all possible operators for tag combinations. Actual recruitment pool may vary based on your server and available operators.</p>
        </div>
    );
}
