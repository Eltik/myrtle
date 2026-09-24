/**
 * What a reader actually knows a chapter by, and what each jump chip says.
 *
 * NOBODY KNOWS THE ACT NAMES. A reader of the main story knows "chapter 8",
 * not "Roaring Flare", and knows "Chapters 0 to 3" long before "Hour of an
 * Awakening"; the page printed only the names, so the section furniture and
 * the card spec lines named things the reader had no way to place. Everything
 * here is the derivation of the NUMBER, and it is pure so the parse, the
 * fallback and the label are testable without a DOM.
 *
 * `StoryGroup.chapterNumber`, `Storyline.chapterRange` and
 * `StorylineArc.chapterRange` now arrive on the wire. Every rule here still
 * reads them DEFENSIVELY and derives the same answer from the group ids when
 * they are missing, because an older backend, a filtered section and a shelf
 * the table does not span all reach these functions with nothing to read.
 */

import { BookMarkedIcon, CompassIcon, DramaIcon, FeatherIcon, FlameIcon, LeafIcon, MoonStarIcon, MountainSnowIcon, ShipIcon, SnowflakeIcon, SwordsIcon, WavesIcon } from "lucide-react";
import type { LibGroup } from "./derive";
import type { FilterKey } from "./sections";

/** A group's chapter number when it has one, whatever the wire sends. */
export function chapterNumberOf(group: Pick<LibGroup, "id" | "category"> & { chapterNumber?: number }): number | null {
    // A MISSING field, never a falsy one: chapter 0 is the Prologue and the
    // most common mainline number on the page, so `group.chapterNumber || ...`
    // would silently re-parse every one of the 17 mainline groups.
    if (typeof group.chapterNumber === "number" && Number.isFinite(group.chapterNumber)) return group.chapterNumber;
    if (group.category !== "main") return null;
    return parseMainOrdinal(group.id);
}

/** `main_8` -> 8. Anything else, including `main_` and `main_x`, is `null`. */
export function parseMainOrdinal(id: string): number | null {
    const match = /^main_(\d+)$/.exec(id);
    if (!match?.[1]) return null;
    const n = Number.parseInt(match[1], 10);
    return Number.isFinite(n) ? n : null;
}

/** A group's release year, or `null` on the 17 mainline groups and anything else the table does not date (`startTime` is -1 there). */
export function releaseYear(group: Pick<LibGroup, "startTime">): number | null {
    if (!(group.startTime > 0)) return null;
    return new Date(group.startTime * 1000).getUTCFullYear();
}

/** The spec line under a card or row title: a chapter number for the mainline, the operation code plus the release year for everything else. */
export type ISpecModel = { kind: "chapter"; chapter: number; entries: number } | { kind: "code"; code: string; year: number | null; entries: number };

export function specModel(group: Pick<LibGroup, "id" | "category" | "zone" | "stories" | "startTime"> & { chapterNumber?: number }, code: string): ISpecModel {
    const chapter = chapterNumberOf(group);
    if (chapter !== null) return { kind: "chapter", chapter, entries: group.stories.length };
    return { kind: "code", code, year: releaseYear(group), entries: group.stories.length };
}

export interface IChapterRange {
    from: number;
    to: number;
}

