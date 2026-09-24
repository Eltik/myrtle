import { CheckIcon, Gamepad2Icon } from "lucide-react";
import type React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { isStoryRead, readSourceOf, type StoryProgress, saveProgress } from "#/lib/story/progress";
import { cn } from "#/lib/utils";
import type { LibEntry } from "./derive";
import { toggleRead } from "./marks";
import type { messages } from "./ReadToggle.messages";

type MarksT = TypedT<typeof messages>;

export interface IReadToggleProps {
    story: Pick<LibEntry, "id" | "name">;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    /** A name for the aria-label when the story's own reads badly on its own, such as a bare "Before". Defaults to the story name. */
    label?: string;
    /** `sm` is the 32 px tick inside a chapter row on a pointer device; both are 44 px under 640. */
    size?: "sm" | "md";
    className?: string;
}

/**
 * THE TICK IS A CONTROL, not a verdict printed beside a link. Clicking it marks
 * the story read, and clicking a filled one withdraws the mark, which is the
 * whole of the "it should be easy" ask: a reader who finished a chapter in the
 * game two years ago has no other way to say so.
 *
 * The SOURCE of a mark is drawn, because the three are not the same claim. A
 * filled tick is the reader's own mark; a filled tick with a controller glyph
 * is the Arknights client's verdict, which arrives with every profile refresh
 * and is never written into the document; an empty ring on a story the game
 * says was played is the reader's override, which outranks the game until they
 * take it back. Every one of them carries the wording in a tooltip as well,
 * because a 10 px glyph is not a sentence.
 *
 * There is no confirmation on a single toggle. It is one click to undo, and a
 * dialog per tick would cost more than the mistake.
 */
export function ReadToggle({ story, progress, gameRead, label, size = "sm", className }: IReadToggleProps): React.ReactElement {
    const t: MarksT = useT("story");
    const read = isStoryRead(progress, gameRead, story.id);
    const source = readSourceOf(progress, gameRead, story.id);
    const name = label ?? story.name;

    const tip = source === "game" ? t("marks.source.game") : source === "cleared" ? t("marks.source.cleared") : source === "own" ? t("marks.source.own") : undefined;

    return (
        <button
            type="button"
            aria-pressed={read}
            aria-label={read ? t("marks.markUnread", { story: name }) : t("marks.markRead", { story: name })}
            title={tip}
            onClick={(event) => {
                // The tick is a sibling of a link and, in a reading-order row,
                // sits inside a surface that opens on click. Neither should
                // fire when the tick is what was hit.
                event.preventDefault();
                event.stopPropagation();
                saveProgress(toggleRead(progress, story.id, read));
            }}
            className={cn("flex pointer-coarse:size-11 size-11 shrink-0 cursor-pointer items-center justify-center rounded-full transition-colors hover:bg-secondary/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60", size === "sm" ? "sm:size-8" : "sm:size-9", className)}
        >
            <span className="relative flex size-4 items-center justify-center">
                {read ? <CheckIcon className="size-4 text-primary" aria-hidden="true" /> : <span aria-hidden="true" className={cn("size-4 rounded-full border", source === "cleared" ? "border-primary/45 border-dashed" : "border-border")} />}
                {source === "game" ? <Gamepad2Icon className="absolute -right-1 -bottom-1 size-2.5 text-muted-foreground" aria-hidden="true" /> : null}
            </span>
        </button>
    );
}
