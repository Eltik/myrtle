import { useMemo, useSyncExternalStore } from "react";

const subscribe = () => () => {};

/** A calendar day as yyyymmdd, so the snapshot is a stable primitive. */
const localDay = (): number => {
    const d = new Date();
    return d.getFullYear() * 10_000 + (d.getMonth() + 1) * 100 + d.getDate();
};
const utcDay = (): number => {
    const d = new Date();
    return d.getUTCFullYear() * 10_000 + (d.getUTCMonth() + 1) * 100 + d.getUTCDate();
};

/**
 * Today as a local-midnight `Date`: the UTC day on the server and through hydration, the
 * reader's own day from the render after.
 *
 * `new Date()` in render put the server's day in the HTML and the browser's in the hydrating
 * render. Whenever the two zones sat on different dates (Tokyo server, browser on UTC, before
 * 09:00 JST), the today callout and the highlighted cell disagreed and React threw #418.
 */
export function useToday(): Date {
    const day = useSyncExternalStore(subscribe, localDay, utcDay);
    return useMemo(() => new Date(Math.floor(day / 10_000), (Math.floor(day / 100) % 100) - 1, day % 100), [day]);
}
