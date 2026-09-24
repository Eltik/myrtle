import { Link } from "@tanstack/react-router";
import { BookOpenIcon, ListTreeIcon, PlayIcon, RotateCcwIcon } from "lucide-react";
import type React from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import type { messages as sharedMessages } from "../../shared.messages";
import type { messages } from "./Browse.messages";
import { chapterNumberOf } from "./chapters";
import { groupCode, type IHeroPick, readFraction } from "./derive";
import type { messages as heroMessages } from "./StoryHero.messages";

export interface IContinueCardProps {
    pick: IHeroPick;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    onViewChapter: (groupId: string) => void;
}

/**
 * ONE ROW under the page head: the story last opened, its breadcrumb, its
 * group's read fraction and the two buttons. The reference page has no
 * continue affordance at all; this keeps ours but stops it eating the fold,
 * which the old cover-art hero did at 1440 (it was 268 px tall over a page
 * whose first card then started below the fold).
 */
export function ContinueCard({ pick, progress, gameRead, onViewChapter }: IContinueCardProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const th: TypedT<typeof heroMessages> = useT("story");
    const tc: TypedT<typeof sharedMessages> = useT("story");
    const { entry, group, recordName, resumeHalt, fresh } = pick;
    const fraction = readFraction(group.stories, progress, gameRead);
    // "Main story / Chapter 0", not "Main story / EPISODE 00: Evil Time Part 1":
    // the breadcrumb names the place the reader is in, in the words they use.
    const chapter = chapterNumberOf(group);
    const chapterLine = recordName ?? (chapter === null ? [group.zone?.nameThird, group.name].filter(Boolean).join(": ") || groupCode(group) : t("browse.card.chapter", { n: chapter }));
    // The set's KEY VISUAL behind the right edge, faded into card colour rather
    // than cropped into a thumbnail: the row is 74 px tall and a picture in it
    // would be a stamp, where the art belongs as the ground the row sits on.
    const banner = group.bannerUrl ? asset(group.bannerUrl) : null;

    return (
        <section className="relative mb-5 flex flex-wrap items-center gap-x-4 gap-y-3 overflow-hidden rounded-[12px] border border-border bg-card px-3.5 py-3">
            {banner ? (
                <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 right-0 hidden w-[46%] sm:block">
                    <img src={banner} alt="" loading="lazy" decoding="async" className="h-full w-full object-cover object-center opacity-45" />
                    <span className="absolute inset-0 bg-gradient-to-r from-card via-card/72 to-card/20" />
                </span>
            ) : null}
            <div className="relative z-1 flex min-w-0 flex-1 flex-col gap-1">
                <div className="flex items-center gap-1.5 font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">
                    <BookOpenIcon className="size-3" aria-hidden="true" />
                    {fresh ? t("browse.continue.kickerFresh") : t("browse.continue.kicker")}
                </div>
                <div className="flex min-w-0 flex-wrap items-baseline gap-x-2">
                    <span className="truncate font-bold font-heading text-[15px] text-foreground leading-tight">{entry.name}</span>
                    <span className="flex min-w-0 items-center gap-1.5 truncate font-sans text-[11.5px] text-muted-foreground">
                        <span className="truncate">
                            {tc(`category.${group.category}`)} / {chapterLine}
                        </span>
                    </span>
                </div>
                <div className="font-mono text-[10px] text-muted-foreground tabular-nums">{th("hero.groupRead", { read: fraction.read, total: fraction.total })}</div>
            </div>
            <div className="relative z-1 flex w-full flex-wrap gap-2 sm:w-auto sm:shrink-0">
                <Button size="sm" className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: entry.id }} search={resumeHalt !== null ? { halt: resumeHalt } : undefined} />}>
                    <PlayIcon /> {fresh ? th("hero.startReading") : th("hero.continue")}
                </Button>
                <Button size="sm" variant="outline" className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: entry.id }} search={{ halt: 0 }} />}>
                    <RotateCcwIcon /> {th("hero.fromStart")}
                </Button>
                <Button size="sm" variant="ghost" className="max-sm:h-11" onClick={() => onViewChapter(group.id)}>
                    <ListTreeIcon /> {th("hero.viewChapter")}
                </Button>
            </div>
        </section>
    );
}
