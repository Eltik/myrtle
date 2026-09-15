import type { IMaterialItem } from "#/lib/api/materials";
import type { messages as helperMessages } from "./helpers.messages";
import type { ItemCategory } from "./types";

export { RARITY_HEX_MUTED as RARITY_COLORS } from "#/lib/utils";

/** A key in `helpers.messages.ts`; resolved by whichever card renders it. */
export type ItemMessageKey = keyof typeof helperMessages & string;

/** Category chip names. */
export const CATEGORY_LABELS: Record<ItemCategory, ItemMessageKey> = {
    all: "profile.items.category.all",
    exp: "profile.items.category.exp",
    lmd: "profile.items.category.lmd",
    mat: "profile.items.category.mat",
    skill: "profile.items.category.skill",
    module: "profile.items.category.module",
    chip: "profile.items.category.chip",
    furniture: "profile.items.category.furniture",
    ticket: "profile.items.category.ticket",
    consume: "profile.items.category.consume",
    other: "profile.items.category.other",
};

/** What a single entry of each category is called on its card. */
export const CATEGORY_ITEM_LABELS: Record<ItemCategory, ItemMessageKey> = {
    all: "profile.items.itemLabel.all",
    exp: "profile.items.itemLabel.exp",
    lmd: "profile.items.itemLabel.lmd",
    mat: "profile.items.itemLabel.mat",
    skill: "profile.items.itemLabel.skill",
    module: "profile.items.itemLabel.module",
    chip: "profile.items.itemLabel.chip",
    furniture: "profile.items.itemLabel.furniture",
    ticket: "profile.items.itemLabel.ticket",
    consume: "profile.items.itemLabel.consume",
    other: "profile.items.itemLabel.other",
};

export const CATEGORY_ORDER: ItemCategory[] = ["all", "exp", "lmd", "mat", "skill", "module", "chip", "furniture", "ticket", "consume", "other"];

export function rarityTierToNumber(tier: string | null | undefined): number {
    if (!tier) return 1;
    const n = Number(String(tier).replace("TIER_", ""));
    return n >= 1 && n <= 6 ? n : 1;
}

// `formatItemType` and `formatVoucherId` title-case a raw API code
// (`SKILL_SUMMARY` -> `Skill Summary`), so what they return is the code itself,
// not prose - which is why neither goes through the catalog.
export function formatItemType(itemType: string | null | undefined): string {
    if (!itemType || itemType === "UNKNOWN" || itemType === "NONE") return "Item";
    return itemType
        .toLowerCase()
        .split("_")
        .map((w) => (w.length > 0 ? w[0].toUpperCase() + w.slice(1) : w))
        .join(" ");
}

export function formatVoucherId(voucherId: string): string {
    return voucherId
        .replace(/_perm$/, " (Permanent)")
        .replace(/_/g, " ")
        .replace(/\b\w/g, (c) => c.toUpperCase());
}

export function categorizeItem(meta: IMaterialItem | null, expIds: Set<string>): ItemCategory {
    if (!meta) return "other";
    if (expIds.has(meta.itemId)) return "exp";
    if (meta.classifyType === "CONSUME") return "consume";
    const t = (meta.itemType || "").toUpperCase();
    if (t.includes("EXP")) return "exp";
    if (t === "GOLD" || t === "DIAMOND" || t === "DIAMOND_SHD" || t === "LMD" || t === "ETH" || t === "EPGS" || t === "VOUCHER_PRTS" || t === "CARD_EXP" || t === "ACTIVITY_COIN") return "lmd";
    if (t.includes("MOD_")) return "module";
    if (t.includes("CHIP") || t.includes("CLASS_TOKEN")) return "chip";
    if (t.includes("FURN")) return "furniture";
    if (t.includes("HGG") || t.includes("LGG") || t.includes("TKT") || t.includes("VOUCHER") || t.includes("HG_") || t.includes("AP_GAMEPLAY") || t.includes("AP_ITEM") || t.includes("AP_BASE")) return "ticket";
    if (t.includes("MATERIAL")) return "mat";
    if (meta.classifyType === "MATERIAL") {
        if (meta.iconId?.toLowerCase().includes("skill")) return "skill";
        if (meta.iconId?.toLowerCase().includes("mod_")) return "module";
        if (meta.iconId?.toLowerCase().includes("chip")) return "chip";
        return "mat";
    }
    return "other";
}
