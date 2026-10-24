import { deflateSync, strToU8 } from "fflate";
import { describe, expect, it } from "vitest";
import { decodeInstance, packShare, SHARE_VERSION, unpackShare } from "./shareCodec";

function rawPack(payload: unknown): string {
    let bin = "";
    for (const b of deflateSync(strToU8(JSON.stringify(payload)))) bin += String.fromCharCode(b);
    return btoa(bin).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

describe("share codec", () => {
    it("round-trips a body", () => {
        const body = { a: [1, null, "x"], b: 0.25 };
        expect(unpackShare("dps", packShare("dps", body))).toEqual(body);
    });

    it("ignores another version instead of guessing at it", () => {
        expect(unpackShare("dps", rawPack([SHARE_VERSION, "dps", { ok: 1 }]))).toEqual({ ok: 1 });
        expect(unpackShare("dps", rawPack([SHARE_VERSION + 1, "dps", { ok: 1 }]))).toBeNull();
        expect(unpackShare("dps", rawPack([0, "dps", { ok: 1 }]))).toBeNull();
        expect(unpackShare("dps", rawPack({ v: SHARE_VERSION }))).toBeNull();
        expect(unpackShare("hps", rawPack([SHARE_VERSION, "dps", { ok: 1 }]))).toBeNull();
    });

    it("carries a card's own zones as a sixteenth entry, and rejects a malformed one", () => {
        const base = ["char_a", 0, 1, null, null, 1, 100, 1, 10, 0, 3, [null, null, null, null], 0, 0, 1];
        expect(decodeInstance([...base, [0.4]])?.config.zones?.atkPercent).toBe(0.4);
        expect(decodeInstance(base)?.config.zones).toBeUndefined();
        expect(decodeInstance([...base, ["x"]])).toBeNull();
    });

    it("never throws on malformed text", () => {
        for (const text of ["", "=", "a", "____", "not-deflate-data", "%7B%7D", "x".repeat(10_000)]) expect(unpackShare("dps", text)).toBeNull();
    });

    it("clamps card numbers to the ranges the controls allow", () => {
        const inst = decodeInstance(["char_a", 99, 1, 7, 500, 0, 900, 9, 42, 1, 8, [null, null, null, null], 0, 0, 1]);
        expect(inst?.color).toBeUndefined();
        expect(inst?.config).toMatchObject({ promotion: 2, level: 90, potential: 1, trust: 100, skillIndex: 3, skillRank: 10, moduleLevel: 3 });
    });

    it("rejects a malformed card tuple", () => {
        expect(decodeInstance(null)).toBeNull();
        expect(decodeInstance(["char_a"])).toBeNull();
        expect(decodeInstance([42, 0, 1, null, null, 1, 100, 1, 10, 0, 3, [null, null, null, null], 0, 0, 1])).toBeNull();
        expect(decodeInstance(["char_a", 0, 1, null, null, 1.5, 100, 1, 10, 0, 3, [null, null, null, null], 0, 0, 1])).toBeNull();
        expect(decodeInstance(["char_a", 0, 1, null, null, 1, 100, 1, 10, 0, 3, [null, "x", null, null], 0, 0, 1])).toBeNull();
    });
});
