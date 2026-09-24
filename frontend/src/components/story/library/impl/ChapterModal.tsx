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
import { XIcon } from "lucide-react";
import type React from "react";
import { useMemo, useRef, useState } from "react";
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
export function ChapterModal({ group, progress, gameRead, onClose }: IChapterModalProps): React.ReactElement {
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
                        <Hero group={group} title={SheetTitle} audio={audio} scrollRoot={scroller} />
                        {sticky}
                        {body}
                    </div>
                    <PinnedClose close={SheetClose} label={t("chapter.close")} />
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
                    <Hero group={group} title={DialogTitle} audio={audio} scrollRoot={scroller} />
                    {sticky}
                    {body}
                </div>
                <PinnedClose close={DialogClose} label={t("chapter.close")} />
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
        <Close aria-label={label} className="absolute top-2 right-2 z-20 flex size-11 cursor-pointer items-center justify-center rounded-full bg-black/45 text-white outline-none backdrop-blur-[2px] transition-colors hover:bg-black/65 focus-visible:ring-2 focus-visible:ring-white/80 sm:size-9">
            <XIcon className="size-4.5" aria-hidden="true" />
        </Close>
    );
}
