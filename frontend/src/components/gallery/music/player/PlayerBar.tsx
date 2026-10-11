import { Disc3Icon, HeartIcon, ListMusicIcon, MicVocalIcon, PauseIcon, PlayIcon, Repeat1Icon, RepeatIcon, Settings2Icon, ShuffleIcon, SkipBackIcon, SkipForwardIcon, SquareIcon, Volume1Icon, Volume2Icon, VolumeXIcon, XIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Slider, SliderPrimitive } from "#/components/ui/slider";
import { Spinner } from "#/components/ui/spinner";
import { useFormatters, useT } from "#/lib/i18n";
import { clockTime } from "#/lib/story/reading";
import { sliderValue } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import { toggleLike, useIsLiked } from "../impl/library/store";
import { clearQueue, cycleRepeat, cycleVolume, next, openPanel, previous, seek, setSettingsOpen, setVolume, stop, togglePanel, togglePlay, toggleShuffle, usePlayer, usePlayhead } from "./engine";
import { IconButton, type PlayerT, subLine, useKicker } from "./parts";

/**
 * THE BAR at the foot of the music page: a seek line across its top edge,
 * the track on the left, the transport in the middle, and levels, panels
 * and settings on the right. On a phone it keeps the cover, the title, play
 * and next; the title opens the queue.
 *
 * It is fixed, not sticky. Its height (plus the home-indicator inset) is
 * reserved as padding at the foot of the body (`data-music-player-bar` in
 * `styles.css`), below the site footer, so appearing never moves what is
 * already on screen and nothing at the bottom of the page sits under it. A
 * spacer inside the page reserved it above the footer, and the bar covered
 * the footer's last 63 px at 375 px wide.
 */
