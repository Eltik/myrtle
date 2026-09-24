/**
 * THE TWO FULL-STAGE CARDS: the one before the first line and the one after the
 * last.
 *
 * They are siblings because they are the same object seen twice, a sheet over
 * the whole stage with the story's own name on it, and because neither of them
 * is a reading surface: nothing on a card advances a halt, so both carry
 * `data-story-ui` and the stage's click handler never sees them.
 */
import { Link } from "@tanstack/react-router";
import type React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import { DARK_FOCUS, TOUCH_TARGET } from "./glass";
import type { messages } from "./reader.messages";

type ReaderT = TypedT<typeof messages>;

/**
 * `useT` is bound HERE rather than taken as a prop, and that is not a style
 * choice: `scripts/i18n-extract.mjs` binds a `t` to its namespace by the
 * `useT(...)` call in the SAME file, so a component handed its translator
 * through a prop takes every key it uses out of the catalogue's reach. Passing
 * it down measured 54 keys newly unused and 59 call sites lost.
 */
export interface ITitleCardProps {
    /** The story's group, code and AVG tag, already joined. */
    meta: string;
    title: string;
    /** The translated category name, from the shared catalogue. */
    category: string;
    /**
     * The halt a previous session stopped on, or null when there is nothing to
     * resume. It is what turns the title card into a two-button choice.
     */
    savedHalt: number | null;
    totalHalts: number;
    onResume: () => void;
    onStart: () => void;
}

export function TitleCard({ meta, title, category, savedHalt, totalHalts, onResume, onStart }: ITitleCardProps): React.ReactElement {
    const t: ReaderT = useT("story");
    return (
        <div className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-gradient-to-b from-black via-neutral-950 to-black px-8 text-center" data-story-card="title">
            <div className="font-heading text-white/55 text-xs uppercase tracking-[0.3em]">{meta}</div>
            <h1 className="font-display font-semibold text-3xl text-white sm:text-5xl">{title}</h1>
            <div className="mt-2 text-sm text-white/60">{category}</div>
            {savedHalt !== null ? (
                <div className="mt-6 flex flex-col items-center gap-3" data-story-ui>
                    <p className="text-sm text-white/75">{t("reader.title.resume", { line: savedHalt + 1, total: totalHalts })}</p>
                    <div className="flex flex-wrap justify-center gap-3">
                        <Button className={cn(TOUCH_TARGET, DARK_FOCUS)} onClick={onResume}>
                            {t("reader.title.resumeButton", { line: savedHalt + 1 })}
                        </Button>
                        <Button variant="outline" className={cn("border-white/30 bg-transparent text-white hover:bg-white/10", TOUCH_TARGET, DARK_FOCUS)} onClick={onStart}>
                            {t("reader.title.startOver")}
                        </Button>
                    </div>
                </div>
            ) : (
                <p className="story-bounce mt-8 text-sm text-white/55">{t("reader.title.begin")}</p>
            )}
        </div>
    );
}

export interface IEndCardProps {
    title: string;
    /** The chapter's neighbours in index order, already resolved by the route. Either may be absent at the ends. */
    previous: StoryEntry | null;
    next: StoryEntry | null;
    onRestart: () => void;
}

export function EndCard({ title, previous, next, onRestart }: IEndCardProps): React.ReactElement {
    const t: ReaderT = useT("story");
    return (
        <div className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-black/85 px-8 text-center" data-story-card="end" data-story-ui>
            <div className="font-heading text-white/55 text-xs uppercase tracking-[0.3em]">{title}</div>
            <h2 className="font-display font-semibold text-3xl text-white">{t("reader.end.title")}</h2>
            <p className="max-w-md text-sm text-white/70">{t("reader.end.body")}</p>
            <div className="mt-4 flex flex-wrap justify-center gap-3">
                {previous ? (
                    <Button variant="outline" className={cn("border-white/30 bg-transparent text-white hover:bg-white/10", TOUCH_TARGET, DARK_FOCUS)} render={<Link to="/stories/$storyId" params={{ storyId: previous.id }} />}>
                        {t("reader.end.previous")}
                    </Button>
                ) : null}
                <Button variant="outline" className={cn("border-white/30 bg-transparent text-white hover:bg-white/10", TOUCH_TARGET, DARK_FOCUS)} onClick={onRestart}>
                    {t("reader.end.again")}
                </Button>
                {next ? (
                    <Button className={cn(TOUCH_TARGET, DARK_FOCUS)} render={<Link to="/stories/$storyId" params={{ storyId: next.id }} />}>
                        {t("reader.end.next")}
                    </Button>
                ) : null}
            </div>
            <Link to="/stories" className={cn("mt-2 inline-flex items-center px-2 text-sm text-white/60 underline-offset-4 hover:underline", TOUCH_TARGET, DARK_FOCUS, "focus-visible:rounded-md focus-visible:ring-2")}>
                {t("reader.end.library")}
            </Link>
        </div>
    );
}
