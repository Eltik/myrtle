/**
 * THE ASK TAB'S THREAD: every turn of one sitting, oldest first, each with
 * its own state.
 *
 * The reader may ask again while earlier questions are still queued or
 * streaming. Each asked turn carries its own `AskState` (askState.ts), moved
 * by events addressed to its id; `useTrevorThread` gives each its own request
 * and watcher. Turns render in the order asked, whatever order they finish in.
 *
 * The backend lets one account hold `MAX_OPEN` open jobs (its
 * `MAX_OPEN_PER_USER`, 3) and refuses a fourth with 409. So a turn asked while
 * that many are open is `held` here, not sent, and the oldest held turn is
 * sent as soon as one settles. Twins (the same question while it is open)
 * join one job server side, so open jobs are counted by distinct job id.
 *
 * The thread lives in the hook's state: closing the sheet keeps it, closing
 * the reader drops it, and nothing is stored or sent.
 */
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import { type AskEvent, type AskState, askPending, askReducer, hasJob } from "./askState";

/** The backend's `MAX_OPEN_PER_USER`: open jobs one account may hold. */
export const MAX_OPEN = 3;

export type TurnState = Exclude<AskState, { kind: "idle" }>;

/** A question the reader asked, followed by its own request and watcher. */
export interface IAskTurn {
    id: number;
    kind: "ask";
    state: TurnState;
    /** The Turnstile token it was asked with, sent with it even if it waited. */
    token: string | null;
    /** Re-shown from "My questions": its spoiler horizon is not known here. */
    restored?: true;
}

/** A suggested question opened from the panel's prepared answer: no request was made. */
interface IBankTurn {
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

export type ThreadAction =
    | { type: "ask"; question: string; token: string | null }
    | { type: "bank"; suggestion: { id: string; question: string; answer: string } }
    /** A question asked earlier (the stories tab's "My questions"), re-shown as it stands: answered, or still open and followed again. */
    | { type: "restore"; state: TurnState }
    | { type: "turn"; id: number; event: AskEvent };

/**
 * Open jobs the thread holds against the backend's cap: each turn being sent
 * (its job id is not known yet), plus each distinct job that is queued,
 * streaming or offline (an offline job is still queued server side).
 */
export function openSlots(thread: IThread): number {
    let sending = 0;
    const jobs = new Set<string>();
    for (const turn of thread.turns) {
        if (turn.kind !== "ask") continue;
        const s = turn.state;
        if (s.kind === "sending") sending++;
        else if (hasJob(s)) jobs.add(s.jobId);
    }
    return sending + jobs.size;
}

/** Send held turns, oldest first, while a slot is free. */
function promote(thread: IThread, cap: number): IThread {
    let open = openSlots(thread);
    if (open >= cap || !thread.turns.some((t) => t.kind === "ask" && t.state.kind === "held")) return thread;
    const turns = thread.turns.map((t) => {
        if (open >= cap || t.kind !== "ask" || t.state.kind !== "held") return t;
        open++;
        return { ...t, state: askReducer(t.state, { type: "send", question: t.state.question }) as TurnState };
    });
    return { ...thread, turns };
}

export function threadReducer(thread: IThread, action: ThreadAction, cap: number = MAX_OPEN): IThread {
    switch (action.type) {
        case "ask": {
            const state: TurnState = openSlots(thread) < cap ? { kind: "sending", question: action.question } : { kind: "held", question: action.question };
            return { turns: [...thread.turns, { id: thread.nextId, kind: "ask", state, token: action.token }], nextId: thread.nextId + 1 };
        }
        case "bank": {
            const s = action.suggestion;
            return { turns: [...thread.turns, { id: thread.nextId, kind: "bank", suggestionId: s.id, question: s.question, answer: s.answer }], nextId: thread.nextId + 1 };
        }
        case "restore": {
            if (action.state.kind === "held" || action.state.kind === "sending") return thread;
            return { turns: [...thread.turns, { id: thread.nextId, kind: "ask", state: action.state, token: null, restored: true }], nextId: thread.nextId + 1 };
        }
        case "turn": {
            let changed = false;
            const turns = thread.turns.map((t) => {
                if (t.id !== action.id || t.kind !== "ask") return t;
                const next = askReducer(t.state, action.event);
                if (next === t.state || next.kind === "idle") return t;
                changed = true;
                return { ...t, state: next };
            });
            return changed ? promote({ ...thread, turns }, cap) : thread;
        }
    }
}

/** Turns not settled yet: waiting, being sent, queued or streaming. */
export function pendingCount(thread: IThread): number {
    return thread.turns.filter((t) => t.kind === "ask" && askPending(t.state)).length;
}

function spoilerOf(s: TurnState): TrevorSpoilerSafe | null | undefined {
    return hasJob(s) || s.kind === "done" ? s.spoiler : undefined;
}

/**
 * The spoiler horizon the backend last reported, newest turn first.
 * `undefined` when no ask has reported one yet (prepared answers carry none);
 * `null` is the backend's "the very start".
 */
export function latestSpoiler(thread: IThread): TrevorSpoilerSafe | null | undefined {
    for (let i = thread.turns.length - 1; i >= 0; i--) {
        const turn = thread.turns[i];
        if (turn?.kind !== "ask" || turn.restored) continue;
        const s = spoilerOf(turn.state);
        if (s !== undefined) return s;
    }
    return undefined;
}
