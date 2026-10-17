/**
 * SPOILER MODE (contract v1.1 section 8): the reader may let Trevor answer
 * from every released story, not only the ones they have read.
 *
 * The choice is per browser (localStorage), off by default, and read through
 * `useSyncExternalStore`, so the reader sheet and the stories tab agree and a
 * change in another tab follows. The server render and the first client
 * render are always off; a stored "on" shows right after hydration. Storage
 * that throws (a private window, blocked site data) keeps the choice in
 * memory for this page load. Turning it on asks for confirmation once per
 * browser (`spoilersConfirmed`).
 */
import { useSyncExternalStore } from "react";
import { trevorSpoilerModeEnabled } from "./flag";

export const SPOILER_MODE_KEY = "myrtle-trevor-spoilers";
export const SPOILER_CONFIRMED_KEY = "myrtle-trevor-spoilers-confirmed";

const listeners = new Set<() => void>();
/** The choice when storage cannot hold it. */
let memory = { on: false, confirmed: false, broken: false };

function storage(): Storage | null {
    return typeof window === "undefined" ? null : window.localStorage;
}

function readOn(): boolean {
    if (memory.broken) return memory.on;
    try {
        return storage()?.getItem(SPOILER_MODE_KEY) === "all";
    } catch {
        return memory.on;
    }
}

function write(key: string, value: string | null): void {
    try {
        const s = storage();
        if (!s) return;
        if (value === null) s.removeItem(key);
        else s.setItem(key, value);
    } catch {
        memory.broken = true;
    }
}

function notify(): void {
    for (const l of listeners) l();
}

function subscribe(listener: () => void): () => void {
    listeners.add(listener);
    const onStorage = (e: StorageEvent) => {
        if (e.key === null || e.key === SPOILER_MODE_KEY) listener();
    };
    window.addEventListener("storage", onStorage);
    return () => {
        listeners.delete(listener);
        window.removeEventListener("storage", onStorage);
    };
}

/** Turn spoiler mode on or off for this browser. */
export function setSpoilersOn(on: boolean): void {
    memory.on = on;
    write(SPOILER_MODE_KEY, on ? "all" : null);
    notify();
}

/** The reader already confirmed spoiler mode once in this browser. */
export function spoilersConfirmed(): boolean {
    if (memory.confirmed) return true;
    try {
        return storage()?.getItem(SPOILER_CONFIRMED_KEY) === "1";
    } catch {
        return false;
    }
}

export function markSpoilersConfirmed(): void {
    memory.confirmed = true;
    write(SPOILER_CONFIRMED_KEY, "1");
}

/** Forget the in-memory fallback (tests). */
export function resetSpoilerMemory(): void {
    memory = { on: false, confirmed: false, broken: false };
}

/** Spoiler mode's state and setter; always off when the build turns the switch off. */
export function useSpoilerMode(): [boolean, (on: boolean) => void] {
    const on = useSyncExternalStore(subscribe, readOn, () => false);
    return [trevorSpoilerModeEnabled() && on, setSpoilersOn];
}
