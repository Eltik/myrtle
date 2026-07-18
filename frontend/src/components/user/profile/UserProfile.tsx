import { useQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { publicPlansQueryOptions } from "#/lib/api/planner";
import { userEncounteredEnemiesQueryOptions, userImprovementsQueryOptions, userInventoryQueryOptions, userQueryOptions, userRosterQueryOptions, userScoreQueryOptions } from "#/lib/api/user";
import { Hero } from "./impl/components/Hero";
import { ProfileTabs } from "./impl/components/ProfileTabs";
import { StatStrip } from "./impl/components/StatStrip";
import { EnemiesTab } from "./impl/components/tabs/Enemies/EnemiesTab";
import { ItemsTab } from "./impl/components/tabs/Items/ItemsTab";
import { PlansTab } from "./impl/components/tabs/Plans/PlansTab";
import { RosterTab } from "./impl/components/tabs/Roster/RosterTab";
import { ScoreTab } from "./impl/components/tabs/Score/ScoreTab";
import { StatsTab } from "./impl/components/tabs/Stats/StatsTab";
import { DynamicArtProvider } from "./impl/dynamic-art";
import type { TabId } from "./impl/types";

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

export function UserProfile() {
    const { id } = useParams({ from: "/user/$id" });
    const [activeTab, setActiveTab] = useState<TabId>("stats");

    // Genuinely-global data: the profile record + roster power the hero, stat
    // strip, and several tab counts, and the full operator table feeds the
    // default (Stats) tab, so both stay eager. Everything else is gated to the
    // tab that needs it and fetched only once that tab first becomes active.
    const { data, isLoading } = useQuery(userQueryOptions(id));
    const { data: roster } = useQuery(userRosterQueryOptions(id));
    const { data: operatorsStatic } = useQuery(operatorsListQueryOptions());

    const { data: inventory } = useQuery({ ...userInventoryQueryOptions(id), enabled: activeTab === "inventory" });
    const { data: score, isLoading: isScoreLoading } = useQuery({ ...userScoreQueryOptions(id), enabled: activeTab === "score" });
    const { data: publicPlans } = useQuery({ ...publicPlansQueryOptions(id), enabled: activeTab === "plans" });
    // Improvements only fire while the Score tab is mounted - it's a heavier
    // payload than the headline score, so don't pay for it on every profile view.
    const { data: improvements, isLoading: isImprovementsLoading } = useQuery({
        ...userImprovementsQueryOptions(id),
        enabled: activeTab === "score",
    });
    const { data: encounteredEnemies, isLoading: isEnemiesLoading } = useQuery({ ...userEncounteredEnemiesQueryOptions(id), enabled: activeTab === "enemies" });
    const { data: operatorsIndex } = useQuery({ ...operatorsIndexQueryOptions(), enabled: activeTab === "roster" });

    const tabs = useMemo(
        () => [
            { id: "stats" as TabId, label: "Stats" },
            { id: "score" as TabId, label: "Score" },
            {
                id: "roster" as TabId,
                label: "Roster",
                count: data?.operator_count ?? roster?.length ?? undefined,
            },
            { id: "plans" as TabId, label: "Plans", count: publicPlans?.length },
            {
                id: "inventory" as TabId,
                label: "Inventory",
                count: data?.item_count ?? inventory?.length ?? undefined,
            },
            {
                id: "enemies" as TabId,
                label: "Enemies",
                count: encounteredEnemies?.encounteredCount ?? undefined,
            },
        ],
        [data, roster, inventory, encounteredEnemies, publicPlans],
    );

    if (isLoading) {
        return (
            <main className="m-[0_auto] flex w-[min(1440px,calc(100%-2rem))] flex-1 flex-col gap-7 p-[24px_0_64px]">
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
                <div className="grid grid-cols-4 gap-2.5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
                    {SKELETON_GRID_IDS.map((id) => (
                        <Skeleton className="aspect-3/4 w-full rounded-xl" key={id} />
                    ))}
                </div>
            </main>
        );
    }
    if (!data) {
        return (
            <main className="m-[0_auto] flex w-[min(1440px,calc(100%-2rem))] flex-1 flex-col gap-7 p-[24px_0_64px]">
                <div className="flex flex-col items-center justify-center gap-3 rounded-3xl border border-border bg-card px-8 py-16 text-center">
                    <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">Doctor</span>
                    <h1 className="font-bold text-2xl tracking-tight">Profile not found</h1>
                    <p className="max-w-sm text-muted-foreground text-sm">
                        No Doctor with ID <code className="font-mono">{id}</code> exists, or their profile is private.
                    </p>
                </div>
            </main>
        );
    }

    return (
        <DynamicArtProvider server={data.server}>
            <main className="m-[0_auto] flex w-[min(1440px,calc(100%-2rem))] flex-1 flex-col gap-7 p-[24px_0_64px]">
                <Hero profile={data} />
                <StatStrip profile={data} rosterCount={roster?.length} />
                <ProfileTabs tabs={tabs} active={activeTab} onChange={setActiveTab} />
                {activeTab === "roster" && <RosterTab roster={roster ?? []} operatorsIndex={operatorsIndex ?? []} operatorsStatic={operatorsStatic ?? []} />}
                {activeTab === "inventory" && <ItemsTab inventory={inventory ?? []} />}
                {activeTab === "plans" && <PlansTab uid={id} roster={roster ?? []} operatorsStatic={operatorsStatic ?? []} />}
                {activeTab === "enemies" && <EnemiesTab encountered={encounteredEnemies} isLoading={isEnemiesLoading} />}
                {activeTab === "stats" && <StatsTab nonDefaultSkinCount={data.non_default_skin_count} operatorsStatic={operatorsStatic ?? []} roster={roster ?? []} secretary={data.secretary} secretarySkinId={data.secretary_skin_id} server={data.server} uid={id} />}
                {activeTab === "score" && <ScoreTab score={score} isLoading={isScoreLoading} improvements={improvements} isImprovementsLoading={isImprovementsLoading} />}
            </main>
        </DynamicArtProvider>
    );
}
