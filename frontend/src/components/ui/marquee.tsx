"use client";

import { useReducedMotion } from "motion/react";
import { cn } from "~/lib/utils";

interface MarqueeProps {
    children: React.ReactNode;
    className?: string;
    reverse?: boolean;
    pauseOnHover?: boolean;
    /** Duration in seconds for one complete cycle */
    duration?: number;
}

/**
 * Uses CSS animations on the compositor thread for smooth 60fps scrolling.
 * No JavaScript animation loop = minimal CPU usage.
 */
export function Marquee({ children, className, reverse = false, pauseOnHover = false, duration = 40 }: MarqueeProps) {
    const prefersReducedMotion = useReducedMotion();

    if (prefersReducedMotion) {
        return (
            <div className={cn("marquee-container marquee-static", className)}>
                <div className="marquee-content">{children}</div>
            </div>
        );
    }

    return (
        <div className={cn("marquee-container", pauseOnHover && "marquee-pause-hover", className)} style={{ "--marquee-duration": `${duration}s` } as React.CSSProperties}>
            <div className={cn("marquee-content", reverse ? "marquee-reverse" : "marquee-forward")}>{children}</div>
            {/* Duplicate for seamless loop */}
            <div aria-hidden="true" className={cn("marquee-content", reverse ? "marquee-reverse" : "marquee-forward")}>
                {children}
            </div>
        </div>
    );
}
