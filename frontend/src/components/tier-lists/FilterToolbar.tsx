import { useState } from "react";
import { Input } from "#/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { useMediaQuery } from "#/hooks/use-media-query";
import { cn } from "#/lib/utils";

export type TierListType = "all" | "official" | "community" | "favorites";

export type TierListSort = "trending" | "recent" | "newest" | "views" | "favorites" | "shares";

export const SORT_OPTIONS: Array<{ value: TierListSort; label: string; hint: string }> = [
    { value: "trending", label: "Trending", hint: "Hot in the last 24h" },
    { value: "recent", label: "Recently updated", hint: "Latest edits first" },
    { value: "newest", label: "Newest", hint: "Most recently created" },
    { value: "views", label: "Most viewed", hint: "All-time view count" },
    { value: "favorites", label: "Most favorited", hint: "All-time favorites" },
    { value: "shares", label: "Most shared", hint: "All-time shares" },
];

export interface IFlairOption {
    code: string;
    label: string;
    color: string | null;
    count?: number;
    displayOrder?: number;
}

interface IFilterToolbarProps {
    type: TierListType;
    sort: TierListSort;
    query: string;
    selectedFlairs: string[];
    flairOptions: IFlairOption[];
    resultCount: number;
    totalCount: number;
    showFavoritesTab: boolean;
    onTypeChange: (next: TierListType) => void;
    onSortChange: (next: TierListSort) => void;
    onQueryChange: (next: string) => void;
    onFlairToggle: (code: string) => void;
    onClearFlairs: () => void;
}

const BASE_TYPE_TABS: Array<{ value: TierListType; label: string }> = [
    { value: "all", label: "All" },
    { value: "official", label: "Official" },
    { value: "community", label: "Community" },
];

// The bar spans the full viewport but its surface stays fully opaque across the centered 1080px content,
// fading into the empty page gutters only on wider screens. On mobile the content fills the width, so
// `max(0px, …)` clamps the fade to zero - no gradient, matching the full-bleed look there.
const CONTENT_WIDTH = "1080px";
const FADE_MASK = `linear-gradient(to right, transparent, #000 max(0px, (100vw - ${CONTENT_WIDTH}) / 2), #000 min(100vw, (100vw + ${CONTENT_WIDTH}) / 2), transparent)`;

