"use client";

import { ExternalLink, X } from "lucide-react";
import Link from "next/link";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogDescription, MorphingDialogTitle } from "~/components/ui/motion-primitives/morphing-dialog";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Progress } from "~/components/ui/shadcn/progress";
import { cn } from "~/lib/utils";
import type { LeaderboardEntry } from "~/types/api";
import { GRADE_COLORS, getAvatarUrl, SCORE_CATEGORIES } from "./constants";
import { GradeBadge } from "./grade-badge";
import { RankBadge } from "./rank-badge";

interface LeaderboardRowDialogProps {
    entry: LeaderboardEntry | null;
    onClose: () => void;
}

export function LeaderboardRowDialog({ entry, onClose }: LeaderboardRowDialogProps) {
    if (!entry) return null;

    const scores = [
        { key: "operatorScore", value: entry.operatorScore },
        { key: "stageScore", value: entry.stageScore },
        { key: "roguelikeScore", value: entry.roguelikeScore },
        { key: "sandboxScore", value: entry.sandboxScore },
        { key: "medalScore", value: entry.medalScore },
        { key: "baseScore", value: entry.baseScore },
    ];

    const maxScore = Math.max(...scores.map((s) => s.value), 1);
    const gradeColors = GRADE_COLORS[entry.grade] || GRADE_COLORS.F;

    return (
        <MorphingDialog onOpenChange={(open) => !open && onClose()} open={!!entry}>
            <MorphingDialogContainer>
                <MorphingDialogContent className="relative w-full max-w-lg rounded-xl border bg-card p-6 shadow-xl">
                    <MorphingDialogClose className="absolute top-4 right-4 rounded-md p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground">
                        <X className="h-5 w-5" />
                    </MorphingDialogClose>

                    {/* Header */}
                    <MorphingDialogTitle className="mb-6">
                        <div className="flex items-start gap-4">
                            <Avatar className="h-16 w-16 border-2 border-border">
                                <AvatarImage alt={entry.nickname} src={getAvatarUrl(entry.avatarId)} />
                                <AvatarFallback className="text-lg">{entry.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                            </Avatar>
                            <div className="min-w-0 flex-1">
                                <div className="flex items-center gap-2">
                                    <h2 className="truncate font-bold text-xl">{entry.nickname}</h2>
                                    <Badge className="uppercase" variant="secondary">
                                        {entry.server}
                                    </Badge>
                                </div>
                                <div className="mt-1 flex items-center gap-3 text-muted-foreground text-sm">
                                    <span>Level {entry.level}</span>
                                    <span>•</span>
                                    <div className="flex items-center gap-1.5">
                                        <span>Rank</span>
                                        <RankBadge rank={entry.rank} />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </MorphingDialogTitle>

                    {/* Grade and Total Score */}
                    <MorphingDialogDescription
                        className="mb-6"
                        variants={{
                            initial: { opacity: 0, y: 10 },
                            animate: { opacity: 1, y: 0 },
                            exit: { opacity: 0, y: 10 },
                        }}
                    >
                        <div className={cn("rounded-lg border p-4", gradeColors.bg, gradeColors.border)}>
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <GradeBadge grade={entry.grade} size="lg" />
                                    <div>
                                        <p className="text-muted-foreground text-xs uppercase tracking-wide">Grade</p>
                                        <p className={cn("font-bold text-lg", gradeColors.text)}>{getGradeLabel(entry.grade)}</p>
                                    </div>
                                </div>
                                <div className="text-right">
                                    <p className="text-muted-foreground text-xs uppercase tracking-wide">Total Score</p>
                                    <p className="font-bold font-mono text-2xl">
                                        <AnimatedNumber springOptions={{ bounce: 0, duration: 800 }} value={entry.totalScore} />
                                    </p>
                                </div>
                            </div>
                        </div>
                    </MorphingDialogDescription>

                    {/* Score Breakdown */}
                    <MorphingDialogDescription
                        className="space-y-4"
                        disableLayoutAnimation
                        variants={{
                            initial: { opacity: 0, y: 10 },
                            animate: { opacity: 1, y: 0, transition: { delay: 0.1 } },
                            exit: { opacity: 0, y: 10 },
                        }}
                    >
                        <h3 className="font-medium text-sm">Score Breakdown</h3>
                        <div className="space-y-3">
                            {scores.map((score) => {
                                const category = SCORE_CATEGORIES.find((c) => c.key === score.key);
                                const percentage = (score.value / maxScore) * 100;

                                return (
                                    <div className="space-y-1.5" key={score.key}>
                                        <div className="flex items-center justify-between text-sm">
                                            <span className="text-muted-foreground">{category?.label || score.key}</span>
                                            <span className="font-mono">{score.value.toLocaleString()}</span>
                                        </div>
                                        <Progress className="h-2" value={percentage} />
                                    </div>
                                );
                            })}
                        </div>

                        {/* Composite Score */}
                        <div className="mt-4 flex items-center justify-between rounded-lg border bg-secondary/30 px-4 py-3">
                            <span className="text-muted-foreground text-sm">Composite Score</span>
                            <span className="font-bold font-mono">{entry.compositeScore.toLocaleString()}</span>
                        </div>
                    </MorphingDialogDescription>

                    {/* View Profile Button */}
                    <MorphingDialogDescription
                        className="mt-6"
                        disableLayoutAnimation
                        variants={{
                            initial: { opacity: 0 },
                            animate: { opacity: 1, transition: { delay: 0.2 } },
                            exit: { opacity: 0 },
                        }}
                    >
                        <Button asChild className="w-full">
                            <Link href={`/user/${entry.uid}`}>
                                View Full Profile
                                <ExternalLink className="ml-2 h-4 w-4" />
                            </Link>
                        </Button>
                    </MorphingDialogDescription>
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
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
            return "Unknown";
    }
}
