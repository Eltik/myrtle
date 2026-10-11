/**
 * Chapter sheet facts and actions. Every number is derived, not stored: counts
 * from `derive.ts`, word spans from `stats.ts`, read verdict from `isStoryRead`
 * (the library's one read predicate).
 */
import { Link } from "@tanstack/react-router";
import type React from "react";
import { lazy, memo, Suspense, useState } from "react";
import type { messages as exportMessages } from "#/components/story/export/export.messages";
import { Button } from "#/components/ui/button";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type StoryProgress, saveProgress } from "#/lib/story/progress";
import { humanTime, minutesFor, useReadingSpeed } from "#/lib/story/reading";
import type { messages } from "./Browse.messages";
import { releaseYear, specModel } from "./chapters";
import { groupWords, type LibGroup, pickContinue, readFraction, sortedStories } from "./derive";
import { clearMarks, markAllRead, markable, markedCount } from "./marks";
import type { messages as marksMessages } from "./ReadToggle.messages";
import { cardCode } from "./sections";
import { setWords } from "./stats";

type BrowseT = TypedT<typeof messages>;
type MarksT = TypedT<typeof marksMessages>;
type ExportT = TypedT<typeof exportMessages>;

// Only mounted once opened, so the PDF and EPUB exporters stay out of the library chunk.
const ExportSheet = lazy(() => import("#/components/story/export/ExportSheet").then((m) => ({ default: m.ExportSheet })));

/**
 * "Continue" opens the first unread story, or the last once all are read.
 * "Start from the beginning" only shows when it would land somewhere else.
 */
