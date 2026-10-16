/**
 * THE ASK TAB'S THREAD: the turns of one sitting, oldest first.
 *
 * `useTrevorAsk` follows one question at a time; this module keeps the ones
 * before it. When the reader asks again (or opens a prepared answer), the
 * question on screen is frozen into the thread as it stands and the hook
 * starts over. The thread lives in component state only: closing the reader
 * drops it, and nothing is stored or sent.
 *
 * A turn is frozen only when no question is in flight (the composer and the
 * suggestion chips are disabled while one is), so a frozen turn is always
 * done, failed, refused or offline. An offline turn frozen this way lost its
 * watcher: it is drawn as "went offline before answering", with no resume.
 */
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import type { AskState } from "./askState";

/** A question the hook followed, frozen as it stood when the next one began. */
export interface IAskTurn {
    id: number;
    kind: "ask";
    state: Exclude<AskState, { kind: "idle" }>;
}

/** A suggested question opened from the panel's prepared answer: no request was made. */
export interface IBankTurn {
    id: number;
    kind: "bank";
    suggestionId: string;
    question: string;
    answer: string;
}

export type ThreadTurn = IAskTurn | IBankTurn;

export interface IThread {
    turns: ThreadTurn[];
    nextId: number;
}

export const EMPTY_THREAD: IThread = { turns: [], nextId: 1 };

/** Freeze the question on screen into the thread; an idle hook adds nothing. */
export function freezeLive(thread: IThread, live: AskState): IThread {
    if (live.kind === "idle") return thread;
    return { turns: [...thread.turns, { id: thread.nextId, kind: "ask", state: live }], nextId: thread.nextId + 1 };
}

/** Freeze the question on screen, then add a prepared answer after it. */
export function addBankTurn(thread: IThread, live: AskState, suggestion: { id: string; question: string; answer: string }): IThread {
    const t = freezeLive(thread, live);
    return { turns: [...t.turns, { id: t.nextId, kind: "bank", suggestionId: suggestion.id, question: suggestion.question, answer: suggestion.answer }], nextId: t.nextId + 1 };
}

/** The thread is empty: nothing frozen and nothing on screen. */
export function threadEmpty(thread: IThread, live: AskState): boolean {
    return thread.turns.length === 0 && live.kind === "idle";
}

function spoilerOf(s: AskState): TrevorSpoilerSafe | null | undefined {
    switch (s.kind) {
        case "queued":
        case "streaming":
        case "offline":
        case "done":
            return s.spoiler;
        default:
            return undefined;
    }
}

/**
 * The spoiler horizon the backend last reported, newest first: the question
 * on screen, then the frozen ones. `undefined` when no ask has reported one
 * yet (prepared answers carry none); `null` is the backend's "the very start".
 */
export function latestSpoiler(thread: IThread, live: AskState): TrevorSpoilerSafe | null | undefined {
    const now = spoilerOf(live);
    if (now !== undefined) return now;
    for (let i = thread.turns.length - 1; i >= 0; i--) {
        const turn = thread.turns[i];
        if (turn?.kind !== "ask") continue;
        const s = spoilerOf(turn.state);
        if (s !== undefined) return s;
    }
    return undefined;
}
