import type { RepeatMode } from "./types";

/**
 * THE QUEUE'S ARITHMETIC, with no audio and no store: a display order (the
 * list the queue panel draws), a play order (indices into it), the current
 * index, and what every edit does to all three. The engine calls these and
 * owns nothing of its own about order, so the rules are tested here once.
 */
export interface IQueue<T> {
    items: T[];
    /** Index into `items` of the current item; -1 when none. */
    index: number;
    /** The play order as indices into `items`. */
    order: number[];
}

export type Rng = () => number;

export function identityOrder(n: number): number[] {
    return Array.from({ length: n }, (_, i) => i);
}

/** A shuffled play order with `first` at the front (when it is a valid index). */
export function shuffledOrder(n: number, first: number, rng: Rng = Math.random): number[] {
    const rest = identityOrder(n).filter((i) => i !== first);
    for (let i = rest.length - 1; i > 0; i -= 1) {
        const j = Math.floor(rng() * (i + 1));
        const a = rest[i] as number;
        rest[i] = rest[j] as number;
        rest[j] = a;
    }
    return first >= 0 && first < n ? [first, ...rest] : rest;
}

/** A play order that names every index exactly once, or `null` when `order` does not. */
export function validOrder(order: readonly unknown[] | undefined, n: number): number[] | null {
    if (!Array.isArray(order) || order.length !== n) return null;
    const seen = new Set<number>();
    for (const v of order) {
        if (typeof v !== "number" || !Number.isInteger(v) || v < 0 || v >= n || seen.has(v)) return null;
        seen.add(v);
    }
    return order as number[];
}

/**
 * The index that plays after the current one, or `null` at the end.
 * `auto` is the track ending on its own: only then does repeat-one hold
 * the current track; a press of Next always moves on.
 */
export function nextIndex(q: IQueue<unknown>, repeat: RepeatMode, auto: boolean): number | null {
    const n = q.items.length;
    if (n === 0) return null;
    if (q.index < 0) return q.order[0] ?? null;
    if (auto && repeat === "one") return q.index;
    const at = q.order.indexOf(q.index);
    if (at + 1 < n) return q.order[at + 1] ?? null;
    return repeat === "all" ? (q.order[0] ?? null) : null;
}

/** The index before the current one in play order, wrapping under repeat-all; `null` at the start. */
export function previousIndex(q: IQueue<unknown>, repeat: RepeatMode): number | null {
    const n = q.items.length;
    if (n === 0 || q.index < 0) return null;
    const at = q.order.indexOf(q.index);
    if (at > 0) return q.order[at - 1] ?? null;
    return repeat === "all" && n > 1 ? (q.order[n - 1] ?? null) : null;
}

/** Insert right after the current item, in BOTH orders, so these play next whatever the shuffle. */
export function insertNext<T>(q: IQueue<T>, added: readonly T[]): IQueue<T> {
    if (added.length === 0) return q;
    const k = added.length;
    const at = q.index + 1;
    const items = [...q.items.slice(0, at), ...added, ...q.items.slice(at)];
    const shifted = q.order.map((i) => (i >= at ? i + k : i));
    const fresh = identityOrder(k).map((i) => at + i);
    const pos = q.index < 0 ? 0 : shifted.indexOf(q.index) + 1;
    const order = [...shifted.slice(0, pos), ...fresh, ...shifted.slice(pos)];
    return { items, index: q.index, order };
}

/**
 * Append to the end of the list. In play order a new item goes to the end
 * when shuffle is off and to a random place after the current item when it
 * is on, so queueing during a shuffle does not make the additions predictable.
 */
export function insertEnd<T>(q: IQueue<T>, added: readonly T[], shuffle: boolean, rng: Rng = Math.random): IQueue<T> {
    if (added.length === 0) return q;
    const base = q.items.length;
    const items = [...q.items, ...added];
    const order = [...q.order];
    for (let i = 0; i < added.length; i += 1) {
        const idx = base + i;
        if (!shuffle) {
            order.push(idx);
            continue;
        }
        const after = q.index < 0 ? 0 : order.indexOf(q.index) + 1;
        const slot = after + Math.floor(rng() * (order.length - after + 1));
        order.splice(slot, 0, idx);
    }
    return { items, index: q.index, order };
}

export interface IRemoval<T> extends IQueue<T> {
    /** The removed item was the current one. */
    removedCurrent: boolean;
    /** When it was: whether something after it in play order took its place (play it), else the queue's new current is only selected. */
    hasNext: boolean;
}

/**
 * Remove one item. Removing the current one makes the next item in PLAY order
 * current (the shuffle's next, not the list's); with nothing after it, the one
 * before it is selected instead and `hasNext` is false. Removing the last
 * item leaves an empty queue with index -1.
 */
export function removeAt<T>(q: IQueue<T>, i: number): IRemoval<T> {
    if (i < 0 || i >= q.items.length) return { ...q, removedCurrent: false, hasNext: false };
    const items = q.items.filter((_, j) => j !== i);
    const at = q.order.indexOf(i);
    const order = q.order.filter((j) => j !== i).map((j) => (j > i ? j - 1 : j));
    if (items.length === 0) return { items, index: -1, order: [], removedCurrent: i === q.index, hasNext: false };
    if (i !== q.index) return { items, index: q.index > i ? q.index - 1 : q.index, order, removedCurrent: false, hasNext: false };
    const hasNext = at < order.length;
    const index = hasNext ? (order[at] as number) : (order[order.length - 1] as number);
    return { items, index, order, removedCurrent: true, hasNext };
}

/**
 * Move one item in the display order. The current index follows its item;
 * the play order is remapped under shuffle and stays the identity without it,
 * because then play order IS the list order.
 */
export function moveItem<T>(q: IQueue<T>, from: number, to: number, shuffle: boolean): IQueue<T> {
    const n = q.items.length;
    if (from < 0 || from >= n || to < 0 || to >= n || from === to) return q;
    const items = [...q.items];
    const [moved] = items.splice(from, 1);
    items.splice(to, 0, moved as T);
    const remap = (i: number): number => {
        if (i === from) return to;
        if (from < to && i > from && i <= to) return i - 1;
        if (from > to && i >= to && i < from) return i + 1;
        return i;
    };
    const order = shuffle ? q.order.map(remap) : identityOrder(n);
    return { items, index: q.index < 0 ? -1 : remap(q.index), order };
}
