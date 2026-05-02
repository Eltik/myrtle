import { useQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { userInventoryQueryOptions, userQueryOptions, userRosterQueryOptions, userScoreQueryOptions } from "#/lib/api/user";
import { Hero } from "./impl/Hero";
import { ProfileTabs } from "./impl/ProfileTabs";
import { StatStrip } from "./impl/StatStrip";
import { RosterTab } from "./impl/tabs/RosterTab";

export type TabId = "roster" | "inventory" | "stats" | "score";

export function UserProfile() {
    const { id } = useParams({ from: "/user/$id" });
    const [activeTab, setActiveTab] = useState<TabId>("roster");

    const { data, isLoading } = useQuery(userQueryOptions(id));
    const { data: roster } = useQuery(userRosterQueryOptions(id));
    const { data: inventory } = useQuery(userInventoryQueryOptions(id));
    const { data: score } = useQuery(userScoreQueryOptions(id));
    const { data: operatorsIndex } = useQuery(operatorsIndexQueryOptions());

    const tabs = useMemo(
        () => [
            { id: "roster" as TabId, label: "Roster", count: data?.operator_count ?? roster?.length ?? undefined },
            { id: "inventory" as TabId, label: "Inventory", count: data?.item_count ?? inventory?.length ?? undefined },
            { id: "stats" as TabId, label: "Stats" },
            { id: "score" as TabId, label: "Score" },
        ],
        [data, roster, inventory],
    );

    if (isLoading) {
        return (
            <main className="flex-1 w-[min(1080px,calc(100%-2rem))] m-[0_auto] p-[24px_0_64px] flex flex-col gap-7">
                <div className="relative h-48 w-full rounded-3xl border border-border/50 bg-card/40 overflow-hidden">
                    <Skeleton className="absolute inset-0 rounded-3xl opacity-60" />
                    <div className="relative flex items-center gap-6 p-6 h-full">
                        <Skeleton className="h-28 w-28 rounded-2xl shrink-0" />
                        <div className="flex-1 flex flex-col gap-3 min-w-0">
                            <div className="flex items-center gap-2">
                                <Skeleton className="h-4 w-16 rounded-md" />
                                <Skeleton className="h-5 w-20 rounded-full" />
                                <Skeleton className="h-5 w-24 rounded-full" />
                            </div>
                            <Skeleton className="h-8 w-48 rounded-lg" />
                            <Skeleton className="h-4 w-40 rounded-md" />
                            <Skeleton className="h-4 w-36 rounded-md mt-1" />
                        </div>
                        <div className="hidden sm:flex items-center gap-2 self-start">
                            <Skeleton className="h-9 w-24 rounded-lg" />
                            <Skeleton className="h-9 w-9 rounded-lg" />
                        </div>
                    </div>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3.5">
                    {[0, 1, 2, 3].map((i) => (
                        <div key={i} className="h-28 w-full rounded-2xl border border-border/50 bg-card/40 p-5 flex flex-col gap-3">
                            <Skeleton className="h-3 w-16 rounded-md" />
                            <Skeleton className="h-8 w-28 rounded-lg" />
                            <Skeleton className="h-3 w-24 rounded-md mt-auto" />
                        </div>
                    ))}
                </div>
                <div className="flex items-center gap-6 border-b border-border/50 pt-2">
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
                        {[64, 80, 72, 88, 76, 84, 80, 92].map((w, i) => (
                            <Skeleton key={i} className="h-8 rounded-full" style={{ width: `${w}px` }} />
                        ))}
                    </div>
                </div>
                <div className="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-2.5">
                    {Array.from({ length: 20 }).map((_, i) => (
                        <Skeleton key={i} className="aspect-3/4 w-full rounded-xl" />
                    ))}
                </div>
            </main>
        );
    }
    if (!data) {
        return (
            <main className="flex-1 w-[min(1080px,calc(100%-2rem))] m-[0_auto] p-[24px_0_64px] flex flex-col gap-7">
                <div className="flex flex-col items-center justify-center gap-3 rounded-3xl border border-border bg-card px-8 py-16 text-center">
                    <span className="font-mono text-[11px] uppercase tracking-widest text-muted-foreground">Doctor</span>
                    <h1 className="text-2xl font-bold tracking-tight">Profile not found</h1>
                    <p className="max-w-sm text-sm text-muted-foreground">
                        No Doctor with ID <code className="font-mono">{id}</code> exists, or their profile is private.
                    </p>
                </div>
            </main>
        );
    }

    return (
        <main className="flex-1 w-[min(1080px,calc(100%-2rem))] m-[0_auto] p-[24px_0_64px] flex flex-col gap-7">
            <Hero profile={data} />
            <StatStrip profile={data} rosterCount={roster?.length} />
            <ProfileTabs tabs={tabs} active={activeTab} onChange={setActiveTab} />
            {activeTab === "roster" && <RosterTab roster={roster ?? []} operatorsIndex={operatorsIndex ?? []} />}
        </main>
    );
}
