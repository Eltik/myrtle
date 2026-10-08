import { useQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { SlidersHorizontalIcon } from "lucide-react";
import { lazy, Suspense, useCallback, useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { publicPlansQueryOptions } from "#/lib/api/planner";
import { userEncounteredEnemiesQueryOptions, userImprovementsQueryOptions, userInventoryQueryOptions, userQueryOptions, userRosterQueryOptions, userScoreQueryOptions } from "#/lib/api/user";
import { type TypedRichT, useGamedataServer, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Hero } from "./impl/components/Hero";
import { ProfileLayoutEditor } from "./impl/components/ProfileLayoutEditor";
import type { messages as layoutMessages } from "./impl/components/ProfileLayoutEditor.messages";
import { ProfileTabs } from "./impl/components/ProfileTabs";
import { StatStrip } from "./impl/components/StatStrip";
import { ScoreTabSkeleton } from "./impl/components/tabs/Score/ScoreTabSkeleton";
import { rosterAccess } from "./impl/components/tabs/Showcase/favourites";
import { StatsTab } from "./impl/components/tabs/Stats/StatsTab";
import { DynamicArtProvider } from "./impl/dynamic-art";
import { resolveActiveTab, shownTabs, tabMemory } from "./impl/layout";
import { isTabId, type TabId } from "./impl/types";
import type { messages } from "./UserProfile.messages";

const SKELETON_TAG_WIDTHS = [
    { id: "tag-1", width: 64 },
    { id: "tag-2", width: 80 },
    { id: "tag-3", width: 72 },
    { id: "tag-4", width: 88 },
    { id: "tag-5", width: 76 },
    { id: "tag-6", width: 84 },
    { id: "tag-7", width: 80 },
    { id: "tag-8", width: 92 },
] as const;

const SKELETON_GRID_IDS = Array.from({ length: 20 }, (_, i) => `grid-${i}`);

// Stats is the default tab and stays in the route chunk; the rest load on first visit so
// their dependencies (recharts via the Score history card, above all) stay out of it.
const loadEnemiesTab = () => import("./impl/components/tabs/Enemies/EnemiesTab");
const loadItemsTab = () => import("./impl/components/tabs/Items/ItemsTab");
const loadOptimizerTab = () => import("./impl/components/tabs/Optimizer/OptimizerTab");
const loadPlansTab = () => import("./impl/components/tabs/Plans/PlansTab");
const loadRosterTab = () => import("./impl/components/tabs/Roster/RosterTab");
const loadScoreTab = () => import("./impl/components/tabs/Score/ScoreTab");
const loadShowcaseTab = () => import("./impl/components/tabs/Showcase/ShowcaseTab");
const EnemiesTab = lazy(() => loadEnemiesTab().then((m) => ({ default: m.EnemiesTab })));
const ItemsTab = lazy(() => loadItemsTab().then((m) => ({ default: m.ItemsTab })));
const OptimizerTab = lazy(() => loadOptimizerTab().then((m) => ({ default: m.OptimizerTab })));
const PlansTab = lazy(() => loadPlansTab().then((m) => ({ default: m.PlansTab })));
const RosterTab = lazy(() => loadRosterTab().then((m) => ({ default: m.RosterTab })));
const ScoreTab = lazy(() => loadScoreTab().then((m) => ({ default: m.ScoreTab })));
const ShowcaseTab = lazy(() => loadShowcaseTab().then((m) => ({ default: m.ShowcaseTab })));
// The editor pulls in the grids' entity picker and the gallery; only an owner who opens it pays for that.
const BackgroundEditor = lazy(() => import("./impl/background-editor/BackgroundEditor").then((m) => ({ default: m.BackgroundEditor })));

const TAB_CHUNKS: Partial<Record<TabId, () => Promise<unknown>>> = {
    enemies: loadEnemiesTab,
    inventory: loadItemsTab,
    optimizer: loadOptimizerTab,
    plans: loadPlansTab,
    roster: loadRosterTab,
    score: loadScoreTab,
    showcase: loadShowcaseTab,
};

/** Tabs that render phases, skills or template ids and so need the full operator table. */
const FULL_TABLE_TABS: ReadonlySet<TabId> = new Set<TabId>(["roster", "optimizer"]);

function GridSkeleton() {
    return (
        <div className="grid grid-cols-4 gap-2.5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
            {SKELETON_GRID_IDS.map((id) => (
                <Skeleton className="aspect-3/4 w-full rounded-xl" key={id} />
            ))}
        </div>
    );
}

/** The page while the profile record loads: the hero, stat strip, tab bar and the Stats tab's first rows. */
function ProfileSkeleton() {
    return (
        <main className="page-shell flex flex-1 flex-col gap-7 [--page-max:1440px]">
            <div className="relative h-48 w-full overflow-hidden rounded-3xl border border-border/50 bg-card/40">
                <Skeleton className="absolute inset-0 rounded-3xl opacity-60" />
                <div className="relative flex h-full items-center gap-6 p-6">
                    <Skeleton className="h-28 w-28 shrink-0 rounded-2xl" />
                    <div className="flex min-w-0 flex-1 flex-col gap-3">
                        <div className="flex items-center gap-2">
                            <Skeleton className="h-4 w-16 rounded-md" />
                            <Skeleton className="h-5 w-20 rounded-full" />
                            <Skeleton className="h-5 w-24 rounded-full" />
                        </div>
                        <Skeleton className="h-8 w-48 rounded-lg" />
                        <Skeleton className="h-4 w-40 rounded-md" />
                        <Skeleton className="mt-1 h-4 w-36 rounded-md" />
                    </div>
                    <div className="hidden items-center gap-2 self-start sm:flex">
                        <Skeleton className="h-9 w-24 rounded-lg" />
                        <Skeleton className="h-9 w-9 rounded-lg" />
                    </div>
                </div>
            </div>
            <div className="grid grid-cols-2 gap-3.5 md:grid-cols-4">
                {[0, 1, 2, 3].map((i) => (
                    <div key={i} className="flex h-28 w-full flex-col gap-3 rounded-2xl border border-border/50 bg-card/40 p-5">
                        <Skeleton className="h-3 w-16 rounded-md" />
                        <Skeleton className="h-8 w-28 rounded-lg" />
                        <Skeleton className="mt-auto h-3 w-24 rounded-md" />
                    </div>
                ))}
            </div>
            <div className="flex items-center gap-6 border-border/50 border-b pt-2">
                {[0, 1, 2, 3].map((i) => (
                    <div key={i} className="flex items-center gap-2 pb-3">
                        <Skeleton className="h-4 w-16 rounded-md" />
                        <Skeleton className="h-5 w-8 rounded-md" />
                    </div>
                ))}
            </div>
            <div className="flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Skeleton className="h-6 w-20 rounded-md" />
                        <Skeleton className="h-5 w-24 rounded-full" />
                    </div>
                    <div className="flex items-center gap-2">
                        <Skeleton className="h-4 w-10 rounded-md" />
                        <Skeleton className="h-8 w-24 rounded-lg" />
                    </div>
                </div>
                <div className="flex flex-wrap gap-2">
                    {SKELETON_TAG_WIDTHS.map(({ id, width }) => (
                        <Skeleton className="h-8 rounded-full" key={id} style={{ width: `${width}px` }} />
                    ))}
                </div>
            </div>
            <GridSkeleton />
        </main>
    );
}

function ProfileNotFound({ id }: { id: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const rt: TypedRichT<typeof messages> = useRichT("user");
    return (
        <main className="page-shell flex flex-1 flex-col gap-7 [--page-max:1440px]">
            <div className="flex flex-col items-center justify-center gap-3 rounded-3xl border border-border bg-card px-8 py-16 text-center">
                <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{t("profile.notFound.eyebrow")}</span>
                <h1 className="font-bold text-2xl tracking-tight">{t("profile.notFound.title")}</h1>
                <p className="max-w-sm text-muted-foreground text-sm">{rt("profile.notFound.desc", { id: <code className="font-mono">{id}</code> })}</p>
            </div>
        </main>
    );
}

export function UserProfile() {
    const t: TypedT<typeof messages> = useT("user");
    const lt: TypedT<typeof layoutMessages> = useT("user");
    const { id } = useParams({ from: "/user/$id" });
    const { user } = useAuth();
    // Survives leaving the page, so going away from Roster and back does not reset to Stats.
    // Stored per browser, not per profile: the tab is a way of reading a profile.
    const [storedTab, setStoredTab] = useLocalStorageState<TabId>("user:profile:tab", "stats", {
        parse: (raw) => (isTabId(raw) ? raw : undefined),
        serialize: (v) => v,
    });
    // A visitor's pick on a customized profile, kept for this visit only (see `tabMemory`).
    const [pickedTab, setPickedTab] = useState<TabId | null>(null);
    const [editingLayout, setEditingLayout] = useState(false);
    // The background editor is open. Its draft lives in the editor, previewed there; the page's header keeps the saved background.
    const [editingBackground, setEditingBackground] = useState(false);

    // The profile record and roster power the hero, stat strip and several tab
    // counts, so both stay eager. Everything else is fetched once its tab first
    // becomes active.
    const { data, isLoading } = useQuery(userQueryOptions(id));

    // With no layout saved (`null`) every line below reduces to the default page:
    // all tabs in canonical order, the remembered tab, the roster fetched for everyone.
    const layout = data?.profile_layout ?? null;
    const isOwner = Boolean(user && data && user.id === data.id);
    const memory = tabMemory(layout, isOwner);
    const shown = useMemo(() => shownTabs(layout, isOwner), [layout, isOwner]);
    const activeTab = resolveActiveTab(shown, memory, storedTab, pickedTab);
    const setActiveTab = memory === "stored" ? setStoredTab : setPickedTab;
    // A visitor whose owner hid the Roster tab is refused `/roster`, so it is not asked.
    const canReadRoster = memory === "stored" || shown.includes("roster");

    const { data: roster } = useQuery({ ...userRosterQueryOptions(id), enabled: canReadRoster });
    const showcaseRosterAccess = useMemo(() => rosterAccess(canReadRoster, roster), [canReadRoster, roster]);
    const gamedataServer = useGamedataServer();
    // The default Stats tab reads only the slim index, so the full table (23.9 MB
    // raw) waits for a tab that renders phases, skills or template ids.
    const { data: operatorsIndex } = useQuery({ ...operatorsIndexQueryOptions(gamedataServer), enabled: activeTab === "stats" || activeTab === "roster" });
    // A tab's chunk loads before the full table is asked for. bun blocks for about 2 s
    // while it serializes the table, and a chunk requested alongside it queues behind
    // that, so the tab could not render until the table had arrived. Hover or focus
    // starts the chunk early; the table itself waits for the click, so passing the
    // pointer over a tab does not download it.
    const [loadedChunks, setLoadedChunks] = useState<ReadonlySet<TabId>>(() => new Set());
    const loadTabChunk = useCallback((tab: TabId) => {
        const load = TAB_CHUNKS[tab];
        if (!load) return;
        void load().then(() => setLoadedChunks((prev) => (prev.has(tab) ? prev : new Set(prev).add(tab))));
    }, []);
    useEffect(() => {
        if (activeTab) loadTabChunk(activeTab);
    }, [activeTab, loadTabChunk]);
    const prefetchTab = useCallback(
        (tab: TabId) => {
            if (tab !== activeTab) loadTabChunk(tab);
        },
        [activeTab, loadTabChunk],
    );

    const { data: inventory } = useQuery({ ...userInventoryQueryOptions(id), enabled: activeTab === "inventory" });
    const { data: score, isLoading: isScoreLoading } = useQuery({ ...userScoreQueryOptions(id), enabled: activeTab === "score" });
    const { data: publicPlans } = useQuery({ ...publicPlansQueryOptions(id), enabled: activeTab === "plans" });
    const { data: operatorsStatic } = useQuery({
        ...operatorsListQueryOptions(gamedataServer),
        enabled: activeTab !== null && FULL_TABLE_TABS.has(activeTab) && loadedChunks.has(activeTab),
    });
    // Improvements only fire while the Score tab is mounted - it's a heavier
    // payload than the headline score, so don't pay for it on every profile view.
    const { data: improvements, isLoading: isImprovementsLoading } = useQuery({
        ...userImprovementsQueryOptions(id),
        enabled: activeTab === "score",
    });
    const { data: encounteredEnemies, isLoading: isEnemiesLoading } = useQuery({ ...userEncounteredEnemiesQueryOptions(id), enabled: activeTab === "enemies" });

    const labels = useMemo<Record<TabId, string>>(
        () => ({
            showcase: t("profile.tab.showcase"),
            stats: t("profile.tab.stats"),
            score: t("profile.tab.score"),
            roster: t("profile.tab.roster"),
            plans: t("profile.tab.plans"),
            inventory: t("profile.tab.inventory"),
            enemies: t("profile.tab.enemies"),
            optimizer: t("profile.tab.optimizer"),
        }),
        [t],
    );
    const tabs = useMemo(() => {
        const counts: Partial<Record<TabId, number>> = {
            roster: data?.operator_count ?? roster?.length ?? undefined,
            plans: publicPlans?.length,
            inventory: data?.item_count ?? inventory?.length ?? undefined,
            enemies: encounteredEnemies?.encounteredCount ?? undefined,
        };
        // Only the owner is sent the private entries, so only they see the marker.
        const hidden = new Set(layout?.tabs.filter((tab) => !tab.visible).map((tab) => tab.id));
        return shown.map((tabId) => ({ id: tabId, label: labels[tabId], count: counts[tabId], private: hidden.has(tabId) }));
    }, [shown, labels, layout, data, roster, inventory, encounteredEnemies, publicPlans]);

    if (isLoading) return <ProfileSkeleton />;
    if (!data) return <ProfileNotFound id={id} />;

    return (
        <DynamicArtProvider server={data.server}>
            <main className="page-shell flex flex-1 flex-col gap-7 [--page-max:1440px]">
                <Hero profile={data} background={layout?.background ?? null} onChangeBackground={isOwner ? () => setEditingBackground(true) : undefined} />
                {isOwner && editingBackground && (
                    <Suspense fallback={null}>
                        <BackgroundEditor profile={data} saved={layout?.background ?? null} onClose={() => setEditingBackground(false)} />
                    </Suspense>
                )}
                <StatStrip profile={data} rosterCount={roster?.length} />
                {editingLayout && isOwner ? (
                    <ProfileLayoutEditor profile={data} labels={labels} onClose={() => setEditingLayout(false)} />
                ) : (
                    shown.length > 0 && (
                        <ProfileTabs
                            tabs={tabs}
                            active={activeTab}
                            onChange={setActiveTab}
                            onIntent={prefetchTab}
                            end={
                                isOwner && (
                                    <Button type="button" variant="ghost" size="sm" onClick={() => setEditingLayout(true)}>
                                        <SlidersHorizontalIcon />
                                        {lt("profile.layout.customize")}
                                    </Button>
                                )
                            }
                        />
                    )
                )}
                {shown.length === 0 && (
                    <div className="flex flex-col items-center justify-center gap-2 rounded-3xl border border-border bg-card px-8 py-12 text-center">
                        <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{t("profile.noTabs.eyebrow")}</span>
                        <p className="max-w-sm text-muted-foreground text-sm">{t("profile.noTabs.desc")}</p>
                    </div>
                )}
                {activeTab === "stats" && <StatsTab nonDefaultSkinCount={data.non_default_skin_count} operatorsIndex={operatorsIndex ?? []} roster={roster ?? []} server={data.server} uid={id} rosterPrivate={!canReadRoster} />}
                {activeTab !== null && activeTab !== "stats" && (
                    <Suspense fallback={activeTab === "score" ? <ScoreTabSkeleton /> : <GridSkeleton />}>
                        {activeTab === "roster" && <RosterTab roster={roster ?? []} operatorsIndex={operatorsIndex ?? []} operatorsStatic={operatorsStatic ?? []} />}
                        {activeTab === "inventory" && <ItemsTab inventory={inventory ?? []} />}
                        {activeTab === "plans" && <PlansTab uid={id} roster={roster ?? []} />}
                        {activeTab === "enemies" && <EnemiesTab encountered={encounteredEnemies} isLoading={isEnemiesLoading} />}
                        {activeTab === "score" && <ScoreTab score={score} isLoading={isScoreLoading} improvements={improvements} isImprovementsLoading={isImprovementsLoading} uid={id} server={data.server} />}
                        {activeTab === "showcase" && <ShowcaseTab uid={id} profile={data} isOwner={isOwner} roster={roster ?? []} rosterAccess={showcaseRosterAccess} />}
                        {activeTab === "optimizer" && <OptimizerTab uid={id} roster={roster ?? []} operatorsStatic={operatorsStatic ?? []} />}
                    </Suspense>
                )}
            </main>
        </DynamicArtProvider>
    );
}