export function FilterToolbar({ type, sort, query, selectedFlairs, flairOptions, resultCount, totalCount, showFavoritesTab, onTypeChange, onSortChange, onQueryChange, onFlairToggle, onClearFlairs }: IFilterToolbarProps) {
    const typeTabs = showFavoritesTab ? [...BASE_TYPE_TABS, { value: "favorites" as const, label: "Favorites" }] : BASE_TYPE_TABS;
    const activeSort = SORT_OPTIONS.find((s) => s.value === sort) ?? SORT_OPTIONS[0];
    const hasActiveFilters = selectedFlairs.length > 0 || query.length > 0 || type !== "all" || sort !== "recent";
    const isDesktop = useMediaQuery("sm");
    const [flairsOpenOverride, setFlairsOpenOverride] = useState<boolean | null>(null);
    const flairsOpen = flairsOpenOverride ?? isDesktop;

    return (
        <div className="sticky top-14 z-30 mx-[calc(50%-50vw)] w-screen border-border border-y bg-card/85 backdrop-blur-md backdrop-saturate-150 sm:top-16" style={{ maskImage: FADE_MASK, WebkitMaskImage: FADE_MASK }}>
            <div className="mx-auto w-[min(1080px,calc(100%-2rem))] py-3">
                <div className="flex flex-wrap items-center gap-2.5">
                    <div role="tablist" aria-label="Tier list type" className="inline-flex shrink-0 gap-0.5 rounded-[10px] border border-border bg-muted p-0.75">
                        {typeTabs.map((tab) => {
                            const active = type === tab.value;
                            const isFavorites = tab.value === "favorites";
                            return (
                                <button
                                    key={tab.value}
                                    role="tab"
                                    type="button"
                                    aria-selected={active}
                                    onClick={() => onTypeChange(tab.value)}
                                    className={cn(
                                        "inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-[7px] border-0 px-3 font-medium font-sans text-xs leading-none transition-colors",
                                        active ? "bg-primary text-primary-foreground shadow-[0_2px_6px_color-mix(in_srgb,var(--primary)_30%,transparent)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                                    )}
                                >
                                    {isFavorites && (
                                        <svg viewBox="0 0 24 24" fill={active ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
                                        </svg>
                                    )}
                                    {tab.label}
                                </button>
                            );
                        })}
                    </div>

                    <Menu>
                        <MenuTrigger
                            className={cn("inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent", "[&>svg]:h-3 [&>svg]:w-3 [&>svg]:opacity-80")}
                            aria-label={`Sort: ${activeSort?.label}`}
                        >
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <path d="M3 6h18" />
                                <path d="M7 12h10" />
                                <path d="M11 18h2" />
                            </svg>
                            <span>{activeSort?.label}</span>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <path d="m6 9 6 6 6-6" />
                            </svg>
                        </MenuTrigger>
                        <MenuPopup align="start" sideOffset={6} className="min-w-56">
                            {SORT_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} onClick={() => onSortChange(opt.value)} className={cn("flex-col items-start gap-0.5 py-1.5", opt.value === sort && "bg-accent/60 text-accent-foreground")}>
                                    <span className="font-medium">{opt.label}</span>
                                    <span className="font-mono text-[10.5px] text-muted-foreground">{opt.hint}</span>
                                </MenuItem>
                            ))}
                        </MenuPopup>
                    </Menu>

                    {flairOptions.length > 0 && (
                        <button
                            type="button"
                            onClick={() => setFlairsOpenOverride(!flairsOpen)}
                            aria-expanded={flairsOpen}
                            aria-controls="tier-list-flairs"
                            className="hidden h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent sm:inline-flex"
                        >
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3 opacity-80" aria-hidden="true">
                                <path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z" />
                                <circle cx="7.5" cy="7.5" r="1.25" fill="currentColor" stroke="none" />
                            </svg>
                            <span>Flairs</span>
                            {selectedFlairs.length > 0 && <span className="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 font-mono font-semibold text-[9.5px] text-primary-foreground tabular-nums leading-none">{selectedFlairs.length}</span>}
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={cn("h-3 w-3 opacity-80 transition-transform duration-200", flairsOpen && "rotate-180")} aria-hidden="true">
                                <path d="m6 9 6 6 6-6" />
                            </svg>
                        </button>
                    )}

                    <div className="relative min-w-0 flex-1 sm:ml-auto sm:w-72 sm:max-w-72 sm:flex-none">
                        <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 left-2.5 z-10 flex items-center text-black dark:text-muted-foreground">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                        </span>
                        <Input type="search" size="sm" value={query} onChange={(e) => onQueryChange((e.target as HTMLInputElement).value)} placeholder="Search lists..." aria-label="Search tier lists" className="pl-7.5" />
                        {query.length > 0 && (
                            <button type="button" onClick={() => onQueryChange("")} aria-label="Clear search" className="absolute inset-y-0 right-1.5 my-auto inline-flex h-5 w-5 items-center justify-center rounded-full text-black hover:bg-accent hover:text-foreground dark:text-muted-foreground">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                    <path d="M18 6 6 18" />
                                    <path d="m6 6 12 12" />
                                </svg>
                            </button>
                        )}
                    </div>
                </div>

                {flairOptions.length > 0 && (
                    <>
                        <button
                            type="button"
                            onClick={() => setFlairsOpenOverride(!flairsOpen)}
                            aria-expanded={flairsOpen}
                            aria-controls="tier-list-flairs"
                            className="mt-2.5 flex w-full items-center justify-between gap-2 rounded-md border border-border bg-muted/40 px-2.5 py-1.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider transition-colors hover:bg-muted hover:text-foreground sm:hidden"
                        >
                            <span className="inline-flex items-center gap-1.5">
                                <span>Flairs</span>
                                {selectedFlairs.length > 0 && <span className="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 font-mono font-semibold text-[9.5px] text-primary-foreground tabular-nums leading-none">{selectedFlairs.length}</span>}
                            </span>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={cn("h-3 w-3 transition-transform duration-200", flairsOpen && "rotate-180")} aria-hidden="true">
                                <path d="m6 9 6 6 6-6" />
                            </svg>
                        </button>
                        <div id="tier-list-flairs" className={cn("mt-2.5 items-center gap-2 overflow-x-auto pb-0.5 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden", flairsOpenOverride === null ? "hidden sm:flex" : flairsOpen ? "flex" : "hidden")}>
                            {flairOptions.map((flair) => {
                                const active = selectedFlairs.includes(flair.code);
                                const color = flair.color ?? "var(--primary)";
                                const count = flair.count ?? 0;
                                const hasMatches = count > 0;
                                return (
                                    <button
                                        key={flair.code}
                                        type="button"
                                        onClick={() => onFlairToggle(flair.code)}
                                        aria-pressed={active}
                                        aria-label={`Flair ${flair.label}${hasMatches ? `, ${count} list${count === 1 ? "" : "s"}` : ", no lists"}`}
                                        disabled={!hasMatches && !active}
                                        className={cn("inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1 font-medium font-sans text-xs leading-none transition-colors disabled:cursor-not-allowed disabled:opacity-50")}
                                        style={
                                            active
                                                ? {
                                                      background: `color-mix(in srgb, ${color} 18%, transparent)`,
                                                      borderColor: `color-mix(in srgb, ${color} 50%, transparent)`,
                                                      color,
                                                  }
                                                : {
                                                      background: "var(--muted)",
                                                      borderColor: "var(--border)",
                                                      color: "var(--muted-foreground)",
                                                  }
                                        }
                                    >
                                        <span className="h-1.5 w-1.5 rounded-full" style={{ background: color }} aria-hidden="true" />
                                        <span>{flair.label}</span>
                                        {flair.count !== undefined && (
                                            <span
                                                className="ml-0.5 inline-flex items-center justify-center rounded-full px-1.5 font-mono font-semibold text-[9.5px] tabular-nums leading-[1.4]"
                                                style={active ? { background: `color-mix(in srgb, ${color} 26%, transparent)`, color } : { background: "color-mix(in srgb, var(--foreground) 6%, transparent)", color: "var(--muted-foreground)" }}
                                                aria-hidden="true"
                                            >
                                                {count}
                                            </span>
                                        )}
                                    </button>
                                );
                            })}
                            {selectedFlairs.length > 0 && (
                                <button
                                    type="button"
                                    onClick={onClearFlairs}
                                    className="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-full border border-border border-dashed bg-transparent px-2.5 py-1 font-medium font-sans text-muted-foreground text-xs leading-none transition-colors hover:border-foreground/40 hover:text-foreground"
                                >
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                        <path d="M18 6 6 18" />
                                        <path d="m6 6 12 12" />
                                    </svg>
                                    Clear
                                </button>
                            )}
                        </div>
                    </>
                )}

                {hasActiveFilters && (
                    <p aria-live="polite" className="mt-2 font-mono text-[10.5px] text-muted-foreground leading-none tracking-wide">
                        <span className="text-foreground tabular-nums">{resultCount}</span> of <span className="tabular-nums">{totalCount}</span>
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
