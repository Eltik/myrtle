import { BrowseCard } from "frontend";

// Browse cards take the flattened `ITierListBrowseItem` the /tier-lists route
// builds: an operator here is the compact card shape (id/name/rarity/role/arch),
// not the full placement record the detail page uses.
const op = (id: string, name: string, rarity: number, role: string, arch: string) => ({ id, name, rarity, role, arch });

const S_TIER = [
    op("char_1035_wisdel", "Wiš'adel", 6, "Sniper", "Besieger"),
    op("char_4064_mlynar", "Młynar", 6, "Guard", "Liberator"),
    op("char_4133_logos", "Logos", 6, "Caster", "Core Caster"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "Specialist", "Executor"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "Guard", "Soloblade"),
];

const A_TIER = [
    op("char_350_surtr", "Surtr", 6, "Guard", "Arts Fighter"),
    op("char_1032_excu2", "Executor the Ex Foedere", 6, "Guard", "Reaper"),
    op("char_311_mudrok", "Mudrock", 6, "Defender", "Juggernaut"),
    op("char_358_lisa", "Suzuran", 6, "Supporter", "Decel Binder"),
    op("char_180_amgoat", "Eyjafjalla", 6, "Caster", "Core Caster"),
    op("char_4039_horn", "Horn", 6, "Defender", "Fortress"),
    op("char_249_mlyss", "Muelsyse", 6, "Vanguard", "Tactician"),
];

const B_TIER = [
    op("char_222_bpipe", "Bagpipe", 6, "Vanguard", "Charger"),
    op("char_103_angel", "Exusiai", 6, "Sniper", "Marksman"),
    op("char_179_cgbird", "Nightingale", 6, "Medic", "Multi-target"),
    op("char_102_texas", "Texas", 5, "Vanguard", "Pioneer"),
    op("char_151_myrtle", "Myrtle", 4, "Vanguard", "Standard Bearer"),
];

const communityList = {
    id: "tl-cc13-risk18",
    slug: "cc13-fake-wave-risk-18",
    title: "CC#13 Fake Wave — Risk 18 core picks",
    tag: "Contingency Contract",
    stage: "Risk 18 clear roster",
    author: { name: "Dr. Kestrel", avatarId: "char_263_skadi" },
    updated: "2d ago",
    votes: 412,
    views: 18420,
    comments: 96,
    hot: false,
    accent: "oklch(0.60 0.15 230)",
    listType: "community" as const,
    description: "Ranked by how much risk each operator lets you skip on their own.",
    createdAtMs: Date.parse("2024-03-02T10:00:00.000Z"),
    updatedAtMs: Date.parse("2024-05-13T09:00:00.000Z"),
    flairCode: "cc",
    flairLabel: "Contingency Contract",
    flairColor: "#e0834a",
    favorites: 412,
    shares: 88,
    views24h: 240,
    views7d: 1830,
    trendingScore: 12.4,
    isTrending: false,
    tiers: [
        { name: "S", color: "#dc4d56", operators: S_TIER },
        { name: "A", color: "#e0834a", operators: A_TIER },
        { name: "B", color: "#d8b54a", operators: B_TIER },
    ],
};

const officialList = {
    ...communityList,
    id: "tl-global-meta",
    slug: "global-6-star-meta",
    title: "Global 6★ meta — May 2024",
    tag: "Meta",
    author: { name: "myrtle.moe", avatarId: "char_151_myrtle" },
    updated: "yesterday",
    views: 214800,
    favorites: 6120,
    votes: 6120,
    listType: "official" as const,
    flairCode: "meta",
    flairLabel: "Meta",
    flairColor: "#5aa9d9",
    shares: 1240,
    views24h: 3180,
    views7d: 22400,
};

const trendingList = {
    ...communityList,
    id: "tl-h6-4-lowend",
    slug: "h6-4-low-end-clears",
    title: "H6-4 low-end clears — no 6★ required",
    author: { name: "Ansel Reruns", avatarId: "char_002_amiya" },
    updated: "6h ago",
    views: 41250,
    favorites: 1980,
    votes: 1980,
    views24h: 5240,
    trendingScore: 88.6,
    isTrending: true,
    hot: true,
    flairCode: "guide",
    flairLabel: "Guide",
    flairColor: "#5dbf86",
    tiers: [
        { name: "Must bring", color: "#dc4d56", operators: B_TIER },
        { name: "Flexible", color: "#5dbf86", operators: A_TIER.slice(0, 5) },
    ],
};

const emptyDraft = {
    ...communityList,
    id: "tl-draft-ex8",
    slug: "chapter-14-draft",
    title: "Chapter 14 — Roaring Flare draft",
    author: { name: "Dr. Vesper", avatarId: "char_102_texas" },
    updated: "just now",
    views: 12,
    favorites: 0,
    votes: 0,
    shares: 0,
    views24h: 12,
    views7d: 12,
    flairCode: null,
    flairLabel: null,
    flairColor: null,
    tiers: [],
};

export const CommunityList = () => (
    <div className="w-70">
        <BrowseCard tl={communityList} />
    </div>
);

export const OfficialList = () => (
    <div className="w-70">
        <BrowseCard tl={officialList} />
    </div>
);

export const TrendingRank = () => (
    <div className="w-80">
        <BrowseCard tl={trendingList} size="trending" rank={1} />
    </div>
);

export const EmptyDraft = () => (
    <div className="w-70">
        <BrowseCard tl={emptyDraft} />
    </div>
);
