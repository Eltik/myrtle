import { useCallback, useMemo } from "react";
import type { IThemeCue } from "./player";
import { stop as stopPlayer, toggle as togglePlayer, useLibraryPlayer } from "./player";

export type { IThemeCue } from "./player";

export interface IChapterAudio {
    /** The key of this group's cue that is sounding, or null. One channel for the whole library, so at most one. */
    playing: string | null;
    /** Start this cue, or pause it when it is the one already sounding. */
    toggle: (key: string, cue: IThemeCue, title: string | null) => void;
    stop: () => void;
}

/**
 * THE CHAPTER SHEET'S VIEW OF THE LIBRARY'S ONE CHANNEL.
 *
 * The hook used to own an audio channel and the reader's volume outright,
 * which is why a theme stopped the moment the sheet unmounted. Both moved to
 * `player.ts`, a module singleton the ticket cards, the list rows and the
 * now-playing bar press on as well; what is left here is the naming.
 *
 * Keys are LOCAL on this side and global in the store. The hero button asks
 * for `theme` and a music row for `track:{id}`, which two chapters would
 * collide on; the store is handed `{groupId}::{local}`, so pressing another
 * group's theme replaces this one rather than sounding beside it, and
 * `playing` is null here whenever the cue that holds the channel belongs to
 * some other chapter.
 */
export function useChapterAudio(group: { id: string; name: string }): IChapterAudio {
    const state = useLibraryPlayer();
    const prefix = `${group.id}::`;

    const playing = useMemo(() => {
        const key = state.track?.key;
        if (key === undefined || state.paused || !key.startsWith(prefix)) return null;
        return key.slice(prefix.length);
    }, [state.track, state.paused, prefix]);

    const toggle = useCallback(
        (key: string, cue: IThemeCue, title: string | null) => {
            togglePlayer({ key: `${group.id}::${key}`, groupId: group.id, groupName: group.name, title, cue });
        },
        [group.id, group.name],
    );

    return { playing, toggle, stop: stopPlayer };
}
