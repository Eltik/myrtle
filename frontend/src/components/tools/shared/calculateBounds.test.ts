/**
 * A share link is decoded on the page and its cards are then sent to the backend, which
 * refuses (400) any field outside `CalculateRequest::validate`. These cases pin the
 * decode-side clamps to the backend's own numbers, read from its source, so the two
 * cannot drift apart silently.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { CALCULATE_BOUNDS, clampField } from "./calculateBounds";
import { decodeInstance, decodeSweep } from "./shareCodec";

// Joined by hand, as in `conditionals.test.ts`: Vite rewrites `new URL(literal, import.meta.url)`.
const BACKEND_DPS = join(dirname(fileURLToPath(import.meta.url)), "../../../../../backend/src/app/services/dps.rs");

/** Every `int_range` / `float_range` call inside `CalculateRequest::validate`, by field name. */
function backendBounds(): Map<string, [number, number]> {
    const src = readFileSync(BACKEND_DPS, "utf8");
    const start = src.indexOf("impl CalculateRequest {");
    const end = src.indexOf("fn validate_zones", start);
    expect(start).toBeGreaterThan(-1);
    expect(end).toBeGreaterThan(start);
    const out = new Map<string, [number, number]>();
    const call = /\b(?:int|float)_range\(\s*"([\w.]+)",\s*[^,]+,\s*(-?[\d_.]+),\s*(-?[\d_.]+)\s*\)/g;
    for (const m of src.slice(start, end).matchAll(call)) out.set(m[1], [Number(m[2].replaceAll("_", "")), Number(m[3].replaceAll("_", ""))]);
    return out;
}

describe("calculate bounds", () => {
    it("match what the backend validates, field for field", () => {
        const backend = backendBounds();
        for (const [field, bounds] of Object.entries(CALCULATE_BOUNDS)) expect([field, backend.get(field)]).toEqual([field, [...bounds]]);
    });

    it("clamp to the backend's bounds, or to a narrower control's", () => {
        expect(clampField("targets", 500)).toBe(50);
        expect(clampField("targets", -3)).toBe(1);
        expect(clampField("res", 900, 0, 100)).toBe(100);
        expect(clampField("defense", 1e9)).toBe(100_000);
        // A control wider than the backend never widens it.
        expect(clampField("moduleIndex", 9, 0, 9)).toBe(5);
    });

    it("cap a hand-edited card's module index at the backend's", () => {
        const inst = decodeInstance(["char_a", 0, 1, 2, 90, 6, 100, 1, 10, 9, 3, [null, null, null, null], 0, 0, 1]);
        expect(inst?.config.moduleIndex).toBe(5);
    });

    it("cut a sweep to its axis control's ceiling, and drop one the cut leaves empty", () => {
        expect(decodeSweep([0, 1e7, 41], 5000)).toEqual({ min: 0, max: 5000, steps: 41 });
        expect(decodeSweep([100, 3000, 41], 5000)).toEqual({ min: 100, max: 3000, steps: 41 });
        expect(decodeSweep([6000, 1e7, 41], 5000)).toBeNull();
    });
});
