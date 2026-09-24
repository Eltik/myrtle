/**
 * The two reading orders the data can DERIVE, the arithmetic behind them, and
 * what one row draws.
 *
 * Release order and storyline order are built here, pure, so the component is a
 * renderer. The third order the reference site offers, a hand-curated timeline
 * of when each story happens in-world, is NOT derivable: no field in
 * `story_review_table` dates a story inside Terra, and inventing one would be a
 * curation we cannot check. Its tab renders an explanation and no list.
 *
 * THE MAINLINE IS PARTLY DATED NOW, which retracts what the head of this file
 * used to say. Measured against :3060 on 2026-09-24 over 451 groups: 70 of the
 * 87 story groups (the 50 side and 20 vignette ones) carry a real `startTime`,
 * and so do 5 of the 17 mainline chapters, `main_10` 1666180800 through
 * `main_14` 1730394000, because the backend now reads
 * `zone_table.MainlineAdditionInfo[].ZoneOpenTime` where the group's own row
 * writes -1. The other 12 mainline zones write -1 there too, so those 12 are
 * the only groups release order cannot place; the 5 dated ones interleave with
 * the events like anything else. The 364 record sets carry `0` and are not
 * listed in any order.
 */

import { isStoryRead, type StoryProgress } from "#/lib/story/progress";
import type { IPlateSource } from "./art";
import { plateSource } from "./art";
import { chapterNumberOf } from "./chapters";
import { groupCode, type LibEntry, type LibGroup, sortedStories } from "./derive";
import { kindOf, type StoryKind } from "./sections";

export type ReadingOrderMode = "release" | "storyline" | "timeline";

export const READING_ORDER_MODES: readonly ReadingOrderMode[] = ["release", "storyline", "timeline"] as const;

export interface IOrderRow {
    group: LibGroup;
    /** Release time in seconds, or `null` when the wire carries none for this group. */
    time: number | null;
}

/**
 * A block of rows under one heading. `kind` is the identity the component keys
 * its wording on, `title` is DATA (an arc's name out of `zone.chapterName`) and
 * is never a translatable string.
 */
export interface IOrderSection {
    key: string;
    kind: "undated" | "dated" | "arc" | "unlinked";
    title: string | null;
    rows: IOrderRow[];
}

/** A group's release time, or `null`: the mainline's `-1` and a record set's `0` are both "no date". */
export function releaseTime(group: Pick<LibGroup, "startTime">): number | null {
    return group.startTime > 0 ? group.startTime : null;
}

/**
 * How the mainline splits on the date question, counted rather than written
 * down, because the note under the shelved block quotes both halves and a
 * constant would rot the first time the backend dates a sixth zone.
 */
export interface IMainlineDates {
    /** Chapters whose zone writes -1 and which therefore stand outside the sequence. */
    undated: number;
    /** Chapters `zone_table` opens on a real date, which sit among the events. */
    dated: number;
}

export function mainlineDateSplit(groups: readonly LibGroup[]): IMainlineDates {
    let undated = 0;
    let dated = 0;
    for (const group of groups) {
        if (group.category !== "main") continue;
        if (releaseTime(group) === null) undated += 1;
        else dated += 1;
    }
    return { undated, dated };
}

/**
 * WHAT ONE ROW DRAWS, as a pure choice, so the branch that fires is pinned in a
 * test rather than read off a screenshot.
 *
 * The picture is `plateSource`, the library's one answer for every surface that
 * draws a chapter's art: the authored key visual first, the Archives cover
 * second, the typographic code third. The row does NOT draw `iconUrl`, the
 * 56x68 chapter deco: it is monochrome line art authored for a 108 px game
 * chrome and at the 18 px this row could give it it reads as a smudge, so the
 * number carries the chapter instead.
 */
export interface IOrderRowModel {
    plate: IPlateSource;
    /** The in-game label the badge prints, the same one a Browse card wears. */
    kind: StoryKind;
    /** The chapter number on a mainline group, `null` on everything else. */
    chapter: number | null;
    /** The typographic fallback printed when {@link IOrderRowModel.plate} is `none`. */
    code: string;
}

export function orderRowModel(group: LibGroup): IOrderRowModel {
    return { plate: plateSource(group), kind: kindOf(group), chapter: chapterNumberOf(group), code: groupCode(group) };
}

function byChapter(a: IOrderRow, b: IOrderRow): number {
    const left = chapterNumberOf(a.group);
    const right = chapterNumberOf(b.group);
    if (left !== null && right !== null) return left - right;
    return a.group.id.localeCompare(b.group.id, undefined, { numeric: true });
}

