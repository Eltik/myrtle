/**
 * THE READING STATISTICS ARE COUNTED OVER `index.groups` ALONE, and that is a
 * correctness rule rather than an economy. The 364 `record` groups and the 315
 * operator record sets list the SAME 367 stories (`StoryTotals` says so on the
 * wire), so a pass over both halves counts every operator record twice and
 * reports a library a fifth larger than it is.
 *
 * Words come from `StoryEntry.wordCount`, which the backend counts once at
 * index build. A story the wire sends no count for contributes nothing and is
 * reported as uncounted rather than as zero: "Not counted yet" is a statement
 * about the index, and a 0 would be a statement about the corpus.
 *
 * Pure over its inputs, so the whole section's arithmetic is pinned in a test
 * against a hand-built index rather than read off a screenshot.
 */

import { isStoryRead, type StoryProgress } from "#/lib/story/progress";
import { groupWords, type LibEntry, type LibGroup, type LibIndex } from "./derive";
import { kindOf, matchesFilter, type StoryKind } from "./sections";

/** The four rows of the category table. They are the browse surface's own pills minus `all`, so a row here and a pill there can never mean different sets: "Side stories" is the vignettes plus the intermezzi, both short-form shelves. */
export type StatBucket = "main" | "events" | "side" | "records";

export const STAT_BUCKETS: readonly StatBucket[] = ["main", "events", "side", "records"] as const;

export interface IStatRow {
    key: StatBucket;
    /** Stories read, by the one predicate the library decides that with. */
    read: number;
    /** Stories that CAN be read: the script-less ones are excluded here exactly as they are from every fraction. */
    total: number;
    /** Words over every readable story in the bucket. */
    words: number;
    /** Words over the ones read. */
    wordsRead: number;
}

/** One entry of the longest-first table: a chapter or an event, never an operator record, because a 400-word record beside a 90,000-word chapter is not the same kind of thing. */
export interface ILongRow {
    id: string;
    name: string;
    kind: StoryKind;
    words: number;
    read: number;
    total: number;
}

export interface IReadingStats {
    /** Words in the stories that count as read. */
    wordsRead: number;
    /** Words in every readable story in the index. */
    wordsTotal: number;
    /** False when the wire carries no word count at all, which is the only case the figures are withheld for. */
    counted: boolean;
    read: number;
    total: number;
    rows: IStatRow[];
    longest: ILongRow[];
}

function bucketOf(group: LibGroup): StatBucket | null {
    const kind = kindOf(group);
    for (const bucket of STAT_BUCKETS) {
        if (matchesFilter(kind, bucket)) return bucket;
    }
    return null;
}

/**
 * Everything the statistics section prints, in one pass.
 *
 * `longestLimit` is the top-N of the longest table. 25 is what the section
 * shows; a test asks for fewer to pin the cut rather than the constant.
 */
export function readingStats(index: LibIndex, progress: StoryProgress, gameRead: ReadonlySet<string>, longestLimit = 25): IReadingStats {
    const rows = new Map<StatBucket, IStatRow>(STAT_BUCKETS.map((key) => [key, { key, read: 0, total: 0, words: 0, wordsRead: 0 }]));
    const longest: ILongRow[] = [];
    let counted = typeof index.totals?.words === "number";
    let wordsRead = 0;
    let wordsTotal = 0;
    let read = 0;
    let total = 0;

    for (const group of index.groups) {
        const bucket = bucketOf(group);
        const row = bucket === null ? null : rows.get(bucket);
        let groupRead = 0;
        let groupTotal = 0;
        for (const story of group.stories) {
            if (!story.hasScript) continue;
            const done = isStoryRead(progress, gameRead, story.id);
            groupTotal += 1;
            if (done) groupRead += 1;
            if (typeof story.wordCount === "number") {
                counted = true;
                wordsTotal += story.wordCount;
                if (done) wordsRead += story.wordCount;
                if (row) {
                    row.words += story.wordCount;
                    if (done) row.wordsRead += story.wordCount;
                }
            }
        }
        read += groupRead;
        total += groupTotal;
        if (row) {
            row.read += groupRead;
            row.total += groupTotal;
        }
        const words = group.category === "record" ? null : groupWords(group);
        if (words !== null && words > 0) longest.push({ id: group.id, name: group.name, kind: kindOf(group), words, read: groupRead, total: groupTotal });
    }

    longest.sort((a, b) => b.words - a.words || a.name.localeCompare(b.name));
    return { wordsRead, wordsTotal, counted, read, total, rows: STAT_BUCKETS.flatMap((key) => ((rows.get(key)?.total ?? 0) > 0 ? [rows.get(key) as IStatRow] : [])), longest: longest.slice(0, Math.max(0, longestLimit)) };
}

/** What a set of stories is worth in words, whole and still to read. Both are `null` when not one story in the set carries a count, so a caller prints "Not counted yet" rather than a zero it cannot justify. */
export interface ISetWords {
    total: number | null;
    left: number | null;
}

/**
 * The word arithmetic of one chapter, one arc or one shelf.
 *
 * `left` counts the UNREAD readable stories, so it falls as the reader ticks
 * rows off and reaches 0 on a finished chapter rather than staying at the
 * chapter's whole length. A story with no script is not left to read and is
 * excluded from both figures.
 */
export function setWords(stories: readonly LibEntry[], progress: StoryProgress, gameRead: ReadonlySet<string>): ISetWords {
    let total = 0;
    let left = 0;
    let counted = false;
    for (const story of stories) {
        if (!story.hasScript || typeof story.wordCount !== "number") continue;
        counted = true;
        total += story.wordCount;
        if (!isStoryRead(progress, gameRead, story.id)) left += story.wordCount;
    }
    return counted ? { total, left } : { total: null, left: null };
}

/** One reading-order block's line: its words, its stories, and how many of its chapters are finished. */
export interface ISectionTotals extends ISetWords {
    /** Chapters finished, by {@link readFraction}'s `done`. */
    done: number;
    groups: number;
}

export function sectionTotals(groups: readonly LibGroup[], progress: StoryProgress, gameRead: ReadonlySet<string>): ISectionTotals {
    const stories = groups.flatMap((group) => group.stories);
    let done = 0;
    for (const group of groups) {
        let total = 0;
        let read = 0;
        for (const story of group.stories) {
            if (!story.hasScript) continue;
            total += 1;
            if (isStoryRead(progress, gameRead, story.id)) read += 1;
        }
        if (total > 0 && read === total) done += 1;
    }
    return { ...setWords(stories, progress, gameRead), done, groups: groups.length };
}
