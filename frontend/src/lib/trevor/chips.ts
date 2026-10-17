/** Suggested questions offered as chips, and which of them the thread has not asked yet. */
import type { IThread } from "./thread";

export interface IQuestionChip {
    key: string;
    question: string;
    /** The chip carries its answer: picking it adds the answer with no request. */
    prepared: boolean;
    onPick: () => void;
}

/** A question as compared with another: trimmed and lower-cased. */
export function foldQuestion(question: string): string {
    return question.trim().toLowerCase();
}

/** The chips whose question is not yet in the thread (asked, opened from a prepared answer, or re-shown). */
export function unaskedChips(chips: IQuestionChip[], thread: IThread): IQuestionChip[] {
    const asked = new Set(thread.turns.map((turn) => foldQuestion(turn.kind === "ask" ? turn.state.question : turn.question)));
    return chips.filter((chip) => !asked.has(foldQuestion(chip.question)));
}
