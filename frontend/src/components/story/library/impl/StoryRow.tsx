import { Link } from "@tanstack/react-router";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { isStoryRead, type StoryProgress } from "#/lib/story/progress";
import { cn } from "#/lib/utils";
import type { LibEntry } from "./derive";
import { ReadToggle } from "./ReadToggle";
import type { messages } from "./StoriesTab.messages";

/**
 * One story in an open chapter or in an operator's record list. `showCode` is
 * false for records, which carry no operation code and would otherwise all sit
 * behind an empty column.
 *
 * The row is a LINK AND A TICK, two controls side by side rather than one: the
 * title opens the reader and the tick marks the story read, which is the same
 * pair the chapter sheet offers. A nested button inside an anchor is invalid,
 * so they are siblings and the row is the flex box around them.
 */
export function StoryRow({ story, progress, gameRead, showCode = true }: { story: LibEntry; progress: StoryProgress; gameRead: ReadonlySet<string>; showCode?: boolean }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const f = useFormatters();
    const read = isStoryRead(progress, gameRead, story.id);
    const pos = progress.pos[story.id];
    const inProgress = !read && pos ? pos.halt : null;

    const inner = (
        <>
            {showCode ? <span className="w-10 shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums sm:w-14">{story.code ?? ""}</span> : null}
            <span className={cn("min-w-0 flex-1 truncate font-sans text-[13px]", !story.hasScript && "text-muted-foreground")}>{story.name}</span>
            {story.avgTag ? <span className="hidden shrink-0 font-mono text-[10px] text-muted-foreground uppercase tracking-wide sm:inline">{story.avgTag}</span> : null}
            {typeof story.wordCount === "number" ? <span className="hidden shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums md:inline">{t("stories.card.words", { count: f.compact(story.wordCount) })}</span> : null}
            {inProgress !== null ? <span className="shrink-0 font-mono text-[10px] text-primary tabular-nums">{t("stories.panel.progressAt", { line: inProgress + 1 })}</span> : null}
            {!story.hasScript ? (
                <Badge variant="outline" className="shrink-0 text-[10px]">
                    {t("stories.panel.notExtracted")}
                </Badge>
            ) : null}
        </>
    );

    const className = "flex min-w-0 flex-1 items-center gap-2.5 rounded-md px-2 py-1.5 max-sm:min-h-11";
    if (!story.hasScript) return <div className={cn(className, "opacity-60")}>{inner}</div>;
    return (
        <div className="flex items-center gap-0.5">
            <Link to="/stories/$storyId" params={{ storyId: story.id }} className={cn(className, "transition-colors hover:bg-accent")}>
                {inner}
            </Link>
            <ReadToggle story={story} progress={progress} gameRead={gameRead} />
        </div>
    );
}
