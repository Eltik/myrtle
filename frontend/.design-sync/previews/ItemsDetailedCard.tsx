import { ItemsDetailedCard } from "frontend";

const stages = (ids: string[]) => ids.map((stageId) => ({ stageId, occPer: "ALWAYS" }));

const ORIROCK_CLUSTER = {
    item_id: "30013",
    quantity: 412,
    name: "Orirock Cluster",
    rarityNum: 3,
    category: "mat",
    iconId: "MTL_SL_G3",
    expValue: null,
    meta: {
        itemId: "30013",
        name: "Orirock Cluster",
        description: "Compressed from Orirock Cubes, it can be formed naturally. It is a fragile material, but industrial technology has made it possible to produce it at scale.",
        rarity: "TIER_3",
        iconId: "MTL_SL_G3",
        overrideBkg: null,
        stackIconId: null,
        sortId: 30013,
        usage: "A conglomerate rock mined from the ground. Can be used for a variety of upgrades and for production in the Factory.",
        obtainApproach: "Stage Drop/Base Production/Store",
        hideInItemGet: false,
        classifyType: "MATERIAL",
        itemType: "MATERIAL",
        stageDropList: stages(["main_01-07", "main_04-04", "main_05-10", "sub_02-01", "main_08-16"]),
        buildingProductList: [{ roomType: "WORKSHOP", formulaId: "35" }],
        voucherRelateList: null,
    },
};

const POLYMER = {
    item_id: "30115",
    quantity: 18,
    name: "Polymerization Preparation",
    rarityNum: 5,
    category: "mat",
    iconId: "MTL_SL_PP",
    expValue: null,
    meta: {
        itemId: "30115",
        name: "Polymerization Preparation",
        description: "A material commonly used as an isolation coating for delicate equipment.",
        rarity: "TIER_5",
        iconId: "MTL_SL_PP",
        overrideBkg: null,
        stackIconId: null,
        sortId: 30115,
        usage: "A complicated liquid industrial product. Can be used for high level upgrades.",
        obtainApproach: "Workshop Synthesis/Event Reward",
        hideInItemGet: false,
        classifyType: "MATERIAL",
        itemType: "MATERIAL",
        stageDropList: [],
        buildingProductList: [{ roomType: "WORKSHOP", formulaId: "14" }],
        voucherRelateList: [{ voucherId: "precious_material_voucher_perm", voucherItemType: "MATERIAL_ISSUE_VOUCHER" }],
    },
};

const BATTLE_RECORD = {
    item_id: "2004",
    quantity: 231,
    name: "Strategic Battle Record",
    rarityNum: 5,
    category: "exp",
    iconId: "sprite_exp_card_t4",
    expValue: 1000,
    meta: {
        itemId: "2004",
        name: "Strategic Battle Record",
        description: "In such a hostile environment, every operation has the potential for a loss of life.",
        rarity: "TIER_5",
        iconId: "sprite_exp_card_t4",
        overrideBkg: null,
        stackIconId: null,
        sortId: 2004,
        usage: "A device that stores battle videos. Gives Operators a huge amount of EXP.",
        obtainApproach: "Stage Drop/Annihilation",
        hideInItemGet: false,
        classifyType: "MATERIAL",
        itemType: "CARD_EXP",
        stageDropList: stages(["main_02-07"]),
        buildingProductList: [],
        voucherRelateList: null,
    },
};

const LMD = {
    item_id: "4001",
    quantity: 4318220,
    name: "LMD",
    rarityNum: 4,
    category: "lmd",
    iconId: "GOLD",
    expValue: null,
    meta: {
        itemId: "4001",
        name: "LMD",
        description: "Since the financial crisis, economic decline and political instability have stood in the way of trade.",
        rarity: "TIER_4",
        iconId: "GOLD",
        overrideBkg: null,
        stackIconId: null,
        sortId: 4001,
        usage: "A widely used currency issued by Lungmen.",
        obtainApproach: "Stage Drop/Trading Post/Mission Reward",
        hideInItemGet: false,
        classifyType: "NORMAL",
        itemType: "GOLD",
        stageDropList: stages(["main_01-07", "sub_02-02", "wk_melee_5", "main_10-08"]),
        buildingProductList: [],
        voucherRelateList: null,
    },
};

export const MaterialCard = () => (
    <div className="w-full max-w-sm">
        <ItemsDetailedCard item={ORIROCK_CLUSTER} />
    </div>
);

export const InventoryGrid = () => (
    <div className="grid w-full max-w-4xl grid-cols-3 gap-4">
        <ItemsDetailedCard item={ORIROCK_CLUSTER} />
        <ItemsDetailedCard item={POLYMER} />
        <ItemsDetailedCard item={BATTLE_RECORD} />
    </div>
);

export const CurrencyCard = () => (
    <div className="w-full max-w-sm">
        <ItemsDetailedCard item={LMD} />
    </div>
);
