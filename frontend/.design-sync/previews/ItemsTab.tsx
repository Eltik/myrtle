import { ItemsTab } from "frontend";

// Real inventory rows as `/inventory?uid=…` returns them: an item id and a
// count. The tab joins these against the material catalog it fetches itself.
const inventory = [
    { item_id: "30012", quantity: 428 },
    { item_id: "30013", quantity: 164 },
    { item_id: "30014", quantity: 37 },
    { item_id: "30062", quantity: 91 },
    { item_id: "30073", quantity: 58 },
    { item_id: "31013", quantity: 24 },
    { item_id: "3303", quantity: 112 },
    { item_id: "2004", quantity: 96 },
    { item_id: "4001", quantity: 3_284_500 },
    { item_id: "mod_unlock_token", quantity: 6 },
];

// The tab fetches the material catalog itself (`materialsQueryOptions`), and
// that server function is stubbed in previews — so this is the real
// catalog-pending branch: every row falls back to its item id for a name, the
// icon slot stays empty and everything lands in the "Other" category. The
// quantities, the chip row, the filters and the grid density are all live.
export const CatalogPending = () => <ItemsTab inventory={inventory} />;

// A Doctor who has synced their account but carries nothing yet — the chip row,
// filters and view toggle stay, and the grid falls back to its empty card.
export const EmptyInventory = () => <ItemsTab inventory={[]} />;
