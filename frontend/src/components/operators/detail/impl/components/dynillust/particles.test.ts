import * as PIXI from "pixi.js";
import { describe, expect, it } from "vitest";
import type { IDriftable, IWorldSpaceState } from "./particles";

// The module builds `PIXI.Texture.WHITE` at load, which paints a 2D canvas; jsdom has
// no canvas backend, so hand it a context that accepts the two calls and draws nothing.
HTMLCanvasElement.prototype.getContext = (() => ({ fillStyle: "", fillRect: () => {} })) as unknown as HTMLCanvasElement["getContext"];
const { holdWorldSpace } = await import("./particles");

/** A container stand-in: `holdWorldSpace` reads only the local transform. */
function containerAt(m: PIXI.Matrix): PIXI.Container {
    return { transform: { updateLocalTransform: () => {}, localTransform: m } } as unknown as PIXI.Container;
}

/** `toEqual` tells -0 from 0, and the Y flip produces -0; adding 0 folds it. */
const plain = (xs: number[] | undefined) => xs?.map((v) => v + 0);

describe("holdWorldSpace", () => {
    it("moves every trail point of a world-space particle and leaves its ages alone", () => {
        const st: IWorldSpaceState = { prev: new PIXI.Matrix() };
        // History is `[x, y, age]` triples, newest first, Y-up.
        const p: IDriftable = { x: 0, y: 0, vx: 0, vy: 0, hist: [0, 0, 0, 5, 5, 0.1, 9, -3, 0.2] };
        // The emitter moved 10 px right since the last frame; the particle must hold
        // its screen position, so in the container's frame it moves 10 px left.
        holdWorldSpace(containerAt(new PIXI.Matrix().translate(10, 0)), st, [p]);
        expect(p.x).toBeCloseTo(-10);
        expect(p.y).toBeCloseTo(0);
        expect(plain(p.hist)).toEqual([-10, 0, 0, -5, 5, 0.1, -1, -3, 0.2]);
    });

    it("flips Y between the particle's Y-up frame and Pixi's Y-down one", () => {
        const st: IWorldSpaceState = { prev: new PIXI.Matrix() };
        const p: IDriftable = { x: 1, y: 2, vx: 0, vy: 1, hist: [1, 2, 0.5] };
        // Pixi +4 in Y is DOWN, so a Y-up particle must rise by 4 to hold still.
        holdWorldSpace(containerAt(new PIXI.Matrix().translate(0, 4)), st, [p]);
        expect(p.y).toBeCloseTo(6);
        expect(p.vy).toBeCloseTo(1);
        expect(plain(p.hist)).toEqual([1, 6, 0.5]);
    });
});
