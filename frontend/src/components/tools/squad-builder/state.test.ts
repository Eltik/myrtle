import { describe, expect, it } from "vitest";
import type { SeatHandoff } from "#/types/generated/SeatHandoff";
import type { SquadMember } from "#/types/generated/SquadMember";
import { buildDpsImport, seatBuffs } from "./handoff";
import { emptyDraft, encodeDpsImport, type IResolvedSeat, SUPPORT_SEAT, toAnalyzeRequest } from "./model";
import { INITIAL_STATE, isDirty, nextEmptySeat, squadReducer, swapSeats } from "./state";

const m = (id: string): SquadMember => ({ char_id: id, skill_index: 0, module_id: null, build: null });

describe("seat moves", () => {
    it("swaps two seats, either of them empty", () => {
        const members = emptyDraft().members;
        members[0] = m("char_a");
        members[3] = m("char_b");
        expect(
            swapSeats(members, 0, 3)
                .map((s) => s?.char_id ?? null)
                .slice(0, 4),
        ).toEqual(["char_b", null, null, "char_a"]);
        expect(swapSeats(members, 0, 5)[5]?.char_id).toBe("char_a");
        expect(swapSeats(members, 0, 5)[0]).toBeNull();
    });

    it("ignores a swap onto itself or off the grid", () => {
        const members = emptyDraft().members;
        members[0] = m("char_a");
        expect(swapSeats(members, 0, 0)[0]?.char_id).toBe("char_a");
        expect(swapSeats(members, 0, 12)[0]?.char_id).toBe("char_a");
    });

    it("finds the next empty seat, wrapping", () => {
        const members = emptyDraft().members;
        members[0] = m("char_a");
        members[1] = m("char_b");
        expect(nextEmptySeat(members, 0)).toBe(2);
        expect(nextEmptySeat(members, 11)).toBe(2);
        expect(
            nextEmptySeat(
                Array.from({ length: 12 }, (_, i) => m(`char_${i}`)),
                3,
            ),
        ).toBeNull();
    });
});

describe("reducer", () => {
    it("is dirty against the saved slot, and a scratch squad once it holds anything", () => {
        expect(isDirty(INITIAL_STATE)).toBe(false);
        const s1 = squadReducer(INITIAL_STATE, { type: "SET_MEMBER", seat: 0, member: m("char_a") });
        expect(isDirty(s1)).toBe(true);
        const saved = squadReducer(s1, { type: "SAVED", slot: 2, draft: s1.draft });
        expect(saved.slot).toBe(2);
        expect(isDirty(saved)).toBe(false);
        const renamed = squadReducer(saved, { type: "SET_NAME", name: "Lowkey" });
        expect(isDirty(renamed)).toBe(true);
        expect(isDirty(squadReducer(renamed, { type: "REVERT" }))).toBe(false);
    });

    it("caps the name at 40 characters", () => {
        expect(squadReducer(INITIAL_STATE, { type: "SET_NAME", name: "x".repeat(60) }).draft.name).toHaveLength(40);
    });

    it("keeps a shared squad read only and restores the working copy on close", () => {
        const working = squadReducer(INITIAL_STATE, { type: "SET_MEMBER", seat: 0, member: m("char_a") });
        const shared = emptyDraft("Shared");
        shared.members[1] = m("char_z");
        const open = squadReducer(working, { type: "OPEN_SHARE", draft: shared });
        expect(open.readOnlyShare).toBe(true);
        expect(squadReducer(open, { type: "SET_MEMBER", seat: 2, member: m("char_q") })).toBe(open);
        const closed = squadReducer(open, { type: "CLOSE_SHARE" });
        expect(closed.draft.members[0]?.char_id).toBe("char_a");
        const kept = squadReducer(open, { type: "KEEP_SHARE" });
        expect(kept.readOnlyShare).toBe(false);
        expect(kept.draft.name).toBe("Shared");
        expect(kept.slot).toBeNull();
    });

    it("clears the seats but keeps the name and stage", () => {
        let s = squadReducer(INITIAL_STATE, { type: "SET_NAME", name: "Mine" });
        s = squadReducer(s, { type: "SET_STAGE", stageId: "main_01-07" });
        s = squadReducer(s, { type: "SET_MEMBER", seat: 4, member: m("char_a") });
        s = squadReducer(s, { type: "SET_SUPPORT", member: m("char_b") });
        const cleared = squadReducer(s, { type: "CLEAR" });
        expect(cleared.draft.members.every((x) => x === null)).toBe(true);
        expect(cleared.draft.support).toBeNull();
        expect(cleared.draft).toMatchObject({ name: "Mine", stageId: "main_01-07" });
    });
});