function byTime(a: IOrderRow, b: IOrderRow): number {
    return (a.time ?? 0) - (b.time ?? 0) || a.group.id.localeCompare(b.group.id, undefined, { numeric: true });
}

function rowsOf(groups: readonly LibGroup[]): IOrderRow[] {
    return groups.map((group) => ({ group, time: releaseTime(group) }));
}

/**
 * RELEASE ORDER: every group that carries a date, oldest first, under the
 * chapters that carry none.
 *
 * The reference interleaves the chapters with the events, and 5 of the 17 now
 * do interleave, because `zone_table` opens their zones on a real date. The
 * other 12 zones write -1 and no other field dates them, so sliding those into
 * the sequence would be a claim the data does not make; they stand first, in
 * chapter order, under a heading that says exactly how many are shelved and how
 * many were placed. An empty block is omitted rather than drawn as a heading
 * over nothing.
 */
export function releaseOrder(groups: readonly LibGroup[]): IOrderSection[] {
    const rows = rowsOf(groups.filter((group) => group.category !== "record"));
    const undated = rows.filter((row) => row.time === null).sort(byChapter);
    const dated = rows.filter((row) => row.time !== null).sort(byTime);
    const sections: IOrderSection[] = [];
    if (undated.length > 0) sections.push({ key: "undated", kind: "undated", title: null, rows: undated });
    if (dated.length > 0) sections.push({ key: "dated", kind: "dated", title: null, rows: dated });
    return sections;
}

/**
 * STORYLINE ORDER: the Main Theme in chapter order, cut into the arcs the game
 * names itself, then everything else.
 *
 * The arcs are real data: `zone.chapterName` is on all 17 chapters and folds
 * them into 4 arcs (Hour of An Awakening, Shatter of A Vision, Shadow of A
 * Dying Sun, Nexus Point of Future), ordered by the lowest chapter each holds.
 *
 * What is NOT in the data is the LINK from a side story or an Intermezzo to the
 * chapter it follows. The index gives a group an id, a name, an `actType`, an
 * `entryType`, a `displayType` and a date, and none of them names a chapter, so
 * the reference's "after Episode 8" placement cannot be derived. Those 70
 * groups therefore fall into one trailing block in RELEASE order, and the tab
 * says that is what it is.
 */
export function storylineOrder(groups: readonly LibGroup[]): IOrderSection[] {
    const story = groups.filter((group) => group.category !== "record");
    const mains = rowsOf(story.filter((group) => group.category === "main")).sort(byChapter);

    const arcs = new Map<string, IOrderRow[]>();
    for (const row of mains) {
        const arc = row.group.zone?.chapterName?.trim() || row.group.name;
        const bucket = arcs.get(arc);
        if (bucket) bucket.push(row);
        else arcs.set(arc, [row]);
    }

    const sections: IOrderSection[] = [...arcs].map(([title, rows]) => ({ key: `arc:${title}`, kind: "arc" as const, title, rows }));

    const rest = rowsOf(story.filter((group) => group.category !== "main")).sort(byTime);
    if (rest.length > 0) sections.push({ key: "unlinked", kind: "unlinked", title: null, rows: rest });
    return sections;
}

export function readingOrder(mode: ReadingOrderMode, groups: readonly LibGroup[]): IOrderSection[] {
    if (mode === "release") return releaseOrder(groups);
    if (mode === "storyline") return storylineOrder(groups);
    return [];
}

/** Rows over every section, for the "N chapters" line above the list. */
export function orderTotal(sections: readonly IOrderSection[]): number {
    return sections.reduce((total, section) => total + section.rows.length, 0);
}

/** Where an order says to go next: the first unread story of the first chapter that is not finished, in the order's own sequence. */
export interface INextInOrder {
    group: LibGroup;
    entry: LibEntry;
}

/**
 * THE POINTER IS THE ORDER'S OWN FIRST UNREAD, not the library's.
 *
 * Release order and storyline order disagree about what comes next by design,
 * so the pointer is computed over the sections as they were built rather than
 * off a global rule: in storyline order it walks the Main Theme's four arcs
 * before it reaches the side material, and in release order it walks the
 * shelved chapters first. `null` means everything in the order is read, which
 * is a finished library rather than a missing answer.
 */
export function nextInOrder(sections: readonly IOrderSection[], progress: StoryProgress, gameRead: ReadonlySet<string>): INextInOrder | null {
    for (const section of sections) {
        for (const row of section.rows) {
            for (const entry of sortedStories(row.group.stories)) {
                if (!entry.hasScript || isStoryRead(progress, gameRead, entry.id)) continue;
                return { group: row.group, entry };
            }
        }
    }
    return null;
}
