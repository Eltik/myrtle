import type React from "react";
import { ToolbarPanel } from "frontend";
import { useState } from "react";
import type { IBrowseToolbarState } from "../../src/components/story/library/impl/BrowseToolbar";
import type { LibrarySort, ReadFilter } from "../../src/components/story/library/impl/derive";
import type { FilterKey, ViewMode } from "../../src/components/story/library/impl/sections";

// ToolbarPanel is the Archives' controls stacked in one column: the panel
// inside the sticky bar's "Search and filters" popover (with the search box)
// and inside the phone's Filters sheet (without it, since the box is on
// screen beside the button). Each field carries its label above it.

/** The toolbar state Browse owns, held locally so every control in the story works. */
function useToolbarState(initial: Partial<Pick<IBrowseToolbarState, "query" | "filter" | "readFilter" | "sort" | "view">> = {}): IBrowseToolbarState {
    const [query, setQuery] = useState(initial.query ?? "");
    const [filter, setFilter] = useState<FilterKey>(initial.filter ?? "all");
    const [readFilter, setReadFilter] = useState<ReadFilter>(initial.readFilter ?? "any");
    const [sort, setSort] = useState<LibrarySort>(initial.sort ?? "default");
    const [view, setView] = useState<ViewMode>(initial.view ?? "grid");
    return { query, setQuery, filter, setFilter, readFilter, setReadFilter, sort, setSort, view, setView };
}

const Surface = ({ children }: { children: React.ReactNode }) => (
    <div className="rounded-xl border border-border bg-popover p-4 shadow-lg/5" style={{ width: 320 }}>
        {children}
    </div>
);

/** The popover's panel, search included, defaults set. */
export const Popover = () => {
    const state = useToolbarState();
    return (
        <Surface>
            <ToolbarPanel state={state} density="popover" withSearch />
        </Surface>
    );
};

/** The phone sheet's panel at 44 px controls, with filters applied. */
export const SheetFiltered = () => {
    const state = useToolbarState({ filter: "main", readFilter: "done", sort: "oldest" });
    return (
        <div style={{ width: 390 }}>
            <ToolbarPanel state={state} density="sheet" withSearch={false} />
        </div>
    );
};
