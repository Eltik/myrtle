/**
 * Pure derivations for the story library: reading fractions, the sub-tab a
 * group belongs to, search, sort, and which story the hero card offers.
 *
 * Nothing here touches the DOM or localStorage, so the whole page's arithmetic
 * is testable and the component layer stays a renderer.
 *
 * The bindings declare `wordCount`, `hasVideo`, `rarity`, `profession`,
 * `avatarUrl`, `illustrationCount`, `spriteCount` and `totals` as REQUIRED, but a backend older than those fields is
 * still answering this page, so the library re-types them as optional and reads
 * them DEFENSIVELY everywhere: absent is a first-class value, never a zero. The
 * :3060 index on 2026-09-23 sends none of them.
 */

import { type IPreparedTarget, type IScoreTarget, prepareQuery, prepareTarget, scorePrepared } from "#/lib/search/fuzzy";
import { isStoryRead, type StoryProgress } from "#/lib/story/progress";
import type { OperatorRecordGroup } from "#/types/generated/OperatorRecordGroup";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryGroup } from "#/types/generated/StoryGroup";
import type { Storyline } from "#/types/generated/Storyline";
import type { StoryTotals } from "#/types/generated/StoryTotals";

export type LibEntry = Omit<StoryEntry, "wordCount" | "hasVideo"> & { wordCount?: number; hasVideo?: boolean };
/**
 * A chapter as the page reads it. `bannerUrl`, `iconUrl`, `chapterNumber` and
 * `titleImageUrl` are the Story Collection's own art, and all four were still
 * ABSENT from every one of the 451 groups the :3060 index answered with on
 * 2026-09-23, so each is typed optional here and every consumer falls back
 * rather than rendering a hole. `titleImageUrl` is additionally not yet in the
 * generated `StoryGroup`, which is why it is named here and not merely widened.
 */
export type LibGroup = Omit<StoryGroup, "stories" | "wordCount" | "illustrationCount" | "spriteCount"> & { stories: LibEntry[]; wordCount?: number; illustrationCount?: number; spriteCount?: number; bannerUrl?: string; iconUrl?: string; chapterNumber?: number; titleImageUrl?: string };
export type LibRecord = Omit<OperatorRecordGroup, "stories" | "wordCount" | "rarity" | "profession" | "avatarUrl" | "illustrationCount" | "spriteCount"> & { stories: LibEntry[]; wordCount?: number; rarity?: number; profession?: string; avatarUrl?: string; illustrationCount?: number; spriteCount?: number };
/** A shelf as the page reads it. `iconUrl` and `chapterRange` were both ABSENT on :3060 on 2026-09-23, on all 14 EN shelves and all 4 EN arcs. */
export type LibStoryline = Storyline & { iconUrl?: string; chapterRange?: { from: number; to: number } };
export type LibIndex = { groups: LibGroup[]; records: LibRecord[]; totals?: StoryTotals; storylines?: LibStoryline[] };

/** The Stories tab's sub-tabs. `intermezzi` is not a `StoryCategory`: the game shelves `displayType === "BRANCHLINE"` events on their own. */
export type StoriesTabKey = "main" | "side" | "intermezzi" | "vignette" | "is" | "reclamation" | "sideContent";

/** Left to right, the order the game's Archives uses. The three at the end are empty today and only render when the data grows them. */
export const STORIES_TAB_ORDER: readonly StoriesTabKey[] = ["main", "side", "intermezzi", "vignette", "is", "reclamation", "sideContent"] as const;

export type SortMode = "release" | "alpha";

/** Which sub-tab a group belongs under, or `null` for an operator record set (those live in the Operators tab). */
export function tabOf(group: Pick<LibGroup, "category" | "displayType">): StoriesTabKey | null {
    if (group.category === "record") return null;
    if (group.displayType === "BRANCHLINE") return "intermezzi";
    return group.category;
}

