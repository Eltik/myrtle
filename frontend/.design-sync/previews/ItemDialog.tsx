import { Dialog, ItemDialog } from "frontend";
import type { ReactNode } from "react";

// The popup portals into a `fixed inset-0` viewport, so every open story needs a
// stage tall enough for the backdrop to fill. Opening/closing is interaction —
// these are the resting open states.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

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
        stageDropList: stages(["main_01-07", "main_04-04", "main_05-10"]),
        buildingProductList: [{ roomType: "WORKSHOP", formulaId: "35" }],
        voucherRelateList: [
            { voucherId: "advanced_material_issue_voucher", voucherItemType: "MATERIAL_ISSUE_VOUCHER" },
            { voucherId: "advanced_material_voucher_perm", voucherItemType: "MATERIAL_ISSUE_VOUCHER" },
        ],
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
        description: "In such a hostile environment, every operation has the potential for a loss of life. We may be able to save more people if we are well prepared.",
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

const MODULE_TOKEN = {
    item_id: "mod_unlock_token",
    quantity: 3,
    name: "Module Data Block",
    rarityNum: 5,
    category: "module",
    iconId: "mod_unlock_token",
    expValue: null,
    meta: {
        itemId: "mod_unlock_token",
        name: "Module Data Block",
        description: "The data blocks record the endless nights the Engineering Department toiled away developing the various modules.",
        rarity: "TIER_5",
        iconId: "mod_unlock_token",
        overrideBkg: null,
        stackIconId: null,
        sortId: 8000,
        usage: "Used to unlock Operator modules.",
        obtainApproach: "Store/Mission Reward/Event Reward",
        hideInItemGet: false,
        classifyType: "MATERIAL",
        itemType: "MATERIAL",
        stageDropList: [],
        buildingProductList: [],
        voucherRelateList: null,
    },
};

export const MaterialDetails = () => (
    <Stage>
        <Dialog open>
            <ItemDialog item={ORIROCK_CLUSTER} />
        </Dialog>
    </Stage>
);

export const BattleRecord = () => (
    <Stage>
        <Dialog open>
            <ItemDialog item={BATTLE_RECORD} />
        </Dialog>
    </Stage>
);

export const NoRecipeOrVouchers = () => (
    <Stage>
        <Dialog open>
            <ItemDialog item={MODULE_TOKEN} />
        </Dialog>
    </Stage>
);
