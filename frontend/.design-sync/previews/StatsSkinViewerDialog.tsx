import { Dialog, StatsSkinViewerDialog } from "frontend";

const SKIN_ACCENT = "var(--primary)";

const skin = (skinId: string, charId: string, skinName: string, skinGroupName: string, skinGroupId: string, skinGroupSortIndex: number, displayTagId: string | null, getTime: number, sortId: number) => ({
    skinId,
    charId,
    displaySkin: { skinName, skinGroupId, skinGroupName, skinGroupSortIndex, displayTagId, getTime, sortId, description: null, content: "", dialog: null, usage: null, obtainApproach: null, designerList: null, drawerList: [] },
});

// Two brand sections (the dialog folds "EPOQUE/XXVIII" and "EPOQUE/VII" into
// one "EPOQUE" section) plus a collab section.
const SKINS = [
    skin("char_4064_mlynar@epoque#28", "char_4064_mlynar", "W Dali", "EPOQUE/XXVIII", "2023#epoque#10", 48, null, 1714496400, 93),
    skin("char_102_texas@epoque#7", "char_102_texas", "Willpower", "EPOQUE/VII", "2020#epoque#6", 27, null, 1588269600, 38),
    skin("char_1028_texas2@epoque#36", "char_1028_texas2", "Il Segreto Della Notte", "EPOQUE/XXXVI", "2024#epoque#7", 56, null, 1745514000, 113),
    skin("char_4045_heidi@epoque#24", "char_4045_heidi", "A Thousand Correspondences", "EPOQUE/XXIV", "2023#epoque#6", 44, null, 1698148800, 87),
    skin("char_263_skadi@summer#3", "char_263_skadi", "Waverider WR04", "Coral Coast/III", "2020#summer#2", 107, null, 1611176400, 184),
    skin("char_180_amgoat@summer#5", "char_180_amgoat", "Summer Flowers FA018", "Coral Coast/V", "2021#summer", 109, null, 1643994000, 194),
    skin("char_350_surtr@summer#9", "char_350_surtr", "Colorful Wonderland CW03", "Coral Coast/IX", "2022#summer#2", 113, null, 1673611200, 206),
    skin("char_1013_chen2@summer#20", "char_1013_chen2", "Holiday HD79", "Coral Coast/XX", "2025#summer#2", 124, null, 1768575600, 229),
    skin("char_180_amgoat@sanrio#2", "char_180_amgoat", "Fluffy Little Witch", "Sanrio characters/II", "2025#sanrio", 227, "From collabs", 1772809200, 403),
    skin("char_293_thorns@it#1", "char_293_thorns", "Cómodo", "i.t", "2021#it", 203, "From collabs", 1639051200, 394),
    skin("char_350_surtr@it#1", "char_350_surtr", "Liberté//Échec", "i.t", "2021#it", 203, "From collabs", 1639051200, 395),
    skin("char_002_amiya@epoque#4", "char_002_amiya", "Fresh Fastener", "Test Collection/II", "2020#sale", 3, "Event Reward", 1609585200, 4),
    skin("char_003_kalts@boc#6", "char_003_kalts", "Remnant", "Bloodline of Combat/VI", "2023#boc", 175, null, 1699376400, 318),
    skin("char_017_huang@witch#5", "char_017_huang", "Explosive Blue Flame", "Witch Feast/V", "2024#witch", 142, null, 1744113600, 447),
];

const OPERATORS_MAP = new Map(
    [
        ["char_4064_mlynar", "Mlynar"],
        ["char_102_texas", "Texas"],
        ["char_1028_texas2", "Texas the Omertosa"],
        ["char_4045_heidi", "Heidi"],
        ["char_263_skadi", "Skadi"],
        ["char_180_amgoat", "Nightingale"],
        ["char_350_surtr", "Surtr"],
        ["char_1013_chen2", "Ch'en the Holungday"],
        ["char_293_thorns", "Thorns"],
        ["char_002_amiya", "Amiya"],
        ["char_003_kalts", "Kal'tsit"],
        ["char_017_huang", "Blaze"],
    ].map(([id, name]) => [id, { id, name, rarity: "TIER_6", profession: "WARRIOR" }]),
);

// Ownership sets are chosen so the *missing* set (the dialog's default filter)
// stays inside the viewport: off-screen `loading="lazy"` skin art stalls the
// capture harness's settle().
const OWNED_OUTSIDE_STORE_BRANDS = new Set(["char_180_amgoat@sanrio#2", "char_293_thorns@it#1", "char_350_surtr@it#1", "char_002_amiya@epoque#4", "char_003_kalts@boc#6", "char_017_huang@witch#5"]);

const OWNED_ALL = new Set(SKINS.map((s) => s.skinId));

const OWNED_ALL_BUT_TWO = new Set(SKINS.filter((s) => s.skinId !== "char_4064_mlynar@epoque#28" && s.skinId !== "char_1028_texas2@epoque#36").map((s) => s.skinId));

export const MissingSkins = () => (
    <div className="min-h-[520px]">
        <Dialog open>
            <StatsSkinViewerDialog color={SKIN_ACCENT} operatorsMap={OPERATORS_MAP} ownedIds={OWNED_OUTSIDE_STORE_BRANDS} profileOwnedCount={6} skins={SKINS} />
        </Dialog>
    </div>
);

export const NearlyComplete = () => (
    <div className="min-h-[520px]">
        <Dialog open>
            <StatsSkinViewerDialog color={SKIN_ACCENT} operatorsMap={OPERATORS_MAP} ownedIds={OWNED_ALL_BUT_TWO} profileOwnedCount={12} skins={SKINS} />
        </Dialog>
    </div>
);

export const NothingMissing = () => (
    <div className="min-h-[520px]">
        <Dialog open>
            <StatsSkinViewerDialog color={SKIN_ACCENT} operatorsMap={OPERATORS_MAP} ownedIds={OWNED_ALL} profileOwnedCount={14} skins={SKINS} />
        </Dialog>
    </div>
);
