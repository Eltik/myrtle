import { describe, expect, it } from "vitest";
import { countCrossings, FLOW_GAP, FLOW_ROW, FLOW_UNIT, layoutFlow, orderFlow } from "./flow";

const name = (id: string) => id;

describe("flow layout", () => {
    it("gives every operator the same band thickness and grows taller instead of squeezing", () => {
        const one = layoutFlow([{ shown: "a", hidden: "x", count: 1 }], name);
        const many = layoutFlow(
            Array.from({ length: 30 }, (_, i) => ({ shown: `s${i}`, hidden: "x", count: 1 })),
            name,
        );
        expect(one.bands[0].width).toBe(FLOW_UNIT - 1);
        expect(many.bands.every((b) => b.width === FLOW_UNIT - 1)).toBe(true);
        expect(many.height).toBe(30 * FLOW_ROW + 29 * FLOW_GAP);
    });

    it("merges duplicate pairings into one band", () => {
        const l = layoutFlow(
            [
                { shown: "a", hidden: "x", count: 1 },
                { shown: "a", hidden: "x", count: 2 },
            ],
            name,
        );
        expect(l.bands).toHaveLength(1);
        expect(l.bands[0].count).toBe(3);
        expect(l.bands[0].width).toBe(3 * FLOW_UNIT - 1);
    });

    it("orders the right column to undo crossings", () => {
        // Sorted by size the left reads a, b; the right starts in name order
        // x, y, which would cross a->y over b->x.
        const links = [
            { shown: "a", hidden: "y", count: 3 },
            { shown: "b", hidden: "x", count: 2 },
        ];
        const { left, right, crossings } = orderFlow(links, name);
        expect(left).toEqual(["a", "b"]);
        expect(right).toEqual(["y", "x"]);
        expect(crossings).toBe(0);
    });

    it("never does worse than the size order it starts from", () => {
        const links = [
            { shown: "a", hidden: "x", count: 4 },
            { shown: "a", hidden: "z", count: 1 },
            { shown: "b", hidden: "y", count: 3 },
            { shown: "c", hidden: "x", count: 1 },
            { shown: "c", hidden: "z", count: 2 },
        ];
        const start = countCrossings(links, new Map(["a", "b", "c"].map((id, i) => [id, i])), new Map(["x", "y", "z"].map((id, i) => [id, i])));
        expect(orderFlow(links, name).crossings).toBeLessThanOrEqual(start);
    });

    it("stacks a node's bands inside its bar, in the order of their other ends", () => {
        const l = layoutFlow(
            [
                { shown: "a", hidden: "x", count: 2 },
                { shown: "a", hidden: "y", count: 1 },
                { shown: "b", hidden: "y", count: 2 },
            ],
            name,
        );
        const a = l.left.find((n) => n.id === "a");
        expect(a).toBeDefined();
        if (!a) return;
        const fromA = l.bands.filter((b) => b.shown === "a").sort((p, q) => p.y1 - q.y1);
        const rightIndex = (id: string) => l.right.findIndex((n) => n.id === id);
        expect(rightIndex(fromA[0].hidden)).toBeLessThan(rightIndex(fromA[1].hidden));
        for (const b of fromA) {
            expect(b.y1 - (b.count * FLOW_UNIT) / 2).toBeGreaterThanOrEqual(a.barTop - 1e-9);
            expect(b.y1 + (b.count * FLOW_UNIT) / 2).toBeLessThanOrEqual(a.barTop + a.barHeight + 1e-9);
        }
    });
});
