/**
 * THE BROWSE TOOLBAR: search, layout, category, read state and sort, as ONE
 * component rendered in two places over the same state that `Browse` owns.
 *
 * Inline in the page head it lays itself out by width. At 1280 and up it is one
 * row; from 640 to 1279 it is two, [search · layout] over [category · read
 * state · sort]; under 640 it is ONE row, [search · Filters], and the Filters
 * button opens a bottom sheet holding category, read state, sort and layout.
 *
 * THE PHONE'S LAYOUT TOGGLE MOVED INTO THE SHEET, which the brief did not ask
 * for, because the brief's own shape missed its budget. [search · layout] over
 * a Filters row measured a 653.6 px head at 390 against a 600 px target; the
 * Filters row was 44 px plus a 12 px gap of it. One row with the toggle in the
 * sheet is 56 px shorter, keeps the word "Filters" on the button, and leaves the
 * search box 258 px where [search · Filters icon · layout] would have left 210.
 *
 * THE ONE-ROW BREAKPOINT IS 1280, NOT 1024, and that was measured rather than
 * chosen. The four fixed controls are 1,013 px wide on one row at 1440 (pills
 * 437.1, read state 292, sort with its label 170.3, layout 82, four 8 px gaps),
 * and the content column at 1024 is 992 px, so a one-row toolbar there has no
 * room left for the search box at all. At 1280 the column is 1,248 px and the
 * search box keeps 235.
 *
 * The same controls stacked are the sticky bar's panel (`ToolbarPanel`), so a
 * reader who has scrolled past the head can still search and filter.
 */
import { LayoutGridIcon, ListIcon, SearchIcon, SlidersHorizontalIcon } from "lucide-react";
import type React from "react";
import { useRef } from "react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Sheet, SheetClose, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle, SheetTrigger } from "#/components/ui/sheet";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./Browse.messages";
import { LIBRARY_SORTS, type LibrarySort, READ_FILTERS, type ReadFilter } from "./derive";
import { ReadMark } from "./GroupCard";
import { FILTER_ORDER, type FilterKey, type ViewMode } from "./sections";
import { activeFilterCount } from "./toolbar";

type BrowseT = TypedT<typeof messages>;

/** Everything the toolbar reads and writes. `Browse` owns it (and the localStorage keys behind three of it); both copies of the toolbar are views of the one state. */
export interface IBrowseToolbarState {
    query: string;
    setQuery: (query: string) => void;
    filter: FilterKey;
    setFilter: (filter: FilterKey) => void;
    readFilter: ReadFilter;
    setReadFilter: (read: ReadFilter) => void;
    sort: LibrarySort;
    setSort: (sort: LibrarySort) => void;
    view: ViewMode;
    setView: (view: ViewMode) => void;
}

/**
 * How big a control is. `inline` is the head's own sizing, 44 px on a phone or
 * a coarse pointer and compact above 640. `sheet` is 44 px everywhere, for the
 * phone's bottom sheets. `popover` is compact unless the pointer is coarse.
 */
type Density = "inline" | "sheet" | "popover";

const TALL: Record<Density, string> = {
    inline: "h-11 pointer-coarse:h-11 sm:h-8",
    sheet: "h-11",
    popover: "h-8 pointer-coarse:h-11",
};

const LABEL = "font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]";

function SearchBox({ state, t, className, inputRef, density }: { state: IBrowseToolbarState; t: BrowseT; className?: string; inputRef?: React.Ref<HTMLInputElement>; density: Density }): React.ReactElement {
    return (
        <div className={cn("relative min-w-0", className)}>
            <SearchIcon className="pointer-events-none absolute top-1/2 left-3.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
            <input
                ref={inputRef}
                value={state.query}
                onChange={(e) => state.setQuery(e.target.value)}
                placeholder={t("browse.search.placeholder")}
                aria-label={t("browse.search.aria")}
                className={cn(
                    "w-full rounded-[9px] border border-border bg-secondary/50 pr-3.5 pl-9 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40",
                    density === "inline" ? "h-11 pointer-coarse:h-11 sm:h-9.5" : density === "sheet" ? "h-11" : "h-9 pointer-coarse:h-11",
                )}
            />
        </div>
    );
}

