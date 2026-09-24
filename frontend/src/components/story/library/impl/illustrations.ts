/**
 * Pure derivations for the Illustrations tab: which category a card belongs
 * under, the counts a card prints, and the rows a group's panel lists.
 *
 * `illustrationCount` and `spriteCount` ride on `StoryGroup` and on
 * `OperatorRecordGroup`, but a backend older than those fields is still
 * answering this page, so they are read as OPTIONAL everywhere and an absent
 * count renders "?" rather than 0. Same rule `derive.ts` already applies to
 * `wordCount`.
 */

import type { IllustrationItem } from "#/types/generated/IllustrationItem";
import type { SpriteItem } from "#/types/generated/SpriteItem";
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import { type IPlateSource, plateSource } from "./art";
import { chapterNumberOf } from "./chapters";
import { buildTabs, groupCode, type LibGroup, type LibRecord, type StoriesTabKey } from "./derive";
import { kindOf, type StoryKind } from "./sections";

/** The two sub-tabs. Each lists the same groups and differs only in which count it prints and which rows the panel opens on. */
export const ART_KINDS = ["illustrations", "sprites"] as const;
export type ArtKind = (typeof ART_KINDS)[number];

/** The Stories tab's own categories, plus the operator records the Operators tab holds. */
export type ArtCategory = StoriesTabKey | "record";

/**
 * WHAT A CARD DRAWS. A story group answers `plateSource`, the library's one
 * answer for every surface that prints a chapter's art, so this grid, the
 * Browse tickets and the Reading Order rows can never disagree about which
 * picture a chapter is. An operator record has no chapter art at all and keeps
 * its 5-star AVATAR, which is a different kind of picture and is named so.
 */
export type IArtPlate = IPlateSource | { kind: "avatar"; url: string };

/** A card in the grid: a story group, or one operator's records addressed by `charId`. */
export interface IArtGroup {
    /** A `StoryGroup.id`, or an operator `charId`. Both address the SAME endpoint. */
    id: string;
    name: string;
    /** The picture the card and the panel header draw; an asset-index path for `asset()`. */
    art: IArtPlate;
    /** The typographic fallback when {@link IArtGroup.art} is `none`: the Stories tab's own short code. */
    code: string;
    /** The badge the card wears, the same `StoryKind` a Browse ticket prints. */
    kind: StoryKind;
    /** The chapter number on a mainline group, `null` on everything else. */
    chapter: number | null;
    illustrationCount?: number;
    spriteCount?: number;
}

/**
 * The categories that have at least one card, story groups in the Stories
 * tab's own order and the operator records last. A category with no groups is
 * dropped rather than rendered empty.
 */
export function artTabs(groups: readonly LibGroup[], records: readonly LibRecord[]): { key: ArtCategory; groups: IArtGroup[] }[] {
    const out: { key: ArtCategory; groups: IArtGroup[] }[] = buildTabs(groups).map((entry) => ({
        key: entry.key as ArtCategory,
        groups: entry.groups.map((group) => ({ id: group.id, name: group.name, art: plateSource(group), code: groupCode(group), kind: kindOf(group), chapter: chapterNumberOf(group), illustrationCount: group.illustrationCount, spriteCount: group.spriteCount })),
    }));
    if (records.length > 0) {
        out.push({
            key: "record",
            groups: records.map((record) => ({
                id: record.charId,
                name: record.name,
                art: record.avatarUrl ? ({ kind: "avatar", url: record.avatarUrl } as const) : ({ kind: "none" } as const),
                code: record.name,
                kind: "record" as const,
                chapter: null,
                illustrationCount: record.illustrationCount,
                spriteCount: record.spriteCount,
            })),
        });
    }
    return out;
}

/** The count one card prints for the open sub-tab, or `null` when the wire carries none. */
export function countOf(group: IArtGroup, kind: ArtKind): number | null {
    const value = kind === "illustrations" ? group.illustrationCount : group.spriteCount;
    return typeof value === "number" ? value : null;
}

/** The sub-tab's total over one category, summed across the cards that carry a count, or `null` when none do. */
export function categoryTotal(groups: readonly IArtGroup[], kind: ArtKind): number | null {
    let sum = 0;
    let seen = false;
    for (const group of groups) {
        const value = countOf(group, kind);
        if (value === null) continue;
        sum += value;
        seen = true;
    }
    return seen ? sum : null;
}

