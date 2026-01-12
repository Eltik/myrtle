import type { SearchResultEntry } from "~/types/api";
import { SearchResultCard } from "./search-result-card";

interface SearchResultsGridProps {
    results: SearchResultEntry[];
}

export function SearchResultsGrid({ results }: SearchResultsGridProps) {
    return (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {results.map((result) => (
                <SearchResultCard key={`${result.uid}-${result.server}`} result={result} />
            ))}
        </div>
    );
}
