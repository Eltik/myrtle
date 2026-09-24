/**
 * THE CHAPTER'S OPERATIONS, in the table's own order.
 *
 * A row is an OPERATION, not a script: `derive.groupOperations` merges the
 * `_beg` and `_end` halves the table stores separately, and 523 of the 525
 * repeated codes merge, which is why the old list printed "Isolated Island"
 * twice.
 *
 * The surface is NEUTRAL. The phase used to be a 10% wash of a palette token
 * across the row plus a full-strength left rule, three hues down a list of
 * thirty-nine, and that is what the page was called ugly for. The phase is now
 * a word on a segment, the only colour is the tick and the hover, and a read
 * row is muted rather than tinted.
 */
import { Link } from "@tanstack/react-router";
import { FilmIcon } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { isStoryRead, type StoryProgress } from "#/lib/story/progress";
import { cn } from "#/lib/utils";
import type { messages } from "./Browse.messages";
import { groupOperations, type IOperationRow, type IOperationSegment, type LibGroup, type StoryPhase } from "./derive";
import { ReadToggle } from "./ReadToggle";
import type { messages as storiesMessages } from "./StoriesTab.messages";

type BrowseT = TypedT<typeof messages>;
type StoriesT = TypedT<typeof storiesMessages>;

/** The chapter's operations, in the table's own order. */
export function Entries({ group, progress, gameRead }: { group: LibGroup; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement {
    const rows = groupOperations(group.stories);
    return (
        <ol className="m-0 flex list-none flex-col p-0">
            {rows.map((row) => (
                <li key={row.key} className="border-border/60 border-b last:border-b-0">
                    <OperationRow row={row} progress={progress} gameRead={gameRead} />
                </li>
            ))}
        </ol>
    );
}

/**
 * ONE OPERATION. A merged row is not a link: its two halves are, each with its
 * own tick, because they are two different places to go. A lone entry has one
 * place to go and the whole row is that link, which is the larger target and
 * the one a thumb finds.
 *
 * The surface is NEUTRAL. The phase used to be a 10% wash of a palette token
 * across the row plus a full-strength left rule, three hues down a list of
 * thirty-nine, and that is what the page was called ugly for. The phase is now
 * a word on a segment, the only colour is the tick and the hover, and a read
 * row is muted rather than tinted.
 */
function OperationRow({ row, progress, gameRead }: { row: IOperationRow; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement {
    const t: BrowseT = useT("story");
    const ts: StoriesT = useT("story");
    const f = useFormatters();
    const words = row.segments.reduce<number | null>((sum, seg) => (typeof seg.entry.wordCount === "number" ? (sum ?? 0) + seg.entry.wordCount : sum), null);
    const video = row.segments.some((seg) => seg.entry.hasVideo);
    const done = row.segments.every((seg) => isStoryRead(progress, gameRead, seg.entry.id));
    const solo = row.segments.length === 1 ? row.segments[0] : undefined;

    const face = (
        <>
            {row.code ? <span className="w-12 shrink-0 truncate rounded-[5px] border border-border bg-secondary/55 px-1 py-0.5 text-center font-mono text-[10px] text-muted-foreground tabular-nums">{row.code}</span> : null}
            <span className="flex min-w-0 flex-1 basis-32 flex-col gap-0.5">
                <span className={cn("font-sans text-[13px] leading-snug", done ? "text-foreground/60" : "text-foreground")}>{row.title}</span>
                <span className="flex flex-wrap items-center gap-x-2 font-mono text-[10px] text-muted-foreground tabular-nums">
                    {solo?.phase ? <span className="uppercase tracking-[0.08em]">{t(`chapter.phase.${solo.phase}`)}</span> : null}
                    {words !== null ? <span>{t("chapter.words", { words: f.number(words) })}</span> : null}
                    {video ? (
                        <span className="flex items-center gap-1 uppercase tracking-[0.08em]">
                            <FilmIcon className="size-3" aria-hidden="true" /> {t("chapter.video")}
                        </span>
                    ) : null}
                </span>
            </span>
        </>
    );

    // The two segments cost about 180 px, which on a 390 px sheet leaves the
    // title 130 and wraps a four-word name onto three lines. Under 640 they
    // take their own line instead, indented past the code chip so the row still
    // reads as one thing.
    const shell = "flex w-full flex-wrap items-center gap-x-2.5 gap-y-2 px-4 py-2.5 text-left transition-colors sm:flex-nowrap sm:px-5";

    if (solo) {
        if (!solo.entry.hasScript)
            return (
                <div className={cn(shell, "opacity-60")}>
                    {face}
                    <Badge variant="outline" className="shrink-0 text-[10px]">
                        {ts("stories.panel.notExtracted")}
                    </Badge>
                </div>
            );
        return (
            <div className={cn(shell, "min-h-11 py-1.5 pe-1.5 sm:pe-2")}>
                <Link
                    to="/stories/$storyId"
                    params={{ storyId: solo.entry.id }}
                    className="-mx-2 -my-1 flex pointer-coarse:min-h-11 min-w-0 flex-1 basis-32 items-center gap-2.5 rounded-[7px] px-2 py-1 transition-colors hover:bg-secondary/45 focus-visible:bg-secondary/45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 focus-visible:ring-inset max-sm:min-h-11"
                >
                    {face}
                </Link>
                <ReadToggle story={solo.entry} progress={progress} gameRead={gameRead} />
            </div>
        );
    }

    return (
        <div className={cn(shell, "min-h-11")}>
            {face}
            <span className="flex shrink-0 items-center gap-1.5 max-sm:w-full max-sm:ps-14.5">
                {row.segments.map((segment) => (
                    <PhaseSegment key={segment.entry.id} segment={segment} title={row.title} progress={progress} gameRead={gameRead} />
                ))}
            </span>
        </div>
    );
}

/**
 * ONE HALF OF AN OPERATION: a chip that opens it and a tick that marks it.
 *
 * The two used to be one link with the tick drawn inside it, which made the
 * read verdict unclickable: the only way to say "I read this in the game two
 * years ago" was to open the script and scroll it. They are siblings now, with
 * the chip's right edge squared into the tick so the pair still reads as one
 * control, and the chip keeps the aria-label naming the operation, because
 * "Before" alone names nothing.
 */
function PhaseSegment({ segment, title, progress, gameRead }: { segment: IOperationSegment; title: string; progress: StoryProgress; gameRead: ReadonlySet<string> }): React.ReactElement {
    const t: BrowseT = useT("story");
    const ts: StoriesT = useT("story");
    const phase: StoryPhase = segment.phase ?? "interlude";
    const label = t(`chapter.phase.${phase}`);
    const read = isStoryRead(progress, gameRead, segment.entry.id);
    const shell = "flex min-h-11 shrink-0 items-center justify-center gap-1.5 rounded-[7px] border px-2.5 font-mono text-[10px] uppercase tracking-[0.08em] transition-colors max-sm:flex-1 pointer-coarse:min-h-11 sm:min-h-8";

    if (!segment.entry.hasScript)
        return (
            <span className={cn(shell, "border-border border-dashed text-muted-foreground/70")} title={ts("stories.panel.notExtracted")}>
                {label}
            </span>
        );

    return (
        <span className="flex shrink-0 items-center max-sm:flex-1">
            <Link
                to="/stories/$storyId"
                params={{ storyId: segment.entry.id }}
                aria-label={t("chapter.segment.aria", { title, phase: label })}
                className={cn(
                    shell,
                    "cursor-pointer rounded-e-none border-e-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60",
                    read ? "border-primary/45 bg-primary/8 text-foreground hover:bg-primary/14" : "border-border bg-secondary/40 text-muted-foreground hover:border-primary/45 hover:text-foreground",
                )}
            >
                {label}
            </Link>
            <ReadToggle story={segment.entry} label={t("chapter.segment.aria", { title, phase: label })} progress={progress} gameRead={gameRead} className={cn("rounded-s-none rounded-e-[7px] border", read ? "border-primary/45 bg-primary/8 hover:bg-primary/14" : "border-border bg-secondary/40")} />
        </span>
    );
}
