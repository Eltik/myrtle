import { describe, expect, it } from "vitest";
import { type IQueue, identityOrder, insertEnd, insertNext, moveItem, nextIndex, playOrder, previousIndex, removeAt, shuffledOrder, validOrder } from "./queue";

function q(items: string[], index: number, order: number[] = identityOrder(items.length)): IQueue<string> {
    return { items, index, order };
}

/** The items in play order: what a listener hears. */
function heard(x: IQueue<string>): string[] {
    return x.order.map((i) => x.items[i] as string);
}

describe("queue arithmetic", () => {
    it("next and previous walk the play order and stop at the ends with repeat off", () => {
        const x = q(["a", "b", "c"], 1, [1, 2, 0]);
        expect(nextIndex(x, "off", true)).toBe(2);
        expect(previousIndex(x, "off")).toBeNull();
        expect(nextIndex({ ...x, index: 0 }, "off", true)).toBeNull();
        expect(previousIndex({ ...x, index: 0 }, "off")).toBe(2);
    });

    it("repeat-all wraps both ways and repeat-one holds only an automatic advance", () => {
        const x = q(["a", "b", "c"], 2);
        expect(nextIndex(x, "all", true)).toBe(0);
        expect(previousIndex({ ...x, index: 0 }, "all")).toBe(2);
        expect(nextIndex(x, "one", true)).toBe(2);
        expect(nextIndex({ ...x, index: 0 }, "one", false)).toBe(1);
    });

    it("a fresh play order is the list order, or the seeded shuffle with the first item in front", () => {
        expect(playOrder(4, 2, false)).toEqual([0, 1, 2, 3]);
        const seeded = () => {
            let n = 0;
            return () => [0.7, 0.1, 0.4][n++ % 3] as number;
        };
        expect(playOrder(4, 2, true, seeded())).toEqual(shuffledOrder(4, 2, seeded()));
        expect(playOrder(0, 0, true)).toEqual([]);
    });

    it("a shuffled order puts the current item first and names every index once", () => {
        let seed = 0.37;
        const rng = () => {
            seed = (seed * 9301 + 0.49297) % 1;
            return seed;
        };
        const order = shuffledOrder(8, 5, rng);
        expect(order[0]).toBe(5);
        expect(validOrder(order, 8)).not.toBeNull();
        expect(validOrder([0, 0, 1], 3)).toBeNull();
    });

    it("enqueueNext lands right after the current item in both orders", () => {
        const x = insertNext(q(["a", "b", "c"], 1, [1, 2, 0]), ["x", "y"]);
        expect(x.items).toEqual(["a", "b", "x", "y", "c"]);
        expect(x.index).toBe(1);
        expect(heard(x)).toEqual(["b", "x", "y", "c", "a"]);
    });

    it("enqueueEnd appends in order without shuffle and after the current item with it", () => {
        const plain = insertEnd(q(["a", "b"], 0), ["c"], false);
        expect(heard(plain)).toEqual(["a", "b", "c"]);
        const shuffled = insertEnd(q(["a", "b", "c"], 2, [1, 2, 0]), ["x"], true, () => 0);
        // rng 0 puts it at the first slot after the current item.
        expect(heard(shuffled)).toEqual(["b", "c", "x", "a"]);
    });

    it("moving an item remaps the current index, and the play order only under shuffle", () => {
        const plain = moveItem(q(["a", "b", "c", "d"], 1), 0, 3, false);
        expect(plain.items).toEqual(["b", "c", "d", "a"]);
        expect(plain.index).toBe(0);
        expect(plain.order).toEqual([0, 1, 2, 3]);
        const shuffled = moveItem(q(["a", "b", "c", "d"], 2, [2, 0, 3, 1]), 3, 0, true);
        expect(shuffled.items).toEqual(["d", "a", "b", "c"]);
        expect(heard(shuffled)).toEqual(["c", "a", "d", "b"]);
        expect(shuffled.items[shuffled.index]).toBe("c");
    });

    it("removing the current item makes the next in play order current", () => {
        const r = removeAt(q(["a", "b", "c", "d"], 1, [1, 3, 0, 2]), 1);
        expect(r.removedCurrent).toBe(true);
        expect(r.hasNext).toBe(true);
        expect(r.items[r.index]).toBe("d");
        expect(heard(r)).toEqual(["d", "a", "c"]);
    });

    it("removing the current item at the end of play order selects the one before, and the only item empties", () => {
        const r = removeAt(q(["a", "b", "c"], 2), 2);
        expect(r.hasNext).toBe(false);
        expect(r.items[r.index]).toBe("b");
        const empty = removeAt(q(["a"], 0), 0);
        expect(empty).toMatchObject({ items: [], index: -1, order: [], removedCurrent: true });
    });

    it("removing another item keeps the current one current", () => {
        const r = removeAt(q(["a", "b", "c"], 2), 0);
        expect(r.removedCurrent).toBe(false);
        expect(r.items[r.index]).toBe("c");
        expect(r.order).toEqual([0, 1]);
    });
});
