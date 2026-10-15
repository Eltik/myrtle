import type { ITrack } from "../impl/model";

/**
 * THE PLAYER'S CONTRACT: what the engine (`engine.ts`) holds and does, and
 * what every view (the bar, the panels, the library lists, the album modal)
 * reads and calls. Views never touch an audio element; they call these.
 */

export type RepeatMode = "off" | "all" | "one";

export type PlaybackStatus = "idle" | "loading" | "playing" | "paused" | "error";

export type PanelName = "lyrics" | "queue";

/** Where the queue came from, for the "Playing from" line and the link back. */
export interface IQueueSource {
    kind: "none" | "album" | "playlist" | "artist" | "list" | "search" | "radio" | "game";
    /** The album id (`siren:<cid>`), playlist id, artist name or list name. */
    id?: string;
    /** What to print. */
    name?: string;
}

export type PreloadChoice = "auto" | "on" | "off";

/** Minutes a sleep timer can run; 0 is off. */
export const SLEEP_CHOICES = [15, 30, 45, 60] as const;
export type SleepMinutes = 0 | (typeof SLEEP_CHOICES)[number];

export interface IPlayerSettings {
    /** Fade between tracks instead of switching. */
    crossfadeOn: boolean;
    /** 1 to 10 s. */
    crossfadeSeconds: number;
    /** When the queue ends, keep playing the rest of the album, then the releases around it. */
    radioOn: boolean;
    /** A compressor on the label tracks evens loud and quiet releases out. */
    levelingOn: boolean;
    /** Buffer the next track ahead. `auto` is on except on coarse-pointer devices, Save-Data or cellular. */
    preload: PreloadChoice;
    /** Seek to where the last session stopped on reload. */
    resumePosition: boolean;
}

export const CROSSFADE_MIN = 1;
export const CROSSFADE_MAX = 10;

export const DEFAULT_SETTINGS: IPlayerSettings = {
    crossfadeOn: false,
    crossfadeSeconds: 3,
    radioOn: false,
    levelingOn: false,
    preload: "auto",
    resumePosition: true,
};

export type LyricsStatus = "none" | "loading" | "ready" | "empty" | "failed";

export interface ILyricLine {
    /** Seconds from the start. */
    time: number;
    text: string;
}

export interface ILyricsState {
    trackId: string | null;
    status: LyricsStatus;
    /** Sorted by time. */
    lines: ILyricLine[];
}

export interface IPlayerState {
    /** The queue in display order. */
    queue: ITrack[];
    /** Index into `queue` of the current track; -1 when nothing is loaded. */
    index: number;
    /**
     * The PLAY order as indices into `queue`: the identity when shuffle is
     * off, a shuffled permutation (current track first) when it is on.
     */
    order: number[];
    current: ITrack | null;
    status: PlaybackStatus;
    /** 0 to 1, the slider's value. */
    volume: number;
    muted: boolean;
    shuffle: boolean;
    repeat: RepeatMode;
    source: IQueueSource;
    settings: IPlayerSettings;
    /** Epoch ms when the sleep timer pauses playback; `null` when off. */
    sleepEndsAt: number | null;
    /** The open side panel, if any. */
    panel: PanelName | null;
    settingsOpen: boolean;
    shortcutsOpen: boolean;
    lyrics: ILyricsState;
    /** Tracks skipped in a row because they failed to load; reset by a successful play. */
    failures: number;
    /** The engine has read its saved session and the catalogue, so views can trust `queue`. */
    restored: boolean;
    /** The catalogue's house artist (`ICatalog.houseArtist`), so views can call `displayArtists` without the catalogue. */
    houseArtist: string | null;
}

/** The playhead, in its own store so a 4 Hz clock does not re-render the queue. */
export interface IPlayhead {
    /** Seconds into the track. */
    position: number;
    /** Seconds; 0 while unknown. For an in-game track, the intro plus two passes of the loop. */
    duration: number;
    /** Seconds where the loop begins on an in-game track; `null` otherwise. */
    loopAt: number | null;
    /** 0 to 1, how much is buffered ahead (label tracks only). */
    buffered: number;
}

/** Maps a keyboard shortcut to what it does, for the shortcuts dialog. The keys are display strings. */
export interface IShortcut {
    /** A message key under `music.shortcut.*`. */
    id: string;
    keys: string[];
    /** A second binding for the same action, shown after an "or". */
    alt?: string[];
}
