import { Search, SearchX } from "lucide-react";
import type * as React from "react";
import { Card } from "#/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IRosterOverlay, ITagCombinationResult, ResultLayout } from "../types";
import { ResultCard } from "./ResultCard";
import { ResultCardDetailed } from "./ResultCardDetailed";
import type { messages } from "./ResultsList.messages";

interface IResultsListProps {
    results: ITagCombinationResult[];
    hasSelection: boolean;
    roster: IRosterOverlay | null;
    layout: ResultLayout;
}

export function ResultsList({ results, hasSelection, roster, layout }: IResultsListProps): React.ReactElement {
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

    if (layout === "detailed") {
        return (
            <div className="flex flex-col gap-3">
                {results.map((result) => (
                    <ResultCardDetailed key={result.tags.join("-")} result={result} roster={roster} />
                ))}
            </div>
        );
    }

    return (
        <Card className="divide-y divide-border/60 overflow-hidden">
            {results.map((result) => (
                <ResultCard key={result.tags.join("-")} result={result} roster={roster} />
            ))}
        </Card>
    );
}
