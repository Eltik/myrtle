/**
 * ONE CHAPTER AS A SHEET, headed by the picture the game itself draws for it.
 *
 * The old popup was a letterboxed banner strip, a "MAIN STORY" eyebrow, a
 * 26 px "CHAPTER 1 · EVIL TIME PART 2" heading, and then one 10%-tinted row
 * per story, which printed "Isolated Island" twice because the table stores an
 * operation as a `_beg` and an `_end` script. Three things changed and each is
 * a claim about the content rather than a preference: the LOGOTYPE is the title
 * (`art.titleSource`, authored per event, on the wire for 81 of the 87
 * non-record groups), the row is an OPERATION rather than a script
 * (`derive.groupOperations`, 523 of 525 repeated codes merge), and the only
 * colour left is on the ticks and the hover, because the per-phase washes were
 * what read as ugly.
 *
 * WHAT THIS FILE OWNS is the shell and the two decisions in it. The first is
 * which shell: a Dialog above 640 px and a bottom Sheet under it, the same
 * rule the operator panel uses. The second is where the scrolling happens, and
 * there is exactly ONE place. The responsive pass gave the entries list a
 * `max-h-[46dvh] overflow-y-auto` of its own INSIDE `DialogPanel`, which is
 * already a `ScrollArea`, so at 1440x900 with 25 rows the right edge carried
 * two stacked scrollbars, the primitive's styled one and the native one the
 * class added. The panel is gone: the popup holds one scrolling content box,
 * the hero scrolls away inside it, and the meta block sticks to its top. The
 * bands inside are `ChapterHero`, `ChapterMeta` and `ChapterEntries`, with the
 * archive's own panel beside them. It still changes no route: the URL stays on
 * `/stories` so the browse position survives the back button.
 */
import { useQuery } from "@tanstack/react-query";
import { ChevronLeftIcon, ChevronRightIcon, XIcon } from "lucide-react";
import type React from "react";
import { Fragment, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Sheet, SheetClose, SheetFooter, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useMediaQuery } from "#/hooks/use-media-query";
import { storyArchiveQueryOptions } from "#/lib/api/story";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { ArchivePanel, type ChapterView, ChapterViewSwitch } from "./ArchivePanel";
import { archiveSections } from "./archive";
import type { messages } from "./Browse.messages";
import { useChapterAudio } from "./ChapterAudio";
import { Entries } from "./ChapterEntries";
import { Hero } from "./ChapterHero";
import { MetaRow } from "./ChapterMeta";
import type { LibGroup } from "./derive";

type BrowseT = TypedT<typeof messages>;

export interface IChapterModalProps {
    group: LibGroup;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    onClose: () => void;
    /** The chapter before this one in the page's own order, or null at the first. */
    prev?: LibGroup | null;
    /** The chapter after this one in the page's own order, or null at the last. */
    next?: LibGroup | null;
    /** Swap the sheet to another chapter in place. */
    onNavigate?: (groupId: string) => void;
}

/** Widgets whose own arrow keys move a selection or a value. */
const ARROW_OWNERS = '[role="tablist"],[role="radiogroup"],[role="menu"],[role="menubar"],[role="listbox"],[role="toolbar"],[role="slider"],[role="grid"],[role="tree"]';

/** True when a keypress belongs to a field or a widget: an arrow key there moves a caret, a value or a selection, never the chapter. */
function typingInto(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    const tag = target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    return target.closest(ARROW_OWNERS) !== null;
}

/**
 * ONE CHAPTER AS A SHEET, headed by the picture the game itself draws for it.
 *
 * The old popup was a letterboxed banner strip, a "MAIN STORY" eyebrow, a
 * 26 px "CHAPTER 1 · EVIL TIME PART 2" heading, and then one 10%-tinted row
 * per story, which printed "Isolated Island" twice because the table stores an
 * operation as a `_beg` and an `_end` script. Three things changed and each is
 * a claim about the content rather than a preference: the LOGOTYPE is the
 * title (`art.titleSource`, authored per event, on the wire for 81 of the 87
 * non-record groups), the row is an OPERATION rather than a script
 * (`derive.groupOperations`, 523 of 525 repeated codes merge), and the only
 * colour left is on the ticks and the hover, because the per-phase washes were
 * what read as ugly.
 *
 * It stays a Dialog above 640 px and a bottom Sheet under it, the same rule
 * the operator panel uses, and it still changes no route: the URL stays on
 * `/stories` so the browse position survives the back button.
 */
