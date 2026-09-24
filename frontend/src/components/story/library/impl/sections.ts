/**
 * How `/stories` is cut into sections, which pill a group answers to, and what
 * badge its card wears. Pure over its inputs, so the whole page's structure is
 * testable without a DOM.
 *
 * THE SECTIONS ARE THE GAME'S OWN. `stage_table.Storylines` is a list of 14
 * themed shelves on EN (`mainLine` "For Tomorrow" plus 13 `ssLine_*`), and the
 * backend resolves each shelf's locations to `StoryGroup.id`s through
 * `StorylineStorySets` (`docs/story-reader.md` section 2). 81 of the 87
 * non-record groups sit on at least one shelf and 17 sit on several, so the
 * shelves are NOT a partition: the page gives each group to the LOWEST-sorted
 * shelf that lists it and drops the repeats, which is why a section can be
 * shorter here than its `groupIds` is on the wire.
 *
 * A backend older than the `storylines` field sends none, and then
 * {@link fallbackSections} cuts the page by main-chapter order, then by event
 * release year, then records. The page says which cut it is showing.
 */

import { iconIsLogo, iconIsWide, sectionIconSource } from "./art";
import type { LibGroup, LibIndex, LibRecord, LibStoryline } from "./derive";

/** The in-game label a card's badge prints. */
export type StoryKind = "main" | "event" | "intermezzo" | "vignette" | "record";

/** The filter pills, left to right. */
export type FilterKey = "all" | "main" | "events" | "side" | "records";

export const FILTER_ORDER: readonly FilterKey[] = ["all", "main", "events", "side", "records"] as const;

export type ViewMode = "grid" | "list";

/**
 * The badge a group wears.
 *
 * `side` splits on the game's own Archives shelf: `displayType BRANCHLINE` is
 * an INTERMEZZO (3 on EN: Darknights Memoir, A Walk in the Dust, Under Tides)
 * and everything else in `side` is an EVENT. The other 47 side groups are NOT
 * all `SIDESTORY`: 34 are, and 13 carry `displayType NONE` because
 * `activity_table` has no row for them, so testing for `SIDESTORY` would leave
 * those 13 with no pill at all.
 *
 * `is`, `reclamation` and `sideContent` are 0 groups on EN and read as events.
 */
export function kindOf(group: Pick<LibGroup, "category" | "displayType">): StoryKind {
    if (group.category === "record") return "record";
    if (group.category === "main") return "main";
    if (group.category === "vignette") return "vignette";
    if (group.displayType === "BRANCHLINE") return "intermezzo";
    return "event";
}

/** Whether a kind answers to a pill. "Side stories" is the vignettes plus the intermezzi, the two short-form shelves. */
export function matchesFilter(kind: StoryKind, filter: FilterKey): boolean {
    switch (filter) {
        case "all":
            return true;
        case "main":
            return kind === "main";
        case "events":
            return kind === "event";
        case "side":
            return kind === "vignette" || kind === "intermezzo";
        case "records":
            return kind === "record";
    }
}

export type SectionKind = "arc" | "storyline" | "other" | "records" | "year" | "main";

export interface ISection {
    /** Stable across renders and used as the jump-list anchor id. */
    id: string;
    title: string;
    kind: SectionKind;
    groups: LibGroup[];
    /** The operator records section only: 315 cards do not belong open. */
    collapsed: boolean;
    /** The shelf this section came off, or the section id when no shelf made it. It is what the glyph is hashed from, so it must not be a translated string. */
    lineId: string;
    /** The arc's own `StorylineArc.iconUrl` where it has one, else the shelf's `Storyline.iconUrl`; absent on a section the wire names no icon for, which then draws a lucide glyph. */
    iconUrl?: string;
    /** True when {@link ISection.iconUrl} is an ARC icon, which is a 184x52 wide banner and is sized by height rather than dropped into a square slot. */
    iconWide?: boolean;
    /** True when {@link ISection.iconUrl} is the shelf's 108x108 LOGO, which fills a larger square than the 44x36 abbreviation. */
    iconLogo?: boolean;
    /** `StorylineArc.chapterRange` on an arc and `Storyline.chapterRange` on a shelf. Absent leaves the majority-rule derivation over the section's own groups. */
    chapterRange?: { from: number; to: number };
}

