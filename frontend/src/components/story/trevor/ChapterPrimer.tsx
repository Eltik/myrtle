/**
 * "Before you start" in the chapter sheet (provisional, behind `VITE_TREVOR=1`).
 *
 * The primer is the panel of the chapter's FIRST story: the panel builder puts
 * the full primer text only on a group's first stories (85 of the 1,862 panels)
 * and a bank link on the rest, and it is gated at that story, so it says
 * nothing the first page of the chapter would not. A chapter whose first story
 * has no primer renders nothing at all, not an empty band.
 */
import { useQuery } from "@tanstack/react-query";
import { ChevronDownIcon } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
import { trevorPanelsQueryOptions } from "#/lib/api/trevor";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { parsePanel, primerOf } from "#/lib/trevor/panels";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

export function ChapterPrimer({ storyId }: { storyId: string | null }): React.ReactElement | null {
    const t: TrevorT = useT("story");
    const { data } = useQuery({ ...trevorPanelsQueryOptions(storyId ?? "", null), enabled: storyId !== null });
    const panel = data ? parsePanel(data.panels) : null;
    const primer = primerOf(panel);
    if (!primer) return null;
    const buildsOn = panel?.recap?.buildsOn ?? [];
    return (
        <section className="border-border border-b px-4 py-2 sm:px-5" data-trevor-primer>
            <Collapsible>
                <CollapsibleTrigger className="group flex min-h-11 w-full items-center justify-between gap-2 rounded-md text-start font-medium text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:min-h-9">
                    {t("trevor.primer.heading")}
                    <ChevronDownIcon className="size-4 shrink-0 text-muted-foreground transition-transform group-data-[panel-open]:rotate-180" aria-hidden />
                </CollapsibleTrigger>
                <CollapsiblePanel>
                    <div className="flex flex-col gap-2 pt-1 pb-2">
                        <p className="text-sm leading-relaxed">{primer}</p>
                        {buildsOn.length > 0 ? (
                            <div className="flex flex-wrap items-center gap-1.5">
                                <span className="text-muted-foreground text-xs">{t("trevor.recap.buildsOn")}</span>
                                {buildsOn.map((name) => (
                                    <Badge key={name} variant="outline">
                                        {name}
                                    </Badge>
                                ))}
                            </div>
                        ) : null}
                        <p className="text-muted-foreground text-xs">{t("trevor.primer.note")}</p>
                    </div>
                </CollapsiblePanel>
            </Collapsible>
        </section>
    );
}
