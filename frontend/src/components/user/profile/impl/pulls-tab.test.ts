import { describe, expect, it } from "vitest";
import { resolveActiveTab, withPullsTab } from "./layout";
import { isTabId } from "./types";

describe("the Pulls tab", () => {
    it("is shown to a visitor only while the owner's pull history is public", () => {
        expect(withPullsTab(["stats", "roster"], false, true)).toEqual(["stats", "roster", "pulls"]);
        expect(withPullsTab(["stats", "roster"], false, false)).toEqual(["stats", "roster"]);
        // A profile read without the switch (an older backend) shows no Pulls tab.
        expect(withPullsTab(["stats", "roster"], false, undefined)).toEqual(["stats", "roster"]);
        expect(withPullsTab(["stats", "roster"], false, null)).toEqual(["stats", "roster"]);
    });

    it("is always shown to the owner, last, so they can reach the switch", () => {
        expect(withPullsTab(["stats"], true, false)).toEqual(["stats", "pulls"]);
        expect(withPullsTab([], true, undefined)).toEqual(["pulls"]);
    });

    it("can be the remembered or linked tab", () => {
        expect(isTabId("pulls")).toBe(true);
        expect(resolveActiveTab(withPullsTab(["stats"], false, true), "stored", "pulls", null)).toBe("pulls");
        // Gone private since: the reader lands on the first tab instead.
        expect(resolveActiveTab(withPullsTab(["stats"], false, false), "stored", "pulls", null)).toBe("stats");
    });
});
