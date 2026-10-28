import { describe, expect, it } from "vitest";
import type { LookbackOperator } from "#/types/generated/LookbackOperator";
import type { LookbackSnapshot } from "#/types/generated/LookbackSnapshot";
import type { LookbackSummary } from "#/types/generated/LookbackSummary";
import { diffBuilds, metricSeries, ownershipSeries, summaryDelta } from "./helpers";

function op(operator_id: string, elite: number, masteries: number[] = [], modules: [string, number][] = []): LookbackOperator {
    return { operator_id, elite, level: 1, potential: 0, skill_level: 7, masteries, modules: modules.map(([module_id, stage]) => ({ module_id, stage })) };
}

function summary(over: Partial<LookbackSummary> = {}): LookbackSummary {
    return { operators: 0, elite2: 0, max_level: 0, skill_level7: 0, masteries: 0, mastery3: 0, modules: 0, module_stage3: 0, max_potential: 0, skins: 0, total_score: null, grade: null, ...over };
}

describe("diffBuilds", () => {
    it("lists gains only, in the later build's order", () => {
        const before = [op("a", 1), op("b", 2, [3, 1], [["mod_b", 1]]), op("c", 2, [3])];
        const after = [
            op("d", 2, [1]),
            op("a", 2),
            op(
                "b",
                2,
                [3, 3, 2],
                [
                    ["mod_b", 3],
                    ["mod_b2", 1],
                ],
            ),
            op("c", 2, []),
        ];
        expect(diffBuilds(before, after)).toEqual({
            gained: ["d"],
            elite2: ["d", "a"],
            masteries: [
                { operatorId: "d", skill: 0, from: 0, to: 1 },
                { operatorId: "b", skill: 1, from: 1, to: 3 },
                { operatorId: "b", skill: 2, from: 0, to: 2 },
            ],
            modules: [
                { operatorId: "b", moduleId: "mod_b", from: 1, to: 3 },
                { operatorId: "b", moduleId: "mod_b2", from: 0, to: 1 },
            ],
        });
    });

    it("finds nothing between a build and itself", () => {
        const build = [op("a", 2, [3, 3], [["m", 2]])];
        expect(diffBuilds(build, build)).toEqual({ gained: [], elite2: [], masteries: [], modules: [] });
    });
});

describe("series", () => {
    const snaps: LookbackSnapshot[] = [
        { id: 100, period_start: "2026-09-16T00:00:00Z", taken_at: "2026-09-16T08:00:00Z", activity_id: "act51side", activity_name: "People, A People", summary: summary({ operators: 200, elite2: 80 }) },
        { id: 200, period_start: "2026-10-14T00:00:00Z", taken_at: "2026-10-15T08:00:00Z", activity_id: null, activity_name: null, summary: summary({ operators: 205, elite2: 84 }) },
    ];

    it("reads one metric per snapshot and appends a later live point", () => {
        const live = { ts: Date.parse("2026-10-28T00:00:00Z"), summary: summary({ operators: 207, elite2: 85 }) };
        expect(metricSeries(snaps, "elite2", live).map((p) => p.value)).toEqual([80, 84, 85]);
        expect(metricSeries(snaps, "operators").map((p) => p.label)).toEqual(["People, A People", null]);
        // A live point no later than the last snapshot is not drawn.
        expect(metricSeries(snaps, "operators", { ts: Date.parse("2026-10-01T00:00:00Z"), summary: summary() })).toHaveLength(2);
    });

    it("puts the ownership line on a millisecond axis", () => {
        expect(ownershipSeries([{ ts: 1_789_516_800, count: 3 }])).toEqual([{ ts: 1_789_516_800_000, value: 3, label: null }]);
    });

    it("subtracts every metric", () => {
        expect(summaryDelta(snaps[0].summary, snaps[1].summary)).toEqual({ operators: 5, elite2: 4, max_level: 0, mastery3: 0, modules: 0, skins: 0 });
    });
});
