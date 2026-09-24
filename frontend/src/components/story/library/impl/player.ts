import { useCallback, useEffect, useSyncExternalStore } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { createStoryAudio, type StoryAudio } from "#/lib/story/audio";
import { clamp01 } from "#/lib/story/num";
import { DEFAULT_SETTINGS, loadSettings, updateSettings } from "#/lib/story/settings";
import type { StoryMusic } from "#/types/generated/StoryMusic";

/** A cue as the wire carries it: asset paths, not playable urls. */
export interface IThemeCue {
    intro?: string;
    loop: string;
}

/** What is sounding, named well enough for a bar at the foot of the page to say so. */
export interface IPlayerTrack {
    /** Unique across the whole library: `{groupId}::{local}`, so a second group's theme REPLACES the first. */
    key: string;
    groupId: string;
    groupName: string;
    /** The track's own title where the music table has one, null where it does not. */
    title: string | null;
    cue: IThemeCue;
}

export interface IPlayerState {
    /** The cue the channel holds, whether it is sounding or paused. Null when the channel is idle. */
    track: IPlayerTrack | null;
    paused: boolean;
    /** The reader's `musicVolume`, 0..1, read from and written to the reader's own settings document. */
    volume: number;
    muted: boolean;
    /** Seconds into the cue's timeline, wrapped at the loop. A paused cue holds the second it stopped on. */
    positionSeconds: number;
    /** The decoded intro's length, so the bar can mark where the loop begins. Zero for a loop-only cue. */
    introSeconds: number;
    /** Intro plus loop, in seconds. Zero until the decode lands, and zero for a channel that fell back to a media element. */
    lengthSeconds: number;
}

/** The local half of a chapter theme's key. An archive track uses `track:{id}` instead. */
export const THEME_LOCAL_KEY = "theme";

/** Long enough that a stop is a fade and not a cut, short enough that a close is instant. */
const FADE_SEC = 0.35;

const INITIAL: IPlayerState = { track: null, paused: false, volume: DEFAULT_SETTINGS.musicVolume, muted: DEFAULT_SETTINGS.muted, positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 };

/**
 * THE LIBRARY'S OWN MUSIC CHANNEL, one per page rather than one per sheet.
 *
 * The channel used to live inside the chapter sheet, so a theme died the
 * moment the sheet closed and browsing on with the music playing was
 * impossible. It is a module singleton now: the sheet, the ticket cards, the
 * list rows and the bar at the foot of the page are four controls over ONE
 * cue, which is why a second group's theme replaces the first instead of
 * layering over it.
 *
 * Volume is the reader's `musicVolume` and there is no second key for it. The
 * settings document publishes no write notification (unlike `onProgressWritten`
 * for the reading document), so this store reads and patches it directly
 * through `loadSettings`/`updateSettings`, which is the same path the reader's
 * own hook writes through. The two never run at once: the reader is a different
 * route, so the library is unmounted while a story is open and each side reads
 * the document fresh on mount.
 *
 * PAUSE IS A STOP THAT REMEMBERS ITS SECOND. `AudioBufferSourceNode` still
 * cannot be resumed, so a pause is still a stop, but the position is read off
 * the audio clock before the sources die and the resume starts new ones that
 * far into the cue, which the listener hears as a continuation rather than the
 * replay from the intro this used to be. The seek slider is the same
 * mechanism pointed anywhere on the timeline. What it costs: the join is
 * rebuilt each time, and a channel whose decode fell back to a media element
 * reports no timeline at all, so there it remains a replay from the top and
 * the bar shows no slider.
 */
let audio: StoryAudio | null = null;
let state: IPlayerState = INITIAL;
const listeners = new Set<() => void>();

function emit(): void {
    for (const fn of listeners) fn();
}

function put(next: Partial<IPlayerState>): void {
    state = { ...state, ...next };
    emit();
}

