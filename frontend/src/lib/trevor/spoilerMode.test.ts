import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { markSpoilersConfirmed, resetSpoilerMemory, SPOILER_CONFIRMED_KEY, SPOILER_MODE_KEY, setSpoilersOn, spoilersConfirmed } from "./spoilerMode";

describe("spoiler mode's per-browser choice", () => {
    beforeEach(() => {
        window.localStorage.clear();
        resetSpoilerMemory();
    });
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("stores on as 'all', off as nothing, and the confirmation once", () => {
        setSpoilersOn(true);
        expect(window.localStorage.getItem(SPOILER_MODE_KEY)).toBe("all");
        setSpoilersOn(false);
        expect(window.localStorage.getItem(SPOILER_MODE_KEY)).toBeNull();
        expect(spoilersConfirmed()).toBe(false);
        markSpoilersConfirmed();
        expect(window.localStorage.getItem(SPOILER_CONFIRMED_KEY)).toBe("1");
        resetSpoilerMemory();
        expect(spoilersConfirmed()).toBe(true);
    });

    it("keeps the choice in memory when storage throws", () => {
        vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
            throw new Error("quota");
        });
        vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
            throw new Error("blocked");
        });
        expect(spoilersConfirmed()).toBe(false);
        markSpoilersConfirmed();
        expect(spoilersConfirmed()).toBe(true);
        expect(() => setSpoilersOn(true)).not.toThrow();
    });
});
