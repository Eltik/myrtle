import { TierListCard } from "frontend";

// One card in the landing page's community tier-list gallery: tag pill, title,
// the top tier as named operator chips, a ghost row summarising the tiers below
// it, and an author/stats footer. Every operator id is checked against
// `https://api.myrtle.moe/api/operators/index`, so the avatars resolve.

const op = (id: string, name: string, rarity: number, role: string, arch: string) => ({ id, name, rarity, role, arch });

const ENDGAME = {
    id: "tl-endgame-cc12",
    slug: "endgame-damage-cores",
    title: "Endgame damage cores — CC#12 Pyrolysis",
    tag: "Endgame",
    stage: "CC#12 Pyrolysis",
    author: { name: "Kal'tsit", avatarId: "char_003_kalts" },
    updated: "2d ago",
    votes: 1284,
    views: 18420,
    comments: 96,
    hot: true,
    accent: "coral",
    tiers: [
        {
            name: "S",
            operators: [op("char_4064_mlynar", "Młynar", 6, "Guard", "Soloblade"), op("char_350_surtr", "Surtr", 6, "Guard", "Arts Fighter"), op("char_2012_typhon", "Typhon", 6, "Sniper", "Besieger"), op("char_377_gdglow", "Goldenglow", 6, "Caster", "Drone Caster")],
        },
        {
            name: "A",
            operators: [op("char_4116_blkkgt", "Degenbrecher", 6, "Guard", "Swordmaster"), op("char_4087_ines", "Ines", 6, "Vanguard", "Agent"), op("char_4133_logos", "Logos", 6, "Caster", "Core Caster"), op("char_4123_ela", "Ela", 6, "Specialist", "Trapmaster")],
        },
        { name: "B", operators: [op("char_103_angel", "Exusiai", 6, "Sniper", "Marksman"), op("char_010_chen", "Ch'en", 6, "Guard", "Swordmaster")] },
    ],
};

const SUSTAIN = {
    id: "tl-sustain-lanes",
    slug: "sustain-and-support",
    title: "Sustain & support — who actually holds a lane",
    tag: "Support",
    stage: "H8-4 Roaring Flare",
    author: { name: "Ptilopsis", avatarId: "char_128_plosis" },
    updated: "6d ago",
    votes: 742,
    views: 9310,
    comments: 41,
    accent: "mint",
    tiers: [
        {
            name: "S",
            operators: [op("char_179_cgbird", "Nightingale", 6, "Medic", "Multi-target Medic"), op("char_147_shining", "Shining", 6, "Medic", "Single-target Medic"), op("char_202_demkni", "Saria", 6, "Defender", "Guardian"), op("char_358_lisa", "Suzuran", 6, "Supporter", "Decel Binder")],
        },
        { name: "A", operators: [op("char_128_plosis", "Ptilopsis", 5, "Medic", "Multi-target Medic"), op("char_171_bldsk", "Warfarin", 5, "Medic", "Single-target Medic"), op("char_136_hsguma", "Hoshiguma", 6, "Defender", "Protector")] },
        { name: "B", operators: [op("char_181_flower", "Perfumer", 4, "Medic", "Multi-target Medic")] },
    ],
};

const BUDGET = {
    id: "tl-budget-core",
    slug: "low-rarity-core",
    title: "Low-rarity core — everything a new Doctor should E2",
    tag: "Budget",
    stage: "1-7 farming loop",
    author: { name: "Eltik", avatarId: "char_151_myrtle" },
    updated: "3w ago",
    votes: 2011,
    views: 44870,
    comments: 158,
    accent: "amber",
    tiers: [
        {
            name: "Core",
            operators: [op("char_151_myrtle", "Myrtle", 4, "Vanguard", "Flagbearer"), op("char_181_flower", "Perfumer", 4, "Medic", "Multi-target Medic"), op("char_117_myrrh", "Myrrh", 4, "Medic", "Single-target Medic"), op("char_123_fang", "Fang", 3, "Vanguard", "Pioneer")],
        },
        { name: "Nice to have", operators: [op("char_240_wyvern", "Vanilla", 3, "Vanguard", "Pioneer"), op("char_401_elysm", "Elysium", 5, "Vanguard", "Flagbearer")] },
    ],
};

const SINGLE_TIER = {
    ...BUDGET,
    id: "tl-ce-5",
    slug: "ce-5-two-op-clear",
    title: "CE-5 two-operator clear",
    tag: "Farming",
    stage: "CE-5 Cargo Escort",
    author: { name: "Muelsyse", avatarId: "char_249_mlyss" },
    updated: "9h ago",
    votes: 318,
    views: 5240,
    comments: 12,
    hot: false,
    accent: "violet",
    tiers: [{ name: "Required", operators: [op("char_263_skadi", "Skadi", 6, "Guard", "Dreadnought"), op("char_202_demkni", "Saria", 6, "Defender", "Guardian")] }],
};

const noop = () => {};

export const Default = () => (
    <div className="max-w-sm">
        <TierListCard tl={SUSTAIN} onOpen={noop} />
    </div>
);

export const Trending = () => (
    <div className="max-w-sm">
        <TierListCard tl={ENDGAME} onOpen={noop} />
    </div>
);

export const SingleTier = () => (
    <div className="max-w-sm">
        <TierListCard tl={SINGLE_TIER} onOpen={noop} />
    </div>
);

export const GalleryGrid = () => (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
        <TierListCard tl={ENDGAME} onOpen={noop} />
        <TierListCard tl={SUSTAIN} onOpen={noop} />
        <TierListCard tl={BUDGET} onOpen={noop} />
    </div>
);