export function PlayerBar(): React.ReactElement | null {
    const t: PlayerT = useT("gallery");
    const current = usePlayer((s) => s.current);
    const status = usePlayer((s) => s.status);
    const source = usePlayer((s) => s.source);
    const houseArtist = usePlayer((s) => s.houseArtist);
    const shuffle = usePlayer((s) => s.shuffle);
    const repeat = usePlayer((s) => s.repeat);
    const panel = usePlayer((s) => s.panel);
    const kicker = useKicker(source);
    if (!current) return null;

    const title = current.title;
    const playing = status === "playing" || status === "loading";
    const repeatLabel = repeat === "one" ? t("music.player.repeat.one") : repeat === "all" ? t("music.player.repeat.all") : t("music.player.repeat.off");

    return (
        <>
            {/* Opaque, not a 95% card over a backdrop blur: a fixed backdrop-filter re-blurs the page under it on every scroll frame, for a 5% see-through. */}
            <div data-music-player-bar="" className="fixed inset-x-0 bottom-0 z-40 border-border border-t bg-card shadow-[0_-8px_24px_-12px_rgba(0,0,0,0.45)]">
                <section aria-label={t("music.player.region")} className="relative">
                    <SeekLine t={t} />
                    <span aria-live="polite" className="sr-only">
                        {status === "playing" ? t("music.player.nowPlaying", { title }) : ""}
                    </span>
                    <div className="mx-auto grid max-w-350 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 px-3 pt-2.5 pb-[max(0.5rem,env(safe-area-inset-bottom))] sm:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] sm:px-6">
                        {/* Left: like, cover, what is playing. */}
                        <div className="relative flex min-w-0 items-center gap-2.5">
                            <LikeButton t={t} trackId={current.id} />
                            <div className="relative size-11 shrink-0 overflow-hidden rounded-md border border-border bg-secondary/50">
                                {current.coverUrl ? <img referrerPolicy="no-referrer" alt="" className="size-full object-cover" decoding="async" key={current.coverUrl} src={sizedCover(current.coverUrl, 44)} /> : <Disc3Icon aria-hidden="true" className="absolute inset-0 m-auto size-6 text-muted-foreground" />}
                            </div>
                            <div className="flex min-w-0 flex-col leading-tight">
                                <span className="truncate font-mono text-[9.5px] text-primary/80 uppercase tracking-[0.14em]">{kicker}</span>
                                <span className="truncate font-sans font-semibold text-[13.5px] text-foreground">{title}</span>
                                <span className="truncate text-[11.5px] text-muted-foreground">{subLine(current, houseArtist)}</span>
                            </div>
                            {/* On a phone the whole track area opens the queue. */}
                            <button aria-label={t("music.player.openQueue")} className="absolute inset-0 cursor-pointer rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring/40 sm:hidden" onClick={() => openPanel("queue")} type="button" />
                        </div>

                        {/* Centre: the transport. */}
                        <div className="flex items-center justify-center gap-0.5 sm:gap-1">
                            <IconButton className="max-sm:hidden" label={shuffle ? t("music.player.shuffle.on") : t("music.player.shuffle.off")} onClick={toggleShuffle} pressed={shuffle}>
                                <ShuffleIcon aria-hidden="true" />
                            </IconButton>
                            <IconButton className="max-sm:hidden" label={t("music.player.previous")} onClick={previous}>
                                <SkipBackIcon aria-hidden="true" />
                            </IconButton>
                            <IconButton className="max-sm:hidden" label={t("music.player.stop")} onClick={stop}>
                                <SquareIcon aria-hidden="true" className="size-3.5!" />
                            </IconButton>
                            <button
                                aria-label={status === "loading" ? t("music.player.loading") : playing ? t("music.player.pause") : t("music.player.play")}
                                className="mx-1 inline-flex pointer-coarse:size-11 size-10 shrink-0 cursor-pointer items-center justify-center rounded-full bg-primary text-primary-foreground shadow-sm outline-none transition-transform hover:scale-105 focus-visible:ring-2 focus-visible:ring-ring/60 focus-visible:ring-offset-2 focus-visible:ring-offset-background active:scale-95 max-sm:size-11"
                                onClick={togglePlay}
                                type="button"
                            >
                                {status === "loading" ? <Spinner aria-hidden="true" className="size-4.5" /> : playing ? <PauseIcon aria-hidden="true" className="size-4.5 fill-current" /> : <PlayIcon aria-hidden="true" className="size-4.5 translate-x-px fill-current" />}
                            </button>
                            <IconButton label={t("music.player.next")} onClick={next}>
                                <SkipForwardIcon aria-hidden="true" />
                            </IconButton>
                            <IconButton className="max-sm:hidden" label={repeatLabel} onClick={cycleRepeat} pressed={repeat !== "off"}>
                                {repeat === "one" ? <Repeat1Icon aria-hidden="true" /> : <RepeatIcon aria-hidden="true" />}
                            </IconButton>
                        </div>

                        {/* Right: panels, settings, levels, time, close. */}
                        <div className="hidden min-w-0 items-center justify-end gap-1 sm:flex">
                            <IconButton className="max-lg:hidden" label={t("music.player.lyrics")} onClick={() => togglePanel("lyrics")} pressed={panel === "lyrics"}>
                                <MicVocalIcon aria-hidden="true" />
                            </IconButton>
                            <IconButton label={t("music.player.queue")} onClick={() => togglePanel("queue")} pressed={panel === "queue"}>
                                <ListMusicIcon aria-hidden="true" />
                            </IconButton>
                            <IconButton label={t("music.player.settings")} onClick={() => setSettingsOpen(true)}>
                                <Settings2Icon aria-hidden="true" />
                            </IconButton>
                            <Volume t={t} />
                            <Clock />
                            <IconButton className="ml-1 border border-border" label={t("music.player.close")} onClick={clearQueue}>
                                <XIcon aria-hidden="true" />
                            </IconButton>
                        </div>
                    </div>
                </section>
            </div>
        </>
    );
}

function LikeButton({ trackId, t }: { trackId: string; t: PlayerT }): React.ReactElement {
    const liked = useIsLiked(trackId);
    return (
        <IconButton className={cn("max-sm:hidden", liked && "text-primary")} label={liked ? t("music.player.unlike") : t("music.player.like")} onClick={() => toggleLike(trackId)} pressed={liked}>
            <HeartIcon aria-hidden="true" className={cn("transition-transform", liked && "scale-110 fill-current")} />
        </IconButton>
    );
}

function Volume({ t }: { t: PlayerT }): React.ReactElement {
    const f = useFormatters();
    const volume = usePlayer((s) => s.volume);
    const muted = usePlayer((s) => s.muted);
    const level = muted ? 0 : volume;
    const Icon = level === 0 ? VolumeXIcon : level < 0.5 ? Volume1Icon : Volume2Icon;
    return (
        <div className="flex items-center gap-1">
            <IconButton label={t("music.player.cycleVolume", { percent: f.percent(level) })} onClick={cycleVolume}>
                <Icon aria-hidden="true" />
            </IconButton>
            <div className="w-20 max-md:hidden">
                <Slider className="[&_[data-slot=slider-control]]:min-w-0" max={1} min={0} onValueChange={(v) => setVolume(sliderValue(v, level))} step={0.01} value={[level]}>
                    <SliderPrimitive.Label className="sr-only">{t("music.player.volume")}</SliderPrimitive.Label>
                </Slider>
            </div>
        </div>
    );
}

