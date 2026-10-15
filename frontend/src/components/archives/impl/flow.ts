/**
 * Layout of the Hidden Factions flow chart: the faction a card shows on the
 * left, the faction the game also counts on the right, one band per pairing.
 *
 * The scale is absolute: every operator is {@link FLOW_UNIT} pixels of band,
 * so one person is always a visible band and the chart grows taller instead
 * of squeezing bands into hairlines. Both columns are ordered by alternating
 * barycentre sweeps (each node sits at the weighted mean position of the
 * nodes it links to), keeping whichever ordering crosses the fewest people,
 * and each node stacks its bands in the order of the nodes at their other
 * end, so no node crosses its own bands.
 */

/** Pixels of band per operator. */
export const FLOW_UNIT = 6;
/** Smallest slot a node takes, so a one-person node still fits its label. */
export const FLOW_ROW = 26;
/** Space between two nodes in a column. */
export const FLOW_GAP = 8;
/** Barycentre sweeps tried; the ordering settles in far fewer on the real data. */
const SWEEPS = 8;

export interface IFlowLink {
    shown: string;
    hidden: string;
    count: number;
}

export interface IFlowNode {
    id: string;
    /** Operators through this node. */
    count: number;
    /** Top of the node's slot. */
    top: number;
    slot: number;
    /** Top and height of the node's bar inside its slot. */
    barTop: number;
    barHeight: number;
}

export interface IFlowBand {
    shown: string;
    hidden: string;
    count: number;
    /** Centre of the band at the left and at the right end. */
    y1: number;
    y2: number;
    /** Stroke width: a pixel narrower than its share, so neighbours keep a gap. */
    width: number;
}

export interface IFlowLayout {
    left: IFlowNode[];
    right: IFlowNode[];
    bands: IFlowBand[];
    height: number;
    /** Weighted crossings of the chosen ordering: pairs of people whose bands cross. */
    crossings: number;
}

/** Pairs of people whose bands cross under the given column positions. */
export function countCrossings(links: readonly IFlowLink[], leftAt: ReadonlyMap<string, number>, rightAt: ReadonlyMap<string, number>): number {
    let n = 0;
    for (let i = 0; i < links.length; i++) {
        for (let j = i + 1; j < links.length; j++) {
            const a = links[i];
            const b = links[j];
            const dl = (leftAt.get(a.shown) ?? 0) - (leftAt.get(b.shown) ?? 0);
            const dr = (rightAt.get(a.hidden) ?? 0) - (rightAt.get(b.hidden) ?? 0);
            if (dl * dr < 0) n += a.count * b.count;
        }
    }
    return n;
}

function positions(order: readonly string[]): Map<string, number> {
    return new Map(order.map((id, i) => [id, i]));
}

function totals(links: readonly IFlowLink[], side: "shown" | "hidden"): Map<string, number> {
    const out = new Map<string, number>();
    for (const l of links) out.set(l[side], (out.get(l[side]) ?? 0) + l.count);
    return out;
}

/** `ids` sorted by the weighted mean position of what they link to; ties by size, then name. */
function barycentre(ids: readonly string[], links: readonly IFlowLink[], side: "shown" | "hidden", otherAt: ReadonlyMap<string, number>, size: ReadonlyMap<string, number>, nameOf: (id: string) => string): string[] {
    const other = side === "shown" ? "hidden" : "shown";
    const mean = new Map<string, number>();
    for (const id of ids) {
        let sum = 0;
        let weight = 0;
        for (const l of links) {
            if (l[side] !== id) continue;
            sum += (otherAt.get(l[other]) ?? 0) * l.count;
            weight += l.count;
        }
        mean.set(id, weight > 0 ? sum / weight : 0);
    }
    return [...ids].sort((a, b) => (mean.get(a) ?? 0) - (mean.get(b) ?? 0) || (size.get(b) ?? 0) - (size.get(a) ?? 0) || nameOf(a).localeCompare(nameOf(b)));
}

