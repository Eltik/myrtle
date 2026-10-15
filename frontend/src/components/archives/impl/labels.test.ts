import { describe, expect, it } from "vitest";
import { declutter, type IView, placeLabel, project } from "./labels";

const view: IView = { box: { x: -100, y: -50, w: 200, h: 100 }, width: 400, height: 200 };

describe("map labels", () => {
    it("projects world points onto the stage", () => {
        expect(project({ x: 0, y: 0 }, view)).toEqual({ x: 200, y: 100 });
        expect(project({ x: -100, y: -50 }, view)).toEqual({ x: 0, y: 0 });
    });

    it("keeps a label inside the stage and drops one whose anchor is out of view", () => {
        expect(placeLabel("a", { x: 390, y: 5 }, 100, 30, view, 4)).toEqual({ id: "a", x: 296, y: 4, w: 100, h: 30 });
        expect(placeLabel("b", { x: -1, y: 50 }, 100, 30, view, 4)).toBeNull();
    });

    it("hides the less important of two overlapping labels and keeps apart ones", () => {
        const boxes = [
            { id: "big", x: 0, y: 0, w: 100, h: 30 },
            { id: "small", x: 50, y: 10, w: 100, h: 30 },
            { id: "far", x: 300, y: 150, w: 80, h: 30 },
        ];
        const people: Record<string, number> = { big: 50, small: 3, far: 1 };
        expect([...declutter(boxes, (id) => people[id], 4)].sort()).toEqual(["big", "far"]);
        // The hovered cluster's label shows even when a bigger one covers it.
        expect([...declutter(boxes, (id) => people[id], 4, "small")].sort()).toEqual(["far", "small"]);
    });
});
