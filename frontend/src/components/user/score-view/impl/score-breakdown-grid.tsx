"use client";

import { BarChart3 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import type { StoredUserScore } from "~/types/api/impl/user";
import { SCORE_CATEGORY_CONFIG } from "./constants";
import { ScoreCategoryBar } from "./score-category-bar";

interface ScoreBreakdownGridProps {
    scoreData: StoredUserScore;
}

// Approximate max values for percentage calculation (based on typical high scores)
const MAX_SCORES: Record<string, number> = {
    operatorScore: 500000,
    stageScore: 100000,
    roguelikeScore: 50000,
    sandboxScore: 30000,
    medalScore: 50000,
    baseScore: 20000,
};

export function ScoreBreakdownGrid({ scoreData }: ScoreBreakdownGridProps) {
    const categories = Object.entries(SCORE_CATEGORY_CONFIG);

    return (
        <Card className="border-border/50 bg-gradient-to-b from-card/60 to-card/40 backdrop-blur-sm">
            <CardHeader className="pb-4">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <BarChart3 className="h-4 w-4" />
                    Score Breakdown
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-5">
                {categories.map(([key, config], index) => (
                    <ScoreCategoryBar
                        bgColor={config.bgColor}
                        color={config.color}
                        delay={index * 0.08}
                        description={config.description}
                        icon={config.icon}
                        key={key}
                        label={config.label}
                        maxScore={MAX_SCORES[key] ?? 100000}
                        progressColor={config.progressColor}
                        score={scoreData[key as keyof StoredUserScore] as number}
                    />
                ))}
            </CardContent>
        </Card>
    );
}
