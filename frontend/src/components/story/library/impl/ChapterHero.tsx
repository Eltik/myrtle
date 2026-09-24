/**
 * THE KEY VISUAL, FULL BLEED, WITH THE AUTHORED LOGOTYPE ON IT, and the
 * chapter's theme button on the picture it belongs to.
 *
 * `ChapterModal.tsx` owns which shell the sheet is (a Dialog above 640 px, a
 * bottom Sheet under it) and where the scrolling happens; this file owns
 * everything above the meta row, which is the one part of the sheet that is a
 * PICTURE rather than a list.
 *
 * THE HERO SCROLLS AWAY. It is neither fixed nor sticky: it is the first band
 * in the sheet's one scroller and it leaves with the scroll, and the parallax
 * is only that the picture goes slower than the box it is in. The close button
 * left with it and now lives on the popup, outside the scroller, because an
 * escape route 25 rows up is not one.
 */

import { PauseIcon, PlayIcon } from "lucide-react";
import type React from "react";
import type { RefObject } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { useParallaxProgress } from "#/components/operators/detail/impl/useParallaxProgress";
import type { DialogTitle } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StoryMusic } from "#/types/generated/StoryMusic";
import type { messages as archiveMessages } from "./Archive.messages";
import { plateCrop, plateSource, titleSource } from "./art";
import type { messages } from "./Browse.messages";
import type { IChapterAudio } from "./ChapterAudio";
import styles from "./ChapterModal.module.css";
import { chapterNumberOf } from "./chapters";
import type { LibGroup } from "./derive";
import { useTicketPalette } from "./palette";
import { cardCode, kindOf } from "./sections";
import { type IFitBounds, useTitleFit } from "./titleFit";

type BrowseT = TypedT<typeof messages>;
type ArchiveT = TypedT<typeof archiveMessages>;

/**
 * THE TEXT-TITLE BOX ON THE HERO, for the 6 groups the wire sends no logotype.
 * The box is the logotype's own, 46% of a 720 px hero, so the two title shapes
 * occupy the same column; the ceiling is the ticket's 32 scaled by the hero's
 * height over the card's (306 / 183 = 1.67), rounded down to 44, which sets two
 * lines at 91.5 px against the logotype's 134.6 px cap.
 */
const SHEET_FIT: IFitBounds = { box: 331, max: 44, min: 26, lines: 2, floorLines: 3 };

/** The one key the hero's theme holds on the sheet's music channel; an archive track holds `track:{id}`. */
const THEME_KEY = "theme";

/** The Dialog and the Sheet are the same base-ui primitive under two names, so the hero takes the one it should label rather than being written twice. */
type TitleComponent = typeof DialogTitle;

/**
 * THE KEY VISUAL, FULL BLEED, WITH THE AUTHORED LOGOTYPE ON IT.
 *
 * `plateSource` picks the picture (the 632x456 or 432x432 key visual first,
 * the derived cover second) and `plateCrop` decides where it is cropped: a
 * mainline plate is a POSTER with the chapter title typeset into rows
 * 0.65..0.78, so a centred 16:7 crop of one would print the chapter's name
 * twice. Biasing to 20% of the overflow leaves the visible band at rows
 * 0.113..0.550 and stops well short of the typeset title.
 *
 * The name is spoken ONCE. With a logotype the plain text is the dialog's
 * accessible title and nothing more; without one the fitted text IS the title
 * and is the same element.
 */