export function ChapterModal({ group, progress, gameRead, onClose, prev = null, next = null, onNavigate }: IChapterModalProps): React.ReactElement {
    const t: BrowseT = useT("story");
    const phone = useMediaQuery("max-sm");
    const server = useGamedataServer();
    // The library's one music channel, seen from this chapter. The hero's theme
    // button and the archive's music rows share it, and so does every ticket on
    // the page behind the sheet: closing the sheet no longer stops the theme,
    // leaving `/stories` does.
    const audio = useChapterAudio(group);
    // The sheet's ONE scroller. The hero's parallax listens on this box rather
    // than on the window, which never moves while a dialog is open.
    const scroller = useRef<HTMLDivElement>(null);
    // A 404 is the answer from a backend that predates the route, and a group
    // that kept no archive answers 200 with no sections. Both land on an empty
    // list here, which is the same thing to a reader.
    const archive = useQuery(storyArchiveQueryOptions(group.id, server));
    const sections = useMemo(() => archiveSections(archive.data), [archive.data]);
    const [view, setView] = useState<ChapterView>("entries");
    const showing: ChapterView = sections.length === 0 ? "entries" : view;
    const body = showing === "archive" ? <ArchivePanel sections={sections} audio={audio} /> : <Entries group={group} progress={progress} gameRead={gameRead} />;
    const views = <ChapterViewSwitch view={showing} onView={setView} sections={sections} />;

    // A NEIGHBOUR SWAPS THE CONTENT, NOT THE SHEET. The popup stays mounted and
    // only the bands inside the scroller are keyed on the group, so the hero,
    // the meta block's confirm state and the list start fresh while the dialog
    // never closes and reopens. The one scroller is kept, so it is put back to
    // the top by hand, before paint.
    const shownId = useRef(group.id);
    useLayoutEffect(() => {
        if (shownId.current === group.id) return;
        shownId.current = group.id;
        if (scroller.current) scroller.current.scrollTop = 0;
    }, [group.id]);

    // ArrowLeft and ArrowRight walk the chapters while the sheet is open, unless
    // a field or an arrow-owning widget has focus or a modifier is held (Alt+Left
    // is the browser's Back). The listener is on the CAPTURE phase because the
    // dialog popup stops keydown from bubbling past itself: measured, a bubble
    // listener on window never saw the ArrowRight that the popup's theme button
    // received.
    useEffect(() => {
        if (!onNavigate) return;
        const onKey = (event: KeyboardEvent): void => {
            if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
            if (typingInto(event.target) || typingInto(document.activeElement)) return;
            const to = event.key === "ArrowLeft" ? prev : event.key === "ArrowRight" ? next : null;
            if (!to) return;
            event.preventDefault();
            onNavigate(to.id);
        };
        window.addEventListener("keydown", onKey, true);
        return () => window.removeEventListener("keydown", onKey, true);
    }, [prev, next, onNavigate]);

    const pinned = (close: typeof DialogClose) => (
        <div className="absolute top-2 right-2 z-20 flex items-center gap-1.5">
            {onNavigate ? (
                <>
                    <PinnedStep to={prev} icon={ChevronLeftIcon} label={t("chapter.nav.prev")} title={prev ? t("chapter.nav.prevTitle", { name: prev.name }) : undefined} onNavigate={onNavigate} />
                    <PinnedStep to={next} icon={ChevronRightIcon} label={t("chapter.nav.next")} title={next ? t("chapter.nav.nextTitle", { name: next.name }) : undefined} onNavigate={onNavigate} />
                </>
            ) : null}
            <PinnedClose close={close} label={t("chapter.close")} />
        </div>
    );

    // The meta block is what the reader steers by, so it is the one band that
    // survives the hero going past: `sticky top-0` inside the scroller, on the
    // sheet's own surface, with the row's existing hairline as its edge.
    const sticky = (
        <div className="sticky top-0 z-10 bg-card">
            <MetaRow group={group} progress={progress} gameRead={gameRead} />
            {views}
        </div>
    );

    if (phone) {
        return (
            <Sheet open onOpenChange={(open) => !open && onClose()}>
                <SheetPopup side="bottom" showCloseButton={false} className="h-[92dvh] bg-card">
                    <div ref={scroller} className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
                        <Fragment key={group.id}>
                            <Hero group={group} title={SheetTitle} audio={audio} scrollRoot={scroller} />
                            {sticky}
                            {body}
                        </Fragment>
                    </div>
                    {pinned(SheetClose)}
                    <SheetFooter>
                        <SheetClose render={<Button variant="outline" className="min-h-11 w-full" />}>{t("chapter.close")}</SheetClose>
                    </SheetFooter>
                </SheetPopup>
            </Sheet>
        );
    }

    return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
            <DialogPopup showCloseButton={false} className="max-w-180 overflow-hidden bg-card">
                <div ref={scroller} className="max-h-[90dvh] min-h-0 overflow-y-auto overscroll-contain">
                    <Fragment key={group.id}>
                        <Hero group={group} title={DialogTitle} audio={audio} scrollRoot={scroller} />
                        {sticky}
                        {body}
                    </Fragment>
                </div>
                {pinned(DialogClose)}
            </DialogPopup>
        </Dialog>
    );
}

