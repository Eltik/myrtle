import { describe, expect, it } from "vitest";
import { crewSource, sameCrew } from "./shift-crews";

describe("crewSource", () => {
    it("reads the draft when no shift tab is active", () => {
        expect(crewSource(null, true, ["a"], ["b"])).toBe("draft");
        expect(crewSource(null, false, ["a"], undefined)).toBe("draft");
    });

    it("reads the rotation cell on a shift tab once a plan exists", () => {
        expect(crewSource(1, true, ["a", "b"], ["c"])).toBe("plan");
    });

    it("reads the draft for a preset crew that is the draft crew", () => {
        expect(crewSource(1, false, ["b", "a"], ["a", "b"])).toBe("draft");
    });

    it("scores nothing for a preset crew the draft did not score", () => {
        // The rotation's cell must never stand in: its Control Center is the plan's.
        expect(crewSource(2, false, ["x", "y"], ["a", "b"])).toBe("preset");
        expect(crewSource(2, false, ["x"], undefined)).toBe("preset");
    });
});

describe("sameCrew", () => {
    it("ignores order and rejects size or member differences", () => {
        expect(sameCrew(["a", "b"], ["b", "a"])).toBe(true);
        expect(sameCrew(["a"], ["a", "a"])).toBe(false);
        expect(sameCrew(["a", "b"], ["a", "c"])).toBe(false);
    });
});
