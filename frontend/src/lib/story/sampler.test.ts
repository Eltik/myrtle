import { afterEach, describe, expect, it, vi } from "vitest";
import { createSampler } from "./sampler";

const KEY = "test.sampler.v1";

function sampler() {
    return createSampler({ storageKey: KEY, width: 1, height: 1, derive: () => "derived", fallback: "fallback" });
}

describe("createSampler's stored record", () => {
    afterEach(() => {
        localStorage.clear();
        vi.restoreAllMocks();
    });

    it("parses an unchanged record once, however many urls ask", () => {
        localStorage.setItem(KEY, JSON.stringify({ a: "A", b: "B", c: "C" }));
        const parse = vi.spyOn(JSON, "parse");
        const s = sampler();
        expect([s.cached("a"), s.cached("b"), s.cached("c"), s.cached("missing")]).toEqual(["A", "B", "C", undefined]);
        expect(parse).toHaveBeenCalledTimes(1);
    });

    it("follows a record another tab rewrote", () => {
        localStorage.setItem(KEY, JSON.stringify({ a: "A" }));
        const s = sampler();
        expect(s.cached("b")).toBeUndefined();
        localStorage.setItem(KEY, JSON.stringify({ a: "A", b: "B" }));
        expect(s.cached("b")).toBe("B");
    });

    it("answers from memory before storage, and nothing when neither has it", () => {
        const s = sampler();
        expect(s.hot("a")).toBeUndefined();
        localStorage.setItem(KEY, JSON.stringify({ a: "A" }));
        expect(s.cached("a")).toBe("A");
        expect(s.hot("a")).toBe("A");
    });
});
