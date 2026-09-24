import { MusicIcon, PauseIcon, PlayIcon, SquareIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Slider, SliderPrimitive } from "#/components/ui/slider";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { clockTime } from "#/lib/story/reading";
import { sliderValue } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import type { messages as archiveMessages } from "./Archive.messages";
import type { messages } from "./NowPlayingBar.messages";
import { type IPlayerState, pause, resume, seek, setVolume, stop, useLibraryPlayer, useNowPlayingClock } from "./player";

type NowT = TypedT<typeof messages & typeof archiveMessages>;

/**
 * WHAT IS SOUNDING, WHERE IT IS, AND THE TWO SLIDERS THAT GOVERN IT.
 *
 * The bar exists only while the library's channel holds a cue, which is the
 * whole visibility rule: no cue, no bar, and stopping is what dismisses it.
 * It is FIXED rather than sticky and it reserves its own height with a spacer
 * at the end of the page, so appearing moves nothing that is already on
 * screen; the cards above it do not reflow at 390 or at 1440.
 *
 * The volume slider is the reader's `musicVolume` and not a second setting. It
 * writes the same `myrtle.story.settings` document the reader's Settings
 * dialog writes, through the store, so a level set here is the level a story
 * opens with and a level set in a story is the level a theme starts at. Zero
 * is silence, and it persists.
 *
 * The seek slider is the cue's own timeline, `intro + loop`, and it is the one
 * control on the bar that can be AHEAD of the store: a drag writes a local
 * scrub value and only the release seeks, because every commit restarts the
 * buffer sources and a drag that seeked on every step would stutter its way
 * across the track. A cue whose decode has not landed, and a channel that fell
 * back to a media element and so has no timeline at all, both report length 0:
 * the slider stays in place and disabled rather than moving under the reader.
 */
export function NowPlayingBar(): React.ReactElement | null {
    const state = useLibraryPlayer();
    if (state.track === null) return null;
    return <Bar state={state} />;
}

/** Split out so the hooks below run only for a bar that exists. */
function Bar({ state }: { state: IPlayerState }): React.ReactElement {
    const t: NowT = useT("story");
    const f = useFormatters();
    const track = state.track;
    // The position clock ticks for this bar only, and only while something sounds.
    useNowPlayingClock(!state.paused);
    // A group whose theme carries no title of its own is announced by what it
    // is, because a bar that says only "Near Light" does not say it is music.
    const name = track?.title ?? t("archive.theme.label");
    const chapter = track?.groupName ?? "";

    return (
        <>
            {/* The bar's own height, in the page flow, so the last row of cards is never
                covered. It is a few pixels GENEROUS on purpose: the bar measures 108 at 390,
                where the seek slider is a second row with a 44 px hit area, and 57 at 1440,
                where it sits inline; the difference is trailing whitespace at the end of the
                page rather than a card clipped by a home indicator. */}
            <div aria-hidden="true" className="h-[calc(120px+env(safe-area-inset-bottom))] sm:h-[calc(64px+env(safe-area-inset-bottom))]" />
            <div className="fixed inset-x-0 bottom-0 z-40 border-border border-t bg-card/95 shadow-[0_-8px_24px_-12px_rgba(0,0,0,0.45)] backdrop-blur-md">
                <section aria-label={t("nowPlaying.region")} className="mx-auto flex max-w-[1400px] flex-wrap items-center gap-x-2 gap-y-1 px-3 pt-2 pb-[max(0.5rem,env(safe-area-inset-bottom))] sm:flex-nowrap sm:gap-x-4 sm:px-6">
                    <MusicIcon aria-hidden="true" className="hidden size-4 shrink-0 text-primary sm:block" />

                    <div className="flex min-w-0 flex-1 flex-col leading-tight">
                        <span aria-hidden="true" className="truncate font-sans text-[12.5px] text-foreground">
                            {name}
                        </span>
                        <span aria-hidden="true" className="truncate font-mono text-[10px] text-muted-foreground uppercase tracking-[0.08em]">
                            {state.paused ? t("nowPlaying.paused") : chapter}
                        </span>
                        {/* The two visible lines TRUNCATE and read in the wrong order to a screen
                            reader, so the announcement is its own sentence and they are hidden from
                            it. A pause empties the region rather than announcing again: the button
                            the reader just pressed has already changed its own name to Resume. */}
                        <span aria-live="polite" className="sr-only">
                            {state.paused ? "" : t("nowPlaying.playing", { title: name, chapter })}
                        </span>
                    </div>

                    <button
                        type="button"
                        onClick={() => (state.paused ? resume() : pause())}
                        aria-label={state.paused ? t("nowPlaying.resume", { name }) : t("nowPlaying.pause", { name })}
                        className="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border bg-background text-foreground transition-colors hover:border-primary/45 hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
                    >
                        {state.paused ? <PlayIcon className="size-4" aria-hidden="true" /> : <PauseIcon className="size-4" aria-hidden="true" />}
                    </button>

                    <button
                        type="button"
                        onClick={stop}
                        aria-label={t("nowPlaying.stop", { name })}
                        className="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border bg-background text-muted-foreground transition-colors hover:border-primary/45 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
                    >
                        <SquareIcon className="size-3.5" aria-hidden="true" />
                    </button>

                    {/* At 390 the seek takes a SECOND ROW of its own, full width, which is what
                        `order-last` plus `w-full` buys out of one wrapping row: the title and
                        the three controls keep the first row exactly as they had it. At 1440
                        the row does not wrap and the seek sits inline between the transport and
                        the volume, sharing the free width with the title column. */}
                    <Seek className="order-last w-full sm:order-none sm:w-auto sm:min-w-[180px] sm:flex-[1.4]" state={state} t={t} />

                    {/* Two things the primitive's defaults get wrong here, both measured. The
                        control box is 44 px tall on a phone so the thumb is a touch target, and
                        the track inside it stays 4 px. And `min-w-44` is CLEARED: the primitive
                        floors a horizontal control at 176 px, which at 390 left the title column
                        88 px and "The Grand Knight Territory" truncated to "The Grand ...".
                        The width then has to come from a BOX AROUND the slider rather than from
                        the slider: its root is `w-full`, so with the floor gone and no definite
                        parent it collapsed to 2 px. */}
                    <div className="flex shrink-0 items-center gap-2">
                        <div className="w-20 sm:w-36">
                            <Slider
                                className="pointer-coarse:[&_[data-slot=slider-control]]:h-11 [&_[data-slot=slider-control]]:min-w-0 pointer-coarse:[&_[data-slot=slider-control]]:items-center max-sm:[&_[data-slot=slider-control]]:h-11 max-sm:[&_[data-slot=slider-control]]:items-center"
                                min={0}
                                max={1}
                                step={0.05}
                                value={[state.volume]}
                                onValueChange={(v) => setVolume(sliderValue(v, state.volume))}
                            >
                                {/* The primitive's own Label, not an `aria-label` on the root: the root renders
                                the GROUP and the thumb's own <input type="range"> is what carries the slider
                                role, so a name on the root leaves the control a reader operates unnamed. */}
                                <SliderPrimitive.Label className="sr-only">{t("nowPlaying.volume")}</SliderPrimitive.Label>
                            </Slider>
                        </div>
                        <span className="hidden w-9 shrink-0 text-right font-mono text-[10px] text-muted-foreground tabular-nums sm:inline">{f.percent(state.volume)}</span>
                    </div>
                </section>
            </div>
        </>
    );
}

