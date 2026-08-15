import { OfficialRail } from "frontend";

const op = (id: string, name: string, rarity: number, role: string, arch: string) => ({ id, name, rarity, role, arch });

const SIX_STARS = [
    op("char_1035_wisdel", "Wiš'adel", 6, "Sniper", "Besieger"),
    op("char_4064_mlynar", "Młynar", 6, "Guard", "Liberator"),
    op("char_4133_logos", "Logos", 6, "Caster", "Core Caster"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "Specialist", "Executor"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "Guard", "Soloblade"),
];

const SUPPORTS = [
    op("char_358_lisa", "Suzuran", 6, "Supporter", "Decel Binder"),
    op("char_179_cgbird", "Nightingale", 6, "Medic", "Multi-target"),
    op("char_1020_reed2", "Reed the Flame Shadow", 6, "Medic", "Incantation"),
    op("char_4039_horn", "Horn", 6, "Defender", "Fortress"),
];

const STARTERS = [
    op("char_151_myrtle", "Myrtle", 4, "Vanguard", "Standard Bearer"),
    op("char_102_texas", "Texas", 5, "Vanguard", "Pioneer"),
    op("char_128_plosis", "Ptilopsis", 5, "Medic", "Multi-target"),
    op("char_298_susuro", "Sussurro", 4, "Medic", "Single-target"),
];

const base = {
    tag: "Meta",
    stage: "",
    comments: 0,
    hot: false,
    accent: "oklch(0.60 0.15 230)",
    listType: "official" as const,
    createdAtMs: Date.parse("2024-01-18T10:00:00.000Z"),
    updatedAtMs: Date.parse("2024-05-13T09:00:00.000Z"),
    shares: 640,
    views7d: 12400,
    trendingScore: 41.2,
    isTrending: false,
    author: { name: "myrtle.moe", avatarId: "char_151_myrtle" },
};

const lists = [
    {
        ...base,
        id: "tl-global-meta",
        slug: "global-6-star-meta",
        title: "Global 6★ meta — May 2024",
        description: "The team's standing ranking of every 6★ on Global.",
        updated: "yesterday",
        votes: 6120,
        views: 214800,
        favorites: 6120,
        views24h: 3180,
        flairCode: "meta",
        flairLabel: "Meta",
        flairColor: "#5aa9d9",
        tiers: [
            { name: "S+", color: "#dc4d56", operators: SIX_STARS },
            { name: "S", color: "#e0834a", operators: SUPPORTS },
        ],
    },
    {
        ...base,
        id: "tl-support-core",
        slug: "support-core-picks",
        title: "Support core — who actually earns the slot",
        description: "Medics, Supporters and Defenders ranked by squad-wide value.",
        updated: "3d ago",
        votes: 2410,
        views: 88300,
        favorites: 2410,
        views24h: 940,
        flairCode: "class",
        flairLabel: "Class guide",
        flairColor: "#5dbf86",
        tiers: [
            { name: "Core", color: "#5dbf86", operators: SUPPORTS },
            { name: "Situational", color: "#52b9b3", operators: STARTERS },
        ],
    },
    {
        ...base,
        id: "tl-f2p-starter",
        slug: "f2p-starter-roster",
        title: "F2P starter roster — first 30 days",
        description: "What to raise first if you started this month.",
        updated: "5d ago",
        votes: 3890,
        views: 126400,
        favorites: 3890,
        views24h: 1520,
        flairCode: "beginner",
        flairLabel: "Beginner",
        flairColor: "#d8b54a",
        tiers: [
            { name: "Raise now", color: "#dc4d56", operators: STARTERS },
            { name: "Later", color: "#8a8a8a", operators: SUPPORTS.slice(0, 3) },
        ],
    },
];

const noop = () => {};

export const FeaturedRail = () => <OfficialRail lists={lists} onOpen={noop} onViewAll={noop} />;

export const TwoLists = () => <OfficialRail lists={lists.slice(0, 2)} onOpen={noop} onViewAll={noop} />;

export const SingleList = () => <OfficialRail lists={lists.slice(0, 1)} onOpen={noop} onViewAll={noop} />;
