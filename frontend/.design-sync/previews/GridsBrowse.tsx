import { GridsBrowse } from "frontend";

// The /grids route container: hero, sort tabs, search and the card gallery.
// The list loads through a server function, stubbed in the design bundle, so
// the page renders its real load-failure branch under the real chrome.

/** The default view: newest first, no query. */
export const LoadFailed = () => <GridsBrowse search={{ sort: "recent", q: "", page: 1 }} />;

/** Sorted by popularity with a query in the search box. */
export const PopularSearch = () => <GridsBrowse search={{ sort: "popular", q: "about me", page: 1 }} />;