/** The sub-tabs that have at least one group, in {@link STORIES_TAB_ORDER}. */
export function buildTabs(groups: readonly LibGroup[]): { key: StoriesTabKey; groups: LibGroup[] }[] {
    const buckets = new Map<StoriesTabKey, LibGroup[]>();
    for (const group of groups) {
        const key = tabOf(group);
        if (key === null) continue;
        const bucket = buckets.get(key);
        if (bucket) bucket.push(group);
        else buckets.set(key, [group]);
    }
    return STORIES_TAB_ORDER.flatMap((key) => {
        const bucket = buckets.get(key);
        return bucket && bucket.length > 0 ? [{ key, groups: bucket }] : [];
    });
}

export interface IReadFraction {
    /** Stories marked read. */
    read: number;
    /** Stories that CAN be read: 90 of 1,887 have no script, and counting those would make a completed group read 12/14 forever. */
    total: number;
    /** Every story listed, readable or not. */
    listed: number;
    done: boolean;
}

/**
 * Reading progress over a set of stories. The verdict is `isStoryRead`
 * everywhere, so the document and the game's own read set are weighed the same
 * way here as in a row: a story the game reports read fills this fraction
 * without ever entering the document.
 */
export function readFraction(stories: readonly Pick<LibEntry, "id" | "hasScript">[], progress: StoryProgress, gameRead: ReadonlySet<string>): IReadFraction {
    let total = 0;
    let hit = 0;
    for (const story of stories) {
        if (!story.hasScript) continue;
        total += 1;
        if (isStoryRead(progress, gameRead, story.id)) hit += 1;
    }
    return { read: hit, total, listed: stories.length, done: total > 0 && hit === total };
}

/** Stories in the order the game lists them. */
export function sortedStories(stories: readonly LibEntry[]): LibEntry[] {
    return [...stories].sort((a, b) => a.sort - b.sort || a.id.localeCompare(b.id));
}

/**
 * A list with its search haystacks normalised ONCE, so a keystroke is one scan
 * over prepared strings and not a re-fold of every name. The browse page
 * builds one per half of the index with `useMemo` and matches every keystroke
 * against it; `filterGroups` and `filterRecords` are the one-shot form of the
 * same thing and answer the same set.
 */
export interface ISearchList<T> {
    items: readonly T[];
    prepared: readonly IPreparedTarget[];
}

export function prepareSearch<T>(items: readonly T[], target: (item: T) => IScoreTarget): ISearchList<T> {
    return { items, prepared: items.map((item) => prepareTarget(target(item))) };
}

/**
 * The items that match, in INPUT order: the caller's sort toggle owns the
 * order and the fuzzy score decides membership only. A blank query keeps
 * everything.
 */
export function searchList<T>(query: string, list: ISearchList<T>): T[] {
    if (query.trim() === "") return [...list.items];
    const q = prepareQuery(query);
    return list.items.filter((_, at) => {
        const target = list.prepared[at];
        return target !== undefined && scorePrepared(q, target) > 0;
    });
}

/** What a group is searched by: its name, then every story's name and operation code. */
export function groupSearchTarget(group: { name: string; stories: readonly { name: string; code?: string }[] }): IScoreTarget {
    return { name: group.name, extra: group.stories.map((s) => `${s.name} ${s.code ?? ""}`).join(" ") };
}

/** What an operator record set is searched by: the operator's name, then every record's name. */
export function recordSearchTarget(record: Pick<LibRecord, "name" | "stories">): IScoreTarget {
    return { name: record.name, extra: record.stories.map((s) => s.name).join(" ") };
}

/** Groups whose name, or one of whose story names, matches, in input order. */
export function filterGroups<T extends { name: string; stories: readonly { name: string; code?: string }[] }>(query: string, groups: readonly T[]): T[] {
    if (query.trim() === "") return [...groups];
    return searchList(query, prepareSearch(groups, groupSearchTarget));
}

/** Operator record sets whose operator name, or one of whose record names, matches. */
export function filterRecords(query: string, records: readonly LibRecord[]): LibRecord[] {
    if (query.trim() === "") return [...records];
    return searchList(query, prepareSearch(records, recordSearchTarget));
}

/**
 * Release order is newest first. The 17 mainline groups all carry
 * `startTime: -1` (the table has no date for them), so the tie-break is the
 * id read NUMERICALLY, which puts `main_0` before `main_10` and keeps the
 * chapters in reading order under a toggle that means nothing to them.
 */