/** Its own component, so the 4 Hz playhead re-renders the time and nothing else in the bar. */
function Clock(): React.ReactElement {
    const p = usePlayhead();
    return (
        <span className="ml-1 shrink-0 whitespace-nowrap font-mono text-[11px] text-muted-foreground tabular-nums max-lg:hidden">
            {clockTime(p.position)} / {clockTime(p.duration)}
        </span>
    );
}

/**
 * The seek line across the bar's top edge. A drag writes a local scrub value
 * and only the release seeks, so dragging does not restart the audio on every
 * step (the in-game engine rebuilds its sources on each seek). Hovering shows
 * the time under the pointer. An in-game track's loop point is a tick.
 */
function SeekLine({ t }: { t: PlayerT }): React.ReactElement {
    const p = usePlayhead();
    const [scrub, setScrub] = useState<number | null>(null);
    const [hover, setHover] = useState<{ x: number; time: number } | null>(null);
    const ready = p.duration > 0;
    const live = ready ? Math.min(p.position, p.duration) : 0;
    const value = scrub ?? live;
    const elapsed = clockTime(value);
    const total = clockTime(p.duration);
    const loopPercent = ready && p.loopAt !== null && p.loopAt > 0 && p.loopAt < p.duration ? (p.loopAt / p.duration) * 100 : null;

    const onMove = (e: React.PointerEvent<HTMLDivElement>) => {
        if (!ready || e.pointerType === "touch") return;
        const rect = e.currentTarget.getBoundingClientRect();
        if (rect.width <= 0) return;
        const frac = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
        setHover({ x: e.clientX - rect.left, time: frac * p.duration });
    };

    return (
        <SliderPrimitive.Root
            className="absolute inset-x-0 top-0 z-10 -translate-y-1/2"
            disabled={!ready}
            largeStep={30}
            max={ready ? p.duration : 1}
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
            <SliderPrimitive.Label className="sr-only">{t("music.player.seek")}</SliderPrimitive.Label>
            <SliderPrimitive.Control className="group flex h-4 w-full cursor-pointer touch-none select-none items-center data-disabled:cursor-default max-sm:h-6" onPointerLeave={() => setHover(null)} onPointerMove={onMove}>
                {/* The bar thickens on hover by scaleY on its three layers, not by animating the track's height (a relayout per frame); the thumb is not scaled. */}
                <SliderPrimitive.Track className="relative h-1.5 w-full grow select-none before:absolute before:inset-0 before:scale-y-[.667] before:bg-foreground/10 before:transition-transform before:duration-150 before:ease-out group-hover:before:scale-y-100">
                    <span aria-hidden="true" className="absolute inset-y-0 left-0 scale-y-[.667] bg-foreground/10 transition-transform duration-150 ease-out group-hover:scale-y-100" style={{ width: `${Math.round(p.buffered * 1000) / 10}%` }} />
                    <SliderPrimitive.Indicator className="scale-y-[.667] bg-primary shadow-[0_0_8px_var(--color-primary)] transition-transform duration-150 ease-out group-hover:scale-y-100" />
                    {loopPercent !== null && <span aria-hidden="true" className="absolute top-1/2 h-2.5 w-0.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-foreground/60" style={{ left: `${loopPercent}%` }} title={t("music.player.loopPoint")} />}
                    <SliderPrimitive.Thumb
                        className="block size-3 rounded-full bg-primary opacity-0 shadow outline-none transition-opacity group-hover:opacity-100 has-focus-visible:opacity-100 has-focus-visible:ring-3 has-focus-visible:ring-ring/40 data-dragging:opacity-100"
                        getAriaValueText={() => t("music.player.seekValue", { elapsed, total })}
                        index={0}
                    />
                </SliderPrimitive.Track>
            </SliderPrimitive.Control>
            {hover && (
                <span aria-hidden="true" className="pointer-events-none absolute bottom-full mb-1 -translate-x-1/2 rounded border border-border bg-popover px-1.5 py-0.5 font-mono text-[10.5px] text-popover-foreground tabular-nums shadow-md" style={{ left: hover.x }}>
                    {clockTime(hover.time)}
                </span>
            )}
        </SliderPrimitive.Root>
    );
}
