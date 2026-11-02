import { useEffect, useState } from "react";
import { secondsUntil } from "./autosave";

/** Whole seconds left until `dueAt`, ticking once a second. Null with no deadline, and no timer runs then. */
export function useCountdownSeconds(dueAt: number | null): number | null {
    const [now, setNow] = useState(() => Date.now());

    useEffect(() => {
        if (dueAt === null) return;
        setNow(Date.now());
        const timer = window.setInterval(() => setNow(Date.now()), 1000);
        return () => window.clearInterval(timer);
    }, [dueAt]);

    return dueAt === null ? null : secondsUntil(dueAt, now);
}
