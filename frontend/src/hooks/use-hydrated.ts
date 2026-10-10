import { useSyncExternalStore } from "react";

const subscribe = () => () => {};

/**
 * `false` on the server and through hydration, `true` from the render right after it, and on
 * every render of a component that mounts later in the browser.
 *
 * For output that depends on the browser (its time zone, today's date): print a fixed form
 * until this turns true, so the hydrating render matches the server HTML, and React re-renders
 * with the browser's value straight after.
 */
export function useHydrated(): boolean {
    return useSyncExternalStore(
        subscribe,
        () => true,
        () => false,
    );
}
