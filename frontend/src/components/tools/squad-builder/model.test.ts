import { describe, expect, it } from "vitest";
import type { IOperatorIndexEntry } from "#/types/operators";
import { clampBuild, decodeDpsImport, decodeShare, emptyDraft, encodeDpsImport, encodeShare, fullBuild, levelCap, maxMastery, maxSkillLevel, memberFromRoster, resolveSeat, rosterModuleRanks } from "./model";

const u = (phase: number, level = 1) => ({ phase, level });
/** Archetto's rules as `character_table` has them: ranks 2-4 at E0, 5-7 at E1, three masteries per skill at E2. */
const sixStarRules = {
    rankUnlocks: [u(0), u(0), u(0), u(1), u(1), u(1)],
    skills: [0, 1, 2].map((i) => ({ unlock: u(i), masteryUnlocks: [u(2), u(2), u(2)] })),
};
const sixStar = {
    skillCount: 3,
    phaseMaxLevels: [50, 80, 90],
    potentialRankCount: 5,
    skillRules: sixStarRules,
    modules: [
        { uniEquipId: "uniequip_001_x", typeName1: "ORIGINAL", typeName2: null, type: "INITIAL", rankCount: 1 },
        { uniEquipId: "uniequip_002_x", typeName1: "MAR", typeName2: "X", type: "ADVANCED", rankCount: 3 },
    ],
} as unknown as IOperatorIndexEntry;
/** Kroos: one skill, ranks to 7, an empty `LevelUpCostCond` so no mastery. */
const threeStar = { skillCount: 1, phaseMaxLevels: [40, 55], potentialRankCount: 5, modules: [], skillRules: { rankUnlocks: sixStarRules.rankUnlocks, skills: [{ unlock: u(0), masteryUnlocks: [] }] } } as unknown as IOperatorIndexEntry;
/** A Reserve 4-star: S1 has no mastery, S2 and S3 have three. */
const reserve = {
    skillCount: 3,
    phaseMaxLevels: [45, 60, 70],
    potentialRankCount: 5,
    modules: [],
    skillRules: {
        rankUnlocks: sixStarRules.rankUnlocks,
        skills: [
            { unlock: u(0), masteryUnlocks: [] },
            { unlock: u(1), masteryUnlocks: [u(2), u(2), u(2)] },
            { unlock: u(1), masteryUnlocks: [u(2), u(2), u(2)] },
        ],
    },
} as unknown as IOperatorIndexEntry;
/** 12F: no skills, no rank-ups, no potentials. */
const twoStar = { skillCount: 0, phaseMaxLevels: [30], potentialRankCount: 0, modules: [], skillRules: { rankUnlocks: [], skills: [] } } as unknown as IOperatorIndexEntry;

describe("caps", () => {
    it("reads the promotion cap from the index entry", () => {
        expect(levelCap(sixStar, 2)).toBe(90);
        expect(levelCap(threeStar, 2)).toBe(55);
        expect(levelCap(threeStar, 1)).toBe(55);
    });
    it("builds a full build from the operator's own rules", () => {
        expect(fullBuild(sixStar, { skill_index: 2, module_id: "uniequip_002_x" })).toEqual({ phase: 2, level: 90, potential: 5, skill_level: 7, mastery: 3, module_level: 3 });
        expect(fullBuild(threeStar, { skill_index: 0, module_id: null })).toEqual({ phase: 1, level: 55, potential: 5, skill_level: 7, mastery: 0, module_level: 0 });
        expect(fullBuild(reserve, { skill_index: 0, module_id: null }).mastery).toBe(0);
        expect(fullBuild(twoStar, { skill_index: 0, module_id: null })).toEqual({ phase: 0, level: 30, potential: 0, skill_level: 1, mastery: 0, module_level: 0 });
    });
    it("clamps ranks and masteries to what the build has unlocked", () => {
        const clamped = clampBuild(sixStar, { phase: 1, level: 95, potential: 9, skill_level: 7, mastery: 3, module_level: 3 }, { skill_index: 0, module_id: "uniequip_002_x" });
        expect(clamped).toEqual({ phase: 1, level: 80, potential: 5, skill_level: 7, mastery: 0, module_level: 3 });
        expect(clampBuild(sixStar, { phase: 0, level: 1, potential: 0, skill_level: 7, mastery: 0, module_level: 0 }, { skill_index: 0, module_id: null }).skill_level).toBe(4);
        expect(maxMastery(reserve, 0, 2, 70, 7)).toBe(0);
        expect(maxMastery(reserve, 1, 2, 70, 7)).toBe(3);
    });
    it("reads a rank unlock's level, not just its promotion", () => {
        const levelGated = { ...sixStar, skillRules: { ...sixStarRules, rankUnlocks: [u(0, 2), ...sixStarRules.rankUnlocks.slice(1)] } } as unknown as IOperatorIndexEntry;
        expect(maxSkillLevel(levelGated, 0, 1)).toBe(1);
        expect(maxSkillLevel(levelGated, 0, 2)).toBe(4);
    });
});

