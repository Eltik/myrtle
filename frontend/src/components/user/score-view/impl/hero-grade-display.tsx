"use client";

import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { TextEffect } from "~/components/ui/motion-primitives/text-effect";
import { cn } from "~/lib/utils";
import type { StoredUserGrade } from "~/types/api/impl/user";
import { GRADE_CONFIG } from "./constants";

interface HeroGradeDisplayProps {
    grade: StoredUserGrade;
}

export function HeroGradeDisplay({ grade }: HeroGradeDisplayProps) {
    const config = GRADE_CONFIG[grade.grade] ?? GRADE_CONFIG.F;
    if (!config) return null;

    return (
        <div className="relative flex h-full flex-col items-center justify-center gap-6 px-6 py-8">
            {/* Main grade display with CSS-only glow */}
            <div className="relative">
                {/* CSS-only glow animation - no flickering */}
                <div
                    className="absolute inset-0 animate-grade-glow rounded-2xl blur-xl"
                    style={{
                        background: `radial-gradient(circle, ${config.glow[0]} 0%, transparent 70%)`,
                    }}
                />
                <motion.div animate={{ scale: 1, opacity: 1 }} className={cn("relative flex h-28 w-28 transform-gpu items-center justify-center rounded-2xl border-2", config.bg, config.border)} initial={{ scale: 0.5, opacity: 0 }} transition={{ type: "spring", stiffness: 200, damping: 20, delay: 0.1 }}>
                    <TextEffect className={cn("font-bold text-6xl tracking-tight", config.text)} delay={0.3} per="char" preset="scale">
                        {grade.grade}
                    </TextEffect>
                </motion.div>
            </div>

            {/* Percentile display */}
            <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1" initial={{ y: 20, opacity: 0 }} transition={{ delay: 0.4, duration: 0.5 }}>
                <div className="flex items-baseline gap-1">
                    <span className="text-muted-foreground text-sm">Top</span>
                    <AnimatedNumber className={cn("font-semibold text-2xl tabular-nums", config.text)} springOptions={{ stiffness: 100, damping: 20 }} value={Math.max(1, Math.round(100 - grade.percentileEstimate))} />
                    <span className={cn("font-medium text-lg", config.text)}>%</span>
                </div>
                <span className="text-muted-foreground text-xs">of all doctors</span>
            </motion.div>

            {/* Composite score badge */}
            <motion.div animate={{ y: 0, opacity: 1 }} className="flex items-center gap-2 rounded-full border border-border/50 bg-muted/30 px-4 py-2" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.5, duration: 0.4 }}>
                <span className="text-muted-foreground text-sm">Composite</span>
                <AnimatedNumber className="font-semibold tabular-nums" springOptions={{ stiffness: 80, damping: 15 }} value={Math.round(grade.compositeScore)} />
            </motion.div>
        </div>
    );
}
