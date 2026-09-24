/**
 * The cutscene layer: a `[Video]` halt drawn as a full-stage player over the
 * canvas, with a Skip pill, which is what the client shows.
 *
 * The clip is the HALT, so the reader stops here until the video ends or the
 * viewer skips; `onEnded` is the advance and the pill is the same advance by
 * hand. WebM is offered first and MP4 second, which is the order the transcode
 * writes them in and the order that keeps Safari off the VP9 file.
 *
 * `play()` is called rather than declared with `autoplay`, because a REJECTED
 * play is the case that matters: the reader is normally past a gesture by the
 * time a cutscene is reached (the title card is a click), but a `?halt=` deep
 * link lands on one with no gesture behind it at all, and a silent refusal
 * would look like a frozen reader. The rejection shows a Play button instead.
 *
 * Three layers draw the same clip, chosen by `cutscenePlayer`. `simple` is the
 * element this file has always rendered, controls and all absent. `native`
 * hands the browser's own control bar the same element. `vidstack` is a
 * separate module behind `React.lazy`, so the player, its layout and its two
 * stylesheets are fetched by the READER THAT MEETS A CUTSCENE and by no other.
 */
import type React from "react";
import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { clamp01 } from "#/lib/story/num";
import type { CutscenePlayer } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import type { VideoSources } from "#/types/generated/VideoSources";

/**
 * The full player, loaded on demand. The import is DYNAMIC and it is the whole
 * point of the file split: a static import would put Vidstack in the reader's
 * own chunk, where every reader pays for it whether a story has a clip or not.
 */
const VidstackCutscene = lazy(() => import("./CutscenePlayer"));

export interface ICutsceneProps {
    /** The resolved transcodes, assets-root-relative as the wire carries them. */
    sources: VideoSources;
    /** The accessible name of the video region. */
    label: string;
    skipLabel: string;
    /** Shown only when the browser refused to start the clip. */
    playLabel: string;
    /** The viewer skipped: advance to the next halt without waiting. */
    onSkip: () => void;
    /** The clip finished, or could not be played at all. */
    onEnded: () => void;
    /**
     * The clip's level, 0..1, applied live. It is the reader's MUSIC volume:
     * a cutscene's track is its score, cut to picture, so it follows the
     * slider the soundtrack follows and not the effects one. The reader's
     * mute silences it the same way it silences the music bus.
     */
    volume: number;
    muted: boolean;
    /** Which controls the clip carries. `simple` is the layer that shipped. */
    player: CutscenePlayer;
    /**
     * The player's OWN volume control moved: the reader's music level and mute
     * follow it, so the two sliders never disagree. Absent means the layer is
     * read-only, which is what `simple` is.
     */
    onVolumeChange?: (volume: number, muted: boolean) => void;
}

/** 44 px, the touch minimum, at every width: the pill is the only control on screen. */
const PILL_BUTTON = "h-11 rounded-lg bg-black/55 px-4 text-white/90 backdrop-blur-sm pointer-coarse:h-11 hover:bg-black/70 hover:text-white focus-visible:ring-white/80 focus-visible:ring-offset-black/40";

/**
 * Which corner the Skip pill takes, and it MOVES with the layer because the
 * bottom-right corner is free on exactly one of the three.
 *
 * `simple` keeps the corner it shipped in: nothing else is drawn. Both control
 * layers put a bar along the bottom whose right end is the fullscreen button,
 * measured at 1440 and at 390, so the pill leaves it: 16 px down the right edge
 * over the browser's own bar, which draws nothing at the top, and 56 px down
 * over the full layout, whose top row is empty at 1440 but carries captions,
 * settings and volume at 390 in a band 40 px tall from the layer's own top.
 */
const PILL_CORNER: Record<CutscenePlayer, string> = {
    simple: "bottom-4",
    native: "top-4",
    vidstack: "top-14",
};

/**
 * How far a level must move before it is written back. A player reports the
 * level it was GIVEN as well as the level a viewer dragged it to, so without a
 * floor the writeback and the prop that caused it chase each other; 0.005 is
 * half the 0.01 step the native slider quantises to and a tenth of the 0.05
 * step the reader's own Music slider uses, so no deliberate drag is swallowed.
 */
export const VOLUME_EPSILON = 0.005;

/**
 * The write a `volumechange` earns, or null for the ones it does not.
 *
 * A mute FLIP always counts, at any level, because muting is not a level
 * change and comparing levels would miss it entirely. A level that is not a
 * finite number is dropped rather than clamped: `Number(null)` is 0 and 0 is
 * SILENCE that would then persist, which is the coercion trap house rule 9
 * names.
 */
export function volumeWriteback(next: { volume: number; muted: boolean }, current: { volume: number; muted: boolean }): { volume: number; muted: boolean } | null {
    if (typeof next.volume !== "number" || !Number.isFinite(next.volume)) return null;
    const moved = Math.abs(next.volume - current.volume) > VOLUME_EPSILON;
    if (!moved && next.muted === current.muted) return null;
    return { volume: clamp01(next.volume), muted: next.muted };
}

/** One transcode as a player reads it: an absolute URL and the type that picks the decoder. */
export interface CutsceneSource {
    src: string;
    type: "video/webm" | "video/mp4";
}