/**
 * THE PANEL SPLITS WHAT THE CARD SUMS, and the card says "pieces" because the
 * index cannot split it.
 *
 * The reference site credits the Prologue with 13 illustrations and that number
 * is the CG list exactly: `main_0` answers 4 backgrounds and 13 images, and
 * `illustrationCount` is 17, the two added together. The index carries only the
 * sum, so a card cannot print 13 without fetching the group it is not allowed
 * to fetch; it prints 17 as "17 pieces", a word that is true of both halves, and
 * the panel that does fetch shows the split as Backgrounds 4 / CGs 13 / Sprites
 * 11. Calling the sum "illustrations" was the defect: it read as the reference's
 * number and was 4 too high on every card.
 */
export const ART_PANES = ["backgrounds", "cgs", "sprites"] as const;
export type ArtPane = (typeof ART_PANES)[number];

/** Which pane a sub-tab opens on: the sprite sub-tab lands on its own pane, the artwork one on the backgrounds. */
export function defaultPane(kind: ArtKind): ArtPane {
    return kind === "sprites" ? "sprites" : "backgrounds";
}

function dedupe(items: readonly IllustrationItem[], seen: Set<string>): IllustrationItem[] {
    const out: IllustrationItem[] = [];
    for (const item of items) {
        if (seen.has(item.name)) continue;
        seen.add(item.name);
        out.push(item);
    }
    return out;
}

/** The backgrounds, in the backend's first-appearance order, each name once. */
export function backgroundRows(data: StoryIllustrations | null | undefined): IllustrationItem[] {
    if (!data) return [];
    return dedupe(data.backgrounds, new Set<string>());
}

/**
 * The CGs, each name once, MINUS any name the backgrounds already listed. The
 * two panes therefore add up to `illustrationCount` exactly, which is the whole
 * reason the card can print the sum and the panel the split.
 */
export function cgRows(data: StoryIllustrations | null | undefined): IllustrationItem[] {
    if (!data) return [];
    return dedupe(data.images, new Set(data.backgrounds.map((item) => item.name)));
}

/**
 * Both halves, backgrounds first: what the card's "N pieces" counts. A name
 * that appears in both is listed ONCE, under the half that saw it first.
 */
export function illustrationRows(data: StoryIllustrations | null | undefined): IllustrationItem[] {
    return [...backgroundRows(data), ...cgRows(data)];
}

/** The Sprites panel's rows, one per sprite base, in wire order. */
export function spriteRows(data: StoryIllustrations | null | undefined): SpriteItem[] {
    if (!data) return [];
    const seen = new Set<string>();
    const out: SpriteItem[] = [];
    for (const item of data.sprites) {
        if (seen.has(item.base)) continue;
        seen.add(item.base);
        out.push(item);
    }
    return out;
}

/** The rows one pane lists. One call site, so the panel has no branch of its own. */
export function paneRows(data: StoryIllustrations | null | undefined, pane: ArtPane): (IllustrationItem | SpriteItem)[] {
    if (pane === "backgrounds") return backgroundRows(data);
    if (pane === "cgs") return cgRows(data);
    return spriteRows(data);
}

/** The count beside each of the panel's three sub-tabs. */
export function paneCounts(data: StoryIllustrations | null | undefined): Record<ArtPane, number> {
    return { backgrounds: backgroundRows(data).length, cgs: cgRows(data).length, sprites: spriteRows(data).length };
}

export interface IArtSummary {
    rows: number;
    /** Rows whose asset resolved to no file: the "not extracted" tiles. */
    missing: number;
    /** Distinct story ids the rows are used in, over the whole panel. */
    stories: number;
}

/** What the panel's header line says, over whichever rows it is showing. */
export function artSummary(rows: readonly (IllustrationItem | SpriteItem)[]): IArtSummary {
    let missing = 0;
    const stories = new Set<string>();
    for (const row of rows) {
        const url = "url" in row ? row.url : row.bodyUrl;
        if (!url) missing += 1;
        for (const id of row.storyIds) stories.add(id);
    }
    return { rows: rows.length, missing, stories: stories.size };
}
