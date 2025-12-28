"use client";

import { CombinationResult } from "./combination-result";
import type { TagCombinationResult } from "./types";

interface ResultsListProps {
    results: TagCombinationResult[];
}

export function ResultsList({ results }: ResultsListProps) {
    // Separate high-value results (5*+) from regular results
    const highValueResults = results.filter((r) => r.guaranteedRarity >= 5);
    const regularResults = results.filter((r) => r.guaranteedRarity < 5);

    const hasResults = results.length > 0;

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
                    <div className="space-y-3">
                        {highValueResults.map((result, index) => (
                            <CombinationResult defaultExpanded={index === 0} key={result.tags.join("-")} result={result} />
                        ))}
                    </div>
                </div>
            )}

            {/* Regular combinations */}
            {regularResults.length > 0 && (
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