describe("DPS calculator hand-off", () => {
    const resolved = (id: string, moduleId: string | null = null): IResolvedSeat => ({
        member: { char_id: id, skill_index: 2, module_id: moduleId, build: null },
        build: { phase: 2, level: 90, potential: 5, skill_level: 7, mastery: 3, module_level: 3 },
        source: "full",
    });
    const handoff: SeatHandoff[] = [
        {
            seat: 0,
            char_id: "char_a",
            buffs: { atk: 0.3, flat_atk: 50, aspd: 10 },
            counted: [
                { from_seat: 1, char_id: "char_b", name: "B", cond: "squad", atk: 0.1, flat_atk: 0, aspd: 10 },
                { from_seat: 2, char_id: "char_c", name: "C", cond: "squad", atk: 0.2, flat_atk: 50, aspd: 0 },
            ],
            left_out: [],
        },
    ];

    it("re-sums the buffs from the givers that are sent along", () => {
        expect(seatBuffs(handoff[0], new Set([0, 1]))).toEqual({ atk: 0.1, flatAtk: 0, aspd: 10 });
        expect(seatBuffs(handoff[0], new Set([0, 1, 2]))).toEqual({ atk: 0.3, flatAtk: 50, aspd: 10 });
        expect(seatBuffs(handoff[0], new Set([0]))).toEqual({ atk: 0, flatAtk: 0, aspd: 0 });
        expect(seatBuffs(undefined, new Set([0]))).toEqual({ atk: 0, flatAtk: 0, aspd: 0 });
    });

    it("sends the seat first, then up to three squadmates in seat order", () => {
        const seats = [resolved("char_a", "uniequip_002_a"), resolved("char_b"), resolved("char_c"), resolved("char_d"), resolved("char_e"), null];
        const out = buildDpsImport(0, [4, 2, 1, 3, 5], seats, null, handoff);
        expect(out.seats.map((s) => s.charId)).toEqual(["char_a", "char_b", "char_c", "char_d"]);
        expect(out.seats[0]).toMatchObject({ skillIndex: 2, skillLevel: 7, mastery: 3, moduleId: "uniequip_002_a", moduleLevel: 3, buffs: { atk: 0.3, flatAtk: 50, aspd: 10 } });
        expect(out.seats[1].moduleLevel).toBe(0);
    });

    it("can send the support", () => {
        const out = buildDpsImport(0, [SUPPORT_SEAT], [resolved("char_a")], resolved("char_s"), []);
        expect(out.seats.map((s) => s.charId)).toEqual(["char_a", "char_s"]);
    });
});

describe("wire bodies keep their shape", () => {
    // Pinned as JSON text, so the field order of the bodies, and with it the
    // `?import=` string and the analysis query key, stays what it was.
    const r = (id: string, moduleId: string | null, moduleLevel: number): IResolvedSeat => ({
        member: { char_id: id, skill_index: 1, module_id: moduleId, build: null },
        build: { phase: 2, level: 80, potential: 3, skill_level: 7, mastery: 2, module_level: moduleLevel },
        source: "roster",
    });
    const seats = [r("char_a", "uniequip_002_a", 2), null, r("char_c", null, 3)];
    const support = r("char_s", "uniequip_003_s", 1);
    const handoff: SeatHandoff[] = [
        {
            seat: 0,
            char_id: "char_a",
            buffs: { atk: 0, flat_atk: 0, aspd: 0 },
            counted: [
                { from_seat: 2, char_id: "char_c", name: "C", cond: "squad", atk: 0.1, flat_atk: 5, aspd: 3 },
                { from_seat: SUPPORT_SEAT, char_id: "char_s", name: "S", cond: "squad", atk: 0.2, flat_atk: 0, aspd: 0 },
            ],
            left_out: [],
        },
    ];

    it("builds the analyze request", () => {
        const body = toAnalyzeRequest(seats, support, { startDp: 12, deployLimit: 8, dub: null });
        const seat = (id: string, moduleId: string | null, moduleLevel: number) => `{"char_id":"${id}","phase":2,"level":80,"potential":3,"skill_index":1,"skill_level":7,"mastery":2,"module_id":${moduleId ? `"${moduleId}"` : "null"},"module_level":${moduleLevel}}`;
        const members = [seat("char_a", "uniequip_002_a", 2), "null", seat("char_c", null, 0), ...Array(9).fill("null")].join(",");
        expect(JSON.stringify(body)).toBe(`{"members":[${members}],"support":${seat("char_s", "uniequip_003_s", 1)},"start_dp":12,"deploy_limit":8,"dub":null}`);
    });

    it("builds the DPS import payload", () => {
        const data = buildDpsImport(0, [SUPPORT_SEAT, 2], seats, support, handoff);
        expect(encodeDpsImport(data)).toBe(
            "eyJzZWF0cyI6W3siY2hhcklkIjoiY2hhcl9hIiwicGhhc2UiOjIsImxldmVsIjo4MCwicG90ZW50aWFsIjozLCJza2lsbEluZGV4IjoxLCJza2lsbExldmVsIjo3LCJtYXN0ZXJ5IjoyLCJtb2R1bGVJZCI6InVuaWVxdWlwXzAwMl9hIiwibW9kdWxlTGV2ZWwiOjIsImJ1ZmZzIjp7ImF0ayI6MC4zLCJmbGF0QXRrIjo1LCJhc3BkIjozfX0seyJjaGFySWQiOiJjaGFyX2MiLCJwaGFzZSI6MiwibGV2ZWwiOjgwLCJwb3RlbnRpYWwiOjMsInNraWxsSW5kZXgiOjEsInNraWxsTGV2ZWwiOjcsIm1hc3RlcnkiOjIsIm1vZHVsZUlkIjpudWxsLCJtb2R1bGVMZXZlbCI6MCwiYnVmZnMiOnsiYXRrIjowLCJmbGF0QXRrIjowLCJhc3BkIjowfX0seyJjaGFySWQiOiJjaGFyX3MiLCJwaGFzZSI6MiwibGV2ZWwiOjgwLCJwb3RlbnRpYWwiOjMsInNraWxsSW5kZXgiOjEsInNraWxsTGV2ZWwiOjcsIm1hc3RlcnkiOjIsIm1vZHVsZUlkIjoidW5pZXF1aXBfMDAzX3MiLCJtb2R1bGVMZXZlbCI6MSwiYnVmZnMiOnsiYXRrIjowLCJmbGF0QXRrIjowLCJhc3BkIjowfX1dfQ",
        );
    });
});
