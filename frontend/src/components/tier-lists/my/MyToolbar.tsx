import { ChevronDownIcon, LayoutGridIcon, ListIcon, SearchIcon, SlidersHorizontalIcon, XIcon } from "lucide-react";
import { Input } from "#/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { MY_SORT_OPTIONS, type MyListSort, type MyListTypeFilter, type MyViewMode } from "./MyToolbar.constants";
import type { messages } from "./MyToolbar.messages";

type MyToolbarT = TypedT<typeof messages>;
type MyToolbarRichT = TypedRichT<typeof messages>;
type MyToolbarMessageKey = keyof typeof messages & string;

// R-03: the 32px desktop controls keep their size and gain a centred 44px hit area on touch.
const COARSE_HIT = "pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 pointer-coarse:after:-translate-1/2";

const TYPE_TABS: Array<{ value: MyListTypeFilter; labelKey: MyToolbarMessageKey }> = [
    { value: "all", labelKey: "my.type.all" },
    { value: "community", labelKey: "my.type.community" },
    { value: "official", labelKey: "my.type.official" },
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
    const t: MyToolbarT = useT("tierLists");
    const rt: MyToolbarRichT = useRichT("tierLists");
    const activeSort = MY_SORT_OPTIONS.find((s) => s.value === sort) ?? MY_SORT_OPTIONS[0];
    const isFiltered = query.length > 0 || type !== "all";

    return (
        <div className="page-bleed sticky top-14 z-30 border-border border-y bg-background/95 px-[var(--page-gutter)] sm:top-16">
            <div className="page-gutter py-3 [--page-max:1080px]">
                <div className="flex flex-wrap items-center gap-2.5">
                    {hasOfficial && (
                        <ToggleGroup aria-label={t("my.type.label")} value={[type]} onValueChange={(next) => next[0] && onTypeChange(next[0] as MyListTypeFilter)} variant="outline" size="sm" className="shrink-0">
                            {TYPE_TABS.map((tab) => (
                                <ToggleGroupItem key={tab.value} value={tab.value} className="px-3 text-xs sm:text-xs">
                                    {t(tab.labelKey)}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    )}

                    <Menu>
                        <MenuTrigger
                            className={cn("relative inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent", "[&>svg]:h-3 [&>svg]:w-3 [&>svg]:opacity-80", COARSE_HIT)}
                            aria-label={t("my.sortTrigger", { label: activeSort ? t(activeSort.labelKey) : "" })}
                        >
                            <SlidersHorizontalIcon aria-hidden="true" />
                            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider">{t("my.sortLabel")}</span>
                            <span>{activeSort ? t(activeSort.labelKey) : ""}</span>
                            <ChevronDownIcon aria-hidden="true" />
                        </MenuTrigger>
                        <MenuPopup align="start" sideOffset={6} className="min-w-56">
                            {MY_SORT_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} onClick={() => onSortChange(opt.value)} className={cn("flex-col items-start gap-0.5 py-1.5", opt.value === sort && "bg-accent/60 text-accent-foreground")}>
                                    <span className="font-medium">{t(opt.labelKey)}</span>
                                    <span className="font-mono text-[10.5px] text-muted-foreground">{t(opt.hintKey)}</span>
                                </MenuItem>
                            ))}
                        </MenuPopup>
                    </Menu>

                    <ToggleGroup aria-label={t("my.view.label")} value={[view]} onValueChange={(next) => next[0] && onViewChange(next[0] as MyViewMode)} variant="outline" size="sm" className="shrink-0">
                        <ToggleGroupItem value="grid" aria-label={t("my.view.grid")}>
                            <LayoutGridIcon className="size-3.5" aria-hidden="true" />
                        </ToggleGroupItem>
                        <ToggleGroupItem value="list" aria-label={t("my.view.list")}>
                            <ListIcon className="size-3.5" aria-hidden="true" />
                        </ToggleGroupItem>
                    </ToggleGroup>

                    <div className="relative w-full sm:ml-auto sm:w-72 sm:max-w-72">
                        <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 left-2.5 flex items-center text-muted-foreground">
                            <SearchIcon className="h-3.5 w-3.5" />
                        </span>
                        <Input type="search" size="sm" value={query} onChange={(e) => onQueryChange((e.target as HTMLInputElement).value)} placeholder={t("my.search.placeholder")} aria-label={t("my.search.label")} className="pl-7.5" />
                        {query.length > 0 && (
                            <button type="button" onClick={() => onQueryChange("")} aria-label={t("my.search.clear")} className={cn("absolute inset-y-0 right-1.5 my-auto inline-flex h-5 w-5 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-foreground", COARSE_HIT)}>
                                <XIcon className="h-3 w-3" />
                            </button>
                        )}
                    </div>
                </div>

                {isFiltered && (
                    <p aria-live="polite" className="mt-2 font-mono text-[10.5px] text-muted-foreground leading-none tracking-wide">
                        {rt("my.results.count", { shown: <span className="text-foreground tabular-nums">{resultCount}</span>, total: <span className="tabular-nums">{totalCount}</span> })}
                        {query.length > 0 && <> {rt("my.results.matching", { query: <span className="text-foreground">"{query}"</span> })}</>}
                    </p>
                )}
            </div>
        </div>
    );
}
