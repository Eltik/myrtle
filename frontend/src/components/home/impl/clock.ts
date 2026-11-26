import * as React from "react";

/** Wall-clock milliseconds, re-read every `intervalMs` (a second by default) while mounted. */
export function useNow(intervalMs = 1000): number {
    const [now, setNow] = React.useState(() => Date.now());
    React.useEffect(() => {
        const id = window.setInterval(() => setNow(Date.now()), intervalMs);
        return () => window.clearInterval(id);
    }, [intervalMs]);
    return now;
}