/**
 * WHAT A SECTION'S CHAPTER NUMBERS ARE ALLOWED TO SAY ABOUT IT.
 *
 * Two different claims, and the page used to make only the first: a section IS
 * a run of main story chapters, or a section merely HOLDS some. The `mainLine`
 * shelf and its four arcs are the first; every themed `ssLine_*` shelf is at
 * most the second.
 *
 * The defect this replaces: `Storyline.chapterRange` was taken as an override
 * wherever the wire sent one, so the 6 themed shelves that carry a range wore a
 * mainline label. "The Blessed" headed "Main story · Chapter 15", "Wildfire"
 * chipped "Chapter 1 / WILDFIRE" and "The Ark" chipped "Chapters 7 to 14 / THE
 * ARK". A themed shelf's primary label is its NAME.
 *
 * `primary` is the run the section IS, and it is non-null only where the
 * numbered groups are a MAJORITY of the section, because a shelf that holds
 * eight events and one chapter is not a chapter run and heading it "Chapter 5"
 * would be a lie about the other eight. The threshold is not a guess, it is
 * where the EN data separates. The four mainline arcs are 4 of 4, 5 of 6, 6 of
 * 9 and 2 of 3 numbered, so the narrowest is 66.7%; every one of the thirteen
 * `ssLine_*` shelves is 0 of n after the lowest-sort assignment, because
 * `mainLine` sorts 0 and takes all 17 mainline chapters before any of them.
 * A strict rule (every group numbered) was measured first and gave a range to
 * ONE arc of four, which is why it is not the rule shipped.
 *
 * `includes` is the weaker claim, and it is read off the section's OWN groups
 * rather than off `Storyline.chapterRange`: that field is the min and max over
 * everything the shelf LISTS, and a shelf that lists `main_15` the `mainLine`
 * shelf already owns holds no chapter at all here. On EN today `includes` is
 * null on all 13 themed shelves, so the field is pinned by test rather than by
 * a shipped row; a server whose shelf order hands a themed shelf a chapter
 * gets the muted secondary instead of a false heading.
 *
 * The override is still honoured on a `primary`, where it is the wire's own
 * wording for the same run (all 4 EN arcs agree with the derivation exactly).
 */
export function sectionChapters(groups: readonly (Pick<LibGroup, "id" | "category"> & { chapterNumber?: number })[], override?: IChapterRange): ISectionChapters {
    const found = groups.map(chapterNumberOf).filter((n): n is number => n !== null);
    if (found.length === 0) return { primary: null, includes: null };
    const span = { from: Math.min(...found), to: Math.max(...found) };
    if (found.length * 2 <= groups.length) return { primary: null, includes: span };
    if (override && Number.isFinite(override.from) && Number.isFinite(override.to)) return { primary: override, includes: null };
    return { primary: span, includes: null };
}

/** The two claims a section's chapter numbers can make. At most one is ever non-null. */
export interface ISectionChapters {
    /** The run the section IS, printed as its primary label over the section's name. */
    primary: IChapterRange | null;
    /** The mainline chapters the section HOLDS while not being a run of them, printed as a muted secondary under its name. */
    includes: IChapterRange | null;
}

/** The twelve glyphs a shelf without an icon of its own draws from. */
export const SECTION_GLYPHS = [BookMarkedIcon, CompassIcon, DramaIcon, FeatherIcon, FlameIcon, LeafIcon, MoonStarIcon, MountainSnowIcon, ShipIcon, SnowflakeIcon, SwordsIcon, WavesIcon] as const;

/** djb2 over the section id, the same hash the reader uses for speaker plates. Stable across reloads and across locales, because the ID is not translated. */
export function hashKey(key: string): number {
    let h = 5381;
    for (let i = 0; i < key.length; i += 1) h = (h * 33) ^ key.charCodeAt(i);
    return h >>> 0;
}

/** Which of {@link SECTION_GLYPHS} a section draws, when the wire sends it no `iconUrl`. */
export function glyphIndexFor(id: string): number {
    return hashKey(id) % SECTION_GLYPHS.length;
}

/** What a jump chip and a section heading print. The NAME is the primary line on both and the chapter range is the secondary, which is the reverse of what shipped first. */
export interface IChipModel {
    id: string;
    /** `null` on a section the chapter numbers do not describe, which is every themed shelf; the caller then prints {@link IChipModel.name} alone. */
    range: IChapterRange | null;
    /** The mainline chapters a NON-run section holds, for the muted "includes" line. Null wherever {@link IChipModel.range} is set. */
    includes: IChapterRange | null;
    /** The section's own name, translated where the data does not name it. */
    name: string;
    count: number;
    /** The arc's or the shelf's `iconUrl` when the wire sends one. */
    iconUrl?: string;
    /** True when {@link IChipModel.iconUrl} is an ARC icon, a 184x52 banner sized by height rather than a square glyph. */
    iconWide?: boolean;
    /** True when {@link IChipModel.iconUrl} is the shelf's 108x108 logo, which takes the larger square slot. */
    iconLogo?: boolean;
    /** The index into {@link SECTION_GLYPHS} used when there is no `iconUrl`. */
    glyph: number;
    /** The one pill every group in the section answers to, which is the lead-in word of the heading's muted line, or `null` where the section mixes kinds. */
    filter?: Exclude<FilterKey, "all"> | null;
}

