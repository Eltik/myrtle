import { describe, expect, it } from "vitest";
import { gachaStorageOff } from "./storage-gate";

describe("gachaStorageOff", () => {
    it("is off only when the switch is explicitly false", () => {
        expect(gachaStorageOff({ store_gacha: false })).toBe(true);
    });

    it("keeps the refresh for an on switch, a missing settings row, or no user", () => {
        expect(gachaStorageOff({ store_gacha: true })).toBe(false);
        expect(gachaStorageOff({ store_gacha: null })).toBe(false);
        expect(gachaStorageOff({})).toBe(false);
        expect(gachaStorageOff(null)).toBe(false);
        expect(gachaStorageOff(undefined)).toBe(false);
    });
});
