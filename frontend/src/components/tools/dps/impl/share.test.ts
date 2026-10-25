import { describe, expect, it } from "vitest";
import { outOfBounds } from "#/components/tools/shared/calculateBounds";
import { CHART_PALETTE } from "#/components/tools/shared/constants";
import { packShare } from "#/components/tools/shared/shareCodec";
import type { IOperatorListEntry } from "#/components/tools/shared/types";
import { decodeDpsShare, encodeDpsShare, resolveDpsShare } from "./share";
import { INITIAL_STATE, instanceToRequest } from "./state";
import type { IDpsState } from "./types";
import { EMPTY_ZONES } from "./zones";

function op(id: string): IOperatorListEntry {
    return { id, name: id, availableSkills: [1, 2, 3], availableModules: [0, 1], availableModuleIds: ["", "uniequip_x"], defaultSkill: 3, defaultModule: 1, conditionals: [] };
}

const OPS = [op("char_a"), op("char_b")];

const STATE: IDpsState = {
    instances: [
        {
            uid: "u1",
            op: OPS[0],
            color: CHART_PALETTE[2],
            visible: true,
            collapsed: false,
            config: { promotion: 2, level: 80, potential: 3, trust: 50, skillIndex: 2, skillRank: 9, moduleIndex: 1, moduleLevel: 2, buffs: {}, zones: { ...EMPTY_ZONES, atkPercent: 0.4, aspd: 30, atkMultiplier: 1.15 }, conditionals: { traitDamage: false, skillDamage: true }, allCond: true },
        },
        {
            uid: "u2",
            op: OPS[1],
            color: CHART_PALETTE[0],
            visible: false,
            collapsed: true,
            config: { potential: 1, trust: 100, skillIndex: 3, skillRank: 10, moduleIndex: 0, moduleLevel: 3, buffs: {}, conditionals: {}, allCond: false },
        },
    ],
    enemy: { defense: 1250.5, res: 35, targets: 3, secondary: { enabled: true, def: 800, res: 50, count: 3 }, cycleModel: "v2" },
    team: { zones: { ...EMPTY_ZONES, spRecovery: 0.2, defPercent: 0.4, resFlat: 10, fragile: 0.3, defSteal: 70 }, supporters: ["warfarin_s2", "pramanix_t1"], investment: { promotion: 2, skillLevel: 9, potential: 6, moduleLevel: 3 } },
    xAxis: "res",
    yMetric: "total_damage",
    sweep: { defense: { min: 100, max: 3000, steps: 41 }, res: { min: 0, max: 60, steps: 11 } },
    markers: false,
    view: "rank",
    rank: { preset: "p6Max", sortBy: "averageDps", professions: ["SNIPER", "CASTER"], rarities: [6, 5], damageTypes: ["arts"] },
};

/** The fields a link is meant to carry: uid and collapse state are per-session. */
function shareable(state: IDpsState) {
    return { ...state, instances: state.instances.map(({ uid: _uid, collapsed: _collapsed, ...rest }) => rest) };
}

