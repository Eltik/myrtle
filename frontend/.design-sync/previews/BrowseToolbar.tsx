import { BrowseToolbar } from "frontend";
import { useState } from "react";
import type { IBrowseToolbarState } from "../../src/components/story/library/impl/BrowseToolbar";
import type { LibrarySort, ReadFilter } from "../../src/components/story/library/impl/derive";
import type { FilterKey, ViewMode } from "../../src/components/story/library/impl/sections";

// BrowseToolbar is the Archives head's inline controls: search, the
// grid/list toggle, the category pills (All, Main story, Events, Side stories,
// Records), the read-state segmented control with its bookmark legend, and the
// order select. At 1280 and up it is one row; below that, two. Browse owns the
// state; here it is held locally so the controls respond.

/** The toolbar state Browse owns, held locally so every control in the story works. */
function useToolbarState(initial: Partial<Pick<IBrowseToolbarState, "query" | "filter" | "readFilter" | "sort" | "view">> = {}): IBrowseToolbarState {
    const [query, setQuery] = useState(initial.query ?? "");
    const [filter, setFilter] = useState<FilterKey>(initial.filter ?? "all");
    const [readFilter, setReadFilter] = useState<ReadFilter>(initial.readFilter ?? "any");
    const [sort, setSort] = useState<LibrarySort>(initial.sort ?? "default");
    const [view, setView] = useState<ViewMode>(initial.view ?? "grid");
    return { query, setQuery, filter, setFilter, readFilter, setReadFilter, sort, setSort, view, setView };
}

/** The page as it opens: nothing filtered, shelves by storyline, grid view. */
export const Default = () => {
    const state = useToolbarState();
    return (
        <div style={{ width: 860 }}>
            <BrowseToolbar state={state} />
        </div>
    );
};

/** A reader narrowing the shelf: a search, Events only, in-progress chapters, newest first, list view. */
export const Filtered = () => {
    const state = useToolbarState({ query: "Kazimierz", filter: "events", readFilter: "progress", sort: "newest", view: "list" });
    return (
        <div style={{ width: 860 }}>
            <BrowseToolbar state={state} />
        </div>
    );
};
