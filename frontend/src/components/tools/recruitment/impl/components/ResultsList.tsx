import { Search, SearchX } from "lucide-react";
import type * as React from "react";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import type { ITagCombinationResult } from "../types";
import { ResultCard } from "./ResultCard";

interface IResultsListProps {
    results: ITagCombinationResult[];
    hasSelection: boolean;
}

export function ResultsList({ results, hasSelection }: IResultsListProps): React.ReactElement {
    if (!hasSelection) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <Search />
                    </EmptyMedia>
                    <EmptyTitle>Pick tags to see combinations</EmptyTitle>
                    <EmptyDescription>Select the tags shown in your in-game recruitment screen. Combinations are ranked by guaranteed minimum rarity.</EmptyDescription>
                </EmptyHeader>
            </Empty>
        );
    }

    if (results.length === 0) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <SearchX />
                    </EmptyMedia>
                    <EmptyTitle>No matching operators</EmptyTitle>
                    <EmptyDescription>No recruitable operators match the current tags and options. Try removing a tag or enabling more options.</EmptyDescription>
                </EmptyHeader>
            </Empty>
        );
    }

    return (
        <div className="flex flex-col gap-3">
            {results.map((result) => (
                <ResultCard key={result.tags.join("-")} result={result} />
            ))}
        </div>
    );
}
