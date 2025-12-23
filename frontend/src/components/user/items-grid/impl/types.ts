import type { InventoryItem } from "~/types/api/impl/user";

export interface ItemWithData extends InventoryItem {
    id: string;
    displayAmount: number;
}