/** Column orders: the left starts by size, then barycentre sweeps alternate and the ordering with the fewest crossings stands. */
export function orderFlow(links: readonly IFlowLink[], nameOf: (id: string) => string): { left: string[]; right: string[]; crossings: number } {
    const leftSize = totals(links, "shown");
    const rightSize = totals(links, "hidden");
    let left = [...leftSize.keys()].sort((a, b) => (leftSize.get(b) ?? 0) - (leftSize.get(a) ?? 0) || nameOf(a).localeCompare(nameOf(b)));
    let right = barycentre([...rightSize.keys()], links, "hidden", positions(left), rightSize, nameOf);
    let best = { left, right, crossings: countCrossings(links, positions(left), positions(right)) };
    for (let i = 0; i < SWEEPS && best.crossings > 0; i++) {
        left = barycentre(left, links, "shown", positions(right), leftSize, nameOf);
        right = barycentre(right, links, "hidden", positions(left), rightSize, nameOf);
        const crossings = countCrossings(links, positions(left), positions(right));
        if (crossings < best.crossings) best = { left, right, crossings };
    }
    return best;
}

function stack(order: readonly string[], size: ReadonlyMap<string, number>, offset: number): IFlowNode[] {
    let y = offset;
    return order.map((id) => {
        const count = size.get(id) ?? 0;
        const barHeight = count * FLOW_UNIT;
        const slot = Math.max(barHeight, FLOW_ROW);
        const node = { id, count, top: y, slot, barTop: y + (slot - barHeight) / 2, barHeight };
        y += slot + FLOW_GAP;
        return node;
    });
}

function columnHeight(nodes: readonly IFlowNode[]): number {
    return nodes.reduce((h, n) => h + n.slot, 0) + FLOW_GAP * Math.max(0, nodes.length - 1);
}

/** Merge duplicate pairings and lay the chart out; the shorter column is centred on the taller. */
export function layoutFlow(raw: readonly IFlowLink[], nameOf: (id: string) => string): IFlowLayout {
    const merged = new Map<string, IFlowLink>();
    for (const l of raw) {
        const key = `${l.shown}\u0000${l.hidden}`;
        const row = merged.get(key);
        if (row) row.count += l.count;
        else merged.set(key, { ...l });
    }
    const links = [...merged.values()];
    const { left: leftOrder, right: rightOrder, crossings } = orderFlow(links, nameOf);
    const leftSize = totals(links, "shown");
    const rightSize = totals(links, "hidden");
    const lh = columnHeight(stack(leftOrder, leftSize, 0));
    const rh = columnHeight(stack(rightOrder, rightSize, 0));
    const height = Math.max(lh, rh);
    const left = stack(leftOrder, leftSize, (height - lh) / 2);
    const right = stack(rightOrder, rightSize, (height - rh) / 2);
    const leftAt = positions(leftOrder);
    const rightAt = positions(rightOrder);

    const y1 = new Map<IFlowLink, number>();
    for (const node of left) {
        let used = 0;
        const out = links.filter((l) => l.shown === node.id).sort((a, b) => (rightAt.get(a.hidden) ?? 0) - (rightAt.get(b.hidden) ?? 0));
        for (const l of out) {
            y1.set(l, node.barTop + (used + l.count / 2) * FLOW_UNIT);
            used += l.count;
        }
    }
    const y2 = new Map<IFlowLink, number>();
    for (const node of right) {
        let used = 0;
        const into = links.filter((l) => l.hidden === node.id).sort((a, b) => (leftAt.get(a.shown) ?? 0) - (leftAt.get(b.shown) ?? 0));
        for (const l of into) {
            y2.set(l, node.barTop + (used + l.count / 2) * FLOW_UNIT);
            used += l.count;
        }
    }
    const bands = links.map((l) => ({ ...l, y1: y1.get(l) ?? 0, y2: y2.get(l) ?? 0, width: Math.max(1.5, l.count * FLOW_UNIT - 1) }));
    return { left, right, bands, height, crossings };
}
