/**
 * THE RECORDS GRID MOUNTS A PAGE AT A TIME. A search forces the operator
 * records section open, and "a" matched 286 record sets, which mounted 286
 * cards (about 5,000 elements) in the keystroke's own task: the DOM went from
 * 2,192 elements to 7,257 and the keystroke took 570 ms. The grid now mounts
 * {@link RECORD_PAGE} and a "Show more" button adds the next page.
 */

/**
 * Cards per page. 60 is the smallest count that divides by every column count
 * the records grid takes (2, 3, 4, 5, 6), so no page ends on a ragged row (40
 * left two orphans at six columns). It costs about 400 elements over the 40
 * measured at 2,911 on the broadest one-letter query, against a keystroke
 * that now takes ~30 ms.
 */
export const RECORD_PAGE = 60;

/** The first `limit` items, and how many are behind the button. A limit under one page is a page. */
export function pageOf<T>(items: readonly T[], limit: number, page: number = RECORD_PAGE): { shown: readonly T[]; hidden: number } {
    const cap = Math.max(page, Math.floor(limit));
    if (items.length <= cap) return { shown: items, hidden: 0 };
    return { shown: items.slice(0, cap), hidden: items.length - cap };
}

/** How many the next press of "Show more" reveals: a page, or the remainder when less than a page is left. */
export function nextPage(hidden: number, page: number = RECORD_PAGE): number {
    return Math.max(0, Math.min(page, hidden));
}
