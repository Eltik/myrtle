"use client";

import { BarChart3 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import type { StoredUserScore } from "~/types/api/impl/user";
import { SCORE_CATEGORY_CONFIG } from "./constants";
import { ScoreCategoryBar } from "./score-category-bar";

interface ScoreBreakdownGridProps {
    scoreData: StoredUserScore;
}

// Get completion info for a category if available from actual game data
function getCompletionInfo(key: string, scoreData: StoredUserScore): { percentage: number; label: string } | null {
    const summary = scoreData.completionSummary;

    if (!summary) return null;

    switch (key) {
        case "operatorScore":
            return summary.operators?.percentage > 0 ? { percentage: summary.operators.percentage, label: `${summary.operators.current}/${summary.operators.maximum} operators` } : null;
        case "stageScore":
            return summary.stages?.percentage > 0 ? { percentage: summary.stages.percentage, label: `${summary.stages.current}/${summary.stages.maximum} stages` } : null;
        case "roguelikeScore":
            // Use consistent percentage from summary for both bar and label
            return summary.roguelike?.percentage > 0 ? { percentage: summary.roguelike.percentage, label: `${summary.roguelike.current}/${summary.roguelike.maximum} collectibles` } : null;
        case "sandboxScore":
            return summary.sandbox?.percentage > 0 ? { percentage: summary.sandbox.percentage, label: `${summary.sandbox.current}/${summary.sandbox.maximum} places` } : null;
        case "medalScore":
            return summary.medals?.percentage > 0 ? { percentage: summary.medals.percentage, label: `${summary.medals.current}/${summary.medals.maximum} medals` } : null;
        default:
            // No completion data for baseScore (efficiency metric, not completable)
            return null;
    }
}

export function ScoreBreakdownGrid({ scoreData }: ScoreBreakdownGridProps) {
    const categories = Object.entries(SCORE_CATEGORY_CONFIG);

    return (
        <Card className="border-border/50 bg-linear-to-b from-card/60 to-card/40 backdrop-blur-sm">
            <CardHeader className="pb-4">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <BarChart3 className="h-4 w-4" />
                    Score Breakdown
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-5">
                {categories.map(([key, config], index) => {
                    const completionInfo = getCompletionInfo(key, scoreData);
                    return (
                        <ScoreCategoryBar
                            bgColor={config.bgColor}
                            color={config.color}
                            completionInfo={completionInfo ?? undefined}
                            delay={index * 0.08}
                            description={config.description}
                            icon={config.icon}
                            key={key}
                            label={config.label}
                            progressColor={config.progressColor}
                            score={scoreData[key as keyof StoredUserScore] as number}
                        />
                    );
                })}
            </CardContent>
        </Card>
    );
}
