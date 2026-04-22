"use client";

import { useCallback, useEffect, useRef, useState } from "react";

interface Options<T> {
    /** Convert a stored string into T. Return `undefined` to keep the initial value. */
    parse?: (raw: string) => T | undefined;
    /** Convert T into the string written to localStorage. */
    serialize?: (value: T) => string;
}

/**
 * Persists state in `window.localStorage`. Returns `initial` on the server and
 * during the first client render, then hydrates from storage in an effect to
 * avoid SSR hydration mismatches. Defaults to JSON encoding.
 */
export function useLocalStorageState<T>(key: string, initial: T, options?: Options<T>): [T, (next: T | ((prev: T) => T)) => void] {
    const optionsRef = useRef(options);
    optionsRef.current = options;

    const [value, setValue] = useState<T>(initial);

    useEffect(() => {
        if (typeof window === "undefined") return;
        const raw = window.localStorage.getItem(key);
        if (raw == null) return;
        const parse = optionsRef.current?.parse;
        try {
            const next = parse ? parse(raw) : (JSON.parse(raw) as T);
            if (next !== undefined) setValue(next);
        } catch {
            // Malformed value — leave initial in place.
        }
    }, [key]);

    const set = useCallback(
        (next: T | ((prev: T) => T)) => {
            setValue((prev) => {
                const resolved = typeof next === "function" ? (next as (p: T) => T)(prev) : next;
                if (typeof window !== "undefined") {
                    const serialize = optionsRef.current?.serialize;
                    const raw = serialize ? serialize(resolved) : JSON.stringify(resolved);
                    window.localStorage.setItem(key, raw);
                }
                return resolved;
            });
        },
        [key],
    );

    return [value, set];
}
