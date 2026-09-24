import { LayoutGridIcon, ListIcon, SearchIcon } from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { cn } from "#/lib/utils";
import type { messages } from "./Browse.messages";
import { FlatHead, JumpBar, SectionHead, sectionTitle, useScrollSpy } from "./BrowseSections";
import { ChapterModal } from "./ChapterModal";
import { glyphIndexFor, type IChipModel, sectionChapters } from "./chapters";
import { filterGroups, filterRecords, LIBRARY_SORTS, type LibGroup, type LibIndex, type LibRecord, type LibrarySort, matchesReadState, READ_FILTERS, type ReadFilter, readFraction, sortLibrary } from "./derive";
import { GroupCard, GroupGrid, GroupRow, GroupRowList } from "./GroupCard";
import { OperatorsTab } from "./OperatorsTab";
import { FILTER_ORDER, type FilterKey, RECORDS_ID, sectionLibrary, type ViewMode } from "./sections";

type BrowseT = TypedT<typeof messages>;

const VIEW_KEY = "story.library.view";
const READ_KEY = "story.library.read";
const SORT_KEY = "story.library.sort";

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
    const [filter, setFilter] = useState<FilterKey>("all");
    const [view, setView] = useLocalStorageState<ViewMode>(VIEW_KEY, "grid", { parse: (raw) => (raw === "list" || raw === "grid" ? raw : undefined), serialize: (v) => v });
    const [readFilter, setReadFilter] = useLocalStorageState<ReadFilter>(READ_KEY, "any", oneOf(READ_FILTERS));
    const [sort, setSort] = useLocalStorageState<LibrarySort>(SORT_KEY, "default", oneOf(LIBRARY_SORTS));
    const [openId, setOpenId] = useState<string | null>(null);
    const [recordsOpen, setRecordsOpen] = useState(false);

    // The fuzzy pass runs ONCE over each half and answers as a set; asking it
    // per card would build 451 searchers per keystroke.
    const matchedGroups = useMemo(() => new Set(filterGroups(query, index.groups).map((g) => g.id)), [query, index.groups]);
    const matchedRecords = useMemo(() => new Set(filterRecords(query, index.records).map((r) => r.charId)), [query, index.records]);

    // ONE PASS over the index answers both the read-state filter and the read
    // sorts. `readFraction` walks a group's stories, and calling it per card
    // per comparison would walk the 1,887 EN stories O(n log n) times inside a
    // sort comparator.
    const fractions = useMemo(() => new Map(index.groups.map((g) => [g.id, readFraction(g.stories, progress, gameRead)])), [index.groups, progress, gameRead]);
    const recordFractions = useMemo(() => new Map(index.records.map((r) => [r.charId, readFraction(r.stories, progress, gameRead)])), [index.records, progress, gameRead]);
    const fractionOf = useCallback((group: LibGroup) => fractions.get(group.id) ?? readFraction(group.stories, progress, gameRead), [fractions, progress, gameRead]);

    const keepGroup = useCallback((group: LibGroup) => matchedGroups.has(group.id) && matchesReadState(readFilter, fractionOf(group)), [matchedGroups, readFilter, fractionOf]);
    const keepRecord = useCallback((record: LibRecord) => matchedRecords.has(record.charId) && matchesReadState(readFilter, recordFractions.get(record.charId) ?? readFraction(record.stories, progress, gameRead)), [matchedRecords, readFilter, recordFractions, progress, gameRead]);
    const library = useMemo(() => sectionLibrary(index, filter, keepGroup, keepRecord), [index, filter, keepGroup, keepRecord]);

    /**
     * A SORT OTHER THAN DEFAULT FLATTENS THE SHELVES. Leaving the 14 sections
     * in place would order each one internally and leave the page still
     * reading by shelf, so "Oldest release" would print the mainline first
     * whatever it says; one list is the only way the order is the thing the
     * reader sees. `null` is the default, and it keeps the sections.
     */
    const flat = useMemo(
        () =>
            sort === "default"
                ? null
                : sortLibrary(
                      library.sections.flatMap((s) => s.groups),
                      sort,
                      fractionOf,
                      collator,
                  ),
        [library.sections, sort, fractionOf, collator],
    );

    // A search or a records-only filter should not leave the 315 cards behind a Show button.
    const recordsForced = query.trim() !== "" || filter === "records";
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
            return { id: s.id, range: chapters.primary, includes: chapters.includes, name: sectionTitle(s, t), count: s.groups.length, iconUrl: s.iconUrl, iconWide: s.iconWide === true, iconLogo: s.iconLogo === true, glyph: glyphIndexFor(s.lineId) };
        });
        if (library.records.length > 0) out.push({ id: RECORDS_ID, range: null, includes: null, name: t("browse.section.records"), count: library.records.length, glyph: glyphIndexFor(RECORDS_ID) });
        return out;
    }, [library, t]);

    const active = useScrollSpy(useMemo(() => (flat === null ? chips.map((c) => c.id) : []), [chips, flat]));
    const chipById = useMemo(() => new Map(chips.map((c) => [c.id, c])), [chips]);

    return (
        <>
            <div className="flex flex-col gap-3">
                {/* Two rows at every width: the search and the layout toggle, then the pills on their own scrollable line. One wrapping row put the toggle over the pills at 375. */}
                <div className="flex items-center gap-2">
                    <div className="relative min-w-0 flex-1">
                        <SearchIcon className="pointer-events-none absolute top-1/2 left-3.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                        <input
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder={t("browse.search.placeholder")}
                            aria-label={t("browse.search.aria")}
                            className="h-11 pointer-coarse:h-11 w-full rounded-[9px] border border-border bg-secondary/50 pr-3.5 pl-9 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 sm:h-9.5"
                        />
                    </div>
                    <fieldset className="flex shrink-0 gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("browse.view.aria")}>
                        {(["grid", "list"] as const).map((mode) => (
                            <button
                                key={mode}
                                type="button"
                                onClick={() => setView(mode)}
                                aria-pressed={view === mode}
                                aria-label={t(`browse.view.${mode}`)}
                                className={cn("flex h-11 pointer-coarse:h-11 pointer-coarse:w-11 w-11 cursor-pointer items-center justify-center rounded-md transition-colors sm:h-7.5 sm:w-9", view === mode ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                            >
                                {mode === "grid" ? <LayoutGridIcon className="size-3.5" aria-hidden="true" /> : <ListIcon className="size-3.5" aria-hidden="true" />}
                            </button>
                        ))}
                    </fieldset>
                </div>
                <div className="msv-scroll -my-1 flex max-w-full items-center gap-1.5 overflow-x-auto py-1">
                    {FILTER_ORDER.map((key) => (
                        <button
                            key={key}
                            type="button"
                            onClick={() => setFilter(key)}
                            aria-pressed={filter === key}
                            className={cn(
                                "h-11 pointer-coarse:h-11 min-w-11 pointer-coarse:min-w-11 shrink-0 cursor-pointer rounded-full border px-3 font-sans font-semibold text-[12px] transition-colors sm:h-8 sm:min-w-0",
                                filter === key ? "border-primary/55 bg-primary/12 text-foreground" : "border-border bg-secondary/40 text-muted-foreground hover:text-foreground",
                            )}
                        >
                            {t(`browse.filter.${key}`)}
                        </button>
                    ))}
                </div>

                {/* The read state is a segmented control and the order a native select: four states are worth four targets, and six orders in a row are a scroller nobody reads. */}
                <div className="flex flex-wrap items-center gap-2">
                    <fieldset className="msv-scroll flex min-w-0 max-w-full gap-0.5 overflow-x-auto rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("browse.read.aria")}>
                        {READ_FILTERS.map((key) => (
                            <button
                                key={key}
                                type="button"
                                onClick={() => setReadFilter(key)}
                                aria-pressed={readFilter === key}
                                className={cn(
                                    "h-11 pointer-coarse:h-11 min-w-11 pointer-coarse:min-w-11 shrink-0 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] transition-colors sm:h-7 sm:min-w-0",
                                    readFilter === key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground",
                                )}
                            >
                                {t(`browse.read.${key}`)}
                            </button>
                        ))}
                    </fieldset>
                    <label className="flex min-w-0 items-center gap-2 sm:ms-auto">
                        <span className="shrink-0 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("browse.sort.label")}</span>
                        <select
                            value={sort}
                            onChange={(e) => setSort(e.target.value as LibrarySort)}
                            className="h-11 pointer-coarse:h-11 min-w-0 rounded-[9px] border border-border bg-secondary/50 px-2.5 font-sans text-[12.5px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 sm:h-8"
                        >
                            {LIBRARY_SORTS.map((key) => (
                                <option key={key} value={key}>
                                    {t(`browse.sort.${key}`)}
                                </option>
                            ))}
                        </select>
                    </label>
                </div>

                {library.fallback ? <p className="m-0 rounded-[10px] border border-warning/40 bg-warning/8 px-3 py-2 font-sans text-[12px] text-muted-foreground">{t("browse.fallback")}</p> : null}
            </div>

            {flat === null && chips.length > 0 ? <JumpBar chips={chips} active={active} /> : null}

            {(flat === null ? library.sections.length === 0 : flat.length === 0) && library.records.length === 0 ? <div className="mt-6 rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("browse.empty")}</div> : null}

            {flat !== null ? (
                flat.length > 0 ? (
                    <section className="mt-7">
                        <FlatHead title={t("browse.section.sorted")} count={t("browse.section.count", { count: flat.length })} />
                        <Cards groups={flat} view={view} progress={progress} gameRead={gameRead} onOpen={setOpenId} />
                    </section>
                ) : null
            ) : (
                library.sections.map((section) => (
                    <section key={section.id} id={section.id} className="mt-7 scroll-mt-32">
                        <SectionHead chip={chipById.get(section.id)} count={t("browse.section.count", { count: section.groups.length })} />
                        <Cards groups={section.groups} view={view} progress={progress} gameRead={gameRead} onOpen={setOpenId} />
                    </section>
                ))
            )}

            {library.records.length > 0 ? (
                <section id={RECORDS_ID} className="mt-7 scroll-mt-32">
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
                    {showRecords ? <OperatorsTab records={library.records} progress={progress} gameRead={gameRead} controls={false} /> : null}
                </section>
            ) : null}

            {openGroupObject ? <ChapterModal key={openGroupObject.id} group={openGroupObject} progress={progress} gameRead={gameRead} onClose={() => setOpenId(null)} /> : null}
        </>
    );
}

/** The cards of one section, in whichever layout the toggle is on. Both branches were written out twice and the flat list would have made it three times. */
function Cards({ groups, view, progress, gameRead, onOpen }: { groups: readonly LibGroup[]; view: ViewMode; progress: StoryProgress; gameRead: ReadonlySet<string>; onOpen: (id: string) => void }): React.ReactElement {
    if (view === "grid")
        return (
            <GroupGrid>
                {groups.map((group, at) => (
                    <GroupCard key={group.id} group={group} progress={progress} gameRead={gameRead} onOpen={onOpen} index={at} />
                ))}
            </GroupGrid>
        );
    return (
        <GroupRowList>
            {groups.map((group, at) => (
                <GroupRow key={group.id} group={group} progress={progress} gameRead={gameRead} onOpen={onOpen} index={at} />
            ))}
        </GroupRowList>
    );
}

/** The disabled second mode tab. Rendered here so the page shell does not have to know why it is dead. */
export function DialogueModeTab(): React.ReactElement {
    const t: BrowseT = useT("story");
    return (
        <Tooltip>
            <TooltipTrigger
                render={
                    <span className="inline-flex h-9 shrink-0 cursor-not-allowed select-none items-center whitespace-nowrap rounded-[9px] px-3 font-sans font-semibold text-[12.5px] text-muted-foreground/60 max-sm:min-h-11" aria-disabled="true">
                        {t("browse.mode.dialogue")}
                    </span>
                }
            />
            <TooltipContent>{t("browse.mode.dialogue.soon")}</TooltipContent>
        </Tooltip>
    );
}
