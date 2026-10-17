/**
 * THE QUESTION CHIPS ABOVE THE COMPOSER, once the thread has turns.
 *
 * The empty thread lists its questions in full (the panel's suggestions in the
 * reader, the era's popular questions on the stories page). After the first
 * question that list is gone, so the same questions stay one tap away here: a
 * single row that scrolls sideways, so it never grows taller than one chip and
 * never pushes the composer down on a phone. A question already in the thread
 * (asked, opened from a prepared answer, or re-shown from My questions) has no
 * chip; when every one has been asked the row is not rendered.
 *
 * Kill switch: `VITE_TREVOR_CHIP_ROW=0` drops the row, so the chips live in the
 * empty thread only, as before 2026-10-17.
 */
import type React from "react";
import type { IThread } from "#/lib/trevor/thread";

export interface IQuestionChip {
    key: string;
    question: string;
    /** The chip carries its answer: picking it adds the answer with no request. */
    prepared: boolean;
    onPick: () => void;
}

/** False only when the build set `VITE_TREVOR_CHIP_ROW=0`. */
export function chipRowEnabled(): boolean {
    return import.meta.env.VITE_TREVOR_CHIP_ROW !== "0";
}

const fold = (question: string): string => question.trim().toLowerCase();

/** The chips whose question is not yet in the thread (compared trimmed, case-folded). */
export function unaskedChips(chips: IQuestionChip[], thread: IThread): IQuestionChip[] {
    const asked = new Set(thread.turns.map((turn) => fold(turn.kind === "ask" ? turn.state.question : turn.question)));
    return chips.filter((chip) => !asked.has(fold(chip.question)));
}

export function QuestionChipRow({ label, chips }: { label: string; chips: IQuestionChip[] }): React.ReactElement | null {
    if (chips.length === 0) return null;
    return (
        <section aria-label={label} data-trevor-chip-row>
            {/* Bleeds to the footer's edges so a chip cut off at the side reads as "scroll for more". */}
            <ul className="-mx-4 flex gap-2 overflow-x-auto overscroll-x-contain px-4 [scrollbar-width:none] sm:-mx-6 sm:px-6">
                {chips.map((chip) => (
                    <li key={chip.key} className="shrink-0">
                        <button
                            type="button"
                            title={chip.question}
                            onClick={chip.onPick}
                            className="h-8 max-w-[22rem] truncate rounded-full border border-border bg-background px-3 text-[13px] transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset max-sm:h-11 max-sm:max-w-[15rem]"
                            data-trevor-chip
                            data-trevor-prepared={chip.prepared ? "" : undefined}
                        >
                            {chip.question}
                        </button>
                    </li>
                ))}
            </ul>
        </section>
    );
}
