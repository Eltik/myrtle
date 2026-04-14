"use client";

import { Loader2, Trophy } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { cn } from "~/lib/utils";
import { useUserScore } from "~/hooks/use-user-score";
import { GRADE_CONFIG } from "./impl/constants";

interface ScoreViewProps {
    userId: string;
}

export function ScoreView({ userId }: ScoreViewProps) {
    const { score: scoreData, isLoading, error } = useUserScore(userId);

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

    const grade = scoreData.grade ?? "F";
    const config = GRADE_CONFIG[grade] ?? GRADE_CONFIG.F ?? {
        bg: "bg-slate-500/10",
        text: "text-slate-400",
        border: "border-slate-500/30",
        glow: ["#94a3b8", "#64748b", "#475569"],
        gradient: "from-slate-500/20 to-slate-500/5",
        ringColor: "stroke-slate-500",
    };
    const totalScore = scoreData.total_score ?? 0;

    return (
        <div className="grid gap-4 pb-8 sm:grid-cols-2">
            {/* Grade Display */}
            <InView
                once
                transition={{ duration: 0.5, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <Card className="border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <CardContent className="flex flex-col items-center justify-center gap-6 px-6 py-8">
                        {/* Grade letter */}
                        <div className="relative">
                            <div
                                className="absolute inset-0 rounded-2xl blur-xl"
                                style={{
                                    background: `radial-gradient(circle, ${config.glow[0]} 0%, transparent 70%)`,
                                    opacity: 0.5,
                                }}
                            />
                            <motion.div
                                animate={{ scale: 1, opacity: 1 }}
                                className={cn(
                                    "relative flex h-28 w-28 items-center justify-center rounded-2xl border-2",
                                    config.bg,
                                    config.border,
                                )}
                                initial={{ scale: 0.5, opacity: 0 }}
                                transition={{ type: "spring", stiffness: 200, damping: 20, delay: 0.1 }}
                            >
                                <span className={cn("font-bold text-6xl tracking-tight", config.text)}>
                                    {grade}
                                </span>
                            </motion.div>
                        </div>

                        <motion.div
                            animate={{ y: 0, opacity: 1 }}
                            className="text-center"
                            initial={{ y: 20, opacity: 0 }}
                            transition={{ delay: 0.4, duration: 0.5 }}
                        >
                            <p className={cn("font-bold text-lg", config.text)}>{getGradeLabel(grade)}</p>
                        </motion.div>
                    </CardContent>
                </Card>
            </InView>

            {/* Total Score */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.1 }}
                variants={{
                    hidden: { opacity: 0, x: 15 },
                    visible: { opacity: 1, x: 0 },
                }}
                viewOptions={{ margin: "-50px 0px" }}
            >
                <Card className="flex h-full flex-col border-border/50 bg-gradient-to-tr from-card/70 to-card/50 backdrop-blur-sm">
                    <CardContent className="flex flex-1 flex-col items-center justify-center gap-4 py-8">
                        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                            <Trophy className="h-6 w-6 text-primary" />
                        </div>
                        <div className="text-center">
                            <p className="text-muted-foreground text-sm">Total Score</p>
                            <AnimatedNumber
                                className="font-bold text-4xl tabular-nums tracking-tight"
                                springOptions={{ stiffness: 60, damping: 20 }}
                                value={totalScore}
                            />
                        </div>
                        <p className="text-center text-muted-foreground text-xs">
                            Combined across all scoring categories
                        </p>
                    </CardContent>
                </Card>
            </InView>
        </div>
    );
}

function getGradeLabel(grade: string): string {
    switch (grade) {
        case "S":
            return "Exceptional";
        case "A":
            return "Outstanding";
        case "B":
            return "Great";
        case "C":
            return "Good";
        case "D":
            return "Average";
        case "F":
            return "Beginner";
        default:
            return "Unranked";
    }
}
