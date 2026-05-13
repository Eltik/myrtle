import { ChevronDownIcon, LayoutGridIcon, ListIcon, SearchIcon, SlidersHorizontalIcon, XIcon } from "lucide-react";
import { Input } from "#/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { cn } from "#/lib/utils";

export type MyListSort = "recent" | "newest" | "alpha" | "views" | "favorites";
export type MyListTypeFilter = "all" | "community" | "official";
export type MyViewMode = "grid" | "list";

export const MY_SORT_OPTIONS: Array<{ value: MyListSort; label: string; hint: string }> = [
    { value: "recent", label: "Recently updated", hint: "Latest edits first" },
    { value: "newest", label: "Newest", hint: "Most recently created" },
    { value: "alpha", label: "Alphabetical", hint: "A → Z by title" },
    { value: "views", label: "Most viewed", hint: "All-time view count" },
    { value: "favorites", label: "Most favorited", hint: "All-time favorites" },
];

const TYPE_TABS: Array<{ value: MyListTypeFilter; label: string }> = [
    { value: "all", label: "All" },
    { value: "community", label: "Community" },
    { value: "official", label: "Official" },
];

interface IMyToolbarProps {
    sort: MyListSort;
    type: MyListTypeFilter;
    view: MyViewMode;
    query: string;
    resultCount: number;
    totalCount: number;
    hasOfficial: boolean;
    onSortChange: (next: MyListSort) => void;
    onTypeChange: (next: MyListTypeFilter) => void;
    onViewChange: (next: MyViewMode) => void;
    onQueryChange: (next: string) => void;
}

export function MyToolbar({ sort, type, view, query, resultCount, totalCount, hasOfficial, onSortChange, onTypeChange, onViewChange, onQueryChange }: IMyToolbarProps) {
    const activeSort = MY_SORT_OPTIONS.find((s) => s.value === sort) ?? MY_SORT_OPTIONS[0];
    const isFiltered = query.length > 0 || type !== "all";

    return (
        <div className="sticky top-14 z-30 -mx-3 border-y border-border bg-background/80 px-3 backdrop-blur-md backdrop-saturate-150 sm:top-16 sm:-mx-4 sm:px-4">
            <div className="mx-auto w-[min(1080px,100%)] py-3">
                <div className="flex flex-wrap items-center gap-2.5">
                    {hasOfficial && (
                        <div role="tablist" aria-label="Filter by list type" className="inline-flex shrink-0 gap-0.5 rounded-[10px] border border-border bg-muted p-0.75">
                            {TYPE_TABS.map((tab) => {
                                const active = type === tab.value;
                                return (
                                    <button
                                        key={tab.value}
                                        role="tab"
                                        type="button"
                                        aria-selected={active}
                                        onClick={() => onTypeChange(tab.value)}
                                        className={cn(
                                            "h-7 cursor-pointer rounded-[7px] border-0 px-3 font-sans text-xs font-medium leading-none transition-colors",
                                            active ? "bg-primary text-primary-foreground shadow-[0_2px_6px_color-mix(in_srgb,var(--primary)_30%,transparent)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                                        )}
                                    >
                                        {tab.label}
                                    </button>
                                );
                            })}
                        </div>
                    )}

                    <Menu>
                        <MenuTrigger
                            className={cn("inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-2.5 font-sans text-xs font-medium leading-none text-foreground transition-colors hover:bg-accent", "[&>svg]:h-3 [&>svg]:w-3 [&>svg]:opacity-80")}
                            aria-label={`Sort: ${activeSort?.label}`}
                        >
                            <SlidersHorizontalIcon aria-hidden="true" />
                            <span className="font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground">Sort</span>
                            <span>{activeSort?.label}</span>
                            <ChevronDownIcon aria-hidden="true" />
                        </MenuTrigger>
                        <MenuPopup align="start" sideOffset={6} className="min-w-56">
                            {MY_SORT_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} onClick={() => onSortChange(opt.value)} className={cn("flex-col items-start gap-0.5 py-1.5", opt.value === sort && "bg-accent/60 text-accent-foreground")}>
                                    <span className="font-medium">{opt.label}</span>
                                    <span className="font-mono text-[10.5px] text-muted-foreground">{opt.hint}</span>
                                </MenuItem>
                            ))}
                        </MenuPopup>
                    </Menu>

                    <div role="tablist" aria-label="View mode" className="inline-flex shrink-0 gap-0.5 rounded-[10px] border border-border bg-muted p-0.75">
                        <button
                            type="button"
                            role="tab"
                            aria-selected={view === "grid"}
                            aria-label="Grid view"
                            onClick={() => onViewChange("grid")}
                            className={cn("inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-[7px] border-0 transition-colors", view === "grid" ? "bg-background text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.06)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground")}
                        >
                            <LayoutGridIcon className="h-3.5 w-3.5" />
                        </button>
                        <button
                            type="button"
                            role="tab"
                            aria-selected={view === "list"}
                            aria-label="List view"
                            onClick={() => onViewChange("list")}
                            className={cn("inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-[7px] border-0 transition-colors", view === "list" ? "bg-background text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.06)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground")}
                        >
                            <ListIcon className="h-3.5 w-3.5" />
                        </button>
                    </div>

                    <div className="relative w-full sm:ml-auto sm:w-72 sm:max-w-72">
                        <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 left-2.5 flex items-center text-muted-foreground">
                            <SearchIcon className="h-3.5 w-3.5" />
                        </span>
                        <Input type="search" size="sm" value={query} onChange={(e) => onQueryChange((e.target as HTMLInputElement).value)} placeholder="Search your lists..." aria-label="Search your tier lists" className="pl-7.5" />
                        {query.length > 0 && (
                            <button type="button" onClick={() => onQueryChange("")} aria-label="Clear search" className="absolute inset-y-0 right-1.5 my-auto inline-flex h-5 w-5 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-foreground">
                                <XIcon className="h-3 w-3" />
                            </button>
                        )}
                    </div>
                </div>

                {isFiltered && (
                    <p aria-live="polite" className="mt-2 font-mono text-[10.5px] leading-none tracking-wide text-muted-foreground">
                        <span className="tabular-nums text-foreground">{resultCount}</span> of <span className="tabular-nums">{totalCount}</span>
                        {query.length > 0 && (
                            <>
                                {" "}
                                · matching <span className="text-foreground">"{query}"</span>
                            </>
                        )}
                    </p>
                )}
            </div>
        </div>
    );
}
