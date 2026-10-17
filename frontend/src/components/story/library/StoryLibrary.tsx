import { useSuspenseQuery } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import type React from "react";
import { lazy, Suspense, useCallback, useEffect, useMemo, useState } from "react";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { emptyProgress, loadProgress, onProgressStorage, onProgressWritten, type StoryProgress } from "#/lib/story/progress";
import { useStoryGameRead, useStoryProgressSync } from "#/lib/story/sync";
import { trevorEnabled } from "#/lib/trevor/flag";
import { cn } from "#/lib/utils";
import type { messages as spriteMessages } from "../sprites/CharactersTab.messages";
import { Browse } from "./impl/Browse";
import { CommunityTab } from "./impl/CommunityTab";
import type { messages as communityMessages } from "./impl/CommunityTab.messages";
import { ContinueCard } from "./impl/ContinueCard";
import type { LibIndex } from "./impl/derive";
import { pickHero } from "./impl/derive";
import { SHELF_HEAD } from "./impl/GroupCard";
import { IllustrationsTab } from "./impl/IllustrationsTab";
import { NowPlayingBar } from "./impl/NowPlayingBar";
import { ProgressTab } from "./impl/ProgressTab";
import { useLibraryPlayerLifetime } from "./impl/player";
import { ReadingOrderTab } from "./impl/ReadingOrderTab";
import { ASK_TAB, isLibraryTab, type LibraryTab } from "./StoryLibrary.constants";
import type { messages } from "./StoryLibrary.messages";

// The Characters tab is its own chunk: Browse's bundle and first load do not
// carry the gallery, and nothing of it is fetched until the tab is opened.
const CharactersTab = lazy(() => import("../sprites/CharactersTab"));
// Ask Trevor (behind VITE_TREVOR=1) is its own chunk too, fetched on first open.
const LibraryAsk = lazy(() => import("../trevor/LibraryAsk").then((m) => ({ default: m.LibraryAsk })));

/**
 * The library at `/stories`: a page head, a compact continue row, one mode row,
 * and then a single continuous browse surface cut into the game's own storyline
 * sections. Reading Order, Illustrations, Progress, Community and Characters are
 * the other modes; operator records are the browse surface's own last section.
 *
 * Progress lives in localStorage, which the server does not have. The first
 * render on BOTH sides uses an empty progress and an effect swaps in the real
 * one, so there is nothing for hydration to disagree about.
 */
