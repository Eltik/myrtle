// Its own module, with no imports, because the /stories route's `validateSearch`
// reads it and `validateSearch` ships in the client entry on every page: importing
// it from StoryLibrary.tsx put the whole story library in that entry.

/** The library's tabs, in strip order. `browse` is the default and never written to the URL. */
export const LIBRARY_TABS = ["browse", "reading", "illustrations", "progress", "community", "characters"] as const;
export type LibraryTab = (typeof LIBRARY_TABS)[number];

export function isLibraryTab(value: unknown): value is LibraryTab {
    return typeof value === "string" && (LIBRARY_TABS as readonly string[]).includes(value);
}
