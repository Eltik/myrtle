import { describe, expect, it } from "vitest";
import { BATCH_LIMIT, chunk, flattenChunks } from "./batch";

describe("chunk", () => {
    it("splits at the backend's 64-request limit, order kept", () => {
        const items = Array.from({ length: 328 }, (_, i) => i);
        const chunks = chunk(items);
        expect(BATCH_LIMIT).toBe(64);
        expect(chunks.map((c) => c.length)).toEqual([64, 64, 64, 64, 64, 8]);
        expect(chunks.flat()).toEqual(items);
    });

    it("gives one chunk for an 8-card chart and none for no cards", () => {
        expect(chunk(Array.from({ length: 8 }, (_, i) => i))).toHaveLength(1);
        expect(chunk(Array.from({ length: 64 }, (_, i) => i))).toHaveLength(1);
        expect(chunk(Array.from({ length: 65 }, (_, i) => i))).toHaveLength(2);
        expect(chunk([])).toEqual([]);
    });

    it("rejects a size the backend would never accept", () => {
        expect(() => chunk([1], 0)).toThrow();
        expect(() => chunk([1], 1.5)).toThrow();
    });
});

describe("flattenChunks", () => {
    it("maps per-chunk answers back to per-request slots in order", () => {
        const slots = flattenChunks(
            [2, 1],
            [
                { data: { results: [{ result: 1 }, { error: "unknown operator" }] }, isPending: false, error: null },
                { data: { results: [{ result: 3 }] }, isPending: false, error: null },
            ],
            (item: { result?: number; error?: string }) => item.result,
        );
        expect(slots.map((s) => s.data)).toEqual([1, undefined, 3]);
        expect(slots[1].error?.message).toBe("unknown operator");
        expect(slots.every((s) => !s.isPending)).toBe(true);
    });

    it("fails every request of a chunk that failed whole, and keeps a pending chunk pending", () => {
        const boom = new Error("400");
        const slots = flattenChunks(
            [1, 2],
            [
                { data: undefined, isPending: false, error: boom },
                { data: undefined, isPending: true, error: null },
            ],
            (item: { result?: number; error?: string }) => item.result,
        );
        expect(slots.map((s) => s.error)).toEqual([boom, null, null]);
        expect(slots.map((s) => s.isPending)).toEqual([false, true, true]);
    });
});
