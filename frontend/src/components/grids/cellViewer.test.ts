import { describe, expect, it } from "vitest";
import { type ITierEntity, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import { entityLink, heroArtPaths, isViewableCell, neighbourIndex, SWIPE_CLOSE_PX, SWIPE_STEP_PX, swipeIntent, viewableIndices } from "./cellViewer";

const cell = (label: string, kind: "operator" | null = null) => ({ label, kind });

function entity(summary: Partial<EntitySummary> & Pick<EntitySummary, "kind" | "id">): ITierEntity {
    return toTierEntity(summary.kind, summary.id, { name: summary.id, icon: null, href: null, facets: {}, ...summary } as EntitySummary, UNPLACED);
}

describe("viewable cells", () => {
    it("opens a cell with a label or a pick, never an empty one or a blank label", () => {
        expect(isViewableCell(cell("Favorite Guard"))).toBe(true);
        expect(isViewableCell(cell("", "operator"))).toBe(true);
        expect(isViewableCell(cell(""))).toBe(false);
        expect(isViewableCell(cell("   "))).toBe(false);
    });

    it("lists them in board order", () => {
        expect(viewableIndices([cell("a"), cell(""), cell("", "operator"), cell(" "), cell("b")])).toEqual([0, 2, 4]);
        expect(viewableIndices([])).toEqual([]);
    });
});

describe("neighbourIndex steps through the viewable cells", () => {
    const order = [0, 2, 4, 7];

    it("skips cells not in the order", () => {
        expect(neighbourIndex(order, 2, 1)).toBe(4);
        expect(neighbourIndex(order, 4, -1)).toBe(2);
    });

    it("stops at either end instead of wrapping", () => {
        expect(neighbourIndex(order, 0, -1)).toBeNull();
        expect(neighbourIndex(order, 7, 1)).toBeNull();
        expect(neighbourIndex([3], 3, 1)).toBeNull();
    });

    it("steps from where a cell outside the order would sit", () => {
        expect(neighbourIndex(order, 5, 1)).toBe(7);
        expect(neighbourIndex(order, 5, -1)).toBe(4);
        expect(neighbourIndex(order, 9, 1)).toBeNull();
        expect(neighbourIndex([], 0, 1)).toBeNull();
    });
});

describe("swipeIntent", () => {
    it("reads left as next, right as prev, down as close", () => {
        expect(swipeIntent(-SWIPE_STEP_PX, 0)).toBe("next");
        expect(swipeIntent(SWIPE_STEP_PX, 10)).toBe("prev");
        expect(swipeIntent(0, SWIPE_CLOSE_PX)).toBe("close");
    });

    it("ignores short moves, an upward swipe and a diagonal", () => {
        expect(swipeIntent(-(SWIPE_STEP_PX - 1), 0)).toBeNull();
        expect(swipeIntent(0, SWIPE_CLOSE_PX - 1)).toBeNull();
        expect(swipeIntent(0, -200)).toBeNull();
        expect(swipeIntent(-100, 90)).toBeNull();
    });
});

describe("entityLink", () => {
    it("links an operator-backed kind to its operator and an enemy to its entry", () => {
        expect(entityLink(entity({ kind: "operator", id: "char_002_amiya", href: "/operators/char_002_amiya" }))).toEqual({ to: "/operators/$id", id: "char_002_amiya" });
        expect(entityLink(entity({ kind: "skin", id: "char_002_amiya@winter#1", href: "/operators/char_002_amiya" }))).toEqual({ to: "/operators/$id", id: "char_002_amiya" });
        expect(entityLink(entity({ kind: "enemy", id: "enemy_1007_slime", href: "/enemies/enemy_1007_slime" }))).toEqual({ to: "/enemies/$id", id: "enemy_1007_slime" });
    });

    it("links nothing without an href, for a route it does not know, or for an unresolved pick", () => {
        expect(entityLink(entity({ kind: "event", id: "act1", href: null }))).toBeNull();
        expect(entityLink(entity({ kind: "event", id: "act1", href: "/events/act1" }))).toBeNull();
        expect(entityLink(toTierEntity("operator", "char_x", null, UNPLACED))).toBeNull();
        expect(entityLink(null)).toBeNull();
    });
});

describe("heroArtPaths", () => {
    it("tries an operator's elite 2 art, reduced first, then elite 1", () => {
        expect(heroArtPaths(entity({ kind: "operator", id: "char_002_amiya" }))).toEqual(["/textures/chararts/char_002_amiya/char_002_amiya_2b.png", "/textures/chararts/char_002_amiya/char_002_amiya_2.png", "/textures/chararts/char_002_amiya/char_002_amiya_1.png"]);
    });

    it("tries an outfit's skinpack art with its # escaped, reduced first", () => {
        const skin = entity({ kind: "skin", id: "char_002_amiya@winter#1", facets: { char_id: "char_002_amiya" } });
        expect(heroArtPaths(skin)).toEqual(["/textures/skinpack/char_002_amiya/char_002_amiya_winter%231b.png", "/textures/skinpack/char_002_amiya/char_002_amiya_winter%231.png"]);
    });

    it("leaves every other kind and an unresolved pick to its tile art", () => {
        expect(heroArtPaths(entity({ kind: "enemy", id: "enemy_1007_slime" }))).toEqual([]);
        expect(heroArtPaths(toTierEntity("operator", "char_x", null, UNPLACED))).toEqual([]);
        expect(heroArtPaths(null)).toEqual([]);
    });
});
