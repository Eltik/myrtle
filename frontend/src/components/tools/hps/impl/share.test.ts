import { describe, expect, it } from "vitest";
import { CHART_PALETTE } from "#/components/tools/shared/constants";
import { packShare } from "#/components/tools/shared/shareCodec";
import type { IOperatorListEntry } from "#/components/tools/shared/types";
import { decodeHpsShare, encodeHpsShare, resolveHpsShare } from "./share";
import { INITIAL_STATE } from "./state";
import type { IHpsState } from "./types";

const OP: IOperatorListEntry = { id: "char_h", name: "Healer", availableSkills: [1, 2], availableModules: [0], availableModuleIds: [""], defaultSkill: 2, defaultModule: 0, conditionals: [] };

const STATE: IHpsState = {
    instances: [
        {
            uid: "u1",
            op: OP,
            color: CHART_PALETTE[5],
            visible: true,
            collapsed: false,
            config: { potential: 6, trust: 100, skillIndex: 1, skillRank: 7, moduleIndex: 0, moduleLevel: 3, buffs: {}, conditionals: { talentDamage: false }, allCond: true },
        },
    ],
    buffs: { buffs: { atk: 0.4, aspd: 20 }, targets: 3, spBoost: 0.5 },
    xAxis: "aspd",
    yMetric: "avg_hps",
    sweep: { targets: { min: 1, max: 6, steps: 6 }, atk: { min: 0, max: 150, steps: 11 }, aspd: { min: 10, max: 100, steps: 21 } },
};

describe("HPS share link", () => {
    it("round-trips every per-card input, the team buffs and the chart controls", () => {
        const decoded = decodeHpsShare(encodeHpsShare(STATE));
        expect(decoded).not.toBeNull();
        if (!decoded) return;
        const { state, missing } = resolveHpsShare(decoded, [OP]);
        expect(missing).toBe(0);
        const strip = (s: IHpsState) => ({ ...s, instances: s.instances.map(({ uid: _u, collapsed: _c, ...rest }) => rest) });
        expect(strip(state)).toEqual(strip(STATE));
    });

    it("round-trips the default page state", () => {
        const decoded = decodeHpsShare(encodeHpsShare(INITIAL_STATE));
        expect(decoded && resolveHpsShare(decoded, [OP]).state).toEqual(INITIAL_STATE);
    });

    it("rejects garbage and the DPS calculator's links", () => {
        expect(decodeHpsShare("%%%")).toBeNull();
        expect(decodeHpsShare(packShare("dps", { i: [], b: [null, null, null, null, 1, 0] }))).toBeNull();
        expect(decodeHpsShare(packShare("hps", { i: [], b: [null, null, null, null, "1", 0] }))).toBeNull();
    });
});
