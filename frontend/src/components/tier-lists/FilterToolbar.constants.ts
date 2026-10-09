// Its own module, with no runtime imports, because the /tier-lists route's
// `validateSearch` reads it and `validateSearch` ships in the client entry on
// every page: importing it from FilterToolbar.tsx put the toolbar there too.
import type { messages } from "./FilterToolbar.messages";

type FilterToolbarMessageKey = keyof typeof messages & string;

export type TierListType = "all" | "official" | "community" | "favorites";

export type TierListSort = "trending" | "recent" | "newest" | "views" | "favorites" | "shares";

export const SORT_OPTIONS: Array<{ value: TierListSort; labelKey: FilterToolbarMessageKey; hintKey: FilterToolbarMessageKey }> = [
    { value: "trending", labelKey: "browse.sort.trending", hintKey: "browse.sort.trending.hint" },
    { value: "recent", labelKey: "browse.sort.recent", hintKey: "browse.sort.recent.hint" },
    { value: "newest", labelKey: "browse.sort.newest", hintKey: "browse.sort.newest.hint" },
    { value: "views", labelKey: "browse.sort.views", hintKey: "browse.sort.views.hint" },
    { value: "favorites", labelKey: "browse.sort.favorites", hintKey: "browse.sort.favorites.hint" },
    { value: "shares", labelKey: "browse.sort.shares", hintKey: "browse.sort.shares.hint" },
];