export function StoryLibrary(): React.ReactElement {
    const t: TypedT<typeof messages & typeof communityMessages & typeof spriteMessages> = useT("story");
    const f = useFormatters();
    const { data } = useSuspenseQuery(storyIndexQueryOptions(useGamedataServer()));
    const index = data as unknown as LibIndex;

    // The document is read once on mount and then FOLLOWED, because the library
    // also writes it: every tick in the sheet, in a reading
    // order row and in an operator's records writes through `saveProgress`, and
    // a fraction that only refreshed on a tab change would leave a reader
    // clicking a tick that does not move. The same subscription carries the
    // account sync's own merges, which land here from outside React entirely.
    const [progress, setProgress] = useState<StoryProgress>(emptyProgress);
    useEffect(() => {
        setProgress(loadProgress());
        // This tab's writes and another tab's writes are two seams; the
        // library follows both so a story finished in a reader tab turns
        // read here without a reload.
        const offWrite = onProgressWritten(setProgress);
        const offStorage = onProgressStorage(setProgress);
        return () => {
            offWrite();
            offStorage();
        };
    }, []);

    // Mounting the library is a sync trigger: it pulls the account's document,
    // merges, and writes back whichever side is behind. The tab that REPORTS
    // the sync mounts the same singleton, so this costs nothing twice.
    useStoryProgressSync();
    // The game's own read verdict is read HERE and passed down beside the
    // document: it is one set per pull, and a subscription per card would
    // resubscribe 451 of them for a value that changes once a session.
    const gameRead = useStoryGameRead();
    // The library's music channel is the PAGE's, not the chapter sheet's: a
    // theme started from a ticket keeps playing while the reader browses on and
    // while a sheet opens and closes over it. It dies with this component,
    // which is what leaving `/stories` is, and opening a story is leaving
    // `/stories`: the reader route has music of its own and the two must never
    // sound at once.
    useLibraryPlayerLifetime();

    // The tab is the URL's `?tab=`, so a link can open one; `?sprite=` is read
    // by the Characters tab itself and never here, so opening a sheet does not
    // re-render the library.
    const tab: LibraryTab = useSearch({ from: "/stories", select: (search) => search.tab ?? "browse" });
    const navigate = useNavigate({ from: "/stories" });
    const trevor = trevorEnabled();
    // Ask Trevor joins the strip and the panels only with the flag on. It is
    // spliced into the SAME flat children arrays rather than added as a
    // conditional slot: React's useId derives ids from a child's index AND the
    // sibling count, so even a null slot renumbers every tab's id, and the flag
    // off must render byte for byte as before.
    const withAsk = (items: React.ReactElement[], ask: React.ReactElement, at: number): React.ReactElement[] => (trevor ? [...items.slice(0, at), ask, ...items.slice(at)] : items);
    const setTab = useCallback(
        (next: string) => {
            const value: LibraryTab = isLibraryTab(next) ? next : next === ASK_TAB && trevor ? ASK_TAB : "browse";
            void navigate({ search: (prev) => ({ ...prev, tab: value === "browse" ? undefined : value, sprite: value === "characters" ? prev.sprite : undefined }), replace: true, resetScroll: false });
        },
        [navigate, trevor],
    );
    // Mounted on first open and KEPT, so a question in flight survives a tab switch.
    const [askSeen, setAskSeen] = useState(tab === ASK_TAB);
    if (tab === ASK_TAB && !askSeen) setAskSeen(true);
    // Mounted on first open and KEPT, so switching to Browse and back keeps the
    // search text, the filters, the page count and the scroll of the grid.
    const [charactersSeen, setCharactersSeen] = useState(tab === "characters");
    if (tab === "characters" && !charactersSeen) setCharactersSeen(true);
    const [openGroup, setOpenGroup] = useState<string | null>(null);
    const viewChapter = useCallback(
        (groupId: string) => {
            setTab("browse");
            setOpenGroup(groupId);
        },
        [setTab],
    );
    const onOpenHandled = useCallback(() => setOpenGroup(null), []);

    const storyGroups = useMemo(() => index.groups.filter((g) => g.category !== "record"), [index]);
    const hero = useMemo(() => pickHero(index, progress), [index, progress]);
    const totals = useMemo(() => {
        if (index.totals) return index.totals.stories;
        return index.groups.reduce((n, g) => n + g.stories.length, 0);
    }, [index]);

    return (
        <div className="page-shell [--page-max:1400px]">
            {/* THE BIG NUMBER IS A DESKTOP ORNAMENT. Under 640 it would be a 130 px block
                of its own over a 923 px head, so there it folds into the subhead as a
                third count and the right-hand stat is hidden. At 640 and up the
                subhead keeps its two counts and the number stays where it was. */}
            <PageHeader
                className={cn("mb-4 border-primary/70 border-b-2 pb-3", SHELF_HEAD)}
                breadcrumbLabel={t("library.breadcrumb")}
                breadcrumb={[t("library.breadcrumb.archive"), t("library.breadcrumb.stories")]}
                title={t("library.title")}
                description={
                    <>
                        <span className="sm:hidden">{t("library.countsAll", { groups: storyGroups.length, records: index.records.length, stories: f.number(totals) })}</span>
                        <span className="max-sm:hidden">{t("library.counts", { groups: storyGroups.length, records: index.records.length })}</span>
                    </>
                }
                actions={
                    <div className="hidden sm:block sm:text-right">
                        <div className="font-light font-sans text-[34px] text-primary tabular-nums leading-none tracking-[-0.02em]">{f.number(totals)}</div>
                        <div className="mt-1.5 font-sans text-[12px] text-muted-foreground">{t("library.storiesLabel")}</div>
                    </div>
                }
            />

            {hero ? <ContinueCard pick={hero} progress={progress} gameRead={gameRead} onViewChapter={viewChapter} /> : null}

            <Tabs value={tab} onValueChange={(value) => setTab(String(value))}>
                {withAsk(
                    [
                        <div key="strip" className="msv-scroll flex max-w-full items-center gap-1 overflow-x-auto">
                            <TabsList className="shrink-0">
                                {withAsk(
                                    [
                                        <TabsTab key="browse" value="browse" className="shrink-0 max-sm:min-h-11">
                                            {t("library.tab.browse")}
                                        </TabsTab>,
                                        <TabsTab key="reading" value="reading" className="shrink-0 max-sm:min-h-11">
                                            {t("library.tab.reading")}
                                        </TabsTab>,
                                        <TabsTab key="illustrations" value="illustrations" className="shrink-0 max-sm:min-h-11">
                                            {t("library.tab.illustrations")}
                                        </TabsTab>,
                                        <TabsTab key="progress" value="progress" className="shrink-0 max-sm:min-h-11">
                                            {t("library.tab.progress")}
                                        </TabsTab>,
                                        <TabsTab key="community" value="community" className="shrink-0 max-sm:min-h-11">
                                            {t("community.tab")}
                                        </TabsTab>,
                                        <TabsTab key="characters" value="characters" className="shrink-0 max-sm:min-h-11">
                                            {t("sprites.tab")}
                                        </TabsTab>,
                                    ],
                                    <TabsTab key="ask" value={ASK_TAB} className="shrink-0 max-sm:min-h-11" data-trevor-library-tab>
                                        {t("library.tab.ask")}
                                    </TabsTab>,
                                    1,
                                )}
                            </TabsList>
                        </div>,
                        <TabsPanel key="browse" value="browse" className="pt-4">
                            <Browse index={index} progress={progress} gameRead={gameRead} openGroup={openGroup} onOpenHandled={onOpenHandled} />
                        </TabsPanel>,
                        <TabsPanel key="reading" value="reading">
                            <ReadingOrderTab groups={storyGroups} progress={progress} gameRead={gameRead} />
                        </TabsPanel>,
                        <TabsPanel key="illustrations" value="illustrations" className="pt-4">
                            <IllustrationsTab groups={storyGroups} records={index.records} />
                        </TabsPanel>,
                        <TabsPanel key="progress" value="progress">
                            <ProgressTab index={index} progress={progress} gameRead={gameRead} onProgressChanged={setProgress} />
                        </TabsPanel>,
                        // A ranked row here opens the chapter sheet through the SAME seam the
                        // continue card uses, which switches to Browse and hands the group id
                        // over rather than teaching a second surface to mount the sheet.
                        <TabsPanel key="community" value="community">
                            <CommunityTab index={index} onViewChapter={viewChapter} />
                        </TabsPanel>,
                        <TabsPanel key="characters" value="characters" keepMounted={charactersSeen}>
                            {charactersSeen ? (
                                <Suspense fallback={null}>
                                    <CharactersTab />
                                </Suspense>
                            ) : null}
                        </TabsPanel>,
                    ],
                    <TabsPanel key="ask" value={ASK_TAB} keepMounted={askSeen}>
                        {askSeen ? (
                            <Suspense fallback={null}>
                                <LibraryAsk active={tab === ASK_TAB} />
                            </Suspense>
                        ) : null}
                    </TabsPanel>,
                    7,
                )}
            </Tabs>

            <NowPlayingBar />
        </div>
    );
}
