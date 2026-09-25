import { useSuspenseQuery } from "@tanstack/react-query";
import type React from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { emptyProgress, loadProgress, onProgressStorage, onProgressWritten, type StoryProgress } from "#/lib/story/progress";
import { useStoryGameRead, useStoryProgressSync } from "#/lib/story/sync";
import { Browse } from "./impl/Browse";
import { CommunityTab } from "./impl/CommunityTab";
import type { messages as communityMessages } from "./impl/CommunityTab.messages";
import { ContinueCard } from "./impl/ContinueCard";
import type { LibIndex } from "./impl/derive";
import { pickHero } from "./impl/derive";
import { IllustrationsTab } from "./impl/IllustrationsTab";
import { NowPlayingBar } from "./impl/NowPlayingBar";
import { ProgressTab } from "./impl/ProgressTab";
import { useLibraryPlayerLifetime } from "./impl/player";
import { ReadingOrderTab } from "./impl/ReadingOrderTab";
import type { messages } from "./StoryLibrary.messages";

/**
 * The library at `/stories`: a page head, a compact continue row, one mode row,
 * and then a single continuous browse surface cut into the game's own storyline
 * sections. Reading Order, Illustrations and Progress are the other modes;
 * operator records are the browse surface's own last section, and their dialog
 * is unchanged.
 *
 * Progress lives in localStorage, which the server does not have. The first
 * render on BOTH sides uses an empty progress and an effect swaps in the real
 * one, so there is nothing for hydration to disagree about.
 */
export function StoryLibrary(): React.ReactElement {
    const t: TypedT<typeof messages & typeof communityMessages> = useT("story");
    const f = useFormatters();
    const { data } = useSuspenseQuery(storyIndexQueryOptions(useGamedataServer()));
    const index = data as unknown as LibIndex;

    // The document is read once on mount and then FOLLOWED, because the library
    // is no longer only a reader of it: every tick in the sheet, in a reading
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

    const [tab, setTab] = useState("browse");
    const [openGroup, setOpenGroup] = useState<string | null>(null);
    const viewChapter = useCallback((groupId: string) => {
        setTab("browse");
        setOpenGroup(groupId);
    }, []);
    const onOpenHandled = useCallback(() => setOpenGroup(null), []);

    const storyGroups = useMemo(() => index.groups.filter((g) => g.category !== "record"), [index]);
    const hero = useMemo(() => pickHero(index, progress), [index, progress]);
    const totals = useMemo(() => {
        if (index.totals) return index.totals.stories;
        return index.groups.reduce((n, g) => n + g.stories.length, 0);
    }, [index]);

    return (
        <div className="page-shell [--page-max:1400px]">
            {/* THE BIG NUMBER IS A DESKTOP ORNAMENT. Under 640 it was a 130 px block of
                its own over a 923 px head, so there it folds into the subhead as a
                third count and the right-hand stat is hidden. At 640 and up the
                subhead keeps its two counts and the number stays where it was. */}
            <PageHeader
                className="mb-4 border-primary/70 border-b-2 pb-3"
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
                        <div className="font-light font-mono text-[34px] text-primary leading-none tracking-[-0.02em]">{f.number(totals)}</div>
                        <div className="mt-1.5 font-mono text-[10px] text-muted-foreground tracking-[0.12em]">{t("library.storiesLabel")}</div>
                    </div>
                }
            />

            {hero ? <ContinueCard pick={hero} progress={progress} gameRead={gameRead} onViewChapter={viewChapter} /> : null}

            <Tabs value={tab} onValueChange={(value) => setTab(String(value))}>
                <div className="msv-scroll flex max-w-full items-center gap-1 overflow-x-auto">
                    <TabsList className="shrink-0">
                        <TabsTab value="browse" className="shrink-0 max-sm:min-h-11">
                            {t("library.tab.browse")}
                        </TabsTab>
                        <TabsTab value="reading" className="shrink-0 max-sm:min-h-11">
                            {t("library.tab.reading")}
                        </TabsTab>
                        <TabsTab value="illustrations" className="shrink-0 max-sm:min-h-11">
                            {t("library.tab.illustrations")}
                        </TabsTab>
                        <TabsTab value="progress" className="shrink-0 max-sm:min-h-11">
                            {t("library.tab.progress")}
                        </TabsTab>
                        <TabsTab value="community" className="shrink-0 max-sm:min-h-11">
                            {t("community.tab")}
                        </TabsTab>
                    </TabsList>
                </div>
                <TabsPanel value="browse" className="pt-4">
                    <Browse index={index} progress={progress} gameRead={gameRead} openGroup={openGroup} onOpenHandled={onOpenHandled} />
                </TabsPanel>
                <TabsPanel value="reading">
                    <ReadingOrderTab groups={storyGroups} progress={progress} gameRead={gameRead} />
                </TabsPanel>
                <TabsPanel value="illustrations" className="pt-4">
                    <IllustrationsTab groups={storyGroups} records={index.records} />
                </TabsPanel>
                <TabsPanel value="progress">
                    <ProgressTab index={index} progress={progress} gameRead={gameRead} onProgressChanged={setProgress} />
                </TabsPanel>
                {/* A ranked row here opens the chapter sheet through the SAME seam the
                    continue card uses, which switches to Browse and hands the group id
                    over rather than teaching a second surface to mount the sheet. */}
                <TabsPanel value="community">
                    <CommunityTab index={index} onViewChapter={viewChapter} />
                </TabsPanel>
            </Tabs>

            <NowPlayingBar />
        </div>
    );
}
