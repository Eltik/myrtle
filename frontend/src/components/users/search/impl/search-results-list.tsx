import type { SearchResultEntry } from "~/types/api";
import { SearchResultRow } from "./search-result-row";

interface SearchResultsListProps {
    results: SearchResultEntry[];
}

export function SearchResultsList({ results }: SearchResultsListProps) {
    return (
        <div className="space-y-2">
            {results.map((result) => (
                <SearchResultRow key={`${result.uid}-${result.server}`} result={result} />
            ))}
        </div>
    );
}