export function sortGroups<T extends { id: string; name: string; startTime: number }>(groups: readonly T[], mode: SortMode, collator: Intl.Collator): T[] {
    const out = [...groups];
    if (mode === "alpha") return out.sort((a, b) => collator.compare(a.name, b.name));
    return out.sort((a, b) => b.startTime - a.startTime || collator.compare(a.id, b.id));
}

/** A group's word count: the field when the backend sends it, else the sum over its stories, else absent. */
export function groupWords(group: Pick<LibGroup, "stories" | "wordCount">): number | null {
    if (typeof group.wordCount === "number") return group.wordCount;
    let sum = 0;
    let seen = false;
    for (const story of group.stories) {
        if (typeof story.wordCount === "number") {
            sum += story.wordCount;
            seen = true;
        }
    }
    return seen ? sum : null;
}

/**
 * The short code a group is known by: the chapter's own third name
 * (`EPISODE 00`) for the mainline, else the operation prefix its stories share
 * (`GT-1` -> `GT`), else the group id. It is the typographic cover fallback and
 * the card's subtitle, so it never returns an empty string.
 */
export function groupCode(group: Pick<LibGroup, "id" | "zone" | "stories">): string {
    const third = group.zone?.nameThird?.trim();
    if (third) return third;
    // A real operation code is hyphenated (`CB-1`, `GT-6`); the two `ENTRY`
    // rows in the corpus (`act3d0`, `act5d0` first-enter stories) are a table
    // constant, not a code, so a hyphen-less value is skipped.
    for (const story of group.stories) {
        const code = story.code?.trim();
        if (!code?.includes("-")) continue;
        const prefix = code.split("-")[0]?.trim();
        if (prefix) return prefix.toUpperCase();
    }
    return group.id.toUpperCase();
}

export interface IHeroPick {
    entry: LibEntry;
    group: LibGroup;
    /** Set when the story is an operator record: the operator's name, which reads better than the record set's. */
    recordName?: string;
    /** The halt to resume at, or `null` when the story was never opened. */
    resumeHalt: number | null;
    resumeTotal: number | null;
    /** True when nothing has been read yet and this is the Prologue offered to a first-time visitor. */
    fresh: boolean;
}

/**
 * The hero story: the last one opened, falling back to the first mainline
 * story. A `last` id that no longer exists in the index (a renamed story, a
 * restored backup from another server) falls back the same way rather than
 * rendering an empty hero.
 */
export function pickHero(index: LibIndex, progress: StoryProgress): IHeroPick | null {
    const last = progress.last;
    if (last) {
        for (const group of index.groups) {
            const entry = group.stories.find((s) => s.id === last);
            if (!entry) continue;
            const pos = progress.pos[last];
            const record = group.category === "record" ? index.records.find((r) => r.stories.some((s) => s.id === last)) : undefined;
            return { entry, group, recordName: record?.name, resumeHalt: pos ? pos.halt : null, resumeTotal: pos ? pos.total : null, fresh: false };
        }
    }
    const mains = index.groups.filter((g) => g.category === "main");
    const first = mains.sort((a, b) => a.startTime - b.startTime || a.id.localeCompare(b.id, undefined, { numeric: true }))[0];
    if (!first) return null;
    const entry = sortedStories(first.stories).find((s) => s.hasScript);
    if (!entry) return null;
    return { entry, group: first, resumeHalt: null, resumeTotal: null, fresh: true };
}

export interface IProgressSummary {
    read: number;
    total: number;
    fraction: number;
    groupsDone: number;
    groups: number;
    /** Words in the stories marked read, or `null` while the backend sends no word counts. */
    words: number | null;
}

/**
 * The Progress tab's arithmetic, over the Stories groups and the operator
 * records together.
 *
 * `words` is `null` only when the WIRE carries no word counts at all, never
 * because the reader has read nothing. Zero stories read over an index that
 * counts words is 0 words, a true number, and the old rule (any READ story
 * with a count) printed "Not counted yet" on a fresh browser against a backend
 * that sends `wordCount` on all 1,797 scripted stories.
 */