/** The records section carries record cards, which are operators, not groups. */
export interface ISectionedLibrary {
    sections: ISection[];
    records: LibRecord[];
    /** True when the wire carried no `storylines` and {@link fallbackSections} cut the page. */
    fallback: boolean;
}

const OTHER_ID = "other-events";
export const RECORDS_ID = "operator-records";

/**
 * The page's sections in the game's own order: the mainline shelf's four arcs,
 * then the twelve side shelves, then the groups no shelf lists, then the
 * operator records.
 *
 * Each group lands ONCE, under the first shelf that lists it in `sort` order,
 * and within the mainline shelf under the first arc that lists it.
 */
export function buildSections(groups: readonly LibGroup[], storylines: readonly LibStoryline[]): ISection[] {
    const byId = new Map(groups.map((g) => [g.id, g]));
    const taken = new Set<string>();
    const sections: ISection[] = [];

    const take = (ids: readonly string[]): LibGroup[] => {
        const out: LibGroup[] = [];
        for (const id of ids) {
            if (taken.has(id)) continue;
            const group = byId.get(id);
            if (!group || group.category === "record") continue;
            taken.add(id);
            out.push(group);
        }
        return out;
    };

    for (const line of [...storylines].sort((a, b) => a.sort - b.sort || a.id.localeCompare(b.id))) {
        if (line.arcs.length > 0) {
            for (const arc of [...line.arcs].sort((a, b) => a.sort - b.sort)) {
                const picked = take(arc.groupIds);
                // The ARC's own icon beats the shelf's: `act_0`..`act_3` name the run
                // the heading is over, where the shelf glyph repeats on all four.
                const icon = sectionIconSource(arc.iconUrl, line.logoUrl, line.iconUrl);
                if (picked.length > 0)
                    sections.push({ id: `arc-${line.id}-${arc.sort}`, title: arc.name, kind: "arc", groups: picked, collapsed: false, lineId: line.id, iconUrl: icon.kind === "glyph" ? undefined : icon.url, iconWide: iconIsWide(icon), iconLogo: iconIsLogo(icon), chapterRange: arc.chapterRange ?? undefined });
            }
            // A shelf can list a group outside every arc; it still belongs to the shelf.
            const rest = take(line.groupIds);
            const icon = sectionIconSource(undefined, line.logoUrl, line.iconUrl);
            if (rest.length > 0) sections.push({ id: `line-${line.id}`, title: line.name, kind: "storyline", groups: rest, collapsed: false, lineId: line.id, iconUrl: icon.kind === "glyph" ? undefined : icon.url, iconLogo: iconIsLogo(icon), chapterRange: line.chapterRange });
            continue;
        }
        const picked = take(line.groupIds);
        const icon = sectionIconSource(undefined, line.logoUrl, line.iconUrl);
        if (picked.length > 0) sections.push({ id: `line-${line.id}`, title: line.name, kind: "storyline", groups: picked, collapsed: false, lineId: line.id, iconUrl: icon.kind === "glyph" ? undefined : icon.url, iconLogo: iconIsLogo(icon), chapterRange: line.chapterRange });
    }

    const rest = groups.filter((g) => g.category !== "record" && !taken.has(g.id)).sort((a, b) => b.startTime - a.startTime || a.id.localeCompare(b.id));
    if (rest.length > 0) sections.push({ id: OTHER_ID, title: "", kind: "other", groups: rest, collapsed: false, lineId: OTHER_ID });
    return sections;
}

/**
 * The cut used while the backend sends no `storylines`: the main chapters in
 * chapter order, then everything else by release YEAR newest first, then the
 * records. `startTime` is -1 on a group the table does not date, and those sit
 * in a final undated section rather than under a wrong year.
 */
export function fallbackSections(groups: readonly LibGroup[]): ISection[] {
    const sections: ISection[] = [];
    const mains = groups.filter((g) => g.category === "main").sort((a, b) => mainOrdinal(a.id) - mainOrdinal(b.id) || a.id.localeCompare(b.id));
    if (mains.length > 0) sections.push({ id: "main-story", title: "", kind: "main", groups: mains, collapsed: false, lineId: "main-story" });

    const rest = groups.filter((g) => g.category !== "record" && g.category !== "main");
    const years = new Map<number, LibGroup[]>();
    for (const group of rest) {
        const year = group.startTime > 0 ? new Date(group.startTime * 1000).getUTCFullYear() : 0;
        const bucket = years.get(year);
        if (bucket) bucket.push(group);
        else years.set(year, [group]);
    }
    for (const [year, bucket] of [...years].sort((a, b) => (a[0] === 0 ? 1 : b[0] === 0 ? -1 : b[0] - a[0]))) {
        sections.push({
            id: year === 0 ? "undated" : `year-${year}`,
            title: year === 0 ? "" : String(year),
            kind: "year",
            groups: bucket.sort((a, b) => b.startTime - a.startTime || a.id.localeCompare(b.id)),
            collapsed: false,
            lineId: year === 0 ? "undated" : `year-${year}`,
        });
    }
    return sections;
}

