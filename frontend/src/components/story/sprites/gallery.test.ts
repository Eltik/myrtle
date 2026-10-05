import { describe, expect, it } from "vitest";
import type { StorySpriteEntry } from "#/types/generated/StorySpriteEntry";
import type { StorySpriteVariant } from "#/types/generated/StorySpriteVariant";
import { aliasesOf, bodyIndexOf, CARD_CROP, cropFor, faceCentre, filterSprites, groupVariants, initialVariant, matchTier, prepareSpriteSearch, primaryName, sheetLayout, spriteSearchTarget } from "./gallery";
import { cardView } from "./SpriteCard";
import { stepCell } from "./SpriteSheet";

function entry(base: string, over: Partial<StorySpriteEntry> = {}): StorySpriteEntry {
    return { base, kind: "npc", names: [], lines: 0, storyCount: 0, variantCount: 1, ...over };
}

function variant(key: string, over: Partial<StorySpriteVariant> = {}): StorySpriteVariant {
    return { key, bodyUrl: `/b/${key}.png`, wholeBody: true, uses: 0, ...over };
}

const collator = new Intl.Collator("en", { sensitivity: "base", numeric: true });

// The rows the EN census gives (story_sprites_real_data_test, 2026-10-05).
const NINE = entry("avg_npc_043_1", { names: [{ name: "Nine", count: 439 }], lines: 439, storyCount: 18, firstSeen: 1_600_000_000, firstOrder: 900 });
const JIE = entry("avg_npc_2125_1", {
    names: [
        { name: "Jie", count: 252 },
        { name: "Minister of Works", count: 14 },
        { name: "Chun", count: 3 },
    ],
    lines: 269,
    storyCount: 8,
    firstSeen: 1_700_000_000,
    firstOrder: 1500,
});
const CHUN = entry("avg_npc_2127_1", {
    names: [
        { name: "Chun", count: 188 },
        { name: "Sui Regulator Supervisor", count: 27 },
    ],
    lines: 220,
    storyCount: 7,
    firstSeen: 1_700_000_000,
    firstOrder: 1499,
});
const AMIYA = entry("char_002_amiya_1", {
    kind: "operator",
    charId: "char_002_amiya",
    operatorName: "Amiya",
    variant: "1",
    names: [
        { name: "Amiya", count: 3315 },
        { name: "Kal'tsit", count: 21 },
    ],
    lines: 3395,
    storyCount: 158,
    firstOrder: 1,
});
const GLADIIA = entry("avg_474_gladiia_1", { kind: "operator", charId: "char_474_glady", operatorName: "Gladiia", names: [{ name: "Gladiia", count: 883.5 }], lines: 927.5, storyCount: 49, firstSeen: 1_650_000_000, firstOrder: 1200 });
const SILENT = entry("avg_npc_999_1");
const ALL = [NINE, JIE, CHUN, AMIYA, GLADIIA, SILENT];
const search = prepareSpriteSearch(ALL);
const bases = (list: StorySpriteEntry[]) => list.map((e) => e.base);

describe("who a card says it is", () => {
    it("leads with the scripts' top name, then the operator's, then the folder", () => {
        expect(primaryName(NINE)).toBe("Nine");
        expect(primaryName(entry("char_x", { operatorName: "Op" }))).toBe("Op");
        expect(primaryName(SILENT)).toBe("avg_npc_999_1");
    });

    it("lists every other name once, the operator's last when it differs", () => {
        expect(aliasesOf(JIE)).toEqual(["Minister of Works", "Chun"]);
        expect(aliasesOf(AMIYA)).toEqual(["Kal'tsit"]);
        expect(aliasesOf(entry("x", { names: [{ name: "Stranger", count: 2 }], operatorName: "Op" }))).toEqual(["Op"]);
    });

    it("searches names on the name tiers and ids as substrings", () => {
        expect(spriteSearchTarget(JIE)).toEqual({ name: "Jie", aliases: ["Minister of Works", "Chun"], extra: "avg_npc_2125_1 " });
    });
});

describe("search", () => {
    it("finds Jie, Nine and Amiya by name, first", () => {
        expect(filterSprites(search, "jie", "all", "appearances", collator)[0]?.base).toBe("avg_npc_2125_1");
        expect(filterSprites(search, "nine", "all", "appearances", collator)[0]?.base).toBe("avg_npc_043_1");
        expect(filterSprites(search, "amiya", "all", "name", collator)[0]?.base).toBe("char_002_amiya_1");
    });

    it("finds a folder by its npc number, and an operator by char id", () => {
        expect(bases(filterSprites(search, "npc_043", "all", "appearances", collator))).toEqual(["avg_npc_043_1"]);
        expect(bases(filterSprites(search, "char_474", "all", "appearances", collator))).toEqual(["avg_474_gladiia_1"]);
    });

    it("ranks an alias hit under the name hit: Chun is Chun first, then the sprite that is once called Chun", () => {
        expect(bases(filterSprites(search, "chun", "all", "appearances", collator))).toEqual(["avg_npc_2127_1", "avg_npc_2125_1"]);
    });

    it("folds scores into tiers at the scorer's own cut points", () => {
        expect([1010, 615, 449, 299, 180, 90].map(matchTier)).toEqual([5, 4, 3, 2, 1, 0]);
    });

    it("filters by kind before matching", () => {
        expect(bases(filterSprites(search, "", "operator", "appearances", collator))).toEqual(["char_002_amiya_1", "avg_474_gladiia_1"]);
        expect(filterSprites(search, "", "npc", "appearances", collator)).toHaveLength(4);
    });
});