export function subscribe(fn: () => void): () => void {
    listeners.add(fn);
    return () => {
        listeners.delete(fn);
    };
}

export function getState(): IPlayerState {
    return state;
}

/**
 * The server renders an idle channel, always. The bar is invisible while the
 * channel is idle, so the first client render agrees with the markup and
 * there is nothing for hydration to disagree about.
 */
export function getServerState(): IPlayerState {
    return INITIAL;
}

/** Read the reader's levels into the store. The mount effect calls this; nothing sounds yet. */
export function hydrate(): void {
    const settings = loadSettings();
    if (settings.musicVolume === state.volume && settings.muted === state.muted) return;
    put({ volume: settings.musicVolume, muted: settings.muted });
}

/**
 * Open a cue on the channel. `offsetSeconds` null is a PRESS, which leaves a
 * cue that is already sounding alone; a number is a resume or a seek, and
 * those restart the sources wherever the number points.
 */
function start(track: IPlayerTrack, offsetSeconds: number | null): void {
    const settings = loadSettings();
    audio ??= createStoryAudio();
    if (!audio.arm()) return;
    audio.setMasterMusic(settings.musicVolume);
    audio.setMuted(settings.muted);
    // A crossfade only makes sense over something that is sounding. Over silence
    // it is a fade-IN the reader hears as a slow start. A seek is neither: the
    // two ends are the same track, so a crossfade there is the track over itself.
    const crossfade = offsetSeconds === null && state.track !== null && !state.paused ? FADE_SEC : 0;
    const req = { intro: track.cue.intro ? asset(track.cue.intro) : undefined, loop: asset(track.cue.loop), volume: 1, crossfade };
    if (offsetSeconds === null) audio.playMusic(req);
    else audio.playMusicAt(req, offsetSeconds);
    // The length survives a seek within one cue and is unknown for a new one,
    // because only the decode that has yet to land can say what it is.
    const sameCue = state.track?.key === track.key;
    put({
        track,
        paused: false,
        volume: settings.musicVolume,
        muted: settings.muted,
        positionSeconds: offsetSeconds ?? 0,
        introSeconds: sameCue ? state.introSeconds : 0,
        lengthSeconds: sameCue ? state.lengthSeconds : 0,
    });
}

/** Start a cue, replacing whatever the channel holds. A browser that refuses an AudioContext leaves the state idle. */
export function play(track: IPlayerTrack): void {
    start(track, null);
}

/** Silence the channel but keep the cue AND the second it reached, so the bar can offer to go on from there. */
export function pause(): void {
    if (state.track === null || state.paused) return;
    // Read before the stop: `stopMusic` drops the timeline the position is derived from.
    const positionSeconds = audio?.musicPosition ?? state.positionSeconds;
    audio?.stopMusic(FADE_SEC);
    put({ paused: true, positionSeconds });
}

export function resume(): void {
    if (state.track === null || !state.paused) return;
    start(state.track, state.positionSeconds);
}

/**
 * Move to any second of the cue. A paused cue only moves its MARK: the
 * listener asked where to go on from, not to start sounding again.
 */
export function seek(seconds: number): void {
    if (state.track === null) return;
    const next = Math.max(0, state.lengthSeconds > 0 ? Math.min(seconds, state.lengthSeconds) : seconds);
    if (!Number.isFinite(next)) return;
    if (state.paused) {
        put({ positionSeconds: next });
        return;
    }
    start(state.track, next);
}

/** Silence the channel and forget the cue, which is what dismisses the bar. */
export function stop(): void {
    if (state.track === null) return;
    audio?.stopMusic(FADE_SEC);
    put({ track: null, paused: false, positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 });
}

/** Press on the cue that is already loaded to pause or resume it; press on any other to replace it. */
export function toggle(track: IPlayerTrack): void {
    if (state.track?.key !== track.key) {
        play(track);
        return;
    }
    if (state.paused) resume();
    else pause();
}