/**
 * The timeline, elapsed and total in mono either side of it.
 *
 * The primitives are composed by hand rather than through the `Slider`
 * wrapper, for two things the wrapper cannot pass down: the thumb's
 * `getAriaValueText`, which is what makes the control read as a time instead
 * of a bare number, and a TICK inside the track at the intro-to-loop join, so
 * the point the theme starts repeating from is visible rather than something
 * the listener discovers by waiting.
 */
function Seek({ state, t, className }: { state: IPlayerState; t: NowT; className?: string }): React.ReactElement {
    const [scrub, setScrub] = useState<number | null>(null);
    const length = state.lengthSeconds;
    const ready = length > 0;
    const live = ready ? Math.min(state.positionSeconds, length) : 0;
    const value = scrub ?? live;
    const elapsed = clockTime(value);
    const total = clockTime(length);
    // Only a cue with an intro has a join, and only a join inside the track has a place to draw.
    const loopPercent = ready && state.introSeconds > 0 && state.introSeconds < length ? (state.introSeconds / length) * 100 : null;

    return (
        <div className={cn("flex items-center gap-2", className)}>
            <span className="w-8 shrink-0 text-right font-mono text-[10px] text-muted-foreground tabular-nums">{elapsed}</span>
            <SliderPrimitive.Root
                className="min-w-0 flex-1"
                disabled={!ready}
                // Five seconds an arrow and thirty a page, on a track that is minutes long:
                // the grid the keyboard walks, not a floor on where a drag can land.
                largeStep={30}
                max={ready ? length : 1}
                min={0}
                onValueChange={(v) => setScrub(sliderValue(v, value))}
                onValueCommitted={(v) => {
                    setScrub(null);
                    seek(sliderValue(v, value));
                }}
                step={5}
                thumbAlignment="edge"
                value={[value]}
            >
                <SliderPrimitive.Label className="sr-only">{t("nowPlaying.seek")}</SliderPrimitive.Label>
                {/* 44 px of control on a phone so the thumb is a touch target, with the track
                    itself still 4 px inside it; inline at sm the row is the buttons' height
                    already and the control only has to hold the thumb. */}
                <SliderPrimitive.Control className="flex h-11 w-full touch-none select-none items-center data-disabled:opacity-64 sm:h-5" data-slot="seek-control">
                    <SliderPrimitive.Track className="relative h-1 w-full grow select-none rounded-full bg-input">
                        <SliderPrimitive.Indicator className="rounded-full bg-primary" />
                        {loopPercent !== null && <span aria-hidden="true" className="absolute top-1/2 h-2.5 w-px -translate-x-1/2 -translate-y-1/2 rounded-full bg-foreground/45" style={{ left: `${loopPercent}%` }} title={t("nowPlaying.loopPoint")} />}
                        <SliderPrimitive.Thumb
                            className="block size-4 shrink-0 select-none rounded-full border border-input bg-background not-dark:bg-clip-padding shadow-xs/5 outline-none transition-[box-shadow,scale] has-focus-visible:ring-3 has-focus-visible:ring-ring/24 data-dragging:scale-120 sm:size-3.5 dark:border-background dark:bg-foreground dark:has-focus-visible:ring-ring/48"
                            getAriaValueText={() => t("nowPlaying.seekValue", { elapsed, total })}
                            index={0}
                        />
                    </SliderPrimitive.Track>
                </SliderPrimitive.Control>
            </SliderPrimitive.Root>
            <span className="w-8 shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">{total}</span>
        </div>
    );
}
