// Its own module, with no runtime imports, because the /tier-lists/my route's
// `validateSearch` reads it and `validateSearch` ships in the client entry on
// every page: importing it from MyToolbar.tsx put the toolbar there too.
import type { messages } from "./MyToolbar.messages";

type MyToolbarMessageKey = keyof typeof messages & string;

export type MyListSort = "recent" | "newest" | "alpha" | "views" | "favorites";
export type MyListTypeFilter = "all" | "community" | "official";
export type MyViewMode = "grid" | "list";

export const MY_SORT_OPTIONS: Array<{ value: MyListSort; labelKey: MyToolbarMessageKey; hintKey: MyToolbarMessageKey }> = [
    { value: "recent", labelKey: "my.sort.recent", hintKey: "my.sort.recent.hint" },
    { value: "newest", labelKey: "my.sort.newest", hintKey: "my.sort.newest.hint" },
    { value: "alpha", labelKey: "my.sort.alpha", hintKey: "my.sort.alpha.hint" },
    { value: "views", labelKey: "my.sort.views", hintKey: "my.sort.views.hint" },
    { value: "favorites", labelKey: "my.sort.favorites", hintKey: "my.sort.favorites.hint" },
];
