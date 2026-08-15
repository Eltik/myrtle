import { TierListBoard } from "frontend";

const op = (id: string, name: string, rarity: number, profession: string, sub: string, position: string, nationId: string | null, description: string | null = null) => ({
    id,
    name,
    appellation: null,
    rarity,
    profession,
    subProfessionId: sub,
    position,
    nationId,
    subOrder: 0,
    description,
    updatedAt: "2024-05-12T14:05:00.000Z",
});

const TIERS = [
    {
        id: "tier-s-plus",
        name: "S+",
        displayOrder: 0,
        color: "#dc4d56",
        description: "Solves a whole risk category on their own. Bring them unless the stage locks them out.",
        operators: [
            op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null, "Deletes the Ritualist wave from off-screen. S3 only; S2 is a trap at this risk level."),
            op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
            op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
            op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
            op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
        ],
    },
    {
        id: "tier-s",
        name: "S",
        displayOrder: 1,
        color: "#e0834a",
        description: "Still stage-defining, but they need a second slot to fully pay off.",
        operators: [
            op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"),
            op("char_1032_excu2", "Executor the Ex Foedere", 6, "WARRIOR", "reaper", "MELEE", "laterano"),
            op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"),
            op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa"),
            op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "leithanien"),
            op("char_4039_horn", "Horn", 6, "TANK", "fortress", "MELEE", "victoria"),
            op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer", "RANGED", null),
        ],
    },
    {
        id: "tier-a",
        name: "A",
        displayOrder: 2,
        color: "#d8b54a",
        description: "Comfortable picks that clear without carrying.",
        operators: [
            op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "victoria"),
            op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "lungmen"),
            op("char_1020_reed2", "Reed the Flame Shadow", 6, "MEDIC", "incantationmedic", "RANGED", "victoria"),
            op("char_377_gdglow", "Goldenglow", 6, "CASTER", "funnel", "RANGED", "victoria"),
            op("char_391_rosmon", "Rosmontis", 6, "SNIPER", "bombarder", "RANGED", "rhodes"),
            op("char_245_cello", "Virtuosa", 6, "SUPPORT", "ritualist", "RANGED", "laterano"),
        ],
    },
    {
        id: "tier-b",
        name: "B",
        displayOrder: 3,
        color: "#5dbf86",
        description: "Budget answers. Worth raising if the S tier is out of reach.",
        operators: [
            op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"),
            op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "siracusa"),
            op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"),
            op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"),
            op("char_298_susuro", "Sussurro", 4, "MEDIC", "physician", "RANGED", "rhodes"),
            op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "rhodes"),
        ],
    },
];

const detail = {
    id: "tl-global-meta",
    slug: "global-6-star-meta",
    title: "Global 6★ meta — May 2024",
    description: "",
    listType: "official" as const,
    createdBy: "u-myrtle",
    isListed: true,
    flair: null,
    author: { id: "u-myrtle", uid: "myrtle", nickname: "myrtle.moe", avatarId: "char_151_myrtle" },
    stats: null,
    tiers: TIERS,
    createdAt: "2024-01-18T10:00:00.000Z",
    updatedAt: "2024-05-14T08:30:00.000Z",
};

export const MetaBoard = () => <TierListBoard detail={detail} />;

export const WithEmptyTier = () => <TierListBoard detail={{ ...detail, tiers: [TIERS[0], TIERS[1], { id: "tier-c", name: "C", displayOrder: 2, color: "#8a8a8a", description: "Nothing has fallen this far yet.", operators: [] }] }} />;

export const NoTiersYet = () => <TierListBoard detail={{ ...detail, tiers: [] }} />;
