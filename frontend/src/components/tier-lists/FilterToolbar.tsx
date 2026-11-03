import { Check } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useMediaQuery } from "#/hooks/use-media-query";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { SORT_OPTIONS, type TierListSort, type TierListType } from "./FilterToolbar.constants";
import type { messages } from "./FilterToolbar.messages";

type FilterToolbarT = TypedT<typeof messages>;
type FilterToolbarRichT = TypedRichT<typeof messages>;
type FilterToolbarMessageKey = keyof typeof messages & string;

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
    /** The author picker, placed before the search box. */
    authorFilter?: React.ReactNode;
    /** An author is picked: counts as an active filter for the results line. */
    authorActive?: boolean;
}

const BASE_TYPE_TABS: Array<{ value: TierListType; labelKey: FilterToolbarMessageKey }> = [
    { value: "all", labelKey: "browse.type.all" },
    { value: "official", labelKey: "browse.type.official" },
    { value: "community", labelKey: "browse.type.community" },
];

// R-03: the 32px desktop controls keep their size and gain a centred 44px hit area on touch.
const COARSE_HIT = "pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 pointer-coarse:after:-translate-1/2";

export function FilterToolbar({ type, sort, query, selectedFlairs, flairOptions, resultCount, totalCount, showFavoritesTab, onTypeChange, onSortChange, onQueryChange, onFlairToggle, onClearFlairs, authorFilter, authorActive = false }: IFilterToolbarProps) {
    const t: FilterToolbarT = useT("tierLists");
    const rt: FilterToolbarRichT = useRichT("tierLists");
    const typeTabs = showFavoritesTab ? [...BASE_TYPE_TABS, { value: "favorites" as const, labelKey: "browse.type.favorites" as const }] : BASE_TYPE_TABS;
    const activeSort = SORT_OPTIONS.find((s) => s.value === sort) ?? SORT_OPTIONS[0];
    const hasActiveFilters = selectedFlairs.length > 0 || query.length > 0 || type !== "all" || sort !== "recent" || authorActive;
    const isDesktop = useMediaQuery("sm");
    const [flairsOpenOverride, setFlairsOpenOverride] = useState<boolean | null>(null);
    const flairsOpen = flairsOpenOverride ?? isDesktop;

    return (
        // Below sm the bar dissolves (display: contents) so only the type/sort row sticks: the full stack was ~240px, 47% of a 375px viewport.
        // At sm and up the bar stays inside the content column: bleeding by the gutter hung its hairlines past the cards at 1440.
        <div className="max-sm:contents sm:sticky sm:top-16 sm:z-30 sm:border-border sm:border-y sm:bg-background">
            <div className="max-sm:contents sm:py-3">
                <div className="flex flex-wrap items-center gap-2.5 max-sm:contents">
                    <div className="page-bleed sticky top-14 z-30 flex items-center gap-2.5 border-border border-y bg-background px-(--page-gutter) py-2 sm:contents">
                        {/* One row below sm (it wrapped to 96px at 375): the group scrolls sideways; the 6px padding, cancelled by the margin, keeps the 44px touch hit area and the focus ring inside the scroller. */}
                        <ToggleGroup
                            aria-label={t("browse.type.label")}
                            value={[type]}
                            onValueChange={(next) => next[0] && onTypeChange(next[0] as TierListType)}
                            variant="outline"
                            size="sm"
                            className="min-w-0 max-sm:-m-1.5 max-sm:overflow-x-auto max-sm:p-1.5 sm:shrink-0 max-sm:[scrollbar-width:none] max-sm:[&::-webkit-scrollbar]:hidden"
                        >
                            {typeTabs.map((tab) => (
                                <ToggleGroupItem key={tab.value} value={tab.value} className="gap-1.5 px-3 text-xs sm:text-xs">
                                    {tab.value === "favorites" && (
                                        <svg viewBox="0 0 24 24" fill={type === tab.value ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="size-3" aria-hidden="true">
                                            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
                                        </svg>
                                    )}
                                    {t(tab.labelKey)}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>

                        <Menu>
                            <MenuTrigger render={<Button variant="outline" size="sm" />} className="text-xs sm:text-xs [&>svg]:h-3 [&>svg]:w-3" aria-label={t("browse.sortTrigger", { label: activeSort ? t(activeSort.labelKey) : "" })}>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                    <path d="M3 6h18" />
                                    <path d="M7 12h10" />
                                    <path d="M11 18h2" />
                                </svg>
                                <span>{activeSort ? t(activeSort.labelKey) : ""}</span>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                            </MenuTrigger>
                            <MenuPopup align="start" sideOffset={6} className="min-w-56">
                                {SORT_OPTIONS.map((opt) => (
                                    <MenuItem key={opt.value} onClick={() => onSortChange(opt.value)} className={cn("flex-col items-start gap-0.5 py-1.5", opt.value === sort && "bg-accent/60 text-accent-foreground")}>
                                        <span className="font-medium">{t(opt.labelKey)}</span>
                                        <span className="font-mono text-[10.5px] text-muted-foreground">{t(opt.hintKey)}</span>
                                    </MenuItem>
                                ))}
                            </MenuPopup>
                        </Menu>
                    </div>

                    {flairOptions.length > 0 && (
                        <Button variant="outline" size="sm" className="hidden text-xs sm:inline-flex sm:text-xs" onClick={() => setFlairsOpenOverride(!flairsOpen)} aria-expanded={flairsOpen} aria-controls="tier-list-flairs">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3 opacity-80" aria-hidden="true">
                                <path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z" />
                                <circle cx="7.5" cy="7.5" r="1.25" fill="currentColor" stroke="none" />
                            </svg>
                            <span>{t("browse.flairs")}</span>
                            {selectedFlairs.length > 0 && <span className="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 font-mono font-semibold text-[9.5px] text-primary-foreground tabular-nums leading-none">{selectedFlairs.length}</span>}
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={cn("h-3 w-3 opacity-80 transition-transform duration-200", flairsOpen && "rotate-180")} aria-hidden="true">
                                <path d="m6 9 6 6 6-6" />
                            </svg>
                        </Button>
                    )}

                    <div className={cn("relative mt-2.5 min-w-0 flex-1 sm:order-2 sm:mt-0 sm:w-72 sm:max-w-72 sm:flex-none", !authorFilter && "sm:ml-auto")}>
                        <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 left-2.5 z-10 flex items-center text-black dark:text-muted-foreground">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3.5 w-3.5" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                        </span>
                        <Input type="search" size="sm" value={query} onChange={(e) => onQueryChange((e.target as HTMLInputElement).value)} placeholder={t("browse.search.placeholder")} aria-label={t("browse.search.label")} className="pl-7.5" />
                        {query.length > 0 && (
                            <button
                                type="button"
                                onClick={() => onQueryChange("")}
                                aria-label={t("browse.search.clear")}
                                className={cn("absolute inset-y-0 right-1.5 my-auto inline-flex h-5 w-5 items-center justify-center rounded-full text-black hover:bg-accent hover:text-foreground dark:text-muted-foreground", COARSE_HIT)}
                            >
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                    <path d="M18 6 6 18" />
                                    <path d="m6 6 12 12" />
                                </svg>
                            </button>
                        )}
                    </div>

                    {authorFilter ? <div className="mt-2.5 w-full sm:order-1 sm:mt-0 sm:ml-auto sm:w-auto">{authorFilter}</div> : null}
                </div>

                {flairOptions.length > 0 && (
                    <>
                        <button
                            type="button"
                            onClick={() => setFlairsOpenOverride(!flairsOpen)}
                            aria-expanded={flairsOpen}
                            aria-controls="tier-list-flairs"
                            className="mt-2.5 flex pointer-coarse:min-h-11 w-full items-center justify-between gap-2 rounded-lg border border-border bg-muted/40 px-2.5 py-1.5 font-mono text-[10.5px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground sm:hidden"
                        >
                            <span className="inline-flex items-center gap-1.5">
                                <span>{t("browse.flairs")}</span>
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
                                        aria-label={hasMatches ? t("browse.flair.chip", { label: flair.label, count }) : t("browse.flair.chipEmpty", { label: flair.label })}
                                        disabled={!hasMatches && !active}
                                        className="inline-flex pointer-coarse:min-h-11 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1 font-medium font-sans text-xs leading-none transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                                        style={
                                            active
                                                ? {
                                                      background: `color-mix(in srgb, ${color} 18%, transparent)`,
                                                      borderColor: `color-mix(in srgb, ${color} 50%, transparent)`,
                                                      // Flair colours are free-form per list, so the text stays foreground: as text on their own tint they measured 2.46 to 2.94.
                                                      color: "var(--foreground)",
                                                  }
                                                : {
                                                      background: "var(--muted)",
                                                      borderColor: "var(--border)",
                                                      color: "var(--muted-foreground)",
                                                  }
                                        }
                                    >
                                        {active ? <Check className="size-3" style={{ color: "var(--foreground)" }} aria-hidden="true" /> : null}
                                        <span className="h-1.5 w-1.5 rounded-full" style={{ background: color }} aria-hidden="true" />
                                        <span>{flair.label}</span>
                                        {flair.count !== undefined && (
                                            <span
                                                className="ml-0.5 inline-flex items-center justify-center rounded-full px-1.5 font-mono font-semibold text-[9.5px] tabular-nums leading-[1.4]"
                                                style={active ? { background: `color-mix(in srgb, ${color} 26%, transparent)`, color: "var(--foreground)" } : { background: "color-mix(in srgb, var(--foreground) 6%, transparent)", color: "var(--muted-foreground)" }}
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
                                    className="inline-flex pointer-coarse:min-h-11 shrink-0 cursor-pointer items-center gap-1 rounded-full border border-border border-dashed bg-transparent px-2.5 py-1 font-medium font-sans text-muted-foreground text-xs leading-none transition-colors hover:border-foreground/40 hover:text-foreground"
                                >
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                        <path d="M18 6 6 18" />
                                        <path d="m6 6 12 12" />
                                    </svg>
                                    {t("browse.flair.clear")}
                                </button>
                            )}
                        </div>
                    </>
                )}

                {hasActiveFilters && (
                    <p aria-live="polite" className="mt-2 font-mono text-[10.5px] text-muted-foreground leading-none tracking-wide">
                        {rt("browse.results.count", { shown: <span className="text-foreground tabular-nums">{resultCount}</span>, total: <span className="tabular-nums">{totalCount}</span> })}
                        {query.length > 0 && <> {rt("browse.results.matching", { query: <span className="text-foreground">"{query}"</span> })}</>}
                    </p>
                )}
            </div>
        </div>
    );
}