export function progressSummary(index: LibIndex, progress: StoryProgress, gameRead: ReadonlySet<string>): IProgressSummary {
    let read = 0;
    let total = 0;
    let groupsDone = 0;
    let groups = 0;
    let words = 0;
    let counted = typeof index.totals?.words === "number";
    for (const group of index.groups) {
        const fraction = readFraction(group.stories, progress, gameRead);
        read += fraction.read;
        total += fraction.total;
        if (group.category !== "record") {
            groups += 1;
            if (fraction.done) groupsDone += 1;
        }
        for (const story of group.stories) {
            if (typeof story.wordCount !== "number") continue;
            counted = true;
            if (isStoryRead(progress, gameRead, story.id)) words += story.wordCount;
        }
    }
    return { read, total, fraction: total > 0 ? read / total : 0, groupsDone, groups, words: counted ? words : null };
}

/** "N of M chapters finished" for one sub-tab. */
export function finishedGroups(groups: readonly LibGroup[], progress: StoryProgress, gameRead: ReadonlySet<string>): { done: number; total: number } {
    let done = 0;
    for (const group of groups) if (readFraction(group.stories, progress, gameRead).done) done += 1;
    return { done, total: groups.length };
}

/** Where a group sits between untouched and finished. `done` is {@link IReadFraction.done}, so a group with nothing readable in it is UNREAD rather than finished: 0 of 0 is not an achievement. */
export type ReadState = "unread" | "progress" | "done";

/** The read-state filter, `any` first because it is the default and the one that hides nothing. */
export type ReadFilter = "any" | ReadState;

export const READ_FILTERS: readonly ReadFilter[] = ["any", "unread", "progress", "done"] as const;

export function readStateOf(fraction: IReadFraction): ReadState {
    if (fraction.done) return "done";
    return fraction.read === 0 ? "unread" : "progress";
}

export function matchesReadState(filter: ReadFilter, fraction: IReadFraction): boolean {
    return filter === "any" || readStateOf(fraction) === filter;
}

/** The orders the browse surface offers. `default` is the game's own shelf order and is the only one that keeps the sections. */
export type LibrarySort = "default" | "newest" | "oldest" | "mostRead" | "leastRead" | "title";

export const LIBRARY_SORTS: readonly LibrarySort[] = ["default", "newest", "oldest", "mostRead", "leastRead", "title"] as const;

/** The share of a group that is read, as a number in [0, 1]. A group with nothing readable in it is 0, never NaN. */
function readShare(fraction: IReadFraction): number {
    return fraction.total > 0 ? fraction.read / fraction.total : 0;
}

/**
 * UNDATED GROUPS SORT LAST IN BOTH DIRECTIONS, and that is a trade rather than
 * a derivation. `story_review_table` carries `startTime: -1` on all 17 mainline
 * chapters and on 13 side groups `activity_table` has no row for, so a plain
 * ascending sort would open "Oldest release" on the whole mainline, which the
 * table does not claim is oldest: it claims nothing. Ranking the undated behind
 * the dated in BOTH directions says "these have no date" in the one way a
 * single column can, and among themselves they keep id order, which is reading
 * order for `main_0`..`main_16` under a numeric collator.
 */
function datedRank(group: Pick<LibGroup, "startTime">): number {
    return group.startTime > 0 ? 0 : 1;
}

/**
 * The flattened browse order. Every mode but `default` returns ONE list,
 * because a sort the reader cannot see is not a sort: leaving the shelves in
 * place would order 14 sections internally and leave the page reading by shelf.
 *
 * `fractionOf` is passed in rather than recomputed so the caller's one pass
 * over the index serves the filter and the sort alike.
 */
export function sortLibrary(groups: readonly LibGroup[], sort: LibrarySort, fractionOf: (group: LibGroup) => IReadFraction, collator: Intl.Collator): LibGroup[] {
    const out = [...groups];
    const byName = (a: LibGroup, b: LibGroup): number => collator.compare(a.name, b.name) || collator.compare(a.id, b.id);
    switch (sort) {
        case "default":
            return out;
        case "title":
            return out.sort(byName);
        case "newest":
            return out.sort((a, b) => datedRank(a) - datedRank(b) || b.startTime - a.startTime || collator.compare(a.id, b.id));
        case "oldest":
            return out.sort((a, b) => datedRank(a) - datedRank(b) || a.startTime - b.startTime || collator.compare(a.id, b.id));
        case "mostRead":
            return out.sort((a, b) => readShare(fractionOf(b)) - readShare(fractionOf(a)) || fractionOf(b).read - fractionOf(a).read || byName(a, b));
        case "leastRead":
            return out.sort((a, b) => readShare(fractionOf(a)) - readShare(fractionOf(b)) || fractionOf(a).read - fractionOf(b).read || byName(a, b));
    }
}

