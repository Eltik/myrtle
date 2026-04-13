"use client";

import { Loader2 } from "lucide-react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { useUserStats } from "~/hooks/use-user-stats";
import { CollectionOverviewCard } from "./impl/collection-overview-card";
import { EliteBreakdownCard } from "./impl/elite-breakdown-card";
import { MasteryStatsCard } from "./impl/mastery-stats-card";
import { ModuleSkinStatsCard } from "./impl/module-skin-stats-card";
import { ProfessionCompletionCard } from "./impl/profession-completion-card";

interface StatsViewProps {
    userId: string;
}

export function StatsView({ userId }: StatsViewProps) {
    const { stats, isLoading, error } = useUserStats(userId);

    if (isLoading) {
        return (
            <div className="flex min-h-100 items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex min-h-100 flex-col items-center justify-center text-center">
                <p className="text-destructive">Failed to load stats data</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">{error}</p>
            </div>
        );
    }

    if (!stats) {
        return (
            <div className="flex flex-col items-center justify-center py-16 text-center">
                <p className="text-muted-foreground">Stats data is not available yet.</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">Check back later for your profile analysis.</p>
            </div>
        );
    }

    return (
        <div className="grid gap-4 pb-8 sm:grid-cols-2">
            {/* Row 1: Collection Overview | Elite Breakdown */}
            <InView
                once
                transition={{ duration: 0.5, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <CollectionOverviewCard className="h-full" collectionPercentage={stats.collectionPercentage} totalAvailable={stats.totalAvailable} totalOwned={stats.totalOwned} />
            </InView>

            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.1 }}
                variants={{
                    hidden: { opacity: 0, x: 15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <EliteBreakdownCard className="h-full" e0={stats.eliteBreakdown.e0} e1={stats.eliteBreakdown.e1} e2={stats.eliteBreakdown.e2} total={stats.eliteBreakdown.total} />
            </InView>

            {/* Row 2: Profession Completion (full width) */}
            <div className="sm:col-span-2">
                <InView
                    once
                    transition={{ duration: 0.4, ease: "easeOut", delay: 0.15 }}
                    variants={{
                        hidden: { opacity: 0, y: 15 },
                        visible: { opacity: 1, y: 0 },
                    }}
                    viewOptions={{ margin: "-50px 0px" }}
                >
                    <ProfessionCompletionCard professions={stats.professions} />
                </InView>
            </div>

            {/* Row 3: Mastery | Modules & Skins */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.2 }}
                variants={{
                    hidden: { opacity: 0, x: -15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <MasteryStatsCard className="h-full" m3Count={stats.masteries.m3Count} m6Count={stats.masteries.m6Count} m9Count={stats.masteries.m9Count} maxPossibleMasteryLevels={stats.masteries.maxPossibleMasteryLevels} totalMasteryLevels={stats.masteries.totalMasteryLevels} />
            </InView>

            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.25 }}
                variants={{
                    hidden: { opacity: 0, x: 15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <ModuleSkinStatsCard className="h-full" modules={stats.modules} skins={stats.skins} />
            </InView>
        </div>
    );
}
