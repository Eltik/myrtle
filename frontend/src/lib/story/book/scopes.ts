/**
 * THE WIDER SCOPES: a mainline arc, a storyline shelf, and a range of a
 * reading order, each as an ordered list of GROUPS that `bookOf` turns into
 * one part per group. The arcs and shelves are the game's own
 * (`index.storylines`, `StorylineArc`); the reading orders are the library's
 * (`readingOrder.ts`), handed in as a flat list of group ids so this file
 * stays free of the component tree.
 */
import type { Storyline } from "#/types/generated/Storyline";
import type { BookScope } from "./types";

/** The shelf that carries the mainline arcs (`mainLine` on EN, the only one with arcs). */
function arcShelves(storylines: readonly Storyline[]): Storyline[] {
    return [...storylines].filter((l) => l.arcs.length > 0).sort((a, b) => a.sort - b.sort);
}

/** The mainline arc holding `groupId`, with its groups in the shelf's order, or null for a group on no arc. */
export function arcOf(storylines: readonly Storyline[], groupId: string): { name: string; groupIds: string[]; key: string } | null {
    for (const line of arcShelves(storylines)) {
        for (const arc of [...line.arcs].sort((a, b) => a.sort - b.sort)) {
            if (arc.groupIds.includes(groupId)) return { name: arc.name, groupIds: [...arc.groupIds], key: `${line.id}-${arc.sort}` };
        }
    }
    return null;
}

/**
 * The themed shelf (`ssLine_*`, or any shelf without arcs) holding `groupId`,
 * lowest `sort` first when a group sits on several (17 do on EN). The mainline
 * shelf is left out: its answer is the arc.
 */
export function storylineOf(storylines: readonly Storyline[], groupId: string): { name: string; groupIds: string[]; key: string } | null {
    const line = [...storylines].filter((l) => l.arcs.length === 0 && l.groupIds.includes(groupId)).sort((a, b) => a.sort - b.sort)[0];
    return line ? { name: line.name, groupIds: [...line.groupIds], key: line.id } : null;
}

/** The inclusive slice of an order between two group ids, in the order's direction whichever end was picked first. */
export function orderRange(order: readonly string[], from: string, to: string): string[] {
    const a = order.indexOf(from);
    const b = order.indexOf(to);
    if (a < 0 || b < 0) return [];
    return order.slice(Math.min(a, b), Math.max(a, b) + 1);
}

export function arcScope(storylines: readonly Storyline[], groupId: string): BookScope | null {
    const arc = arcOf(storylines, groupId);
    return arc ? { kind: "groups", ids: arc.groupIds, title: arc.name, id: `arc-${arc.key}` } : null;
}

export function storylineScope(storylines: readonly Storyline[], groupId: string): BookScope | null {
    const line = storylineOf(storylines, groupId);
    return line ? { kind: "groups", ids: line.groupIds, title: line.name, id: `storyline-${line.key}` } : null;
}

export function rangeScope(mode: string, order: readonly string[], from: string, to: string, title: string): BookScope {
    const ids = orderRange(order, from, to);
    return { kind: "groups", ids, title, id: `order-${mode}-${ids[0] ?? "none"}-${ids[ids.length - 1] ?? "none"}` };
}