/** Grid or list. Inline it is two icon buttons; in a panel it spells its two words out, because a panel has the room and a stacked label over two bare icons reads as a riddle. */
function ViewToggle({ state, t, className, density }: { state: IBrowseToolbarState; t: BrowseT; className?: string; density: Density }): React.ReactElement {
    const inline = density === "inline";
    return (
        <fieldset className={cn("flex shrink-0 gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75", inline ? "" : "grid grid-cols-2", className)} aria-label={t("browse.view.aria")}>
            {(["grid", "list"] as const).map((mode) => (
                <button
                    key={mode}
                    type="button"
                    onClick={() => state.setView(mode)}
                    aria-pressed={state.view === mode}
                    aria-label={inline ? t(`browse.view.${mode}`) : undefined}
                    className={cn(
                        "flex cursor-pointer items-center justify-center gap-1.5 rounded-md font-sans font-semibold text-[12px] transition-colors focus-visible:ring-2 focus-visible:ring-ring/60",
                        inline ? "h-11 pointer-coarse:h-11 pointer-coarse:w-11 w-11 sm:h-7.5 sm:w-9" : density === "sheet" ? "h-10" : "h-7 pointer-coarse:h-10",
                        state.view === mode ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    {mode === "grid" ? <LayoutGridIcon className="size-3.5" aria-hidden="true" /> : <ListIcon className="size-3.5" aria-hidden="true" />}
                    {inline ? null : t(`browse.view.${mode}`)}
                </button>
            ))}
        </fieldset>
    );
}

/** The category pills. Inline they are a scroller of one line; in a panel they wrap, because a scroller inside a 320 px popover hides the last two. */
function CategoryPills({ state, t, className, density }: { state: IBrowseToolbarState; t: BrowseT; className?: string; density: Density }): React.ReactElement {
    const inline = density === "inline";
    return (
        <fieldset aria-label={t("browse.toolbar.category")} className={cn("min-w-0", inline ? "msv-scroll -my-1 flex max-w-full items-center gap-1.5 overflow-x-auto py-1" : "flex flex-wrap gap-1.5", className)}>
            {FILTER_ORDER.map((key) => (
                <button
                    key={key}
                    type="button"
                    onClick={() => state.setFilter(key)}
                    aria-pressed={state.filter === key}
                    className={cn(
                        "min-w-11 pointer-coarse:min-w-11 shrink-0 cursor-pointer rounded-full border px-3 font-sans font-semibold text-[12px] transition-colors focus-visible:ring-2 focus-visible:ring-ring/60",
                        TALL[density],
                        inline ? "sm:min-w-0" : "",
                        state.filter === key ? "border-primary/55 bg-primary/12 text-foreground" : "border-border bg-secondary/40 text-muted-foreground hover:text-foreground",
                    )}
                >
                    {t(`browse.filter.${key}`)}
                </button>
            ))}
        </fieldset>
    );
}

/**
 * The read-state segmented control. THE THREE STATE PILLS ARE ALSO THE
 * TICKET'S LEGEND: each wears the card's own bookmark in that state's ink,
 * which is what the colour meant all along and nowhere said. In a panel the
 * four states sit two by two, because "In progress" and "Finished" beside
 * "Any" and "Unread" are 292 px on one line and a 320 px popover has 286.
 */
function ReadSegmented({ state, t, className, density }: { state: IBrowseToolbarState; t: BrowseT; className?: string; density: Density }): React.ReactElement {
    const inline = density === "inline";
    return (
        <fieldset className={cn("gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75", inline ? "msv-scroll flex min-w-0 max-w-full overflow-x-auto" : "grid grid-cols-2", className)} aria-label={t("browse.read.aria")}>
            {READ_FILTERS.map((key) => (
                <button
                    key={key}
                    type="button"
                    onClick={() => state.setReadFilter(key)}
                    aria-pressed={state.readFilter === key}
                    className={cn(
                        "flex min-w-11 pointer-coarse:min-w-11 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-md px-2.5 font-sans font-semibold text-[11.5px] transition-colors focus-visible:ring-2 focus-visible:ring-ring/60",
                        inline ? "h-11 pointer-coarse:h-11 sm:h-7 sm:min-w-0" : density === "sheet" ? "h-10" : "h-7 pointer-coarse:h-10",
                        state.readFilter === key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground",
                    )}
                >
                    <ReadMark state={key} />
                    {t(`browse.read.${key}`)}
                </button>
            ))}
        </fieldset>
    );
}

/** The order, a native select: six orders in a row are a scroller nobody reads. */
function SortSelect({ state, t, className, density }: { state: IBrowseToolbarState; t: BrowseT; className?: string; density: Density }): React.ReactElement {
    const inline = density === "inline";
    return (
        <label className={cn("flex min-w-0 gap-2", inline ? "items-center" : "flex-col items-stretch", className)}>
            <span className={cn("shrink-0", LABEL)}>{t("browse.sort.label")}</span>
            <select
                value={state.sort}
                onChange={(e) => state.setSort(e.target.value as LibrarySort)}
                className={cn("min-w-0 rounded-[9px] border border-border bg-secondary/50 px-2.5 font-sans text-[12.5px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40", TALL[density])}
            >
                {LIBRARY_SORTS.map((key) => (
                    <option key={key} value={key}>
                        {t(`browse.sort.${key}`)}
                    </option>
                ))}
            </select>
        </label>
    );
}

/** A panel section: the mono label over its control. The fieldsets carry their own accessible names, so the visible label is `aria-hidden` rather than read twice. */
function PanelField({ label, children }: { label: string | null; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2">
            {label ? (
                <span aria-hidden="true" className={LABEL}>
                    {label}
                </span>
            ) : null}
            {children}
        </div>
    );
}

/** The controls stacked, for the phone's Filters sheet (no search: the box is on screen beside the button) and for the sticky bar's panel (everything). */
export function ToolbarPanel({ state, density, withSearch, searchRef }: { state: IBrowseToolbarState; density: "sheet" | "popover"; withSearch: boolean; searchRef?: React.Ref<HTMLInputElement> }): React.ReactElement {
    const t: BrowseT = useT("story");
    return (
        <div className="flex flex-col gap-4">
            {withSearch ? <SearchBox state={state} t={t} inputRef={searchRef} density={density} /> : null}
            <PanelField label={t("browse.toolbar.category")}>
                <CategoryPills state={state} t={t} density={density} />
            </PanelField>
            <PanelField label={t("browse.read.aria")}>
                <ReadSegmented state={state} t={t} density={density} />
            </PanelField>
            {/* The select's own label is its visible one, so the field prints no second. */}
            <SortSelect state={state} t={t} density={density} />
            <PanelField label={t("browse.view.aria")}>
                <ViewToggle state={state} t={t} density={density} />
            </PanelField>
        </div>
    );
}

/** The count of non-default filters, as a small pill on a button's corner. Hidden from the accessibility tree: the button's own label carries the count in words. */
function CountBadge({ count, className }: { count: number; className?: string }): React.ReactElement | null {
    if (count === 0) return null;
    return (
        <span aria-hidden="true" className={cn("flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 font-mono font-semibold text-[9.5px] text-primary-foreground tabular-nums leading-none", className)}>
            {count}
        </span>
    );
}

/** The inline toolbar in the page head. */
export function BrowseToolbar({ state }: { state: IBrowseToolbarState }): React.ReactElement {
    const t: BrowseT = useT("story");
    const count = activeFilterCount(state);
    return (
        // `xl:contents` on the two row wrappers makes their children items of
        // the outer row, so the same five controls are one row at 1280 and two
        // below it without being rendered twice. The layout toggle is `order-last`
        // there so it ends the row, as it ends the first row below it.
        <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:gap-2">
            <div className="flex items-center gap-2 xl:contents">
                <SearchBox state={state} t={t} density="inline" className="flex-1 xl:max-w-[360px]" />
                <ViewToggle state={state} t={t} density="inline" className="max-sm:hidden xl:order-last" />
                <Sheet>
                    <SheetTrigger render={<Button variant="outline" className="h-11 shrink-0 sm:hidden" aria-label={count > 0 ? t("browse.toolbar.filtersAria", { count }) : undefined} />}>
                        <SlidersHorizontalIcon aria-hidden="true" />
                        {t("browse.toolbar.filters")}
                        <CountBadge count={count} />
                    </SheetTrigger>
                    <SheetPopup side="bottom" className="max-h-[88dvh]" closeProps={{ className: "absolute end-2 top-2 size-11" }}>
                        <SheetHeader>
                            <SheetTitle>{t("browse.toolbar.filters")}</SheetTitle>
                        </SheetHeader>
                        <SheetPanel>
                            <ToolbarPanel state={state} density="sheet" withSearch={false} />
                        </SheetPanel>
                        <SheetFooter>
                            <SheetClose render={<Button className="h-11 w-full" />}>{t("browse.toolbar.done")}</SheetClose>
                        </SheetFooter>
                    </SheetPopup>
                </Sheet>
            </div>
            <div className="flex flex-wrap items-center gap-2 max-sm:hidden xl:contents">
                <CategoryPills state={state} t={t} density="inline" className="xl:min-w-0 xl:shrink" />
                <ReadSegmented state={state} t={t} density="inline" className="shrink-0" />
                <SortSelect state={state} t={t} density="inline" className="ms-auto xl:ms-0" />
            </div>
        </div>
    );
}

/**
 * The sticky bar's "Search and filters" button: a popover at 640 and up, a
 * bottom sheet under it. Both are rendered and one is hidden by CSS, so the
 * server and the first client render agree whatever the width.
 *
 * It wears the non-default count as a badge, and a DOT while the search box has
 * text in it, so a filtered page says so from under the pinned bar. The search
 * box takes focus when either opens: this is the only search a scrolled reader
 * has.
 */
export function ToolbarButton({ state }: { state: IBrowseToolbarState }): React.ReactElement {
    const t: BrowseT = useT("story");
    const count = activeFilterCount(state);
    const searching = state.query.trim() !== "";
    const label = count > 0 ? t("browse.toolbar.openActive", { count }) : t("browse.toolbar.open");
    const popoverSearch = useRef<HTMLInputElement | null>(null);
    const sheetSearch = useRef<HTMLInputElement | null>(null);
    const marks = (
        <>
            <SearchIcon className="size-4" aria-hidden="true" />
            <CountBadge count={count} className="absolute -top-1 -right-1" />
            {searching ? <span aria-hidden="true" className={cn("absolute size-2 rounded-full bg-primary ring-2 ring-background", count > 0 ? "-right-0.5 -bottom-0.5" : "-top-0.5 -right-0.5")} /> : null}
        </>
    );
    const face =
        "relative flex shrink-0 cursor-pointer items-center justify-center rounded-[10px] border border-border bg-secondary/50 text-foreground transition-colors hover:bg-secondary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 data-popup-open:border-primary/55 data-popup-open:bg-primary/12";

    return (
        <>
            <Popover>
                <Tooltip>
                    <TooltipTrigger render={<PopoverTrigger aria-label={label} className={cn(face, "pointer-coarse:size-11 size-9 max-sm:hidden")} />}>{marks}</TooltipTrigger>
                    <TooltipContent>{t("browse.toolbar.open")}</TooltipContent>
                </Tooltip>
                <PopoverPopup align="end" sideOffset={8} className="w-80" initialFocus={popoverSearch}>
                    <ToolbarPanel state={state} density="popover" withSearch searchRef={popoverSearch} />
                </PopoverPopup>
            </Popover>
            <Sheet>
                <SheetTrigger aria-label={label} className={cn(face, "size-11 sm:hidden")}>
                    {marks}
                </SheetTrigger>
                <SheetPopup side="bottom" className="max-h-[88dvh]" initialFocus={sheetSearch} closeProps={{ className: "absolute end-2 top-2 size-11" }}>
                    <SheetHeader>
                        <SheetTitle>{t("browse.toolbar.open")}</SheetTitle>
                    </SheetHeader>
                    <SheetPanel>
                        <ToolbarPanel state={state} density="sheet" withSearch searchRef={sheetSearch} />
                    </SheetPanel>
                    <SheetFooter>
                        <SheetClose render={<Button className="h-11 w-full" />}>{t("browse.toolbar.done")}</SheetClose>
                    </SheetFooter>
                </SheetPopup>
            </Sheet>
        </>
    );
}
