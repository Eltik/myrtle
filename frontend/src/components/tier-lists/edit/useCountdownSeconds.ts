import { useEffect, useState } from "react";
import { secondsUntil } from "./autosave";

/**
 * Whole seconds left until `dueAt`, re-rendering as each second boundary passes. Null with no
 * deadline, and no timer runs then.
 *
 * The count is read from the clock at render, not from a stored tick: a stored tick is stale on
 * the render where a new deadline arrives, which flashed a count measured from the last tick.
 */
export function useCountdownSeconds(dueAt: number | null): number | null {
    const [, setTick] = useState(0);

    useEffect(() => {
        if (dueAt === null) return;
        let timer: number;
        // Wake just past each boundary of `dueAt - now`, where the rounded-up count changes.
        const schedule = () => {
            const toBoundary = (dueAt - Date.now()) % 1000;
            timer = window.setTimeout(
                () => {
                    setTick((n) => n + 1);
                    schedule();
                },
                (toBoundary > 0 ? toBoundary : 1000) + 5,
            );
        };
        schedule();
        return () => window.clearTimeout(timer);
    }, [dueAt]);

    return dueAt === null ? null : secondsUntil(dueAt, Date.now());
}
