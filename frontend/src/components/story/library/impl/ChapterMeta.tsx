/**
 * WHAT THE CHAPTER IS AND WHERE THE READER IS IN IT, then the one thing they
 * came to do.
 *
 * Two rows of facts and one row of actions, and every number on them is derived
 * rather than stored: the counts come from `derive.ts`, the word spans from
 * `stats.ts` and the read verdict from `isStoryRead`, which is the one
 * predicate the whole library decides that with.
 */
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type StoryProgress, saveProgress } from "#/lib/story/progress";
import { humanTime, minutesFor, useReadingSpeed } from "#/lib/story/reading";
import type { messages } from "./Browse.messages";
import { releaseYear } from "./chapters";
import { groupWords, type LibGroup, pickContinue, readFraction, sortedStories } from "./derive";
import { clearMarks, markAllRead, markable, markedCount } from "./marks";
import type { messages as marksMessages } from "./ReadToggle.messages";
import { cardCode } from "./sections";
import { setWords } from "./stats";

type BrowseT = TypedT<typeof messages>;
type MarksT = TypedT<typeof marksMessages>;

/**
 * WHAT THE CHAPTER IS AND WHERE THE READER IS IN IT, then the one thing they
 * came to do.
 *
 * ONE primary. "Continue" opens the first story with no read mark, or the last
 * story once every one of them has one, and "Start from the beginning" only
 * appears when it would land somewhere else: on an untouched chapter the two
 * are the same story and a second button offering the same link is noise.
 */
export function MetaRow({ group, progress, gameRead }: { group: LibGroup; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement {
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
            <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 font-mono text-[10.5px] text-muted-foreground uppercase tabular-nums tracking-[0.07em]">
                <span>{cardCode(group)}</span>
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
                    <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("chapter.readFraction", { read: fraction.read, total: fraction.total })}</span>
                </div>
            ) : null}

            {/* ONE ROW: the reading actions lead, the mark actions sit at the
                right end of the same line, and on a phone the row wraps. */}
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
                <div className="flex min-w-0 items-center max-sm:w-full sm:ml-auto">
                    <MarkActions group={group} progress={progress} gameRead={gameRead} />
                </div>
            </div>
        </div>
    );
}

/**
 * MARK THE WHOLE CHAPTER, in one press.
 *
 * "Mark all read" is not confirmed: it is additive, every tick it fills can be
 * emptied again, and a dialog in front of it would cost more than the mistake.
 * "Clear all" IS confirmed, once, inline, because it withdraws marks the reader
 * may have spent an evening making and some of them may be the game's own
 * verdict rather than theirs. The confirmation quotes the COUNT, so a misfire
 * on a 39-entry chapter says what it would take.
 *
 * It is not `window.confirm`: a native dialog blocks the page, cannot be
 * styled, and reads in the browser's language rather than the site's.
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

/** The hairline between two meta facts. A middle dot is a character and inherits the line's tracking; this is 1 px wide at every size. */
function Rule(): React.ReactElement {
    return <span aria-hidden="true" className="h-3 w-px bg-border" />;
}
