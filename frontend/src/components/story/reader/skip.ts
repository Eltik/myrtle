/**
 * SKIP, the client's own: the button opens the story's SUMMARY, and confirming
 * it goes straight to the end. Nothing is fast-forwarded.
 *
 * This replaced a ratio-0 fast-forward (every scene duration times zero, one
 * advance per 16 ms frame, stopped by a decision, the end or a click) that
 * stepped visibly through the whole script. The game does not do that: its Skip
 * shows the story summary (`story_review_table.infoUnlockDatas[].storyInfo`,
 * served as the script's `synopsis`) with a skip and a cancel, and the skip
 * lands on the end. Every press shows the sheet, so the old once-per-session
 * confirm has nothing left to guard and is gone with it.
 *
 * Pure, so the rule is tested without a DOM.
 */

/** `open` while the synopsis sheet is up. */
export interface SkipState {
    open: boolean;
}

export const SKIP_CLOSED: SkipState = { open: false };

/**
 * What moves it. `press` is the button and `S`; `confirm` and `cancel` are the
 * sheet's two answers (Escape and a click outside are `cancel`).
 */
export type SkipEvent = "press" | "confirm" | "cancel";

/**
 * Whether Skip means anything right now: only while a story is being READ.
 * On the title card there is nothing to skip past yet, and on the end card
 * nothing left.
 */
export function skipAvailable(phase: string): boolean {
    return phase === "reading";
}

/**
 * The next state, and whether the reader jumps to the end. A press toggles the
 * sheet, so the button and `S` also close it; a press outside a reading phase
 * does nothing.
 */
export function nextSkip(s: SkipState, e: SkipEvent, phase: string): { state: SkipState; toEnd: boolean } {
    switch (e) {
        case "press":
            if (s.open) return { state: SKIP_CLOSED, toEnd: false };
            return { state: { open: skipAvailable(phase) }, toEnd: false };
        case "confirm":
            // Confirming from a sheet left open over the end card is a close.
            return { state: SKIP_CLOSED, toEnd: s.open && skipAvailable(phase) };
        case "cancel":
            return { state: SKIP_CLOSED, toEnd: false };
    }
}

/**
 * The synopsis as paragraphs: one per line of the file, trimmed, blanks
 * dropped. The EN files are one to a few paragraphs of 18 to 490 bytes, so a
 * missing or blank synopsis is an empty list, and the sheet says so.
 */
export function synopsisParagraphs(synopsis: string | null | undefined): string[] {
    if (!synopsis) return [];
    return synopsis
        .split(/\r?\n/)
        .map((p) => p.trim())
        .filter((p) => p !== "");
}
