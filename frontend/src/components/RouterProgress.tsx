import { useRouterState } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import { cn } from "#/lib/utils";

const SHOW_DELAY_MS = 80;
const TRICKLE_MS = 200;
const FINISH_MS = 220;

export function RouterProgress() {
    const isLoading = useRouterState({
        select: (s) => s.isLoading || s.isTransitioning,
    });

    const [visible, setVisible] = useState(false);
    const [progress, setProgress] = useState(0);
    const showTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const trickle = useRef<ReturnType<typeof setInterval> | null>(null);

    useEffect(() => {
        const clear = () => {
            if (showTimer.current) clearTimeout(showTimer.current);
            if (hideTimer.current) clearTimeout(hideTimer.current);
            if (trickle.current) clearInterval(trickle.current);
            showTimer.current = null;
            hideTimer.current = null;
            trickle.current = null;
        };

        if (isLoading) {
            clear();
            showTimer.current = setTimeout(() => {
                setVisible(true);
                setProgress(10);
                trickle.current = setInterval(() => {
                    setProgress((p) => (p < 90 ? p + (90 - p) * 0.1 : p));
                }, TRICKLE_MS);
            }, SHOW_DELAY_MS);
            return clear;
        }

        if (showTimer.current) {
            clearTimeout(showTimer.current);
            showTimer.current = null;
        }
        if (trickle.current) {
            clearInterval(trickle.current);
            trickle.current = null;
        }
        if (!visible) return;

        setProgress(100);
        hideTimer.current = setTimeout(() => {
            setVisible(false);
            setProgress(0);
        }, FINISH_MS);
        return clear;
    }, [isLoading, visible]);

    return <div aria-hidden className={cn("pointer-events-none fixed inset-x-0 top-0 z-60 h-0.5 origin-left bg-primary", "transition-[transform,opacity] duration-200 ease-out motion-reduce:transition-none", visible ? "opacity-100" : "opacity-0")} style={{ transform: `scaleX(${progress / 100})` }} />;
}
