import { LibraryToolbarButton } from "frontend";
import { useEffect, useRef, useState } from "react";
import type { IBrowseToolbarState } from "../../src/components/story/library/impl/BrowseToolbar";
import type { LibrarySort, ReadFilter } from "../../src/components/story/library/impl/derive";
import type { FilterKey, ViewMode } from "../../src/components/story/library/impl/sections";

// LibraryToolbarButton (the source's `ToolbarButton`) ends the Archives'
// sticky jump bar: a 36 px search button that opens "Search and filters" as a
// popover (a bottom sheet on a phone). It wears the number of non-default
// filters as a primary badge on its corner, and a dot while the search box has
// text, so a filtered page says so from under the pinned bar.

/** The toolbar state Browse owns, held locally so every control in the story works. */
function useToolbarState(initial: Partial<Pick<IBrowseToolbarState, "query" | "filter" | "readFilter" | "sort" | "view">> = {}): IBrowseToolbarState {
    const [query, setQuery] = useState(initial.query ?? "");
    const [filter, setFilter] = useState<FilterKey>(initial.filter ?? "all");
    const [readFilter, setReadFilter] = useState<ReadFilter>(initial.readFilter ?? "any");
    const [sort, setSort] = useState<LibrarySort>(initial.sort ?? "default");
    const [view, setView] = useState<ViewMode>(initial.view ?? "grid");
    return { query, setQuery, filter, setFilter, readFilter, setReadFilter, sort, setSort, view, setView };
}

/** Opens the popover the way a reader does: a click on the trigger, two frames after mount. */
function useOpenOnMount() {
    const root = useRef<HTMLDivElement | null>(null);
    useEffect(() => {
        let raf = requestAnimationFrame(() => {
            raf = requestAnimationFrame(() => {
                root.current?.querySelector<HTMLButtonElement>("button[aria-haspopup]")?.click();
            });
        });
        return () => cancelAnimationFrame(raf);
    }, []);
    return root;
}

/** At rest: nothing filtered, no badge. */
export const Resting = () => {
    const state = useToolbarState();
    return (
        <div className="flex justify-end" style={{ width: 360 }}>
            <LibraryToolbarButton state={state} />
        </div>
    );
};

/** Two filters on and a search typed: the count badge and the search dot. */
export const Filtered = () => {
    const state = useToolbarState({ query: "Siracusa", filter: "events", readFilter: "unread" });
    return (
        <div className="flex justify-end" style={{ width: 360 }}>
            <LibraryToolbarButton state={state} />
        </div>
    );
};

/** Open: the popover with the whole control panel, the search box focused. */
export const Open = () => {
    const state = useToolbarState({ filter: "main" });
    const root = useOpenOnMount();
    return (
        <div ref={root} className="flex justify-end" style={{ width: 560, minHeight: 640 }}>
            <LibraryToolbarButton state={state} />
        </div>
    );
};
