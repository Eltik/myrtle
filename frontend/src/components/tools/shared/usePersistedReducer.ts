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
 * A state source that outranks `localStorage`, such as a share link. `none`: there is
 * none, hydrate from storage as usual. `pending`: one exists but is not resolved yet
 * (it may need the operator list), so hydration waits. `ready`: hydrate from `state`,
 * or from storage when it resolved to null.
 */
export type ExternalHydration<S> = { status: "none" } | { status: "pending" } | { status: "ready"; state: S | null };

const NO_EXTERNAL: ExternalHydration<never> = { status: "none" };

/**
 * `useReducer` with `localStorage` persistence and a `{ type: "HYDRATE" }`
 * action used to restore saved state on mount. Hydration runs in an effect to
 * avoid SSR mismatches; until then the initial state renders. An `external`
 * source, when present, is used instead of the stored value.
 */
export function usePersistedReducer<S, A extends { type: string }>(storageKey: string, reducer: (state: S, action: A) => S, initialState: S, merge: (raw: unknown) => S | null, hydrateAction: (state: S) => A, external: ExternalHydration<S> = NO_EXTERNAL): IPersistedReducerHandle<S, A> {
    const [state, dispatch] = useReducer(reducer, initialState);
    const hydratedRef = useRef(false);
    const [hydrationToken, setHydrationToken] = useState(0);

    // Hydrate once, as soon as the external source (if any) has settled;
    // storageKey/merge/hydrateAction are stable per tool.
    // biome-ignore lint/correctness/useExhaustiveDependencies: runs until hydrated, once
    useEffect(() => {
        if (typeof window === "undefined" || hydratedRef.current || external.status === "pending") return;
        const fromExternal = external.status === "ready" ? external.state : null;
        if (fromExternal) dispatch(hydrateAction(fromExternal));
        else {
            const raw = window.localStorage.getItem(storageKey);
            if (raw != null) {
                try {
                    const merged = merge(JSON.parse(raw) as unknown);
                    if (merged) dispatch(hydrateAction(merged));
                } catch {
                    // Malformed stored value: keep the default state.
                }
            }
        }
        hydratedRef.current = true;
        setHydrationToken((t) => t + 1);
    }, [external]);

    useEffect(() => {
        if (typeof window === "undefined" || !hydratedRef.current) return;
        try {
            window.localStorage.setItem(storageKey, JSON.stringify(state));
        } catch {
            // Quota or a blocked store: the state just isn't saved this time.
        }
    }, [storageKey, state]);

    return { state, dispatch, hydrationToken };
}
