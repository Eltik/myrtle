import { Search, SearchX } from "lucide-react";
import type * as React from "react";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { ITagCombinationResult } from "../types";
import { ResultCard } from "./ResultCard";
import type { messages } from "./ResultsList.messages";

interface IResultsListProps {
    results: ITagCombinationResult[];
    hasSelection: boolean;
}

export function ResultsList({ results, hasSelection }: IResultsListProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    if (!hasSelection) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <Search />
                    </EmptyMedia>
                    <EmptyTitle>{t("recruit.empty.noSelection.title")}</EmptyTitle>
                    <EmptyDescription>{t("recruit.empty.noSelection.desc")}</EmptyDescription>
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
                    <EmptyTitle>{t("recruit.empty.noResults.title")}</EmptyTitle>
                    <EmptyDescription>{t("recruit.empty.noResults.desc")}</EmptyDescription>
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
