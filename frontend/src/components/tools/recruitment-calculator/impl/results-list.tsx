"use client";

import { CombinationResult } from "./combination-result";
import type { TagCombinationResult } from "./types";

interface ResultsListProps {
    results: TagCombinationResult[];
    showLowRarity: boolean;
}

export function ResultsList({ results, showLowRarity }: ResultsListProps) {
    // Separate high-value results (5*+) from regular results
    const highValueResults = results.filter((r) => r.guaranteedRarity >= 5);
    const regularResults = results.filter((r) => r.guaranteedRarity < 5);

    const hasResults = results.length > 0;
    const showRegular = regularResults.length > 0 && (showLowRarity || highValueResults.length === 0);

    if (!hasResults) {
        return (
            <div className="flex flex-col items-center justify-center py-12 text-center">
                <p className="text-muted-foreground">Select tags above to see possible operators</p>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {/* High value combinations */}
            {highValueResults.length > 0 && (
                <div className="space-y-3">
                    <div className="flex items-center gap-2">
                        <span className="font-semibold text-amber-400 text-sm">Guaranteed 5★+ Combinations</span>
                        <span className="rounded bg-amber-500/20 px-1.5 py-0.5 font-medium text-amber-400 text-xs">{highValueResults.length}</span>
                    </div>
                    <div className="space-y-3">
                        {highValueResults.map((result, index) => (
                            <CombinationResult defaultExpanded={index === 0} key={result.tags.join("-")} result={result} />
                        ))}
                    </div>
                </div>
            )}

            {/* Regular combinations */}
            {showRegular && (
                <div className="space-y-3">
                    {highValueResults.length > 0 && <span className="font-semibold text-muted-foreground text-sm">Other Combinations</span>}
                    <div className="space-y-3">
                        {regularResults.map((result, index) => (
                            <CombinationResult defaultExpanded={highValueResults.length === 0 && index === 0} key={result.tags.join("-")} result={result} />
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
