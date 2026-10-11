import { useQueryClient } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { lazy, Suspense, useCallback, useState } from "react";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { archiveDemographicsQueryOptions, archiveDossiersQueryOptions } from "#/lib/api/archives";
import { useGamedataServer } from "#/lib/i18n";
import { themeStore } from "#/lib/theme/store";
import type { ArchivesTab, IArchivesSearch } from "./Archives.constants";
import { DossiersTab } from "./impl/DossiersTab";
import { HiddenTab } from "./impl/HiddenTab";
import { NetworkTab } from "./impl/NetworkTab";
import { SpoilerNote, useArchivesT } from "./impl/shared";

// The two views that draw charts load on demand, so recharts stays out of the
// route's chunk: Network and Dossiers, the tabs most visits open, never use it.
const loadNumbersTab = () => import("./impl/NumbersTab");
const NumbersTab = lazy(() => loadNumbersTab().then((m) => ({ default: m.NumbersTab })));
const PersonnelSheet = lazy(() => import("./impl/PersonnelSheet").then((m) => ({ default: m.PersonnelSheet })));

/**
 * The Archives: the game's network chart, every operator's file, the roster
 * counted up and, behind the spoilers setting, the hidden factions. The
 * setting lives in localStorage, so SSR and the first client render read it
 * as off and the hidden tab arrives after hydration, never as a mismatch.
 */
export function Archives(): React.ReactElement {
    const t = useArchivesT();
    const search = useSearch({ from: "/archives" });
    const navigate = useNavigate({ from: "/archives" });
    const spoilers = useSelector(themeStore, (s) => s.spoilers);
    const tab: ArchivesTab = search.tab ?? "network";
    const queryClient = useQueryClient();
    const server = useGamedataServer();
    // The sheet mounts with the first file opened and stays, so closing it still animates.
    const [sheetMounted, setSheetMounted] = useState(search.id !== undefined);
    if (search.id !== undefined && !sheetMounted) setSheetMounted(true);
    // Hovering or focusing the tab warms its chunk and its data, so the click finds both.
    const warmNumbers = useCallback(() => {
        void loadNumbersTab();
        void queryClient.prefetchQuery(archiveDemographicsQueryOptions(server));
    }, [queryClient, server]);
    const warmDossiers = useCallback(() => void queryClient.prefetchQuery(archiveDossiersQueryOptions(server)), [queryClient, server]);

    const setSearch = useCallback(
        (patch: Partial<IArchivesSearch>, replace = false) => {
            void navigate({ search: (prev: IArchivesSearch) => ({ ...prev, ...patch }), replace, resetScroll: false });
        },
        [navigate],
    );
    const openFile = useCallback((id: string) => setSearch({ id }), [setSearch]);
    const openChart = useCallback((chart: string | undefined) => setSearch({ tab: undefined, chart, id: undefined }), [setSearch]);

    return (
        <section className="mx-auto flex w-full max-w-330 flex-col gap-5 px-8 pt-7 pb-15 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
            <PageHeader title={t("page.title")} description={t("page.description")} />

            <Tabs value={tab} onValueChange={(v) => v && setSearch({ tab: v === "network" ? undefined : (v as ArchivesTab) })}>
                <div className="flex flex-col gap-2">
                    <TabsList aria-label={t("tabs.label")} className="w-full overflow-x-auto sm:w-auto">
                        <TabsTab className="pointer-coarse:h-11" value="network">
                            {t("tabs.network")}
                        </TabsTab>
                        <TabsTab className="pointer-coarse:h-11" value="dossiers" onPointerEnter={warmDossiers} onFocus={warmDossiers}>
                            {t("tabs.dossiers")}
                        </TabsTab>
                        <TabsTab className="pointer-coarse:h-11" value="numbers" onPointerEnter={warmNumbers} onFocus={warmNumbers}>
                            {t("tabs.numbers")}
                        </TabsTab>
                        {spoilers ? (
                            <TabsTab className="pointer-coarse:h-11" value="hidden">
                                {t("tabs.hidden")}
                            </TabsTab>
                        ) : null}
                    </TabsList>
                    {spoilers ? null : <SpoilerNote className="text-[12px]" />}
                </div>
                <TabsPanel value="network" className="mt-4">
                    <NetworkTab chart={search.chart} onChart={(chart) => setSearch({ chart })} onOpenFile={openFile} />
                </TabsPanel>
                <TabsPanel value="dossiers" className="mt-4">
                    <DossiersTab onOpenFile={openFile} />
                </TabsPanel>
                <TabsPanel value="numbers" className="mt-4">
                    <Suspense fallback={null}>
                        <NumbersTab onOpenFile={openFile} />
                    </Suspense>
                </TabsPanel>
                <TabsPanel value="hidden" className="mt-4">
                    {spoilers ? <HiddenTab onOpenFile={openFile} /> : <SpoilerNote className="rounded-[14px] border border-border bg-card p-5" />}
                </TabsPanel>
            </Tabs>

            {sheetMounted ? (
                <Suspense fallback={null}>
                    <PersonnelSheet id={search.id} onClose={() => setSearch({ id: undefined })} onOpenFile={openFile} onOpenChart={openChart} fromDossiers={tab === "dossiers"} />
                </Suspense>
            ) : null}
        </section>
    );
}
