import { beforeEach, describe, expect, it, vi } from "vitest";
import type { IOperatorsStaticMap } from "#/types/operators";

const backendFetch = vi.fn();
vi.mock("#/lib/fetch", () => ({ backendFetch: (...args: unknown[]) => backendFetch(...args) }));
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({
            handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }),
        }),
    }),
}));

const { deepCamelize, operatorsRosterQueryOptions, toRosterItems } = await import("./operators");

// Two raw `/static/operators` rows: camelCase on top, PascalCase + trailing underscore inside
// some nested shapes, and fields the roster never reads (handbook, talents, audio).
const RAW = {
    char_a: {
        id: "char_a",
        name: "A",
        profession: "WARRIOR",
        professionName: "Guard",
        phases: [{ MaxLevel: 50, AttributesKeyFrames: [{ Level: 1, Data: { MaxHp: 1000, Type_: "x" } }] }],
        skills: [{ skillId: "skchr_a_1", static: { levels: [{ name: "S1", Duration: 10 }] } }],
        modules: [],
        potentialRanks: [{ Type_: "BUFF", Description: "Cost -1" }],
        favorKeyFrames: [{ Level: 0, Data: { Atk: 0 } }],
        baseSkills: [],
        skin: "/skin/a.png",
        portrait: "/portrait/a.png",
        itemObtainApproach: "Recruitment",
        obtainChannel: null,
        handbook: { storyTextAudio: [{ Stories: [] }] },
        talents: [{ Candidates: [] }],
        audio: { Voices: [] },
    },
    char_b: {
        id: "char_b",
        name: "B",
        profession: "MEDIC",
        professionName: "Medic",
        phases: [],
        skills: [],
        modules: [{ id: "uniequip_b", Phases: [] }],
        potentialRanks: [],
        favorKeyFrames: [],
        baseSkills: [{ buffId: "b1" }],
        skin: null,
        portrait: null,
        itemObtainApproach: null,
        obtainChannel: "event",
        tmplDefault: "char_b2",
        drones: [{ Id: "d" }],
    },
} as unknown as IOperatorsStaticMap;

const FIELDS = ["id", "name", "profession", "professionName", "phases", "skills", "modules", "potentialRanks", "favorKeyFrames", "baseSkills", "skin", "portrait", "itemObtainApproach", "obtainChannel", "tmplDefault"];

describe("toRosterItems", () => {
    it("gives the same rows as camelizing the whole table and picking the roster fields", () => {
        const whole = Object.values(deepCamelize(RAW)) as unknown as Record<string, unknown>[];
        const picked = whole.map((op) => Object.fromEntries(FIELDS.filter((f) => f in op).map((f) => [f, op[f]])));
        expect(toRosterItems(Object.values(RAW))).toEqual(picked);
    });

    it("drops what the tabs never read and camelizes nested keys", () => {
        const [a, b] = toRosterItems(Object.values(RAW)) as unknown as Record<string, unknown>[];
        expect(Object.keys(a)).not.toContain("handbook");
        expect(Object.keys(a)).not.toContain("talents");
        expect(Object.keys(b)).not.toContain("drones");
        expect(a.phases).toEqual([{ maxLevel: 50, attributesKeyFrames: [{ level: 1, data: { maxHp: 1000, type: "x" } }] }]);
        expect(b.tmplDefault).toBe("char_b2");
    });
});

describe("operatorsRosterQueryOptions", () => {
    beforeEach(() => backendFetch.mockReset());

    it("reads the roster rows from the backend, unprefixed for the default server", async () => {
        const served = toRosterItems(Object.values(RAW));
        backendFetch.mockResolvedValue(new Response(JSON.stringify(Object.values(RAW)), { status: 200 }));
        const rows = await (operatorsRosterQueryOptions().queryFn as () => Promise<unknown[]>)();
        expect(backendFetch).toHaveBeenCalledWith("/operators/roster");
        expect(rows).toEqual(served);
    });

    it("prefixes another server and rejects a failed response", async () => {
        backendFetch.mockResolvedValue(new Response("{}", { status: 503 }));
        await expect((operatorsRosterQueryOptions("jp").queryFn as () => Promise<unknown>)()).rejects.toThrow("503");
        expect(backendFetch).toHaveBeenCalledWith("/jp/operators/roster");
    });
});
