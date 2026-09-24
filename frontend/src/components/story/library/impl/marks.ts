/**
 * TURNING A MARK ON AND OFF IS ONE OPERATION, not two screens' worth of
 * handlers. Every tick in the library runs through {@link toggleRead}, and the
 * two chapter-wide actions run through {@link markAllRead} and
 * {@link clearMarks}, so a mark made in the chapter sheet and one made in a
 * reading-order row cannot end up meaning different things in the document.
 *
 * Pure over `StoryProgress`: the caller saves the returned document, which is
 * what reaches the account sync. Nothing here touches storage.
 */

import { isStoryRead, type StoryProgress, withRead, withUnread } from "#/lib/story/progress";
import type { LibEntry } from "./derive";

/** The stories a mark can apply to. 90 of the 1,887 EN stories have no script, are never read, and are never counted in a fraction, so a bulk action must not mark them either. */
export function markable<T extends Pick<LibEntry, "hasScript">>(stories: readonly T[]): T[] {
    return stories.filter((story) => story.hasScript);
}

/**
 * Flip one story's mark.
 *
 * `read` is the CALLER's verdict, `isStoryRead`, rather than a second reading
 * of the document here: the tick the reader clicked is drawn from that
 * predicate, and deriving the action from anything else would let a tick
 * disagree with what clicking it does. Marking a game-read story unread writes
 * the override that outranks the game (`own`/`game` -> `cleared`), and marking
 * it read again takes the override away (`cleared` -> `own`).
 */
export function toggleRead(progress: StoryProgress, storyId: string, read: boolean, now: number = Date.now()): StoryProgress {
    return read ? withUnread(progress, storyId, now) : withRead(progress, storyId, now);
}

/** Every readable story in the set marked read, in one document. */
export function markAllRead(progress: StoryProgress, stories: readonly LibEntry[], now: number = Date.now()): StoryProgress {
    let out = progress;
    for (const story of markable(stories)) out = withRead(out, story.id, now);
    return out;
}

/**
 * Every mark in the set withdrawn.
 *
 * Only the stories that COUNT as read are touched. `withUnread` on an untouched
 * story would write an override against a verdict nobody made, which the merge
 * then carries to every device as a fact about a story the reader never saw.
 */
export function clearMarks(progress: StoryProgress, stories: readonly LibEntry[], gameRead: ReadonlySet<string>, now: number = Date.now()): StoryProgress {
    let out = progress;
    for (const story of markable(stories)) {
        if (isStoryRead(out, gameRead, story.id)) out = withUnread(out, story.id, now);
    }
    return out;
}

/** How many marks "Clear all" would withdraw, which is the number its confirmation quotes. */
export function markedCount(stories: readonly LibEntry[], progress: StoryProgress, gameRead: ReadonlySet<string>): number {
    let count = 0;
    for (const story of markable(stories)) {
        if (isStoryRead(progress, gameRead, story.id)) count += 1;
    }
    return count;
}
