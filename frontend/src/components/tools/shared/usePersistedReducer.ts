import { type Dispatch, useEffect, useReducer, useRef, useState } from "react";

export interface IPersistedReducerHandle<S, A> {
    state: S;
    dispatch: Dispatch<A>;
    /**
     * Increments once after hydration from `localStorage` completes. Children
     * with uncontrolled inputs can include this in their `key` so they remount
     * with the hydrated value rather than the initial-render placeholder.
     */
    hydrationToken: number;
}

/**
 * `useReducer` with `localStorage` persistence and a `{ type: "HYDRATE" }`
 * action used to restore saved state on mount. Hydration runs in an effect to
 * avoid SSR mismatches; until then the initial state renders.
 */
export function usePersistedReducer<S, A extends { type: string }>(storageKey: string, reducer: (state: S, action: A) => S, initialState: S, merge: (raw: unknown) => S | null, hydrateAction: (state: S) => A): IPersistedReducerHandle<S, A> {
    const [state, dispatch] = useReducer(reducer, initialState);
    const hydratedRef = useRef(false);
    const [hydrationToken, setHydrationToken] = useState(0);

    // Hydrate once on mount; storageKey/merge/hydrateAction are stable per tool.
    // biome-ignore lint/correctness/useExhaustiveDependencies: run once on mount
    useEffect(() => {
        if (typeof window === "undefined") return;
        const raw = window.localStorage.getItem(storageKey);
        if (raw == null) {
            hydratedRef.current = true;
            setHydrationToken((t) => t + 1);
            return;
        }
        try {
            const parsed = JSON.parse(raw) as unknown;
            const merged = merge(parsed);
            if (merged) dispatch(hydrateAction(merged));
        } catch {
            // Malformed stored value - ignore and fall back to the default state.
        }
        hydratedRef.current = true;
        setHydrationToken((t) => t + 1);
    }, []);

    useEffect(() => {
        if (typeof window === "undefined" || !hydratedRef.current) return;
        try {
            window.localStorage.setItem(storageKey, JSON.stringify(state));
        } catch {
            // Storage quota exhausted or other failure - silently ignore.
        }
    }, [storageKey, state]);

    return { state, dispatch, hydrationToken };
}
