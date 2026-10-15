import * as React from "react";

/**
 * `value` once it has stopped changing for `delayMs`. Changes are detected by
 * `key` (the value's JSON by default), so a caller that rebuilds an equal
 * object every render does not keep restarting the timer.
 */
export function useDebouncedValue<T>(value: T, delayMs: number, key: string = JSON.stringify(value) ?? ""): T {
    const [settled, setSettled] = React.useState<{ key: string; value: T }>({ key, value });
    const latest = React.useRef(value);
    latest.current = value;
    React.useEffect(() => {
        if (key === settled.key) return;
        const id = setTimeout(() => setSettled({ key, value: latest.current }), delayMs);
        return () => clearTimeout(id);
    }, [key, delayMs, settled.key]);
    return settled.value;
}
