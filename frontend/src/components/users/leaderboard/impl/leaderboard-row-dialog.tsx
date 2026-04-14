"use client";

import { ExternalLink, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Link from "next/link";
import { useCallback, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { LeaderboardEntry } from "~/types/api";
import { GRADE_COLORS, getAvatarURL, SCORE_CATEGORIES } from "./constants";
import { GradeBadge } from "./grade-badge";
import { RankBadge } from "./rank-badge";

interface LeaderboardRowDialogProps {
    entry: LeaderboardEntry | null;
    onClose: () => void;
    isLoading?: boolean;
}

export function LeaderboardRowDialog({ entry, onClose, isLoading }: LeaderboardRowDialogProps) {
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!entry) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") onClose();
        };

        document.addEventListener("keydown", handleKeyDown);
        document.body.style.overflow = "hidden";

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            document.body.style.overflow = "";
        };
    }, [entry, onClose]);

    const handleBackdropClick = useCallback(
        (e: React.MouseEvent) => {
            if (e.target === e.currentTarget) onClose();
        },
        [onClose],
    );

    if (typeof window === "undefined") return null;

    return createPortal(
        <AnimatePresence mode="wait">
            {entry && (
                <>
                    {/* Backdrop */}
                    <motion.div animate={{ opacity: 1 }} className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm" exit={{ opacity: 0 }} initial={{ opacity: 0 }} onClick={handleBackdropClick} />

                    {/* Dialog */}
                    <motion.div animate={{ opacity: 1, scale: 1, y: 0 }} className="fixed inset-0 z-50 flex items-center justify-center p-4" exit={{ opacity: 0, scale: 0.95, y: 10 }} initial={{ opacity: 0, scale: 0.95, y: 10 }} onClick={handleBackdropClick} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                        <motion.div className="relative max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-xl border bg-card p-6 shadow-xl" layoutId="dialog-content" onClick={(e) => e.stopPropagation()} ref={containerRef}>
                            <DialogContent entry={entry} isLoading={isLoading} onClose={onClose} />
                        </motion.div>
                    </motion.div>
                </>
            )}
        </AnimatePresence>,
        document.body,
    );
}

function DialogContent({ entry, onClose, isLoading }: { entry: LeaderboardEntry; onClose: () => void; isLoading?: boolean }) {
    const nickname = entry.nickname ?? "Unknown";
    const scores = [
        { key: "operator_score", value: entry.operator_score ?? 0 },
        { key: "stage_score", value: entry.stage_score ?? 0 },
        { key: "roguelike_score", value: entry.roguelike_score ?? 0 },
        { key: "sandbox_score", value: entry.sandbox_score ?? 0 },
        { key: "medal_score", value: entry.medal_score ?? 0 },
        { key: "base_score", value: entry.base_score ?? 0 },
        { key: "skin_score", value: entry.skin_score ?? 0 },
    ];

    const gradeColors = GRADE_COLORS[entry.grade ?? "F"] ?? GRADE_COLORS.F;

    return (
        <>
            <button className="absolute top-4 right-4 z-10 rounded-md p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground" onClick={onClose} type="button">
                <X className="h-5 w-5" />
            </button>

            {/* Header */}
            <motion.div animate={{ opacity: 1, y: 0 }} className="mb-6" initial={{ opacity: 0, y: -10 }} transition={{ delay: 0.05 }}>
                <div className="flex items-start gap-4">
                    <Avatar className="h-16 w-16 border-2 border-border">
                        <AvatarImage alt={nickname} src={getAvatarURL(entry.avatar_id)} />
                        <AvatarFallback className="text-lg">{nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                    </Avatar>
                    <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                            <h2 className="truncate font-bold text-xl">{nickname}</h2>
                            <Badge className="uppercase" variant="secondary">
                                {entry.server}
                            </Badge>
                        </div>
                        <div className="mt-1 flex items-center gap-3 text-muted-foreground text-sm">
                            <span>Level {entry.level}</span>
                            <span>•</span>
                            <div className="flex items-center gap-1.5">
                                <span>Rank</span>
                                <RankBadge rank={entry.rank_global ?? 0} />
                            </div>
                            {entry.rank_server && (
                                <>
                                    <span>•</span>
                                    <span>Server #{entry.rank_server}</span>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </motion.div>

            {/* Grade and Total Score */}
            <motion.div animate={{ opacity: 1, y: 0 }} className="mb-6" initial={{ opacity: 0, y: 10 }} transition={{ delay: 0.1 }}>
                <div className={cn("rounded-lg border p-4", gradeColors.bg, gradeColors.border)}>
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <GradeBadge grade={entry.grade ?? "F"} size="lg" />
                            <div>
                                <p className="text-muted-foreground text-xs uppercase tracking-wide">Grade</p>
                                <p className={cn("font-bold text-lg", gradeColors.text)}>{getGradeLabel(entry.grade ?? "F")}</p>
                            </div>
                        </div>
                        <div className="text-right">
                            <p className="text-muted-foreground text-xs uppercase tracking-wide">Total Score</p>
                            <p className="font-bold font-mono text-2xl">
                                <AnimatedNumber springOptions={{ bounce: 0, duration: 800 }} value={entry.total_score ?? 0} />
                            </p>
                        </div>
                    </div>
                </div>
            </motion.div>

            {/* Score Breakdown */}
            <motion.div animate={{ opacity: 1, y: 0 }} className="space-y-4" initial={{ opacity: 0, y: 10 }} transition={{ delay: 0.15 }}>
                <h3 className="font-medium text-sm">Score Breakdown</h3>
                <div className="space-y-2">
                    {scores.map((score, index) => {
                        const category = SCORE_CATEGORIES.find((c) => c.key === score.key);

                        return (
                            <motion.div animate={{ opacity: 1, x: 0 }} initial={{ opacity: 0, x: -10 }} key={score.key} transition={{ delay: 0.2 + index * 0.03 }}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <div className="flex cursor-help items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-secondary/50">
                                            <span className="text-muted-foreground text-sm">{category?.label ?? score.key}</span>
                                            <span className="font-mono text-sm">{score.value.toLocaleString()}</span>
                                        </div>
                                    </TooltipTrigger>
                                    <TooltipContent className="z-9999" side="top" variant="dark">
                                        {category?.description}
                                    </TooltipContent>
                                </Tooltip>
                            </motion.div>
                        );
                    })}
                </div>
            </motion.div>

            {/* View Profile Button */}
            <motion.div animate={{ opacity: 1 }} className="mt-6" initial={{ opacity: 0 }} transition={{ delay: 0.45 }}>
                <Button asChild className="w-full">
                    <Link href={`/user/${entry.uid}`}>
                        View Full Profile
                        <ExternalLink className="ml-2 h-4 w-4" />
                    </Link>
                </Button>
            </motion.div>
        </>
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
