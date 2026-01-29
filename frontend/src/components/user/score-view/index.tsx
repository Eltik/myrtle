"use client";

import { Loader2 } from "lucide-react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { useUserScore } from "~/hooks/use-user-score";
import { ActivityMetricsCard } from "./impl/activity-metrics-card";
import { EngagementMetricsCard } from "./impl/engagement-metrics-card";
import { HeroGradeDisplay } from "./impl/hero-grade-display";
import { ScoreBreakdownGrid } from "./impl/score-breakdown-grid";
import { ScoreOverviewCard } from "./impl/score-overview-card";

interface ScoreViewProps {
    userId: string;
}

export function ScoreView({ userId }: ScoreViewProps) {
    const { score: scoreData, isLoading, error } = useUserScore(userId);

    if (isLoading) {
        return (
            <div className="flex min-h-[400px] items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex min-h-[400px] flex-col items-center justify-center text-center">
                <p className="text-destructive">Failed to load score data</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">{error}</p>
            </div>
        );
    }

    if (!scoreData) {
        return (
            <div className="flex flex-col items-center justify-center py-16 text-center">
                <p className="text-muted-foreground">Score data is not available yet.</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">Check back later for your profile analysis.</p>
            </div>
        );
    }

    return (
        <div className="grid gap-4 pb-8 sm:grid-cols-2">
            {/* Row 1: Grade | Total Score */}
            <InView
                once
                transition={{ duration: 0.5, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <div className="h-full rounded-xl border border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <HeroGradeDisplay grade={scoreData.grade} />
                </div>
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
                <ScoreOverviewCard className="h-full" scoreData={scoreData} />
            </InView>

            {/* Row 2: Engagement | Activity (equal height) */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.15 }}
                variants={{
                    hidden: { opacity: 0, x: -15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <EngagementMetricsCard className="h-full" grade={scoreData.grade} />
            </InView>

            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.2 }}
                variants={{
                    hidden: { opacity: 0, x: 15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <ActivityMetricsCard className="h-full" grade={scoreData.grade} />
            </InView>

            {/* Row 3: Score Breakdown (full width) */}
            <div className="sm:col-span-2">
                <InView
                    once
                    transition={{ duration: 0.4, ease: "easeOut", delay: 0.25 }}
                    variants={{
                        hidden: { opacity: 0, y: 15 },
                        visible: { opacity: 1, y: 0 },
                    }}
                    viewOptions={{ margin: "-50px 0px" }}
                >
                    <ScoreBreakdownGrid scoreData={scoreData} />
                </InView>
            </div>
        </div>
    );
}
