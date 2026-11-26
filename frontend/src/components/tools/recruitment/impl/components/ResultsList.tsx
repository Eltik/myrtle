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

const EMPTY = "m-0 flex min-h-40 items-center justify-center rounded-2xl border border-border border-dashed px-6 py-10 text-balance text-center text-muted-foreground text-sm";

export function ResultsList({ results, hasSelection, roster, layout }: IResultsListProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    // A plain sentence, not the kit Empty: an icon in a tinted square read as decoration beside a column of chips.
    // It sits in a dashed frame where the results will land, level with the tag card's top, rather than
    // as one line floating in the empty column.
    if (!hasSelection) return <p className={EMPTY}>{t("recruit.empty.noSelection.desc")}</p>;
    if (results.length === 0) return <p className={EMPTY}>{t("recruit.empty.noResults.desc")}</p>;

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
