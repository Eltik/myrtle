import type React from "react";
import { useCallback, useDeferredValue, useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import type { messages } from "./Browse.messages";
import { FlatHead, JumpBar, SectionHead, sectionTitle } from "./BrowseSections";
import { BrowseToolbar, type IBrowseToolbarState, ToolbarButton } from "./BrowseToolbar";
import { ChapterModal } from "./ChapterModal";
import { glyphIndexFor, type IChipModel, sectionChapters } from "./chapters";
import { groupSearchTarget, type IReadFraction, LIBRARY_SORTS, type LibGroup, type LibIndex, type LibRecord, type LibrarySort, matchesReadState, prepareSearch, READ_FILTERS, type ReadFilter, readFraction, recordSearchTarget, searchList, sortLibrary } from "./derive";
import { GroupCard, GroupGrid, GroupRow, GroupRowList } from "./GroupCard";
import { OperatorsTab } from "./OperatorsTab";
import { type FilterKey, RECORDS_ID, sectionFilterKey, sectionLibrary, type ViewMode } from "./sections";
import { TOOLBAR_DEFAULTS } from "./toolbar";

type BrowseT = TypedT<typeof messages>;

const VIEW_KEY = "story.library.view";
const READ_KEY = "story.library.read";
const SORT_KEY = "story.library.sort";

/** The bar's chips under a sort: none, as one constant so the bar's spy is not handed a fresh empty list every render. */
const NO_CHIPS: readonly IChipModel[] = [];

/**
 * The three toggles are stored as the BARE token, not as JSON, which is the
 * convention `story.library.view` already set. A stored value that is not one
 * of the tokens returns `undefined` and the hook keeps the default, so a
 * hand-edited or stale key degrades to "Any" and "Default" rather than
 * filtering the page down to nothing.
 */
function oneOf<T extends string>(allowed: readonly T[]): { parse: (raw: string) => T | undefined; serialize: (value: T) => string } {
    return { parse: (raw) => (allowed.includes(raw as T) ? (raw as T) : undefined), serialize: (value) => value };
}

export interface IBrowseProps {
    index: LibIndex;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    /** A group the continue card asked to open. */
    openGroup: string | null;
    onOpenHandled: () => void;
}

/**
 * The browse surface: one continuous page cut into the game's own storyline
 * sections, with a sticky jump list over it. Not sub-tabs, because the
 * sections ARE the navigation and a tab row hides eleven of them.
 */
export function Browse({ index, progress, gameRead, openGroup, onOpenHandled }: IBrowseProps): React.ReactElement {
    const t: BrowseT = useT("story");
    const collator = useFormatters().collator;
    const [query, setQuery] = useState("");
    const [filter, setFilter] = useState<FilterKey>(TOOLBAR_DEFAULTS.filter);
    const [view, setView] = useLocalStorageState<ViewMode>(VIEW_KEY, "grid", { parse: (raw) => (raw === "list" || raw === "grid" ? raw : undefined), serialize: (v) => v });
    const [readFilter, setReadFilter] = useLocalStorageState<ReadFilter>(READ_KEY, TOOLBAR_DEFAULTS.readFilter, oneOf(READ_FILTERS));
    const [sort, setSort] = useLocalStorageState<LibrarySort>(SORT_KEY, TOOLBAR_DEFAULTS.sort, oneOf(LIBRARY_SORTS));
    const [openId, setOpenId] = useState<string | null>(null);
    const [recordsOpen, setRecordsOpen] = useState(false);

    /**
     * THE CONTROLS ANSWER AT ONCE AND THE PAGE FOLLOWS. The toolbar is
     * controlled by the immediate state; everything the page derives from it
     * reads React's DEFERRED copies below, so a keystroke or a pill press
     * commits the control first and the re-filtered page renders after it in an
     * interruptible pass. Filtering on `query` directly made each keystroke one
     * 570 to 610 ms task (INP 634 ms at 1440, 631 at 390), and the pills were
     * the same shape: "All" after "Events" was a 195 ms task once the search
     * alone had been deferred.
     */
    const searched = useDeferredValue(query);
    const shownFilter = useDeferredValue(filter);
    const shownRead = useDeferredValue(readFilter);
    const shownSort = useDeferredValue(sort);
    const shownView = useDeferredValue(view);
    // The haystacks are normalised ONCE per index, and the fuzzy pass then runs
    // once over each half per keystroke and answers as a set; asking it per
    // card would build 451 searchers, and re-folding every name per keystroke
    // re-normalised the 1,887 story names each time.
    const groupSearch = useMemo(() => prepareSearch(index.groups, groupSearchTarget), [index.groups]);
    const recordSearch = useMemo(() => prepareSearch(index.records, recordSearchTarget), [index.records]);
    const matchedGroups = useMemo(() => new Set(searchList(searched, groupSearch).map((g) => g.id)), [searched, groupSearch]);
    const matchedRecords = useMemo(() => new Set(searchList(searched, recordSearch).map((r) => r.charId)), [searched, recordSearch]);

    // ONE PASS over the index answers both the read-state filter and the read
    // sorts. `readFraction` walks a group's stories, and calling it per card
    // per comparison would walk the 1,887 EN stories O(n log n) times inside a
    // sort comparator.
    const fractions = useMemo(() => new Map(index.groups.map((g) => [g.id, readFraction(g.stories, progress, gameRead)])), [index.groups, progress, gameRead]);
    const recordFractions = useMemo(() => new Map(index.records.map((r) => [r.charId, readFraction(r.stories, progress, gameRead)])), [index.records, progress, gameRead]);
    const fractionOf = useCallback((group: LibGroup) => fractions.get(group.id) ?? readFraction(group.stories, progress, gameRead), [fractions, progress, gameRead]);

    const keepGroup = useCallback((group: LibGroup) => matchedGroups.has(group.id) && matchesReadState(shownRead, fractionOf(group)), [matchedGroups, shownRead, fractionOf]);
    const keepRecord = useCallback((record: LibRecord) => matchedRecords.has(record.charId) && matchesReadState(shownRead, recordFractions.get(record.charId) ?? readFraction(record.stories, progress, gameRead)), [matchedRecords, shownRead, recordFractions, progress, gameRead]);
    const library = useMemo(() => sectionLibrary(index, shownFilter, keepGroup, keepRecord), [index, shownFilter, keepGroup, keepRecord]);

    /**
     * A SORT OTHER THAN DEFAULT FLATTENS THE SHELVES. Leaving the 14 sections
     * in place would order each one internally and leave the page still
     * reading by shelf, so "Oldest release" would print the mainline first
     * whatever it says; one list is the only way the order is the thing the
     * reader sees. `null` is the default, and it keeps the sections.
     */
    const flat = useMemo(
        () =>
            shownSort === "default"
                ? null
                : sortLibrary(
                      library.sections.flatMap((s) => s.groups),
                      shownSort,
                      fractionOf,
                      collator,
                  ),
        [library.sections, shownSort, fractionOf, collator],
    );

    // A search or a records-only filter should not leave the 315 cards behind a Show button.
    const recordsForced = searched.trim() !== "" || shownFilter === "records";
    const showRecords = recordsOpen || recordsForced;

    const openGroupObject = useMemo(() => (openId ? (index.groups.find((g) => g.id === openId) ?? null) : null), [openId, index.groups]);

    useEffect(() => {
        if (!openGroup) return;
        setOpenId(openGroup);
        onOpenHandled();
    }, [openGroup, onOpenHandled]);

    const chips: IChipModel[] = useMemo(() => {
        const out: IChipModel[] = library.sections.map((s) => {
            const chapters = sectionChapters(s.groups, s.chapterRange);
            // An ARC is main story by definition, even when an intermezzo is
            // shelved inside it (Act I holds Darknights' Memoir): the kind word
            // names the run, not the majority of its cards.
            // "Other events" is the one wire section the game marks with
            // nothing, so it is the one that needs an abbreviation to collapse
            // to. A shelf collapses to its monogram and an arc to its banner.
            return {
                id: s.id,
                range: chapters.primary,
                includes: chapters.includes,
                name: sectionTitle(s, t),
                abbr: s.kind === "other" ? t("browse.chip.otherAbbr") : undefined,
                count: s.groups.length,
                iconUrl: s.iconUrl,
                iconWide: s.iconWide === true,
                iconLogo: s.iconLogo === true,
                glyph: glyphIndexFor(s.lineId),
                filter: s.kind === "arc" ? "main" : sectionFilterKey(s.groups),
            };
        });
        // The records section is NOT given a kind word: its name already is
        // one, and "Operator records · Operator records" is what that reads as.
        if (library.records.length > 0) out.push({ id: RECORDS_ID, range: null, includes: null, name: t("browse.section.records"), abbr: t("browse.chip.recordsAbbr"), count: library.records.length, glyph: glyphIndexFor(RECORDS_ID), filter: null });
        return out;
    }, [library, t]);

    const chipById = useMemo(() => new Map(chips.map((c) => [c.id, c])), [chips]);
    const toolbar: IBrowseToolbarState = { query, setQuery, filter, setFilter, readFilter, setReadFilter, sort, setSort, view, setView };

    return (
        <>
            <div className="flex flex-col gap-3">
                <BrowseToolbar state={toolbar} />

                {library.fallback ? <p className="m-0 rounded-[10px] border border-warning/40 bg-warning/8 px-3 py-2 font-sans text-[12px] text-muted-foreground">{t("browse.fallback")}</p> : null}
            </div>

            {/* The bar is pinned under a sort too, with no chips in it: it is the only
                search and filter a scrolled reader has, and a sorted page is the one
                most likely to want its sort changed back. */}
            <JumpBar chips={flat === null ? chips : NO_CHIPS} tools={<ToolbarButton state={toolbar} />} />

            {(flat === null ? library.sections.length === 0 : flat.length === 0) && library.records.length === 0 ? <div className="mt-6 rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("browse.empty")}</div> : null}

            {flat !== null ? (
                flat.length > 0 ? (
                    <section className="mt-7">
                        <FlatHead title={t("browse.section.sorted")} count={t("browse.section.count", { count: flat.length })} />
                        <Cards groups={flat} view={shownView} fractionOf={fractionOf} onOpen={setOpenId} />
                    </section>
                ) : null
            ) : (
                library.sections.map((section) => (
                    <section key={section.id} id={section.id} className="mt-7 scroll-mt-32 sm:scroll-mt-36">
                        <SectionHead chip={chipById.get(section.id)} count={t("browse.section.count", { count: section.groups.length })} />
                        <Cards groups={section.groups} view={shownView} fractionOf={fractionOf} onOpen={setOpenId} />
                    </section>
                ))
            )}

            {library.records.length > 0 ? (
                <section id={RECORDS_ID} className="mt-7 scroll-mt-32 sm:scroll-mt-36">
                    <SectionHead
                        chip={chipById.get(RECORDS_ID)}
                        count={t("browse.section.recordCount", { count: library.records.length })}
                        action={
                            recordsForced ? null : (
                                <Button variant="ghost" size="sm" className="max-sm:min-h-11" onClick={() => setRecordsOpen((open) => !open)} aria-expanded={showRecords}>
                                    {showRecords ? t("browse.section.hide") : t("browse.section.show")}
                                </Button>
                            )
                        }
                    />
                    {showRecords ? <OperatorsTab records={library.records} progress={progress} gameRead={gameRead} controls={false} paged pageKey={`${searched}|${shownFilter}|${shownRead}`} /> : null}
                </section>
            ) : null}

            {openGroupObject ? <ChapterModal key={openGroupObject.id} group={openGroupObject} progress={progress} gameRead={gameRead} onClose={() => setOpenId(null)} /> : null}
        </>
    );
}

/**
 * The cards of one section, in whichever layout the toggle is on. Both branches were written out twice and the flat list would have made it three times.
 *
 * `fractionOf` hands each card the fraction Browse already holds, so the
 * memoised card sees the SAME object until progress changes and skips.
 */
function Cards({ groups, view, fractionOf, onOpen }: { groups: readonly LibGroup[]; view: ViewMode; fractionOf: (group: LibGroup) => IReadFraction; onOpen: (id: string) => void }): React.ReactElement {
    if (view === "grid")
        return (
            <GroupGrid>
                {groups.map((group, at) => (
                    <GroupCard key={group.id} group={group} fraction={fractionOf(group)} onOpen={onOpen} index={at} />
                ))}
            </GroupGrid>
        );
    return (
        <GroupRowList>
            {groups.map((group, at) => (
                <GroupRow key={group.id} group={group} fraction={fractionOf(group)} onOpen={onOpen} index={at} />
            ))}
        </GroupRowList>
    );
}