/**
 * The reader's `musicVolume`, applied to the live gain and persisted in the
 * reader's own document. Zero is SILENCE and it persists, which is what a
 * reader who drags the slider to the floor is asking for.
 */
export function setVolume(volume: number): void {
    const next = clamp01(volume);
    updateSettings({ musicVolume: next });
    audio?.setMasterMusic(next);
    put({ volume: next });
}

/** Leaving `/stories` takes the channel with it: the reader has music of its own and the two must never overlap. */
export function dispose(): void {
    audio?.dispose();
    audio = null;
    state = { ...state, track: null, paused: false, positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 };
    emit();
}

/** The theme cue for a group, or null for the one group whose bank names a loop clip the tree does not hold. */
export function themeTrack(group: { id: string; name: string; music?: StoryMusic }): IPlayerTrack | null {
    const music = group.music;
    if (!music) return null;
    const title = music.title?.trim();
    return { key: `${group.id}::${THEME_LOCAL_KEY}`, groupId: group.id, groupName: group.name, title: title ? title : null, cue: { intro: music.introUrl, loop: music.loopUrl } };
}

/** True while this exact cue is the one sounding. A paused cue is not sounding. */
export function isSounding(current: IPlayerState, key: string): boolean {
    return current.track?.key === key && !current.paused;
}

export function useLibraryPlayer(): IPlayerState {
    return useSyncExternalStore(subscribe, getState, getServerState);
}

/**
 * ONE BOOLEAN, for the 86 glyphs on the shelf.
 *
 * A card that subscribed to the whole state would re-render all 86 of them on
 * every step of a volume drag. The snapshot here is a primitive, so React
 * compares it and re-renders only the glyph whose own cue started or stopped;
 * the other 85 cost one string comparison each.
 */
export function useIsSounding(key: string | null): boolean {
    const get = useCallback(() => (key === null ? false : isSounding(state, key)), [key]);
    return useSyncExternalStore(subscribe, get, () => false);
}

/**
 * THE CLOCK RUNS ONLY WHILE SOMEONE IS WATCHING IT.
 *
 * Nothing about the music needs a tick: the position is a subtraction on the
 * audio clock and is right whether or not anybody asks. The only reason to
 * loop at all is to move a slider on screen, so the loop belongs to the bar's
 * mount and to a cue that is actually sounding. A stopped channel, a paused
 * cue and a page with no bar on it each cost zero frames and zero timers, and
 * the frame that is running is cancelled the moment any of those becomes true.
 */
let frame: number | null = null;

/** One reading of the audio clock into the store. The loop below is this, once a frame. */
export function samplePosition(): void {
    if (audio === null || state.track === null || state.paused) return;
    const positionSeconds = audio.musicPosition;
    const introSeconds = audio.musicIntroSeconds;
    const lengthSeconds = audio.musicLength;
    if (positionSeconds !== state.positionSeconds || introSeconds !== state.introSeconds || lengthSeconds !== state.lengthSeconds) put({ positionSeconds, introSeconds, lengthSeconds });
}

function tickPosition(): void {
    frame = null;
    if (audio === null || state.track === null || state.paused) return;
    samplePosition();
    frame = window.requestAnimationFrame(tickPosition);
}

export function useNowPlayingClock(running: boolean): void {
    useEffect(() => {
        if (!running || typeof window === "undefined" || typeof window.requestAnimationFrame !== "function") return;
        if (frame === null) frame = window.requestAnimationFrame(tickPosition);
        return () => {
            if (frame !== null) window.cancelAnimationFrame(frame);
            frame = null;
        };
    }, [running]);
}

/**
 * The channel's LIFETIME, mounted once by the library page. The store outlives
 * every card and sheet on the page, and it must not outlive the page itself.
 */
export function useLibraryPlayerLifetime(): void {
    useEffect(() => {
        hydrate();
        return dispose;
    }, []);
}
