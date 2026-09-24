"use client";

import { useCallback, useEffect, useRef, useState } from "react";

interface IOptions<T> {
    /** Convert a stored string into T. Return `undefined` to keep the initial value. */
    parse?: (raw: string) => T | undefined;
    /** Convert T into the string written to localStorage. */
    serialize?: (value: T) => string;
}

/**
 * One stored value, read outside React.
 *
 * The hook below is this plus a `useState`, and a non-React module that owns
 * the same key (the story library's music channel is the one in the tree) calls
 * it directly rather than writing a second copy of the encoding. A miss, a
 * malformed value and a `parse` that declines all give `fallback`, so a caller
 * never sees a half-read document.
 */
export function readStoredValue<T>(key: string, fallback: T, parse?: (raw: string) => T | undefined): T {
    if (typeof window === "undefined") return fallback;
    const raw = window.localStorage.getItem(key);
    if (raw == null) return fallback;
    try {
        const next = parse ? parse(raw) : (JSON.parse(raw) as T);
        return next === undefined ? fallback : next;
    } catch {
        return fallback;
    }
}

/** The write half, same encoding, same no-op on the server. */
export function writeStoredValue<T>(key: string, value: T, serialize?: (value: T) => string): void {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(key, serialize ? serialize(value) : JSON.stringify(value));
}

/**
 * Persists state in `window.localStorage`. Returns `initial` on the server and
 * during the first client render, then hydrates from storage in an effect to
 * avoid SSR hydration mismatches. Defaults to JSON encoding.
 */
export function useLocalStorageState<T>(key: string, initial: T, options?: IOptions<T>): [T, (next: T | ((prev: T) => T)) => void] {
    const optionsRef = useRef(options);
    optionsRef.current = options;

    const [value, setValue] = useState<T>(initial);

    useEffect(() => {
        // `undefined` is the sentinel for "nothing readable", which is what
        // leaves `initial` in place on a miss and on a malformed value alike.
        const next = readStoredValue<T | undefined>(key, undefined, optionsRef.current?.parse);
        if (next !== undefined) setValue(next);
    }, [key]);

    // The LATEST value, kept outside React state so the write can happen in the
    // caller and not inside the updater.
    const latest = useRef(value);
    latest.current = value;

    /**
     * The write is a side effect, so it does NOT live inside the `setValue`
     * updater. React treats an updater as a reducer and may call it more than
     * once for one dispatch: measured on the story volume slider, one click
     * wrote `myrtle.story.settings` TWICE, once from `dispatchSetStateInternal`
     * and once from the render-phase replay in `updateReducerImpl`. A replay
     * runs against whatever base state React is replaying from, so the second
     * write can persist a value the user has already moved past.
     */
    const set = useCallback(
        (next: T | ((prev: T) => T)) => {
            const resolved = typeof next === "function" ? (next as (p: T) => T)(latest.current) : next;
            latest.current = resolved;
            writeStoredValue(key, resolved, optionsRef.current?.serialize);
            setValue(resolved);
        },
        [key],
    );

    return [value, set];
}