describe("sort", () => {
    it("orders by stories, then lines", () => {
        expect(bases(filterSprites(search, "", "all", "appearances", collator))).toEqual(["char_002_amiya_1", "avg_474_gladiia_1", "avg_npc_043_1", "avg_npc_2125_1", "avg_npc_2127_1", "avg_npc_999_1"]);
    });

    it("orders by name with the folder breaking a tie", () => {
        expect(bases(filterSprites(search, "", "all", "name", collator))).toEqual(["char_002_amiya_1", "avg_npc_999_1", "avg_npc_2127_1", "avg_474_gladiia_1", "avg_npc_2125_1", "avg_npc_043_1"]);
    });

    it("orders by first date, library order inside one date, undated last", () => {
        expect(bases(filterSprites(search, "", "all", "firstSeen", collator))).toEqual(["avg_npc_043_1", "avg_474_gladiia_1", "avg_npc_2127_1", "avg_npc_2125_1", "char_002_amiya_1", "avg_npc_999_1"]);
    });
});

describe("the sheet", () => {
    const variants = [variant("#1$1", { uses: 3 }), variant("#2$1"), variant("#1$2", { uses: 9 }), variant("#3$1"), variant("@smile")];

    it("groups expressions by body, in body order, keeping the hub's face order", () => {
        expect(groupVariants(variants).map((g) => [g.body, g.variants.map((v) => v.key)])).toEqual([
            [1, ["#1$1", "#2$1", "#3$1", "@smile"]],
            [2, ["#1$2"]],
        ]);
        expect(bodyIndexOf("#12$3")).toBe(3);
    });

    it("opens on the card's own expression, else the most used, else the first", () => {
        expect(initialVariant(variants, "#2$1")).toBe(1);
        expect(initialVariant(variants, undefined)).toBe(2);
        expect(initialVariant([variant("#1$1"), variant("#2$1")], "#9$9")).toBe(0);
    });

    it("moves by arrow over the laid-out cells, down to the nearest in the next row", () => {
        // Two rows of three, then a group break and a row of one.
        const at = (col: number, row: number) => ({ left: col * 100, top: row * 100, width: 90, height: 90 });
        const rects = [at(0, 0), at(1, 0), at(2, 0), at(0, 1), at(1, 1), at(2, 1), at(0, 2.3)];
        expect(stepCell(rects, 1, "ArrowDown")).toBe(4);
        expect(stepCell(rects, 4, "ArrowUp")).toBe(1);
        expect(stepCell(rects, 5, "ArrowDown")).toBe(6);
        expect(stepCell(rects, 0, "ArrowUp")).toBe(0);
        expect(stepCell(rects, 2, "ArrowRight")).toBe(3);
        expect(stepCell(rects, 0, "ArrowLeft")).toBe(0);
        expect(stepCell(rects, 3, "End")).toBe(6);
    });

    it("lays the download out in rows of at most six", () => {
        expect(sheetLayout(13, { cell: 220, label: 26, head: 64, gap: 12, maxCols: 6 })).toEqual({ cols: 6, rows: 3, width: 1404, height: 850, cell: 220, label: 26, head: 64, gap: 12 });
        expect(sheetLayout(2, { cell: 220, label: 26, head: 64, gap: 12, maxCols: 6 }).cols).toBe(2);
    });
});

describe("the head crop", () => {
    it("reads the face centre off facePos over the TEXTURE size", () => {
        // Kal'tsit-like: a 1280 texture, so 570/1280 and never 570/1024.
        const f = faceCentre({ facePos: { x: 570, y: 233, w: 100, h: 100 }, bodySize: { w: 1280, h: 1280 } });
        expect(f.measured).toBe(true);
        expect(f.x).toBeCloseTo(620 / 1280, 6);
        expect(f.y).toBeCloseTo(283 / 1280, 6);
        expect(faceCentre({})).toEqual({ x: 0.5, y: 0.3, measured: false });
    });

    it("centres the face and clamps the plate to cover the frame", () => {
        const mid = cropFor({ x: 0.5, y: 0.3 }, CARD_CROP);
        expect(mid.size).toBeCloseTo(170, 6);
        expect(mid.left).toBeCloseTo(-35, 6);
        // 0.34 x 4/3 - 0.3 x 1.7 = -0.0567 of the width, over the 4/3 height.
        expect(mid.top).toBeCloseTo((-0.056667 / (4 / 3)) * 100, 3);
        // A face at the far left edge cannot pull the plate off the frame.
        expect(cropFor({ x: 0, y: 0 }, CARD_CROP)).toEqual({ size: 170, left: 0, top: 0 });
        expect(cropFor({ x: 1, y: 1 }, CARD_CROP).left).toBeCloseTo(-70, 6);
    });
});

describe("a card's props", () => {
    it("are derived from its entry alone", () => {
        expect(cardView({ ...JIE, thumb: variant("#1$1", { uses: 68 }) })).toMatchInlineSnapshot(`
          {
            "aliases": 2,
            "base": "avg_npc_2125_1",
            "kind": "npc",
            "name": "Jie",
            "stories": 8,
            "thumbKey": "#1$1",
          }
        `);
    });
});
