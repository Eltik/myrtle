import type * as React from "react";
import { Card } from "#/components/ui/card";
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
    // A plain sentence, not the kit Empty: an icon in a tinted square read as decoration beside a column of chips.
    if (!hasSelection) return <p className="px-1 py-2 text-muted-foreground text-sm">{t("recruit.empty.noSelection.desc")}</p>;
    if (results.length === 0) return <p className="px-1 py-2 text-muted-foreground text-sm">{t("recruit.empty.noResults.desc")}</p>;

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
