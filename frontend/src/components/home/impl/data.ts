export const ROLE_CLS: Record<string, string> = {
    Vanguard: "c-vanguard",
    Supporter: "c-support",
    Defender: "c-defender",
    Sniper: "c-sniper",
    Caster: "c-caster",
    Medic: "c-medic",
    Guard: "c-guard",
    Specialist: "c-special",
};

export interface Operator {
    id: string;
    name: string;
    rarity: number;
    role: string;
    arch: string;
    e?: number;
    lvl?: number;
    hp?: number;
    atk?: number;
    def?: number;
    trust?: number;
    owned?: boolean;
}

export interface TierList {
    id: string;
    title: string;
    tag: string;
    stage: string;
    author: { name: string; initials: string };
    updated: string;
    votes: number;
    comments: number;
    hot?: boolean;
    accent: string;
    tiers: { S: string[]; A: string[]; B: string[] };
}

export const OPERATORS: Operator[] = [
    { id: "char_1028_texas2", name: "Texas Alter", rarity: 6, role: "Guard", arch: "Reaper" },
    { id: "char_311_mudrok", name: "Mudrock", rarity: 6, role: "Defender", arch: "Arts Protector" },
    { id: "char_358_lisa", name: "Suzuran", rarity: 6, role: "Supporter", arch: "Hexer" },
    { id: "char_350_surtr", name: "Surtr", rarity: 6, role: "Guard", arch: "Dreadnought" },
    { id: "char_103_angel", name: "Exusiai", rarity: 6, role: "Sniper", arch: "Marksman" },
    { id: "char_180_amgoat", name: "Eyjafjalla", rarity: 6, role: "Caster", arch: "Core Caster" },
    { id: "char_202_demkni", name: "Saria", rarity: 6, role: "Defender", arch: "Guardian" },
    { id: "char_003_kalts", name: "Kal'tsit", rarity: 6, role: "Medic", arch: "Summoner" },
    { id: "char_2013_cerber", name: "Ceobe", rarity: 6, role: "Caster", arch: "Core Caster" },
    { id: "char_293_thorns", name: "Thorns", rarity: 6, role: "Guard", arch: "Lord" },
    { id: "char_151_myrtle", name: "Myrtle", rarity: 4, role: "Vanguard", arch: "Tactician" },
    { id: "char_377_gdglow", name: "Goldenglow", rarity: 6, role: "Caster", arch: "Phalanx" },
    { id: "char_181_flower", name: "Ptilopsis", rarity: 5, role: "Medic", arch: "Wandering" },
    { id: "char_017_huang", name: "Blaze", rarity: 6, role: "Guard", arch: "Centurion" },
    { id: "char_136_hsguma", name: "Hoshiguma", rarity: 6, role: "Defender", arch: "Guardian" },
    { id: "char_172_svrash", name: "SilverAsh", rarity: 6, role: "Guard", arch: "Lord" },
];

export const OP_BY_ID: Record<string, Operator> = Object.fromEntries(OPERATORS.map((o) => [o.id, o]));

export const TIER_LISTS: TierList[] = [
    {
        id: "endgame-2026",
        title: "Endgame Stages · Current Meta",
        tag: "Endgame",
        stage: "IS#4 · CC#14 · Annihilation",
        author: { name: "AceShippo", initials: "AS" },
        updated: "2h ago",
        votes: 1284,
        comments: 312,
        hot: true,
        accent: "coral",
        tiers: {
            S: ["char_1028_texas2", "char_311_mudrok", "char_358_lisa", "char_350_surtr"],
            A: ["char_103_angel", "char_180_amgoat", "char_202_demkni", "char_003_kalts"],
            B: ["char_2013_cerber", "char_293_thorns", "char_151_myrtle"],
        },
    },
    {
        id: "beginner-6mo",
        title: "Starter Box · First 6 Months",
        tag: "Beginner",
        stage: "Main Theme · Chapter 1–8",
        author: { name: "Perro_Salchicha", initials: "PS" },
        updated: "yesterday",
        votes: 842,
        comments: 198,
        accent: "mint",
        tiers: {
            S: ["char_151_myrtle", "char_181_flower", "char_358_lisa"],
            A: ["char_103_angel", "char_017_huang", "char_136_hsguma"],
            B: ["char_172_svrash", "char_202_demkni"],
        },
    },
    {
        id: "is4-relic",
        title: "Integrated Strategies #4",
        tag: "Roguelike",
        stage: "Expeditioner's Joklumarkar",
        author: { name: "Viktor R.", initials: "VR" },
        updated: "3d ago",
        votes: 516,
        comments: 71,
        accent: "amber",
        tiers: {
            S: ["char_377_gdglow", "char_293_thorns", "char_311_mudrok"],
            A: ["char_1028_texas2", "char_350_surtr", "char_003_kalts"],
            B: ["char_2013_cerber", "char_180_amgoat"],
        },
    },
    {
        id: "arts-dps",
        title: "Arts DPS Showdown",
        tag: "Niche",
        stage: "High-RES bosses, sarkaz crawls",
        author: { name: "dps_nerd", initials: "DN" },
        updated: "last week",
        votes: 394,
        comments: 88,
        accent: "violet",
        tiers: {
            S: ["char_180_amgoat", "char_377_gdglow", "char_2013_cerber"],
            A: ["char_003_kalts", "char_358_lisa"],
            B: ["char_181_flower"],
        },
    },
];
