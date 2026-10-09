import { describe, expect, it } from "vitest";
import { levelVariant } from "#/components/admin/shell/model";
import { canDeleteList, canEditList, canPublishList, canSetFlair, filterCounts, filterRows, grantedRows, type IBrowseListLike, levelChangePlan, matchesQuery, nextVersionNumber, staffRows } from "./model";

function list(slug: string, over: Partial<IBrowseListLike> = {}): IBrowseListLike {
    return {
        slug,
        title: slug,
        listType: "official",
        author: { name: "Doctor" },
        flairCode: null,
        flairLabel: null,
        flairColor: null,
        isTrending: false,
        tiers: [{ operators: [1, 2] }, { operators: [3] }],
        views24h: 10,
        updatedAtMs: 1_700_000_000_000,
        ...over,
    };
}

const browse = [list("endgame-dps", { title: "Endgame DPS rankings", flairLabel: "Endgame", isTrending: true }), list("budget-squads", { title: "Budget squads", flairLabel: "Beginner" }), list("my-faves", { listType: "community", title: "Ceobe's picks" })];

describe("staffRows", () => {
    it("scopes to the chosen list type, every row at admin", () => {
        expect(staffRows(browse, "official").map((r) => r.slug)).toEqual(["endgame-dps", "budget-squads"]);
        expect(staffRows(browse, "community").map((r) => r.slug)).toEqual(["my-faves"]);
        expect(staffRows(browse, "all")).toHaveLength(3);
        expect(staffRows(browse, "all").every((r) => r.level === "admin")).toBe(true);
    });

    it("counts tiers and placements", () => {
        const [row] = staffRows(browse, "official");
        expect(row?.tiers).toBe(2);
        expect(row?.placements).toBe(3);
        expect(row?.hot).toBe(true);
    });
});

describe("grantedRows", () => {
    it("joins grants to browse stats and keeps an unlisted list with no stats", () => {
        const rows = grantedRows(browse, [
            { slug: "budget-squads", title: "Budget squads", listType: "official", permission: "publish" },
            { slug: "hidden", title: "Hidden draft", listType: "community", permission: "view" },
        ]);
        expect(rows.map((r) => [r.slug, r.level])).toEqual([
            ["budget-squads", "publish"],
            ["hidden", "view"],
        ]);
        expect(rows[0]?.placements).toBe(3);
        expect(rows[1]).toMatchObject({ title: "Hidden draft", author: null, tiers: null, placements: null, views24h: null, hot: false });
    });

    it("is empty without grants", () => {
        expect(grantedRows(browse, [])).toEqual([]);
    });
});

describe("filtering", () => {
    const rows = staffRows(browse, "all");

    it("matches title, slug or flair, ignoring case and punctuation", () => {
        const [dps, budget, faves] = rows;
        if (!dps || !budget || !faves) throw new Error("fixture rows missing");
        expect(matchesQuery(dps, "ENDGAME")).toBe(true);
        expect(matchesQuery(dps, "endgame-dps")).toBe(true);
        expect(matchesQuery(budget, "beginner")).toBe(true);
        expect(matchesQuery(faves, "ceobes")).toBe(true);
        expect(matchesQuery(budget, "dps")).toBe(false);
        expect(matchesQuery(budget, "  ")).toBe(true);
    });

    it("applies the trending filter and the query together", () => {
        expect(filterRows(rows, "hot", "").map((r) => r.slug)).toEqual(["endgame-dps"]);
        expect(filterRows(rows, "all", "squads").map((r) => r.slug)).toEqual(["budget-squads"]);
        expect(filterRows(rows, "hot", "squads")).toEqual([]);
    });

    it("counts both filter tabs from the unfiltered rows", () => {
        expect(filterCounts(rows)).toEqual({ all: 3, hot: 1 });
    });
});

describe("level gates", () => {
    it("matches the backend's per-action levels", () => {
        expect([canEditList("view"), canEditList("edit")]).toEqual([false, true]);
        expect([canPublishList("edit"), canPublishList("publish"), canPublishList("admin")]).toEqual([false, true, true]);
        expect([canSetFlair("view"), canSetFlair("edit")]).toEqual([false, true]);
        expect([canDeleteList("publish"), canDeleteList("admin")]).toEqual([false, true]);
    });

    it("maps levels to the design's badge variants", () => {
        expect(["view", "edit", "publish", "admin"].map((l) => levelVariant(l))).toEqual(["outline", "info", "success", "default"]);
    });
});

describe("nextVersionNumber", () => {
    it("is one past the highest version, or 1 for an unpublished list", () => {
        expect(nextVersionNumber([])).toBe(1);
        expect(nextVersionNumber([{ version: 3 }, { version: 14 }, { version: 2 }])).toBe(15);
    });
});

describe("levelChangePlan", () => {
    it("grants the new level before revoking the old one, and skips a no-op", () => {
        expect(levelChangePlan("edit", "publish")).toEqual({ grant: "publish", revoke: "edit" });
        expect(levelChangePlan("edit", "edit")).toBeNull();
    });
});
