import { ModulesSkinsCard } from "frontend";

interface IGapOp {
    id: string;
    operatorId: string;
    name: string;
    charId: string;
    rarity: number;
    sub?: string;
}

const gapOp = (charId: string, name: string, rarity: number, sub: string): IGapOp => ({ id: `${charId}-${sub}`, operatorId: charId, charId, name, rarity, sub });

const LOCKED = [gapOp("char_4087_ines", "Ines", 6, "ARC-Y"), gapOp("char_1013_chen2", "Ch'en the Holungday", 6, "SWD-Y"), gapOp("char_017_huang", "Blaze", 5, "CEN-X"), gapOp("char_4045_heidi", "Heidi", 5, "MSC-X")];

const BELOW_MAX = [gapOp("char_263_skadi", "Skadi", 6, "Lv 1 / 3"), gapOp("char_102_texas", "Texas", 5, "Lv 2 / 3"), gapOp("char_350_surtr", "Surtr", 6, "Lv 2 / 3")];

// A slice of the /skins/index projection — the shape the Stats tab passes down.
const skin = (skinId: string, charId: string, skinName: string, skinGroupName: string, skinGroupId: string, skinGroupSortIndex: number, displayTagId: string | null, getTime: number, sortId: number) => ({
    skinId,
    charId,
    displaySkin: { skinName, skinGroupId, skinGroupName, skinGroupSortIndex, displayTagId, getTime, sortId, description: null, content: "", dialog: null, usage: null, obtainApproach: null, designerList: null, drawerList: [] },
});

const SKIN_LIST = [
    skin("char_4064_mlynar@epoque#28", "char_4064_mlynar", "W Dali", "EPOQUE/XXVIII", "2023#epoque#10", 48, null, 1714496400, 93),
    skin("char_263_skadi@marthe#5", "char_263_skadi", "The Next Afternoon Tea", "MARTHE/V", "2023#marthe", 130, null, 1701190800, 237),
    skin("char_102_texas@winter#1", "char_102_texas", "Winter Messenger", "Cambrian/I", "2018#winter", 143, null, 1580943600, 256),
    skin("char_180_amgoat@sanrio#2", "char_180_amgoat", "Fluffy Little Witch", "Sanrio characters/II", "2025#sanrio", 227, "From collabs", 1772809200, 403),
    skin("char_1028_texas2@iteration#1", "char_1028_texas2", "Wingbreaker", "Iteration Provident", "2023#iteration", 216, null, 1714496400, 450),
    skin("char_4045_heidi@epoque#24", "char_4045_heidi", "A Thousand Correspondences", "EPOQUE/XXIV", "2023#epoque#6", 44, null, 1698148800, 87),
    skin("char_002_amiya@epoque#4", "char_002_amiya", "Fresh Fastener", "Test Collection/II", "2020#sale", 3, "Event Reward", 1609585200, 4),
    skin("char_293_thorns@boc#8", "char_293_thorns", "Blade-cleaved Tides", "Bloodline of Combat/VIII", "2024#boc", 177, null, 1730394000, 326),
    skin("char_350_surtr@summer#9", "char_350_surtr", "Colorful Wonderland CW03", "Coral Coast/IX", "2022#summer#2", 113, null, 1673611200, 206),
    skin("char_4087_ines@boc#8", "char_4087_ines", "Under the Flaming Dome", "Bloodline of Combat/VIII", "2024#boc", 177, null, 1730394000, 325),
    skin("char_003_kalts@boc#6", "char_003_kalts", "Remnant", "Bloodline of Combat/VI", "2023#boc", 175, null, 1699376400, 318),
    skin("char_017_huang@witch#5", "char_017_huang", "Explosive Blue Flame", "Witch Feast/V", "2024#witch", 142, null, 1744113600, 447),
];

const CHAR_SKINS = Object.fromEntries(SKIN_LIST.map((s) => [s.skinId, s]));

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Mlynar", rarity: "TIER_6", profession: "WARRIOR" },
    { id: "char_263_skadi", name: "Skadi", rarity: "TIER_6", profession: "WARRIOR" },
    { id: "char_102_texas", name: "Texas", rarity: "TIER_5", profession: "PIONEER" },
    { id: "char_180_amgoat", name: "Nightingale", rarity: "TIER_6", profession: "MEDIC" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", rarity: "TIER_6", profession: "SPECIAL" },
    { id: "char_4045_heidi", name: "Heidi", rarity: "TIER_5", profession: "SUPPORT" },
    { id: "char_002_amiya", name: "Amiya", rarity: "TIER_5", profession: "CASTER" },
    { id: "char_293_thorns", name: "Thorns", rarity: "TIER_6", profession: "WARRIOR" },
    { id: "char_350_surtr", name: "Surtr", rarity: "TIER_6", profession: "WARRIOR" },
    { id: "char_4087_ines", name: "Ines", rarity: "TIER_6", profession: "PIONEER" },
    { id: "char_003_kalts", name: "Kal'tsit", rarity: "TIER_6", profession: "MEDIC" },
    { id: "char_017_huang", name: "Blaze", rarity: "TIER_5", profession: "WARRIOR" },
];

const OWNED = new Set(["char_263_skadi@marthe#5", "char_102_texas@winter#1", "char_293_thorns@boc#8", "char_350_surtr@summer#9", "char_017_huang@witch#5"]);

export const Veteran = () => (
    <div className="w-full max-w-md">
        <ModulesSkinsCard
            charSkins={CHAR_SKINS}
            modules={{ unlocked: 117, atMax: 94, totalAvailable: 211, details: { locked: LOCKED, belowMax: BELOW_MAX } }}
            operatorsStatic={OPERATORS}
            ownedSkinIds={OWNED}
            skins={{ totalOwned: 63, totalAvailable: 472, percentage: 13.3 }}
        />
    </div>
);

export const EarlyAccount = () => (
    <div className="w-full max-w-md">
        <ModulesSkinsCard
            charSkins={CHAR_SKINS}
            modules={{ unlocked: 6, atMax: 1, totalAvailable: 38, details: { locked: LOCKED, belowMax: BELOW_MAX } }}
            operatorsStatic={OPERATORS}
            ownedSkinIds={new Set()}
            skins={{ totalOwned: 2, totalAvailable: 472, percentage: 0.4 }}
        />
    </div>
);

export const ModulesMaxed = () => (
    <div className="w-full max-w-md">
        <ModulesSkinsCard
            charSkins={CHAR_SKINS}
            modules={{ unlocked: 211, atMax: 211, totalAvailable: 211, details: { locked: [], belowMax: [] } }}
            operatorsStatic={OPERATORS}
            ownedSkinIds={OWNED}
            skins={{ totalOwned: 472, totalAvailable: 472, percentage: 100 }}
        />
    </div>
);
