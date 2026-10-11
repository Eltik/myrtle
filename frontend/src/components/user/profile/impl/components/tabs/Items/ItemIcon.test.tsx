import { cleanup, fireEvent, render, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { IMaterialItem, IMaterials } from "#/lib/api/materials";
import { ItemIcon, itemIconURL } from "./ItemIcon";
import type { IItemEntry } from "./types";
import { useItems } from "./useItems";

afterEach(cleanup);

function entry(iconId: string | null): IItemEntry {
    return { item_id: "30043", quantity: 12, meta: null, name: "Oriron Cluster", rarityNum: 3, category: "mat", iconId, expValue: null };
}

const MATERIALS = {
    items: { "30043": { itemId: "30043", iconId: "MTL_SL_IRON3", name: "Oriron Cluster", rarity: "TIER_3" } as IMaterialItem },
    expItems: {},
    potentialItems: {},
    apSupplies: {},
    charVoucherItems: {},
} as IMaterials;

const INVENTORY = [
    { item_id: "30043", quantity: 12 },
    { item_id: "act12side_token_key01", quantity: 1 },
];

describe("ItemIcon", () => {
    it("requests nothing while the icon is unknown", () => {
        const { container } = render(<ItemIcon item={entry(null)} />);
        expect(container.querySelector("img")).toBeNull();
        expect(container.textContent).toBe("OC");
        expect(itemIconURL(null)).toBeNull();
    });

    it("tries a new icon after an earlier one failed", () => {
        const { container, rerender } = render(<ItemIcon item={entry("30043")} />);
        const first = container.querySelector("img");
        expect(first?.getAttribute("src")).toMatch(/\/api\/item-icon\/30043$/);
        if (first) fireEvent.error(first);
        expect(container.querySelector("img")).toBeNull();
        expect(container.textContent).toBe("OC");

        rerender(<ItemIcon item={entry("MTL_SL_IRON3")} />);
        expect(container.querySelector("img")?.getAttribute("src")).toMatch(/\/api\/item-icon\/MTL_SL_IRON3$/);
    });
});

describe("useItems icon ids", () => {
    it("holds icons back while the item table loads", () => {
        const { result } = renderHook(() => useItems(INVENTORY, undefined, true));
        expect(result.current.entries.map((e) => e.iconId)).toEqual([null, null]);
    });

    it("uses the table's iconId, and the inventory id only for items the table lacks", () => {
        const { result } = renderHook(() => useItems(INVENTORY, MATERIALS, false));
        expect(result.current.entries.map((e) => e.iconId)).toEqual(["MTL_SL_IRON3", "act12side_token_key01"]);
    });

    it("falls back to the inventory id when the table failed to load", () => {
        const { result } = renderHook(() => useItems(INVENTORY, undefined, false));
        expect(result.current.entries.map((e) => e.iconId)).toEqual(["30043", "act12side_token_key01"]);
    });
});