export const MetaRow = memo(function MetaRow({ group, progress, gameRead }: { group: LibGroup; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement {
    const t: BrowseT = useT("story");
    const f = useFormatters();
    const { wpm } = useReadingSpeed();
    const fraction = readFraction(group.stories, progress, gameRead);
    const words = groupWords(group);
    const spans = setWords(group.stories, progress, gameRead);
    const year = releaseYear(group);
    const resume = pickContinue(group.stories, progress, gameRead);
    const first = sortedStories(group.stories).find((s) => s.hasScript) ?? null;
    const pct = fraction.total > 0 ? Math.round((fraction.read / fraction.total) * 100) : 0;
    // The chapter's own length is worth a span; what is LEFT of it is only
    // worth one while the two differ, so a fresh chapter and a finished one
    // both print a single figure.
    const showLeft = spans.left !== null && spans.total !== null && spans.left > 0 && spans.left < spans.total;

    return (
        <div className="flex shrink-0 flex-col gap-3 border-border border-b px-4 pt-3 pb-4 sm:px-5">
            {/* The end padding is the pinned cluster's (previous, next, close), which
                sits over this line once the hero has scrolled away: 3 x 44 px plus
                two 6 px gaps and the 8 px inset is 152 px on a phone, 3 x 36 + 20 is
                128 px on a fine pointer. */}
            <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 pe-38 font-sans text-[13px] text-muted-foreground tabular-nums sm:pointer-fine:pe-32">
                <span>{sheetCode(group, t)}</span>
                <Rule />
                <span>{t("browse.card.entries", { count: group.stories.length })}</span>
                <Rule />
                <span>{words === null ? t("chapter.wordsUnknown") : t("chapter.words", { words: f.number(words) })}</span>
                {words !== null ? (
                    <>
                        <Rule />
                        <span>{t("chapter.about", { time: humanTime(minutesFor(words, wpm)) })}</span>
                    </>
                ) : null}
                {showLeft ? (
                    <>
                        <Rule />
                        <span className="text-foreground/75">{t("chapter.timeLeft", { time: humanTime(minutesFor(spans.left ?? 0, wpm)) })}</span>
                    </>
                ) : null}
                {year !== null ? (
                    <>
                        <Rule />
                        <span>{year}</span>
                    </>
                ) : null}
            </div>

            {fraction.total > 0 ? (
                <div className="flex items-center gap-2.5">
                    <span aria-hidden="true" className="h-1 min-w-0 flex-1 overflow-hidden rounded-full bg-secondary">
                        <span className="block h-full rounded-full bg-primary transition-[width] duration-300 ease-out motion-reduce:transition-none" style={{ width: `${pct}%` }} />
                    </span>
                    <span className="shrink-0 font-sans text-[12.5px] text-muted-foreground tabular-nums">{t("chapter.readFraction", { read: fraction.read, total: fraction.total })}</span>
                </div>
            ) : null}

            <div className="flex flex-wrap items-center gap-2">
                {resume ? (
                    <>
                        <Button render={<Link to="/stories/$storyId" params={{ storyId: resume.entry.id }} />} className="max-sm:min-h-11 max-sm:flex-1">
                            {t("chapter.continue")}
                        </Button>
                        {first && first.id !== resume.entry.id ? (
                            <Button variant="outline" render={<Link to="/stories/$storyId" params={{ storyId: first.id }} />} className="max-sm:min-h-11 max-sm:flex-1">
                                {t("chapter.restart")}
                            </Button>
                        ) : null}
                    </>
                ) : null}
                <div className="flex min-w-0 flex-wrap items-center gap-2 max-sm:w-full sm:ml-auto">
                    <ExportAction group={group} />
                    <MarkActions group={group} progress={progress} gameRead={gameRead} />
                </div>
            </div>
        </div>
    );
});

/**
 * "Mark all read" is additive and undoable, so unconfirmed. "Clear all"
 * confirms inline with the count, since some marks may be the game's own
 * verdict. Not `window.confirm`: it blocks, can't be styled, and speaks the
 * browser's language, not the site's.
 */
function MarkActions({ group, progress, gameRead }: { group: LibGroup; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement | null {
    const t: MarksT = useT("story");
    const [confirming, setConfirming] = useState(false);
    const readable = markable(group.stories).length;
    const marks = markedCount(group.stories, progress, gameRead);
    if (readable === 0) return null;

    if (confirming) {
        return (
            <div className="flex flex-wrap items-center gap-2">
                <span className="font-sans text-[12.5px] text-foreground">{t("marks.clearAll.confirm", { count: marks })}</span>
                <Button
                    variant="destructive"
                    size="sm"
                    className="max-sm:h-11"
                    onClick={() => {
                        saveProgress(clearMarks(progress, group.stories, gameRead));
                        setConfirming(false);
                    }}
                >
                    {t("marks.clearAll.yes")}
                </Button>
                <Button variant="outline" size="sm" className="max-sm:h-11" onClick={() => setConfirming(false)}>
                    {t("marks.clearAll.no")}
                </Button>
            </div>
        );
    }

    return (
        <div className="flex flex-wrap items-center gap-2">
            <Button variant="outline" size="sm" className="max-sm:h-11" disabled={marks === readable} onClick={() => saveProgress(markAllRead(progress, group.stories))}>
                {t("marks.markAll")}
            </Button>
            <Button variant="ghost" size="sm" className="max-sm:h-11" disabled={marks === 0} onClick={() => setConfirming(true)}>
                {t("marks.clearAll")}
            </Button>
        </div>
    );
}

/** Hidden when no story has a script. The sheet mounts only while open. */
function ExportAction({ group }: { group: LibGroup }): React.ReactElement | null {
    const t: ExportT = useT("story");
    const [open, setOpen] = useState(false);
    if (!group.stories.some((s) => s.hasScript)) return null;
    return (
        <>
            <Button variant="outline" size="sm" className="max-sm:h-11" onClick={() => setOpen(true)} data-story-export>
                {t("export.open")}
            </Button>
            {open ? (
                <Suspense fallback={null}>
                    <ExportSheet open={open} onOpenChange={setOpen} group={group} />
                </Suspense>
            ) : null}
        </>
    );
}

/** The hairline between two meta facts. A middle dot is a character and inherits the line's tracking; this is 1 px wide at every size. */
function Rule(): React.ReactElement {
    return <span aria-hidden="true" className="h-3 w-px bg-border" />;
}

/** As the ticket prints it: "Chapter 0" for mainline (not the data's EPISODE 00), else the operation code. */
function sheetCode(group: LibGroup, t: BrowseT): string {
    const model = specModel(group, cardCode(group));
    return model.kind === "chapter" ? t("browse.card.chapter", { n: model.chapter }) : model.code;
}
