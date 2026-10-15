// Design-bundle stand-in for `@vidstack/react` (826 KB unminified; the bundle must
// stay under the 12 MiB upload cap). Only story/reader/CutscenePlayer.tsx uses it.
// `MediaPlayer` becomes a native <video> with the same sources and native controls,
// and its ref exposes the play/pause/state.paused surface CutscenePlayer drives, so
// a design still gets a working player, without Vidstack's skin and control bar.
import { forwardRef, useImperativeHandle, useRef } from "react";
import type React from "react";

export interface MediaPlayerInstance {
    play(): Promise<void>;
    pause(): void;
    readonly state: { readonly paused: boolean };
}

interface IMediaPlayerProps {
    className?: string;
    style?: React.CSSProperties;
    src?: { src: string; type: string }[];
    title?: string;
    "aria-label"?: string;
    playsInline?: boolean;
    autoPlay?: boolean;
    volume?: number;
    muted?: boolean;
    onEnded?: () => void;
    onError?: () => void;
    onVolumeChange?: (detail: { volume: number; muted: boolean }) => void;
    children?: React.ReactNode;
    [prop: string]: unknown;
}

export const MediaPlayer = forwardRef<MediaPlayerInstance, IMediaPlayerProps>(function MediaPlayer(props, ref) {
    const video = useRef<HTMLVideoElement>(null);
    useImperativeHandle(ref, () => ({
        play: () => video.current?.play() ?? Promise.resolve(),
        pause: () => video.current?.pause(),
        get state() {
            return { paused: video.current?.paused ?? true };
        },
    }));
    return (
        <div className={props.className} style={{ ...props.style, background: "#000" }} aria-label={props["aria-label"]}>
            <video
                ref={video}
                title={props.title}
                playsInline={props.playsInline}
                autoPlay={props.autoPlay}
                muted={props.muted}
                controls
                onEnded={props.onEnded}
                onError={props.onError}
                onVolumeChange={(e) => props.onVolumeChange?.({ volume: e.currentTarget.volume, muted: e.currentTarget.muted })}
                style={{ width: "100%", height: "100%", objectFit: "contain" }}
            >
                {props.src?.map((s) => <source key={s.src} src={s.src} type={s.type} />)}
            </video>
        </div>
    );
});

export function MediaProvider(): null {
    return null;
}