function mainOrdinal(id: string): number {
    const n = Number.parseInt(id.replace("main_", ""), 10);
    return Number.isNaN(n) ? Number.MAX_SAFE_INTEGER : n;
}

/**
 * The whole page: sections from the game's shelves when the wire carries them,
 * the fallback cut when it does not, filtered by the pill and the search box,
 * with empty sections dropped.
 */
export function sectionLibrary(index: LibIndex, filter: FilterKey, keep: (group: LibGroup) => boolean, keepRecord: (record: LibRecord) => boolean): ISectionedLibrary {
    const storyGroups = index.groups.filter((g) => g.category !== "record");
    const lines = index.storylines ?? [];
    const fallback = lines.length === 0;
    const all = fallback ? fallbackSections(storyGroups) : buildSections(storyGroups, lines);

    const sections = all.map((section) => ({ ...section, groups: section.groups.filter((g) => matchesFilter(kindOf(g), filter) && keep(g)) })).filter((section) => section.groups.length > 0);

    const records = matchesFilter("record", filter) ? index.records.filter(keepRecord) : [];
    return { sections, records, fallback };
}

/**
 * The short code a card prints under its title: the chapter's own third name
 * (`EPISODE 00`) on the mainline, else the operation prefix its stories share
 * (`CB-1` -> `CB`), else the uppercased group id. It never returns an empty
 * string and it never returns a placeholder word.
 *
 * THE FIRST CODE IS NOT ALWAYS AN OPERATION CODE. `story_review_table` writes
 * the literal `ENTRY` as the `storyCode` of an act's first-enter entry, and
 * `derive.groupCode` returns the first non-empty code verbatim, so "Code of
 * Brawl" (`act5d0`) printed "ENTRY · 2020 · 27 entries" and "Heart of Surging
 * Flame" (`act3d0`) printed "ENTRY · 2020 · 15 entries" where the acts are CB
 * and OF. An operation code is recognised by its HYPHEN: over the 451 EN groups
 * `ENTRY` is the only code in the whole corpus with no hyphen in it, 2 of 2
 * occurrences, and all 72 other prefixes come off a hyphenated code. The same
 * count holds on CN (2 of 2 hyphen-less, both `ENTRY`, same two groups), so the
 * word is a constant in the table rather than a translated string, and the
 * hyphen test does not need a word list to survive a server change.
 *
 * The trade this ships knowingly: a server that one day writes a REAL code with
 * no hyphen in it falls back to the group id rather than printing that code. No
 * such code exists on EN or CN today, and the id is a true handle where `ENTRY`
 * was not.
 *
 * Kept here rather than in `derive.groupCode` so the five other surfaces that
 * call `groupCode` (the chapter modal, the hero, the continue card, the reading
 * order and the illustrations tab) keep the behaviour they were verified with.
 */
export function cardCode(group: Pick<LibGroup, "id" | "zone" | "stories">): string {
    const third = group.zone?.nameThird?.trim();
    if (third) return third;
    for (const story of group.stories) {
        const code = story.code?.trim();
        if (!code?.includes("-")) continue;
        const prefix = code.split("-")[0]?.trim();
        if (prefix) return prefix.toUpperCase();
    }
    return group.id.toUpperCase();
}

/**
 * The jump list, one chip per rendered section plus the records when they are
 * shown. `count` is what the section is currently showing, not what it holds,
 * because the pills and the search box both shrink it.
 */
export function jumpChips(library: ISectionedLibrary, recordsTitle: string): { id: string; title: string; count: number }[] {
    const chips = library.sections.map((s) => ({ id: s.id, title: s.title, count: s.groups.length }));
    if (library.records.length > 0) chips.push({ id: RECORDS_ID, title: recordsTitle, count: library.records.length });
    return chips;
}
