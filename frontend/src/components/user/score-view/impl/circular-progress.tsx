"use client";

import { motion } from "motion/react";
import { cn } from "~/lib/utils";

interface CircularProgressProps {
    value: number;
    max: number;
    size?: number;
    strokeWidth?: number;
    className?: string;
    trackClassName?: string;
    progressClassName?: string;
    delay?: number;
}

export function CircularProgress({ value, max, size = 48, strokeWidth = 4, className, trackClassName, progressClassName, delay = 0 }: CircularProgressProps) {
    const radius = (size - strokeWidth) / 2;
    const circumference = radius * 2 * Math.PI;
    const percentage = max > 0 ? Math.min((value / max) * 100, 100) : 0;
    const offset = circumference - (percentage / 100) * circumference;

    return (
        <div className={cn("relative", className)} style={{ width: size, height: size }}>
            <svg aria-hidden="true" className="-rotate-90" height={size} viewBox={`0 0 ${size} ${size}`} width={size}>
                {/* Track */}
                <circle className={cn("stroke-muted/30", trackClassName)} cx={size / 2} cy={size / 2} fill="none" r={radius} strokeWidth={strokeWidth} />
                {/* Progress */}
                <motion.circle
                    animate={{ strokeDashoffset: offset }}
                    className={cn("stroke-primary", progressClassName)}
                    cx={size / 2}
                    cy={size / 2}
                    fill="none"
                    initial={{ strokeDashoffset: circumference }}
                    r={radius}
                    strokeLinecap="round"
                    strokeWidth={strokeWidth}
                    style={{
                        strokeDasharray: circumference,
                    }}
                    transition={{ delay, duration: 0.8, ease: "easeOut" }}
                />
            </svg>
            {/* Center value */}
            <div className="absolute inset-0 flex items-center justify-center">
                <span className="font-medium text-xs tabular-nums">{Math.round(percentage)}</span>
            </div>
        </div>
    );
}
