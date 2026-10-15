import { describe, expect, it } from "vitest";
import { pityAt } from "./odds";
import { buildPlan } from "./plan";
import { loadPulls, sanitise } from "./store";

describe("the stored pity counters", () => {
    it("seeds BOTH counters from a plan saved with the old single pity", () => {
        const s = sanitise({ pity: 37 });
        expect(s.standardPity).toBe(37);
        expect(s.kernelPity).toBe(37);
    });

    it("reads an old saved blob out of localStorage with both counters seeded", () => {
        localStorage.setItem("release-planner:pulls:v1", JSON.stringify({ pity: 52, allocations: { S1: 40 } }));
        const s = loadPulls();
        localStorage.removeItem("release-planner:pulls:v1");
        expect([s.standardPity, s.kernelPity]).toEqual([52, 52]);
        expect(s.allocations).toEqual({ S1: 40 });
    });

    it("keeps the two counters once they are stored separately", () => {
        const s = sanitise({ pity: 37, standardPity: 12, kernelPity: 0 });
        expect(s.standardPity).toBe(12);
        expect(s.kernelPity).toBe(0);
    });

    it("defaults a missing counter to zero and clamps above hard pity", () => {
        const s = sanitise({ standardPity: 400 });
        expect(s.standardPity).toBe(98);
        expect(s.kernelPity).toBe(0);
    });

    it("gives a migrated plan the seeds the old single counter gave it", () => {
        // The old build seeded `standard: pityAt(pity), kernel: pityAt(pity)`.
        const s = sanitise({ pity: 37 });
        const enStart = Math.floor(Date.UTC(2026, 8, 20) / 1000);
        const banner = (id: string, ruleType: string, at: number) =>
            ({ cnPoolId: id, ruleType, nameCn: id, cnOpen: at, cnEnd: at + 14 * 86_400, returning: false, featured6: ["a", "b"], enFeatured6: ["a", "b"], overrideFeatured: [], resolution: { status: "confirmed", enId: id, enStart: at, enEnd: at + 14 * 86_400 } }) as never;
        const rows = buildPlan({
            banners: [banner("S1", "DOUBLE", enStart), banner("K1", "CLASSIC", enStart + 86_400)],
            days: [],
            model: null,
            today: new Date(Date.UTC(2026, 8, 16)),
            standardPity: s.standardPity,
            kernelPity: s.kernelPity,
            allocations: {},
            targets: {},
            spendOriginite: false,
            countFreePulls: false,
        }).rows;
        expect(Array.from(rows[0].pityDist ?? [])).toEqual(Array.from(pityAt(37)));
        expect(Array.from(rows[1].pityDist ?? [])).toEqual(Array.from(pityAt(37)));
    });
});
