import { playerStore, playQueue, togglePlay } from "../player/engine";
import type { IQueueSource } from "../player/types";
import { type ITrack, isPlayable } from "./model";

/**
 * Play `list` as the queue from `track`, or pause/resume when `track` is
 * already the current one. Tracks with no audio are left out of the queue, so
 * the engine never stalls on an Unavailable in-game track.
 */
export function playFrom(list: readonly ITrack[], track: ITrack, source: IQueueSource): void {
    if (playerStore.state.current?.id === track.id) {
        togglePlay();
        return;
    }
    if (!isPlayable(track)) return;
    const playable = list.filter(isPlayable);
    playQueue(playable, Math.max(0, playable.indexOf(track)), source);
}

/** A random index into a list of `length`; 0 for an empty list. */
export function randomIndex(length: number): number {
    return length > 0 ? Math.floor(Math.random() * length) : 0;
}