/** The transcodes as a source list, WebM first, absolute against the assets root. */
export function cutsceneSources(sources: VideoSources): CutsceneSource[] {
    const out: CutsceneSource[] = [];
    if (sources.webmUrl) out.push({ src: asset(sources.webmUrl), type: "video/webm" });
    if (sources.mp4Url) out.push({ src: asset(sources.mp4Url), type: "video/mp4" });
    return out;
}

export function Cutscene({ sources, label, skipLabel, playLabel, onSkip, onEnded, volume, muted, player, onVolumeChange }: ICutsceneProps): React.ReactElement {
    const ref = useRef<HTMLVideoElement | null>(null);
    const full = player === "vidstack";
    // Level and mute follow the settings while the clip runs, not only at mount.
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        el.volume = clamp01(Number.isFinite(volume) ? volume : 1);
        el.muted = muted;
    }, [volume, muted]);
    const [blocked, setBlocked] = useState(false);
    // The full player fills this with its own play; the element branch uses the
    // element. Either way the Play button is one call site.
    const startRef = useRef<(() => void) | null>(null);
    const toggleRef = useRef<(() => void) | null>(null);
    const startElement = useCallback(() => {
        const el = ref.current;
        if (!el) return;
        try {
            const started = el.play() as Promise<void> | undefined;
            if (started && typeof started.catch === "function") {
                void started.then(() => setBlocked(false)).catch(() => setBlocked(true));
            }
        } catch {
            // jsdom has no media stack and a locked-down browser can throw
            // synchronously; either way the Play button is the answer.
            setBlocked(true);
        }
    }, []);
    const start = useCallback(() => {
        if (startRef.current) startRef.current();
        else startElement();
    }, [startElement]);
    useEffect(() => {
        if (!full) startElement();
    }, [full, startElement]);

    const report = useCallback(
        (level: number, isMuted: boolean) => {
            if (!onVolumeChange) return;
            const write = volumeWriteback({ volume: level, muted: isMuted }, { volume, muted });
            if (write) onVolumeChange(write.volume, write.muted);
        },
        [onVolumeChange, volume, muted],
    );
    const onElementVolume = useCallback(
        (e: React.SyntheticEvent<HTMLVideoElement>) => {
            const el = e.currentTarget;
            report(el.volume, el.muted);
        },
        [report],
    );

    /**
     * SPACE belongs to the player while the full layout is up, and to the
     * reader everywhere else. Vidstack's own shortcuts are off (`keyDisabled`),
     * because its defaults claim ArrowLeft, ArrowRight, `m`, `f` and `l`, every
     * one of which the reader already answers for; this listener hands back the
     * single key a video player is expected to own. It runs in the CAPTURE
     * phase on `window`, ahead of the reader's own bubble-phase listener on the
     * same object, and stops the event there so a press does not also advance
     * past the clip it just paused.
     */
    useEffect(() => {
        if (!full) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key !== " " && e.key !== "Spacebar") return;
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
            toggleRef.current?.();
        };
        window.addEventListener("keydown", onKey, true);
        return () => window.removeEventListener("keydown", onKey, true);
    }, [full]);

    const list = useMemo(() => cutsceneSources(sources), [sources]);

    return (
        // `data-story-ui` keeps a tap on the clip from reaching the stage, so
        // only the pill and the reader's own advance keys move past it.
        <div data-story-ui data-story-cutscene data-story-cutscene-player={player} className="absolute inset-0 z-50 flex items-center justify-center bg-black">
            {full ? (
                // No fallback art: the layer is already black and the clip's
                // first frame is black too, so a spinner would be the only
                // thing that flashes.
                <Suspense fallback={null}>
                    <VidstackCutscene sources={list} label={label} volume={volume} muted={muted} onEnded={onEnded} onVolume={report} onBlocked={setBlocked} startRef={startRef} toggleRef={toggleRef} />
                </Suspense>
            ) : (
                /* biome-ignore lint/a11y/useMediaCaption: the game ships no caption
                   track for these clips and inventing one would put words in them.
                   The dialogue around the cutscene is the script's own text. */
                <video
                    ref={ref}
                    aria-label={label}
                    className="h-full w-full object-contain"
                    playsInline
                    preload="auto"
                    controls={player === "native"}
                    // The clips are ours to stream and not to hand out, and the
                    // download item is the one native control with no reader
                    // equivalent. Chromium honours it; Firefox and Safari do not.
                    controlsList="nodownload"
                    onEnded={onEnded}
                    onError={onEnded}
                    onVolumeChange={onElementVolume}
                >
                    {/* The same list the full player is handed, so the two
                        layers can never disagree about which file is first. */}
                    {list.map((s) => (
                        <source key={s.src} src={s.src} type={s.type} />
                    ))}
                </video>
            )}
            {blocked ? (
                <Button variant="ghost" size="sm" data-story-cutscene-play onClick={start} className={cn("absolute z-[200]", PILL_BUTTON)}>
                    {playLabel}
                </Button>
            ) : null}
            <Button variant="ghost" size="sm" data-story-cutscene-skip aria-label={skipLabel} onClick={onSkip} className={cn("absolute end-4 z-[200]", PILL_CORNER[player], PILL_BUTTON)}>
                {skipLabel}
            </Button>
        </div>
    );
}
