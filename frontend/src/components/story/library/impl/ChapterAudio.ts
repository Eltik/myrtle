import { useCallback, useMemo, useSyncExternalStore } from "react";
import type { IPlayerState, IThemeCue } from "./player";
import { getState, stop as stopPlayer, subscribe, toggle as togglePlayer } from "./player";

export type { IThemeCue } from "./player";

export interface IChapterAudio {
    /** The key of this group's cue that is sounding, or null. One channel for the whole library, so at most one. */
    playing: string | null;
    /** Start this cue, or pause it when it is the one already sounding. */
    toggle: (key: string, cue: IThemeCue, title: string | null) => void;
    stop: () => void;
}

/**
 * The local key of this group's cue that is SOUNDING, or null: the one thing
 * the sheet draws from the channel. A paused cue, an idle channel and a cue
 * that belongs to another chapter are all null.
 */
export function playingKeyOf(state: IPlayerState, prefix: string): string | null {
    const key = state.track?.key;
    if (key === undefined || state.paused || !key.startsWith(prefix)) return null;
    return key.slice(prefix.length);
}

/**
 * THE CHAPTER SHEET'S VIEW OF THE LIBRARY'S ONE CHANNEL.
 *
 * The audio channel and the reader's volume live in `player.ts`, a module singleton the ticket
 * cards, the list rows and the now-playing bar press on as well, so a theme survives the sheet
 * unmounting. What is left here is the naming.
 *
 * Keys are LOCAL on this side and global in the store. The hero button asks
 * for `theme` and a music row for `track:{id}`, which two chapters would
 * collide on; the store is handed `{groupId}::{local}`, so pressing another
 * group's theme replaces this one rather than sounding beside it, and
 * `playing` is null here whenever the cue that holds the channel belongs to
 * some other chapter.
 */
export function useChapterAudio(group: { id: string; name: string }): IChapterAudio {
    const prefix = `${group.id}::`;

    // THE SHEET SUBSCRIBES TO ONE STRING, NOT TO THE CHANNEL. While a cue
    // sounds, the now-playing bar samples the audio clock into the store once a
    // frame (`useNowPlayingClock`), and a hook that read the whole state
    // re-rendered the sheet at the frame rate for a playhead it never draws:
    // hero, meta block and every row, 150 commits of 7,200 components in one
    // second of Near Light's theme (preview, 1440, a 120 Hz panel). The
    // snapshot here is the key or null, which React compares by value.
    const read = useCallback(() => playingKeyOf(getState(), prefix), [prefix]);
    const playing = useSyncExternalStore(subscribe, read, serverPlaying);

    const toggle = useCallback(
        (key: string, cue: IThemeCue, title: string | null) => {
            togglePlayer({ key: `${group.id}::${key}`, groupId: group.id, groupName: group.name, title, cue });
        },
        [group.id, group.name],
    );

    // One object per change, so the memoised bands that take it skip a render
    // of the sheet that did not touch the music.
    return useMemo(() => ({ playing, toggle, stop: stopPlayer }), [playing, toggle]);
}

function serverPlaying(): string | null {
    return null;
}