export function Hero({ group, title: Title, audio, scrollRoot }: { group: LibGroup; title: TitleComponent; audio: IChapterAudio; scrollRoot: RefObject<HTMLElement | null> }): React.ReactElement {
    const t: BrowseT = useT("story");
    const plate = plateSource(group);
    const art = plate.kind === "none" ? null : asset(plate.url);
    const logo = titleSource(group);
    const fit = useTitleFit(group.name, SHEET_FIT);
    const chapter = chapterNumberOf(group);
    // The SAME sampler the ticket runs over the SAME file, so a card and its
    // sheet can never disagree about the chapter's colour, and the second call
    // is free: `useTicketPalette` answers from the cache the card filled.
    const palette = useTicketPalette(art);
    // The sheet's content box is the scroller, so the hook is told to listen
    // on it: the window does not move while a dialog is open.
    const parallax = useParallaxProgress<HTMLDivElement>("--parallax-progress", scrollRoot);

    return (
        <div ref={parallax} className={cn("relative shrink-0", styles.hero, styles.ground, styles.parallax)} style={{ "--ticket-c1": palette.c1 } as React.CSSProperties}>
            <div className={cn("relative aspect-16/7 w-full overflow-hidden", styles.plate, styles.ground)}>
                {/* The moving layer, and the reason the plate above it keeps its own ground: at progress 1 the picture has drifted 80 px down and the band it vacates is this box's gradient rather than a hole. That band is never seen, because a hero at progress 1 is a hero entirely above the scroller's top edge. */}
                <div aria-hidden="true" className={cn("absolute inset-0", styles.ground, styles.parallaxPlate)}>
                    {art ? (
                        <img src={art} alt="" aria-hidden="true" crossOrigin="anonymous" decoding="async" data-source={plate.kind} className="absolute inset-0 h-full w-full object-cover" style={{ objectPosition: plateCrop(group) }} />
                    ) : (
                        <span aria-hidden="true" className="absolute inset-0 flex items-center justify-center font-black font-heading text-[34px] text-white/45 uppercase tracking-tight">
                            {cardCode(group)}
                        </span>
                    )}
                </div>
                {/* The veil is what makes this a HEADER rather than a picture with text on it: the art goes down into the dark ground and the ground into the sheet, so the meta row below has no seam to sit on and no bright plate to end against. */}
                <span aria-hidden="true" className={cn("absolute inset-0", styles.veil)} />
                <span aria-hidden="true" className="absolute inset-x-0 top-0 h-20 bg-linear-to-b from-black/45 to-transparent" />
            </div>

            <div className={cn("absolute top-2.5 left-3 flex items-center gap-1.5 sm:left-4", styles.parallaxLift)}>
                <span className={styles.badge}>{t(`browse.badge.${kindOf(group)}`)}</span>
                {chapter !== null ? <span className={styles.badge}>{t("browse.card.chapter", { n: chapter })}</span> : null}
            </div>

            <div className="absolute inset-x-0 bottom-0 flex items-end gap-2 px-3 pb-3 sm:px-4 sm:pb-4">
                <div className={cn("flex min-w-0 items-end", styles.parallaxLift)}>
                    {logo.kind === "logotype" ? (
                        <>
                            <img src={asset(logo.url)} alt={group.name} decoding="async" className={styles.logotype} />
                            <Title className="sr-only">{group.name}</Title>
                        </>
                    ) : (
                        <Title className={styles.textTitle} style={{ "--sheet-title-size": fit.size, "--sheet-title-wrap": fit.breakAnywhere ? "anywhere" : "normal" } as React.CSSProperties}>
                            {fit.lines.map((line) => (
                                <span key={line} className="block">
                                    {line}
                                </span>
                            ))}
                        </Title>
                    )}
                </div>
                {/* The theme pill LIFTS AND FADES with the logotype: the close button
                    is pinned to the popup above this band, and a pill that rode the
                    hero at full opacity passed straight under it mid-scroll. */}
                {group.music ? (
                    <div className={cn("ml-auto flex shrink-0", styles.parallaxLift)}>
                        <ThemeButton music={group.music} audio={audio} />
                    </div>
                ) : null}
            </div>
        </div>
    );
}

/**
 * THE CHAPTER'S THEME, on the picture it belongs to.
 *
 * 86 of the 87 EN groups carry one and 6 of those carry a title, so the
 * button prints the title where the music table has one and says what it is
 * where it does not: a bare glyph on a key visual would read as a video.
 * `act24side` is the one group with no theme, whose bank names a loop clip the
 * tree does not hold, and it gets no button at all rather than a dead one.
 *
 * The press is the GESTURE. A browser refuses to start an AudioContext without
 * one, so the module is armed here and nowhere earlier.
 */
function ThemeButton({ music, audio }: { music: StoryMusic; audio: IChapterAudio }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const name = music.title?.trim() ? music.title : t("archive.theme.label");
    const playing = audio.playing === THEME_KEY;

    return (
        <button
            type="button"
            onClick={() => audio.toggle(THEME_KEY, { intro: music.introUrl, loop: music.loopUrl }, music.title?.trim() ? music.title : null)}
            aria-label={playing ? t("archive.theme.pause", { name }) : t("archive.theme.play", { name })}
            className="ms-auto flex h-11 min-w-0 cursor-pointer items-center gap-2 rounded-full bg-black/45 px-3 text-white outline-none backdrop-blur-[2px] transition-colors hover:bg-black/65 focus-visible:ring-2 focus-visible:ring-white/80"
        >
            {playing ? <PauseIcon className="size-4 shrink-0" aria-hidden="true" /> : <PlayIcon className="size-4 shrink-0" aria-hidden="true" />}
            <span className="max-w-26 truncate font-mono text-[10px] uppercase tracking-[0.08em] sm:max-w-40">{name}</span>
        </button>
    );
}
