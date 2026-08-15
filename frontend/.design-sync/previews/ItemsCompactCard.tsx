import { ItemsCompactCard } from "frontend";

const entry = (item_id: string, name: string, rarityNum: number, iconId: string, quantity: number, category = "mat") => ({
    item_id,
    quantity,
    name,
    rarityNum,
    category,
    iconId,
    expValue: null,
    meta: null,
});

const INVENTORY = [
    entry("30012", "Orirock Cube", 2, "MTL_SL_G2", 1284),
    entry("30013", "Orirock Cluster", 3, "MTL_SL_G3", 412),
    entry("31013", "Coagulating Gel", 3, "MTL_SL_PGEL3", 96),
    entry("31033", "Crystalline Component", 3, "MTL_SL_OC3", 141),
    entry("30104", "RMA70-24", 4, "MTL_SL_RMA7024", 33),
    entry("30084", "Manganese Trihydrate", 4, "MTL_SL_MANGANESE2", 74),
    entry("30115", "Polymerization Preparation", 5, "MTL_SL_PP", 18),
    entry("30125", "Bipolar Nanoflake", 5, "MTL_SL_BN", 6),
    entry("30145", "Crystalline Electronic Unit", 5, "MTL_SL_OEU", 4),
    entry("2004", "Strategic Battle Record", 5, "sprite_exp_card_t4", 231, "exp"),
    entry("4001", "LMD", 4, "GOLD", 4318220, "lmd"),
    entry("mod_unlock_token", "Module Data Block", 5, "mod_unlock_token", 3, "module"),
];

export const InventoryGrid = () => (
    <div className="grid w-full max-w-3xl grid-cols-6 gap-2.5 p-2">
        {INVENTORY.map((item) => (
            <ItemsCompactCard item={item} key={item.item_id} />
        ))}
    </div>
);

export const RarityStrip = () => (
    <div className="grid w-full max-w-md grid-cols-4 gap-2.5 p-2">
        {[INVENTORY[0], INVENTORY[1], INVENTORY[4], INVENTORY[6]].map((item) => (
            <ItemsCompactCard item={item} key={item.item_id} />
        ))}
    </div>
);

export const LargeQuantities = () => (
    <div className="grid w-full max-w-xs grid-cols-3 gap-2.5 p-2">
        {[INVENTORY[10], INVENTORY[9], INVENTORY[0]].map((item) => (
            <ItemsCompactCard item={item} key={item.item_id} />
        ))}
    </div>
);
