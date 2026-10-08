import { describe, expect, it } from "vitest";
import type { IUserScore } from "#/lib/api/user";
import { fastestSection, formatPct, GRADE_LADDER, nextGradeStep, SUBSCORES, toPct, weightShare } from "./helpers";

function score(partial: Partial<IUserScore>): IUserScore {
    return {
        user_id: "u",
        total_score: 0,
        operator_score: 0,
        stage_score: 0,
        roguelike_score: 0,
        sandbox_score: 0,
        medal_score: 0,
        base_score: 0,
        base_utilization: null,
        base_infrastructure: null,
        skin_score: 0,
        grade: null,
        calculated_at: "2026-10-01T00:00:00Z",
        ...partial,
    };
}

describe("section weights", () => {
    it("mirror the backend SECTION_WEIGHT_* constants", () => {
        expect(Object.fromEntries(SUBSCORES.map((s) => [s.key, s.weight]))).toEqual({
            operator_score: 0.85,
            base_score: 0.35,
            stage_score: 0.6,
            roguelike_score: 0.3,
            sandbox_score: 0.2,
            medal_score: 0.2,
        });
    });

    it("share out to 100%", () => {
        expect(weightShare(0.85)).toBeCloseTo(34, 10);
        expect(weightShare(0.6)).toBeCloseTo(24, 10);
        expect(weightShare(0.2)).toBeCloseTo(8, 10);
        expect(SUBSCORES.reduce((sum, s) => sum + weightShare(s.weight), 0)).toBeCloseTo(100, 10);
    });
});

describe("toPct and formatPct", () => {
    it("scale a 0-1 score to a clamped percentage", () => {
        expect(toPct(0.4567)).toBeCloseTo(45.67, 10);
        expect(toPct(1.2)).toBe(100);
        expect(toPct(-0.1)).toBe(0);
    });

    it("treat missing and NaN as zero", () => {
        expect(toPct(null)).toBe(0);
        expect(toPct(undefined)).toBe(0);
        expect(toPct(Number.NaN)).toBe(0);
    });

    it("format with one decimal by default", () => {
        expect(formatPct(0.4567)).toBe("45.7%");
        expect(formatPct(0.4567, 0)).toBe("46%");
        expect(formatPct(null)).toBe("0.0%");
        expect(formatPct(2)).toBe("100.0%");
    });
});

describe("grade ladder", () => {
    it("mirrors the backend score_to_grade thresholds", () => {
        expect(GRADE_LADDER.map((g) => [g.grade, g.min])).toEqual([
            ["F", 0],
            ["D", 0.15],
            ["C", 0.3],
            ["B", 0.45],
            ["A", 0.6],
            ["S", 0.75],
            ["S+", 0.9],
        ]);
    });

    it("names the next rung and the points to it", () => {
        const step = nextGradeStep(0.5);
        expect(step?.grade).toBe("A");
        expect(step?.min).toBe(0.6);
        expect(step?.pointsAway).toBeCloseTo(10, 10);
    });

    it("steps past a rung the score already sits on", () => {
        expect(nextGradeStep(0)?.grade).toBe("D");
        expect(nextGradeStep(0.75)?.grade).toBe("S+");
        // Within float noise of a threshold counts as reaching it.
        expect(nextGradeStep(0.6 - 1e-12)?.grade).toBe("S");
    });

    it("has nothing above S+", () => {
        expect(nextGradeStep(0.9)).toBeNull();
        expect(nextGradeStep(1)).toBeNull();
    });
});

describe("fastestSection", () => {
    it("picks the heaviest section that still has headroom", () => {
        expect(fastestSection(score({}))?.key).toBe("operator_score");
        expect(fastestSection(score({ operator_score: 1 }))?.key).toBe("stage_score");
        expect(fastestSection(score({ operator_score: 1, stage_score: 0.996 }))?.key).toBe("base_score");
    });

    it("keeps the first listed section on a weight tie", () => {
        const s = score({ operator_score: 1, stage_score: 1, base_score: 1, roguelike_score: 1 });
        expect(fastestSection(s)?.key).toBe("sandbox_score");
    });

    it("returns null once everything is effectively maxed", () => {
        const s = score({ operator_score: 1, stage_score: 1, base_score: 1, roguelike_score: 1, sandbox_score: 0.995, medal_score: 0.999 });
        expect(fastestSection(s)).toBeNull();
    });
});