/**
 * THE ONE WAY OUT, ALWAYS REACHABLE. It used to sit on the hero, which was
 * fine while the hero could not move; now the hero scrolls away and a close
 * button that goes with it would leave the escape route 25 rows up. It is
 * pinned to the POPUP instead, outside the scroller, so it holds the same
 * corner at every scroll position. The black pill and the blur are the hero's,
 * unchanged, because it still has to read on a bright key visual at the top of
 * the scroll.
 */
function PinnedClose({ close: Close, label }: { close: typeof DialogClose; label: string }): React.ReactElement {
    return (
        <Close aria-label={label} className="flex size-11 cursor-pointer items-center justify-center rounded-full bg-black/45 text-white outline-none backdrop-blur-[2px] transition-colors hover:bg-black/65 focus-visible:ring-2 focus-visible:ring-white/80 sm:pointer-fine:size-9">
            <XIcon className="size-4.5" aria-hidden="true" />
        </Close>
    );
}

/**
 * ONE STEP TO A NEIGHBOURING CHAPTER, pinned beside the close button for the
 * same reason it is: the hero scrolls away and the way on has to stay put. The
 * same black pill, 36 px on a fine pointer and 44 px on a phone or a coarse
 * one. At an end the step is disabled rather than hidden, so the pair never
 * shifts under the reader's pointer and the end reads as an end.
 */
function PinnedStep({ to, icon: Icon, label, title, onNavigate }: { to: LibGroup | null; icon: typeof ChevronLeftIcon; label: string; title: string | undefined; onNavigate: (groupId: string) => void }): React.ReactElement {
    return (
        <button
            type="button"
            aria-label={label}
            title={title}
            disabled={to === null}
            onClick={() => to && onNavigate(to.id)}
            className="flex size-11 cursor-pointer items-center justify-center rounded-full bg-black/45 text-white outline-none backdrop-blur-[2px] transition-colors hover:bg-black/65 focus-visible:ring-2 focus-visible:ring-white/80 disabled:cursor-default disabled:opacity-35 disabled:hover:bg-black/45 sm:pointer-fine:size-9"
        >
            <Icon className="size-5" aria-hidden="true" />
        </button>
    );
}
