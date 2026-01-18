"use client";

import type { LucideIcon } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";

interface ScoreCategoryBarProps {
    label: string;
    description: string;
    score: number;
    icon: LucideIcon;
    color: string;
    bgColor: string;
    progressColor: string;
    delay?: number;
    /** Real completion info from game data (if available) */
    completionInfo?: {
        percentage: number;
        label: string;
    };
}

export function ScoreCategoryBar({ label, description, score, icon: Icon, color, bgColor, progressColor, delay = 0, completionInfo }: ScoreCategoryBarProps) {
    // Only show progress bar if we have real completion data
    const hasCompletionData = !!completionInfo;
    const percentage = completionInfo?.percentage ?? 0;

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <motion.div animate={{ x: 0, opacity: 1 }} className="group -mx-3 cursor-help rounded-lg px-3 py-2 transition-colors hover:bg-muted/40" initial={{ x: -20, opacity: 0 }} transition={{ delay, duration: 0.4, ease: "easeOut" }}>
                    <div className="mb-2 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className={cn("flex h-7 w-7 items-center justify-center rounded-md", bgColor)}>
                                <Icon className={cn("h-4 w-4", color)} />
                            </div>
                            <div>
                                <span className="font-medium text-sm">{label}</span>
                                <p className="hidden text-muted-foreground text-xs sm:block">{description}</p>
                            </div>
                        </div>
                        <div className="flex flex-col items-end gap-0.5">
                            <AnimatedNumber className={cn("font-semibold text-sm tabular-nums", color)} decimals={2} springOptions={{ stiffness: 100, damping: 20 }} value={score} />
                            {completionInfo && <span className="text-[10px] text-muted-foreground">{completionInfo.label}</span>}
                        </div>
                    </div>

                    {/* Progress bar - only shown when we have real completion data */}
                    {hasCompletionData && (
                        <div className="relative h-2 w-full overflow-hidden rounded-full bg-muted/50">
                            <motion.div animate={{ width: `${percentage}%` }} className={cn("h-full rounded-full", progressColor)} initial={{ width: 0 }} transition={{ delay: delay + 0.2, duration: 0.6, ease: "easeOut" }} />
                        </div>
                    )}
                </motion.div>
            </TooltipTrigger>
            <TooltipContent className="max-w-[220px]" sideOffset={5} variant="dark">
                <p className="font-medium">{label}</p>
                <p className="text-muted-foreground">{description}</p>
                {hasCompletionData && <p className="mt-1 text-[10px] text-muted-foreground/70">{percentage.toFixed(1)}% completion</p>}
            </TooltipContent>
        </Tooltip>
    );
}