describe("DPS share link", () => {
    it("round-trips every per-card input, the enemy panel and the chart controls", () => {
        const text = encodeDpsShare(STATE);
        expect(text).toMatch(/^[A-Za-z0-9_-]+$/);
        const decoded = decodeDpsShare(text);
        expect(decoded).not.toBeNull();
        if (!decoded) return;
        expect(decoded.invalid).toBe(0);
        const { state, missing } = resolveDpsShare(decoded, OPS);
        expect(missing).toBe(0);
        expect(shareable(state)).toEqual(shareable(STATE));
    });

    it("keeps an unset conditional distinct from an explicit false", () => {
        const decoded = decodeDpsShare(encodeDpsShare(STATE));
        expect(decoded?.instances[0].config.conditionals).toEqual({ traitDamage: false, skillDamage: true });
        expect(decoded?.instances[1].config.conditionals).toEqual({});
    });

    it("round-trips the default page state", () => {
        const decoded = decodeDpsShare(encodeDpsShare(INITIAL_STATE));
        expect(decoded && resolveDpsShare(decoded, OPS).state).toEqual(INITIAL_STATE);
    });

    it("drops and counts a card whose operator is gone", () => {
        const decoded = decodeDpsShare(encodeDpsShare(STATE));
        if (!decoded) throw new Error("decode failed");
        const { state, missing } = resolveDpsShare(decoded, [OPS[1]]);
        expect(missing).toBe(1);
        expect(state.instances.map((i) => i.op.id)).toEqual(["char_b"]);
    });

    it("rejects garbage, other versions and the other calculator's links", () => {
        expect(decodeDpsShare("")).toBeNull();
        expect(decodeDpsShare("not base64!")).toBeNull();
        expect(decodeDpsShare("AAAA")).toBeNull();
        expect(decodeDpsShare(encodeDpsShare(STATE).slice(0, -6))).toBeNull();
        expect(decodeDpsShare(packShare("hps", { i: [], e: [0, 0, 1, 0, null, null, null, null] }))).toBeNull();
        expect(decodeDpsShare("x".repeat(9000))).toBeNull();
    });

    it("rejects a body with a malformed enemy block", () => {
        expect(decodeDpsShare(packShare("dps", { i: [], e: [0, 0] }))).toBeNull();
        expect(decodeDpsShare(packShare("dps", { i: [], e: ["0", 0, 1] }))).toBeNull();
        expect(decodeDpsShare(packShare("dps", { i: [], e: [0, 0, 1], z: [0, "x"] }))).toBeNull();
        expect(decodeDpsShare(packShare("dps", { i: [], e: [0, 0, 1, 0, null, null, null, null] }))).toBeNull();
        expect(decodeDpsShare(packShare("dps", []))).toBeNull();
    });

    it("drops a malformed card but keeps the rest", () => {
        const text = packShare("dps", {
            i: [
                ["char_a", 0, 1, null, null, 1, 100, 1, 10, 0, 3, [null, null, null, null], 0, 0, 1],
                ["char_b", "x"],
            ],
            e: [0, 0, 1],
        });
        const decoded = decodeDpsShare(text);
        expect(decoded?.instances.map((i) => i.opId)).toEqual(["char_a"]);
        expect(decoded?.invalid).toBe(1);
    });

    it("falls back to defaults for an out-of-range axis, metric or sweep", () => {
        const decoded = decodeDpsShare(packShare("dps", { i: [], e: [0, 0, 1], x: 7, y: "length", s: [[5, 1, 21]], v: 9, r: [7, 7, ["sniper"], [0, 7], ["fire"]] }));
        expect(decoded?.xAxis).toBe(INITIAL_STATE.xAxis);
        expect(decoded?.yMetric).toBe(INITIAL_STATE.yMetric);
        expect(decoded?.sweep).toEqual(INITIAL_STATE.sweep);
        expect(decoded?.view).toBe(INITIAL_STATE.view);
        expect(decoded?.rank).toEqual({ ...INITIAL_STATE.rank });
    });

    it("clamps a hand-edited link to what the backend accepts, so no card request is a 400", () => {
        const text = packShare("dps", {
            i: [
                // moduleIndex 9, and zone values far past `validate_zones` (`buildRequestZones` clamps those).
                ["char_a", 0, 1, 2, 90, 1, 100, 2, 10, 9, 3, [1e6, null, null, null], 0, 0, 1],
                ["char_b", 1, 1, 2, 90, 1, 100, 2, 10, 1, 3, [null, null, null, null], 0, 0, 1, [1e6, 1e9, -1e6]],
            ],
            e: [1e9, 5000, 500],
            q: [1, 1e9, 1e9, 99],
            z: [1e6, 1e9, 1e6, 1e6, 1e6],
            s: [
                [0, 1e7, 41],
                [0, 1e4, 11],
            ],
        });
        const decoded = decodeDpsShare(text);
        expect(decoded).not.toBeNull();
        if (!decoded) return;
        const { state } = resolveDpsShare(decoded, OPS);
        expect(state.enemy).toMatchObject({ defense: 100_000, res: 100, targets: 50, secondary: { def: 100_000, res: 100, count: 49 } });
        expect(state.sweep).toEqual({ defense: { min: 0, max: 5000, steps: 41 }, res: { min: 0, max: 100, steps: 11 } });
        for (const enemy of [state.enemy, { ...state.enemy, secondary: { ...state.enemy.secondary, enabled: false } }]) {
            for (const inst of state.instances) expect(outOfBounds(instanceToRequest(inst, enemy, state.team, []))).toEqual([]);
        }
    });
});
