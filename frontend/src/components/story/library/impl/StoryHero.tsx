import { Link } from "@tanstack/react-router";
import { BookOpenIcon, ListTreeIcon, PlayIcon, RotateCcwIcon } from "lucide-react";
import type React from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { cn } from "#/lib/utils";
import type { messages as sharedMessages } from "../../shared.messages";
import { groupCode, type IHeroPick, readFraction } from "./derive";
import type { messages } from "./StoryHero.messages";

export interface IStoryHeroProps {
    pick: IHeroPick;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    onViewChapter: (groupId: string) => void;
}

/**
 * The card at the top of the library: the story the reader left off in, over
 * its chapter's cover art. All 17 mainline groups ship without a `coverUrl`
 * today, so the typographic panel is the common case here, not the exception.
 */
export function StoryHero({ pick, progress, gameRead, onViewChapter }: IStoryHeroProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const tc: TypedT<typeof sharedMessages> = useT("story");
    const { entry, group, recordName, resumeHalt, resumeTotal, fresh } = pick;
    const fraction = readFraction(group.stories, progress, gameRead);
    const code = groupCode(group);
    const cover = group.coverUrl ? asset(group.coverUrl) : null;
    const chapterLine = recordName ?? [group.zone?.nameThird, group.name].filter(Boolean).join(": ");

    return (
        <section className="relative isolate mb-6 overflow-hidden rounded-[14px] border border-border bg-card sm:mb-7">
            {cover ? (
                <img src={cover} alt="" aria-hidden="true" loading="eager" decoding="async" className="pointer-events-none absolute inset-0 h-full w-full object-cover object-center opacity-45" />
            ) : (
                <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 right-0 flex w-1/2 items-center justify-end overflow-hidden whitespace-nowrap pr-4 font-black font-heading text-[56px] text-foreground/4 leading-none tracking-tighter sm:pr-8 sm:text-[110px]">
                    {code}
                </span>
            )}
            <span aria-hidden="true" className="pointer-events-none absolute inset-0 bg-gradient-to-r from-card via-card/92 to-card/55" />
            <span aria-hidden="true" className="pointer-events-none absolute inset-0 bg-gradient-to-t from-card/85 to-transparent" />

            <div className="relative flex flex-col gap-4 p-4 sm:gap-5 sm:p-7">
                <div className="flex min-w-0 flex-col gap-1.5">
                    <div className="flex items-center gap-1.5 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">
                        <BookOpenIcon className="size-3" aria-hidden="true" />
                        {fresh ? t("hero.kickerFresh") : t("hero.kicker")}
                    </div>
                    <nav aria-label={t("hero.kicker")} className="flex flex-wrap items-baseline gap-x-2 font-sans text-[12px] text-muted-foreground">
                        <span>{tc(`category.${group.category}`)}</span>
                        <span aria-hidden="true">/</span>
                        <span className="text-foreground/80">{chapterLine}</span>
                    </nav>
                    <h2 className="wrap-break-word m-0 font-bold font-sans text-[22px] text-foreground leading-[1.15] tracking-tight sm:text-[30px]">{entry.name}</h2>
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground tabular-nums">
                        <span>{t("hero.groupRead", { read: fraction.read, total: fraction.total })}</span>
                        <span aria-hidden="true" className="h-3 w-px bg-border" />
                        <span>{resumeHalt !== null && resumeTotal ? t("hero.resumeAt", { line: resumeHalt + 1, total: resumeTotal }) : t("hero.neverOpened")}</span>
                    </div>
                </div>

                <div className="flex flex-wrap gap-2">
                    {resumeHalt !== null ? (
                        <Button className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: entry.id }} search={{ halt: resumeHalt }} />}>
                            <PlayIcon /> {t("hero.continue")}
                        </Button>
                    ) : (
                        <Button className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: entry.id }} />}>
                            <PlayIcon /> {fresh ? t("hero.startReading") : t("hero.continue")}
                        </Button>
                    )}
                    <Button variant="outline" className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: entry.id }} search={{ halt: 0 }} />}>
                        <RotateCcwIcon /> {t("hero.fromStart")}
                    </Button>
                    <Button variant="ghost" className={cn("max-sm:h-11")} onClick={() => onViewChapter(group.id)}>
                        <ListTreeIcon /> {t("hero.viewChapter")}
                    </Button>
                </div>
            </div>
        </section>
    );
}
