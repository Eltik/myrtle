/**
 * The full cutscene player: Vidstack's `MediaPlayer` under its default video
 * layout, which is what buys the scrubber, the volume slider, the settings
 * menu, fullscreen and picture-in-picture without us drawing any of them.
 *
 * This module is reached only through `React.lazy` from `Cutscene.tsx`. Its two
 * stylesheets are imported HERE and nowhere else, so Vite emits them beside
 * this chunk and a reader who never opens a cutscene never fetches either.
 *
 * `keyDisabled` is the load-bearing prop. Vidstack's default shortcuts claim
 * ArrowLeft, ArrowRight, `m`, `f`, `l`, `j` and `k`, and the reader answers
 * five of those already (advance, back, mute, fullscreen, log), so a player
 * that kept them would quietly eat the reader's own keyboard for the length of
 * a clip. Every key is left to the reader and `Cutscene` hands back exactly
 * one, Space, through `toggleRef`.
 */
import { MediaPlayer, type MediaPlayerInstance, MediaProvider } from "@vidstack/react";
import { DefaultVideoLayout, defaultLayoutIcons } from "@vidstack/react/player/layouts/default";
import "@vidstack/react/player/styles/default/theme.css";
import "@vidstack/react/player/styles/default/layouts/video.css";
import type React from "react";
import { useEffect, useRef } from "react";
import type { CutsceneSource } from "./Cutscene";

export interface ICutscenePlayerProps {
    /** The transcodes, already absolute, WebM first. */
    sources: CutsceneSource[];
    label: string;
    volume: number;
    muted: boolean;
    onEnded: () => void;
    /** The layout's own volume slider or mute button moved. */
    onVolume: (volume: number, muted: boolean) => void;
    /** Autoplay was refused, or a later play succeeded: the Play pill follows this. */
    onBlocked: (blocked: boolean) => void;
    /** Filled with this player's play, for the Play pill `Cutscene` owns. */
    startRef: React.RefObject<(() => void) | null>;
    /** Filled with this player's play/pause, for the Space key `Cutscene` intercepts. */
    toggleRef: React.RefObject<(() => void) | null>;
}

/**
 * The layout in the reader's own accent, and the whole theme is three
 * variables. `--media-brand` is the one Vidstack documents, but on its own it
 * paints NOTHING here: the shipped theme defines `--media-slider-track-fill-bg`
 * as `--color-inverse`, measured as rgb(245, 245, 245), which wins over the
 * brand fallback on both sliders, so the accent has to be handed to that
 * variable directly. `--media-font-family` keeps the time code and the menu on
 * the site's face instead of the player's own stack.
 */
const THEME = {
    "--media-brand": "var(--primary)",
    "--media-slider-track-fill-bg": "var(--primary)",
    "--media-font-family": "inherit",
} as const;

export default function CutscenePlayer({ sources, label, volume, muted, onEnded, onVolume, onBlocked, startRef, toggleRef }: ICutscenePlayerProps): React.ReactElement {
    const player = useRef<MediaPlayerInstance>(null);
    useEffect(() => {
        startRef.current = () => {
            const p = player.current;
            if (!p) return;
            try {
                void p
                    .play()
                    .then(() => onBlocked(false))
                    .catch(() => onBlocked(true));
            } catch {
                onBlocked(true);
            }
        };
        toggleRef.current = () => {
            const p = player.current;
            if (!p) return;
            // `paused` is the player's own state, so this is the same toggle the
            // layout's play button performs and the two never disagree.
            if (p.state.paused) startRef.current?.();
            else p.pause();
        };
        return () => {
            startRef.current = null;
            toggleRef.current = null;
        };
    }, [startRef, toggleRef, onBlocked]);

    return (
        <MediaPlayer
            ref={player}
            className="h-full w-full"
            style={THEME}
            src={sources}
            title={label}
            aria-label={label}
            viewType="video"
            playsInline
            autoPlay
            keyDisabled
            volume={volume}
            muted={muted}
            onEnded={onEnded}
            onError={onEnded}
            // Autoplay refused is the deep-link case: no gesture stands behind a
            // `?halt=` that lands on a clip, and a silent refusal reads as a
            // frozen reader. The Play pill is the answer, the same one the bare
            // element gets.
            onAutoPlayFail={() => onBlocked(true)}
            onAutoPlay={() => onBlocked(false)}
            onVolumeChange={(detail) => onVolume(detail.volume, detail.muted)}
        >
            <MediaProvider />
            <DefaultVideoLayout icons={defaultLayoutIcons} noKeyboardAnimations noGestures={false} />
        </MediaPlayer>
    );
}
