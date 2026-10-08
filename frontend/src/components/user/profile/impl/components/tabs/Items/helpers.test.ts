import { describe, expect, it } from "vitest";
import type { IMaterialItem } from "#/lib/api/materials";
import { CATEGORY_ITEM_LABELS, CATEGORY_LABELS, CATEGORY_ORDER, categorizeItem, formatItemType, formatVoucherId, rarityTierToNumber } from "./helpers";

function item(partial: Partial<IMaterialItem>): IMaterialItem {
    return { itemId: "x", classifyType: "NORMAL", itemType: "UNKNOWN", iconId: "", ...partial } as IMaterialItem;
}

const NO_EXP = new Set<string>();

describe("rarityTierToNumber", () => {
    it("reads TIER_n and bare numbers", () => {
        expect(rarityTierToNumber("TIER_1")).toBe(1);
        expect(rarityTierToNumber("TIER_5")).toBe(5);
        expect(rarityTierToNumber("TIER_6")).toBe(6);
        expect(rarityTierToNumber("3")).toBe(3);
    });

    it("falls back to 1 for anything out of range or unreadable", () => {
        for (const tier of [null, undefined, "", "TIER_0", "TIER_7", "TIER_X", "rare"]) {
            expect(rarityTierToNumber(tier)).toBe(1);
        }
    });
});

describe("formatItemType", () => {
    it("title-cases the raw code", () => {
        expect(formatItemType("SKILL_SUMMARY")).toBe("Skill Summary");
        expect(formatItemType("MATERIAL")).toBe("Material");
        expect(formatItemType("TKT_GACHA10")).toBe("Tkt Gacha10");
    });

    it("names unknown or empty types plainly", () => {
        for (const t of [null, undefined, "", "UNKNOWN", "NONE"]) {
            expect(formatItemType(t)).toBe("Item");
        }
    });
});

describe("formatVoucherId", () => {
    it("spells out the permanent suffix and title-cases words", () => {
        expect(formatVoucherId("voucher_skin_perm")).toBe("Voucher Skin (Permanent)");
        expect(formatVoucherId("class_token")).toBe("Class Token");
        expect(formatVoucherId("perm_voucher")).toBe("Perm Voucher");
    });
});

describe("categorizeItem", () => {
    it("treats a missing meta as other and the EXP id set as exp", () => {
        expect(categorizeItem(null, NO_EXP)).toBe("other");
        expect(categorizeItem(item({ itemId: "2004", itemType: "MATERIAL", classifyType: "MATERIAL" }), new Set(["2004"]))).toBe("exp");
    });

    it("sorts consumables before any item type check", () => {
        expect(categorizeItem(item({ classifyType: "CONSUME", itemType: "AP_SUPPLY" }), NO_EXP)).toBe("consume");
    });

    it("buckets by item type", () => {
        const cases: [Partial<IMaterialItem>, string][] = [
            [{ itemType: "EXP_PLAYER" }, "exp"],
            [{ itemType: "GOLD" }, "lmd"],
            [{ itemType: "DIAMOND" }, "lmd"],
            [{ itemType: "DIAMOND_SHD" }, "lmd"],
            [{ itemType: "ACTIVITY_COIN" }, "lmd"],
            [{ itemType: "HGG_SHD" }, "ticket"],
            [{ itemType: "LGG_SHD" }, "ticket"],
            [{ itemType: "TKT_GACHA10" }, "ticket"],
            [{ itemType: "VOUCHER_PICK" }, "ticket"],
            [{ itemType: "AP_GAMEPLAY" }, "ticket"],
            [{ itemType: "AP_ITEM" }, "ticket"],
            [{ itemType: "MATERIAL_ISSUE_VOUCHER" }, "ticket"],
            [{ itemType: "MATERIAL" }, "mat"],
            [{ itemType: "SOCIAL_PT" }, "other"],
            [{ itemType: "RENAMING_CARD", classifyType: "NORMAL" }, "other"],
        ];
        for (const [meta, category] of cases) {
            expect(categorizeItem(item(meta), NO_EXP), meta.itemType).toBe(category);
        }
    });

    it("files Battle Records under exp", () => {
        // CURRENT BEHAVIOR, suspected bug: "CARD_EXP" is also listed in the lmd
        // branch, but `includes("EXP")` claims it first, so that part of the lmd
        // check is dead code.
        expect(categorizeItem(item({ itemType: "CARD_EXP" }), NO_EXP)).toBe("exp");
    });

    it("files skill summaries as plain materials", () => {
        // CURRENT BEHAVIOR, suspected bug: skill summaries ship as itemType
        // "MATERIAL", so `includes("MATERIAL")` returns "mat" before the
        // classifyType MATERIAL branch can read the "skill" icon id. The skill,
        // module and chip buckets are reachable only for a MATERIAL-class item
        // whose itemType does not mention MATERIAL.
        expect(categorizeItem(item({ itemId: "3303", itemType: "MATERIAL", classifyType: "MATERIAL", iconId: "MTL_SKILL3" }), NO_EXP)).toBe("mat");
        expect(categorizeItem(item({ itemType: "UNKNOWN", classifyType: "MATERIAL", iconId: "MTL_SKILL3" }), NO_EXP)).toBe("skill");
        expect(categorizeItem(item({ itemType: "UNKNOWN", classifyType: "MATERIAL", iconId: "mod_unlock_token" }), NO_EXP)).toBe("module");
        expect(categorizeItem(item({ itemType: "UNKNOWN", classifyType: "MATERIAL", iconId: "MTL_CHIP_A" }), NO_EXP)).toBe("chip");
        expect(categorizeItem(item({ itemType: "UNKNOWN", classifyType: "MATERIAL", iconId: "MTL_SL_G2" }), NO_EXP)).toBe("mat");
    });
});

describe("category tables", () => {
    it("cover every category in display order", () => {
        expect(CATEGORY_ORDER).toEqual(["all", "exp", "lmd", "mat", "skill", "module", "chip", "furniture", "ticket", "consume", "other"]);
        expect(Object.keys(CATEGORY_LABELS).sort()).toEqual([...CATEGORY_ORDER].sort());
        expect(Object.keys(CATEGORY_ITEM_LABELS).sort()).toEqual([...CATEGORY_ORDER].sort());
    });
});