/** The three phases `avgTag` carries, in the order a chapter plays them. 699 Interlude, 589 Before Operation and 599 After Operation over the 1,887 EN stories; nothing else appears. */
export type StoryPhase = "before" | "after" | "interlude";

export function phaseOf(entry: Pick<LibEntry, "avgTag">): StoryPhase | null {
    switch (entry.avgTag) {
        case "Before Operation":
            return "before";
        case "After Operation":
            return "after";
        case "Interlude":
            return "interlude";
        default:
            return null;
    }
}

export interface IOperationSegment {
    phase: StoryPhase | null;
    entry: LibEntry;
}

/** One row of a chapter sheet: an operation the reader plays once, not a script the table stores twice. */
export interface IOperationRow {
    /** The first entry's id, which is unique within the group. */
    key: string;
    /** The operation code the row is known by, or `null` on the 494 stories the table gives no code. */
    code: string | null;
    title: string;
    /** Two segments on a merged operation, one on everything else. */
    segments: IOperationSegment[];
}

/**
 * A CHAPTER IS A LIST OF OPERATIONS, NOT A LIST OF SCRIPTS. `1-1 Isolated
 * Island` is one battle the reader plays once, and the table stores it as two
 * rows, `_beg` and `_end`, with the SAME code and the same name; printed one
 * per row the sheet says "Isolated Island" twice and a 39-entry chapter reads
 * as 39 unrelated things.
 *
 * The merge is ADJACENCY plus a shared code, never a code index. Over the 451
 * EN groups 525 codes repeat, and 3 of them do not repeat adjacently: the
 * `act19side` interludes `DV-S-1` and `DV-S-2` sit between their operation's
 * two halves, and `main_15` gives four entries the code `15-17` (Before, two
 * Interludes, After). A code index would pull those halves together across the
 * interlude that plays between them and break the one thing the reader is owed,
 * the ORDER. Those three stay as separate rows, which is what the table says
 * happens. 523 of the 525 merge.
 *
 * The merged row takes the FIRST entry's name: over all 525 repeated codes the
 * two names are identical, 0 disagreements, so there is nothing to choose.
 */
export function groupOperations(stories: readonly LibEntry[]): IOperationRow[] {
    const order = sortedStories(stories);
    const rows: IOperationRow[] = [];
    for (let i = 0; i < order.length; i += 1) {
        const entry = order[i];
        if (!entry) continue;
        const next = order[i + 1];
        const code = entry.code?.trim() ?? "";
        if (next && code !== "" && next.code?.trim() === code && phaseOf(entry) === "before" && phaseOf(next) === "after") {
            rows.push({
                key: entry.id,
                code,
                title: entry.name,
                segments: [
                    { phase: "before", entry },
                    { phase: "after", entry: next },
                ],
            });
            i += 1;
            continue;
        }
        rows.push({ key: entry.id, code: code === "" ? null : code, title: entry.name, segments: [{ phase: phaseOf(entry), entry }] });
    }
    return rows;
}

/** Which story a chapter sheet's primary action opens: the first unread, or the last one in the chapter once everything is read. `null` when nothing in the chapter has a script. */
export function pickContinue(stories: readonly LibEntry[], progress: StoryProgress, gameRead: ReadonlySet<string>): { entry: LibEntry; resumed: boolean } | null {
    const readable = sortedStories(stories).filter((s) => s.hasScript);
    const first = readable[0];
    if (!first) return null;
    const unread = readable.find((s) => !isStoryRead(progress, gameRead, s.id));
    if (unread) return { entry: unread, resumed: unread !== first };
    const last = readable[readable.length - 1];
    return last ? { entry: last, resumed: true } : null;
}
