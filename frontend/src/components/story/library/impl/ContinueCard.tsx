import { Link } from "@tanstack/react-router";
import { BookOpenIcon, EllipsisVerticalIcon, ListTreeIcon, PlayIcon, RotateCcwIcon } from "lucide-react";
import type React from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
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
 *
 * ON A PHONE IT IS ONE ROW TOO. The three buttons used to wrap under the text
 * there and the card measured 191 px at 390; under 640 the text column now
 * stacks beside ONE primary button, and "From the start" and "View chapter"
 * move into a menu at the row's end rather than off the page. At 640 and up
 * nothing changes.
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
    const resume = <Link to="/stories/$storyId" params={{ storyId: entry.id }} search={resumeHalt !== null ? { halt: resumeHalt } : undefined} />;
    const fromStart = <Link to="/stories/$storyId" params={{ storyId: entry.id }} search={{ halt: 0 }} />;
    const primaryLabel = fresh ? th("hero.startReading") : th("hero.continue");

    return (
        <section className="relative mb-5 flex items-center gap-3 overflow-hidden rounded-xl border border-border bg-card px-3.5 py-3 sm:flex-wrap sm:gap-x-4">
            {banner ? (
                <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 right-0 hidden w-[46%] sm:block">
                    <img src={banner} alt="" loading="lazy" decoding="async" className="h-full w-full object-cover object-center opacity-45" />
                    <span className="absolute inset-0 bg-linear-to-r from-card via-card/72 to-card/20" />
                </span>
            ) : null}
            <div className="relative z-1 flex min-w-0 flex-1 flex-col gap-1">
                <div className="flex items-center gap-1.5 font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">
                    <BookOpenIcon className="size-3" aria-hidden="true" />
                    {fresh ? t("browse.continue.kickerFresh") : t("browse.continue.kicker")}
                </div>
                {/* Stacked under 640, where the column is about 150 px beside the button
                    and a title and its breadcrumb on one baseline would truncate both. */}
                <div className="flex min-w-0 flex-col gap-x-2 sm:flex-row sm:flex-wrap sm:items-baseline">
                    <span className="truncate font-bold font-heading text-[15px] text-foreground leading-tight">{entry.name}</span>
                    <span className="flex min-w-0 items-center gap-1.5 truncate font-sans text-[11.5px] text-muted-foreground">
                        <span className="truncate">
                            {tc(`category.${group.category}`)} / {chapterLine}
                        </span>
                    </span>
                </div>
                <div className="font-mono text-[10px] text-muted-foreground tabular-nums">{th("hero.groupRead", { read: fraction.read, total: fraction.total })}</div>
            </div>
            <div className="relative z-1 flex shrink-0 items-center gap-1 sm:hidden">
                <Button className="h-11" render={resume}>
                    <PlayIcon /> {primaryLabel}
                </Button>
                <Menu>
                    <MenuTrigger render={<Button variant="ghost" size="icon" className="size-11" aria-label={t("browse.continue.more")} />}>
                        <EllipsisVerticalIcon aria-hidden="true" />
                    </MenuTrigger>
                    <MenuPopup align="end">
                        <MenuItem className="min-h-11" render={fromStart}>
                            <RotateCcwIcon /> {th("hero.fromStart")}
                        </MenuItem>
                        <MenuItem className="min-h-11" onClick={() => onViewChapter(group.id)}>
                            <ListTreeIcon /> {th("hero.viewChapter")}
                        </MenuItem>
                    </MenuPopup>
                </Menu>
            </div>
            <div className="relative z-1 hidden shrink-0 gap-2 sm:flex sm:flex-wrap">
                <Button size="sm" render={resume}>
                    <PlayIcon /> {primaryLabel}
                </Button>
                <Button size="sm" variant="outline" render={fromStart}>
                    <RotateCcwIcon /> {th("hero.fromStart")}
                </Button>
                <Button size="sm" variant="ghost" onClick={() => onViewChapter(group.id)}>
                    <ListTreeIcon /> {th("hero.viewChapter")}
                </Button>
            </div>
        </section>
    );
}
