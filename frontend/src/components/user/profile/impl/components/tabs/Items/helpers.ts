import type { IMaterialItem } from "#/lib/api/materials";
import type { ItemCategory } from "./types";

export { RARITY_HEX_MUTED as RARITY_COLORS } from "#/lib/utils";

export const CATEGORY_LABELS: Record<ItemCategory, string> = {
    all: "All",
    exp: "EXP",
    lmd: "Currency",
    mat: "Materials",
    skill: "Skill Books",
    module: "Module Mats",
    chip: "Chips",
    furniture: "Furniture",
    ticket: "Tickets",
    consume: "Consumables",
    other: "Other",
};

export const CATEGORY_ITEM_LABELS: Record<ItemCategory, string> = {
    all: "Item",
    exp: "Battle Record",
    lmd: "Currency",
    mat: "Material",
    skill: "Skill book",
    module: "Module mat",
    chip: "Chip",
    furniture: "Furniture",
    ticket: "Ticket",
    consume: "Consumable",
    other: "Item",
};

export const CATEGORY_ORDER: ItemCategory[] = ["all", "exp", "lmd", "mat", "skill", "module", "chip", "furniture", "ticket", "consume", "other"];

export function rarityTierToNumber(tier: string | null | undefined): number {
    if (!tier) return 1;
    const n = Number(String(tier).replace("TIER_", ""));
    return n >= 1 && n <= 6 ? n : 1;
}

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