/** "Chapters 0 to 3", or "Chapter 9" when a section holds one. The caller owns the wording; this owns which of the two it is. */
export function rangeIsSingle(range: IChapterRange): boolean {
    return range.from === range.to;
}

/**
 * The act ordinal a section's name carries, and the name with it taken off.
 *
 * THE ORDINAL IS IN THE ART, NOT IN THE STRING. All four EN arcs are named by
 * `MainlineSplitData.SubName` alone ("HOUR OF AN AWAKENING", "SHATTER OF A
 * VISION", "SHADOW OF A DYING SUN", "NEXUS POINT OF FUTURE"), and the ordinal
 * a reader asks for is typeset into the 184x52 banner beside them, which reads
 * AD INITIUM, ACT I, ACT II, ACT III. So this returns `null` on 4 of 4 EN arcs
 * and the chip prints no ordinal of its own: printing one would either repeat
 * the banner or invent a numbering the game does not use.
 *
 * It exists for the server that one day names an arc "Act I: Shatter of a
 * Vision", which is the shape the second reader asked for. A bare "Act 1" with
 * nothing after it splits to an EMPTY name, and the chip then prints the
 * ordinal line alone rather than printing "Act 1" twice.
 */
export function splitActOrdinal(name: string): { ordinal: string | null; name: string } {
    const match = /^act\s+([ivxlcdm]+|\d{1,3})\b[\s:.·-]*(.*)$/i.exec(name.trim());
    if (!match?.[1]) return { ordinal: null, name };
    return { ordinal: match[1].toUpperCase(), name: (match[2] ?? "").trim() };
}

/**
 * WHAT ONE JUMP CHIP PRINTS, as a structure rather than as words.
 *
 * A chip has two forms and the bar is mostly the first. COLLAPSED is the
 * inactive form: one line, and on a mainline arc that line is the compact
 * chapter range alone, because the arc's banner IS its name and printing the
 * name beside a picture of the name cost 255.0 px a chip. EXPANDED is the
 * active one: the name over the range, which is the form the reader is given
 * once a section is the one they are in, and the form the tooltip shows on
 * hover for every collapsed chip.
 *
 * A themed shelf keeps its name in both forms, because its 108x108 logo is a
 * monogram (RL, UR, LA) and names nothing. Its `includes` line, the weaker
 * "holds these chapters" claim, is printed on the expanded chip only: it is
 * true of no EN shelf today and it is never worth a line in a bar that
 * overflows.
 */
export interface IChipLines {
    /** The name line. `null` only where the arc banner already prints it, which is a collapsed mainline chip. */
    name: string | null;
    /** The chapter run on the mono line, compact when collapsed. */
    range: IChapterRange | null;
    /** True when {@link IChipLines.range} is the weaker "holds these chapters" claim rather than the run the section IS. */
    holds: boolean;
    /** The act ordinal off the section's own name, printed before the range on an expanded chip. `null` on every EN arc; see {@link splitActOrdinal}. */
    ordinal: string | null;
}

export function chipLines(chip: Pick<IChipModel, "name" | "range" | "includes" | "iconWide">, active: boolean): IChipLines {
    const split = splitActOrdinal(chip.name);
    // The banner carries the name only where there IS a banner and the chip is
    // a chapter run: a themed shelf with `iconWide` would lose its name here.
    const bannerNames = chip.iconWide === true && chip.range !== null;
    const name = active || !bannerNames ? (split.name === "" ? null : split.name) : null;
    const range = active ? (chip.range ?? chip.includes) : chip.range;
    return { name, range, holds: range !== null && chip.range === null, ordinal: active && range !== null ? split.ordinal : null };
}
