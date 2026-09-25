import { describe, expect, it } from "vitest";
import { activeFilterCount, scrollBehaviorFor, TOOLBAR_DEFAULTS } from "./toolbar";

describe("activeFilterCount", () => {
    it("is 0 on the defaults", () => {
        expect(activeFilterCount(TOOLBAR_DEFAULTS)).toBe(0);
    });

    it("counts each control that is off its default once", () => {
        expect(activeFilterCount({ ...TOOLBAR_DEFAULTS, filter: "main" })).toBe(1);
        expect(activeFilterCount({ ...TOOLBAR_DEFAULTS, readFilter: "unread" })).toBe(1);
        expect(activeFilterCount({ ...TOOLBAR_DEFAULTS, sort: "newest" })).toBe(1);
        expect(activeFilterCount({ filter: "records", readFilter: "done", sort: "title" })).toBe(3);
    });

    it("counts a control, not how far it moved", () => {
        expect(activeFilterCount({ filter: "events", readFilter: "progress", sort: "default" })).toBe(2);
        expect(activeFilterCount({ filter: "side", readFilter: "any", sort: "leastRead" })).toBe(2);
    });

    it("keeps the defaults the Browse state starts on", () => {
        expect(TOOLBAR_DEFAULTS).toEqual({ filter: "all", readFilter: "any", sort: "default" });
    });
});

describe("scrollBehaviorFor", () => {
    it("is instant when the reader asked for reduced motion", () => {
        expect(scrollBehaviorFor(true)).toBe("auto");
        expect(scrollBehaviorFor(false)).toBe("smooth");
    });
});