describe("roster resolution", () => {
    const roster = {
        elite: 2,
        level: 60,
        potential: 2,
        skill_level: 7,
        default_skill: 2,
        current_equip: "uniequip_002_x",
        masteries: [
            { index: 0, mastery: 0 },
            { index: 2, mastery: 3 },
        ],
        modules: [
            { id: "uniequip_001_x", level: 1, locked: false },
            { id: "uniequip_002_x", level: 2, locked: false },
        ],
    };
    it("follows the roster when the member has no build", () => {
        const member = memberFromRoster("char_x", roster, sixStar);
        expect(member).toEqual({ char_id: "char_x", skill_index: 2, module_id: "uniequip_002_x", build: null });
        const seat = resolveSeat(member, sixStar, roster, false);
        expect(seat.source).toBe("roster");
        expect(seat.build).toEqual({ phase: 2, level: 60, potential: 2, skill_level: 7, mastery: 3, module_level: 2 });
    });
    it("ignores the INITIAL module and locked modules", () => {
        expect([...rosterModuleRanks(sixStar, roster).keys()]).toEqual(["uniequip_002_x"]);
        expect(rosterModuleRanks(sixStar, { ...roster, modules: [{ id: "uniequip_002_x", level: 3, locked: 1 }] }).size).toBe(0);
    });
    it("falls back to a full build for an unowned operator and for a support", () => {
        const member = memberFromRoster("char_x", undefined, sixStar);
        expect(resolveSeat(member, sixStar, undefined, false).source).toBe("full");
        expect(resolveSeat(member, sixStar, roster, true).source).toBe("support");
        expect(resolveSeat(member, sixStar, roster, true).build.level).toBe(90);
    });
    it("a hand-set build wins over the roster", () => {
        const member = { ...memberFromRoster("char_x", roster, sixStar), build: { phase: 1, level: 70, potential: 0, skill_level: 5, mastery: 0, module_level: 0 } };
        const seat = resolveSeat(member, sixStar, roster, false);
        expect(seat.source).toBe("hand");
        expect(seat.build.level).toBe(70);
    });
});

describe("share link", () => {
    it("round-trips without builds", () => {
        const draft = emptyDraft("Squad 1");
        draft.members[0] = { char_id: "char_332_archet", skill_index: 2, module_id: "uniequip_003_archet", build: null };
        draft.members[5] = { char_id: "char_222_bpipe", skill_index: 2, module_id: null, build: { phase: 2, level: 60, potential: 0, skill_level: 7, mastery: 3, module_level: 0 } };
        draft.support = { char_id: "char_479_sleach", skill_index: 2, module_id: null, build: null };
        draft.stageId = "main_04-04";
        const search = encodeShare(draft, false);
        expect(search.s).toBe("char_332_archet.2.003_archet|||||char_222_bpipe.2.-||||||");
        expect(search.sup).toBe("char_479_sleach.2.-");
        const back = decodeShare(search);
        expect(back).not.toBeNull();
        expect(back?.name).toBe("Squad 1");
        expect(back?.stageId).toBe("main_04-04");
        expect(back?.members[0]).toEqual(draft.members[0]);
        expect(back?.members[5]).toEqual({ ...draft.members[5], build: null });
        expect(back?.support).toEqual(draft.support);
    });
    it("round-trips builds and pins roster-following seats to what the sender saw", () => {
        const draft = emptyDraft("");
        const member = { char_id: "char_002_amiya", skill_index: 1, module_id: "uniequip_002_amiya", build: null };
        draft.members[3] = member;
        const resolved = { members: draft.members.map((m) => (m ? { member: m, build: { phase: 2, level: 90, potential: 5, skill_level: 7, mastery: 3, module_level: 3 }, source: "roster" as const } : null)), support: null };
        const search = encodeShare(draft, true, resolved);
        expect(search.s?.split("|")[3]).toBe("char_002_amiya.1.002_amiya.2.90.5.7.3.3");
        expect(search.n).toBeUndefined();
        const back = decodeShare(search);
        expect(back?.members[3]).toEqual({ ...member, build: { phase: 2, level: 90, potential: 5, skill_level: 7, mastery: 3, module_level: 3 } });
    });
    it("drops malformed seats and refuses a bad stage id instead of throwing", () => {
        const back = decodeShare({ s: "nonsense|char_x.-1.-|char_1.1.-.a.b", st: "../x" });
        expect(back?.members.every((m) => m === null)).toBe(true);
        expect(back?.stageId).toBeNull();
        expect(decodeShare({})).toBeNull();
    });
});

describe("skill index", () => {
    it("decodes any index and clamps it to the operator's skills on resolve, as the backend does", () => {
        const back = decodeShare({ s: "char_x.9.-" });
        expect(back?.members[0]?.skill_index).toBe(9);
        const member = back?.members[0];
        if (!member) throw new Error("decoded");
        expect(resolveSeat(member, sixStar, undefined, false).member.skill_index).toBe(2);
    });
});

describe("dps import", () => {
    it("round-trips through base64url", () => {
        const data = { seats: [{ charId: "char_332_archet", phase: 2, level: 90, potential: 5, skillIndex: 2, skillLevel: 7, mastery: 3, moduleId: "uniequip_003_archet", moduleLevel: 3, buffs: { atk: 0.16, flatAtk: 0, aspd: 14 } }] };
        const text = encodeDpsImport(data);
        expect(text).not.toMatch(/[+/=]/);
        expect(decodeDpsImport(text)).toEqual(data);
        expect(decodeDpsImport("%%%")).toBeNull();
    });
});
