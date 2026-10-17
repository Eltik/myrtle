// Its own module, with no imports, because the /stories route's `validateSearch`
// reads it and `validateSearch` ships in the client entry on every page: importing
// it from StoryLibrary.tsx put the whole story library in that entry.

/** The library's tabs, in strip order. `browse` is the default and never written to the URL. */
export const LIBRARY_TABS = ["browse", "reading", "illustrations", "progress", "community", "characters"] as const;
/**
 * `ask` (Ask Trevor) is a tab only while the Trevor flag is on, so it is not in
 * `LIBRARY_TABS`: with the flag off `?tab=ask` is dropped like any unknown tab.
 * The flag is the caller's to check (`lib/trevor/flag.ts`), which keeps this
 * module import-free.
 */
export const ASK_TAB = "ask";
export type LibraryTab = (typeof LIBRARY_TABS)[number] | typeof ASK_TAB;

export function isLibraryTab(value: unknown): value is LibraryTab {
    return typeof value === "string" && (LIBRARY_TABS as readonly string[]).includes(value);
}
