/**
 * The browse toolbar's pure half: what "default" is for each control, and how
 * many controls have been moved off it. The count is what the Filters button
 * and the sticky bar's search button badge, so a reader scrolled past the head
 * can see that the page under them is filtered without opening anything.
 */
import type { LibrarySort, ReadFilter } from "./derive";
import type { FilterKey } from "./sections";

export interface IToolbarFilters {
    filter: FilterKey;
    readFilter: ReadFilter;
    sort: LibrarySort;
}

/** The value each control starts on, and the one that hides and reorders nothing. */
export const TOOLBAR_DEFAULTS: Readonly<IToolbarFilters> = { filter: "all", readFilter: "any", sort: "default" };

/**
 * How many of category, read state and sort are off their default, 0 to 3.
 *
 * The SEARCH QUERY IS NOT COUNTED, and the layout toggle is not either. The
 * query is text the reader can see in the box (the sticky button marks it with
 * a dot instead of a number), and grid or list changes how the page looks, not
 * what is on it, so counting it would badge a page that hides nothing.
 */
export function activeFilterCount(values: IToolbarFilters): number {
    return (values.filter !== TOOLBAR_DEFAULTS.filter ? 1 : 0) + (values.readFilter !== TOOLBAR_DEFAULTS.readFilter ? 1 : 0) + (values.sort !== TOOLBAR_DEFAULTS.sort ? 1 : 0);
}

/** `smooth`, unless the reader asked the system for less motion, in which case the jump is instant. */
export function scrollBehaviorFor(reducedMotion: boolean): ScrollBehavior {
    return reducedMotion ? "auto" : "smooth";
}
