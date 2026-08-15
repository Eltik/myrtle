import { TierListHero } from "frontend";

const op = (id: string, name: string, rarity: number, profession: string, sub: string, position: string, nationId: string | null) => ({
    id,
    name,
    appellation: null,
    rarity,
    profession,
    subProfessionId: sub,
    position,
    nationId,
    subOrder: 0,
    description: null,
    updatedAt: "2024-05-12T14:05:00.000Z",
});

const TIERS = [
    {
        id: "tier-s-plus",
        name: "S+",
        displayOrder: 0,
        color: "#dc4d56",
        description: "Bring these unless the stage actively locks them out.",
        operators: [op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null), op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"), op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes")],
    },
    {
        id: "tier-s",
        name: "S",
        displayOrder: 1,
        color: "#e0834a",
        description: null,
        operators: [op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"), op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"), op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa")],
    },
];

const officialDetail = {
    id: "tl-global-meta",
    slug: "global-6-star-meta",
    title: "Global 6★ meta — May 2024",
    description:
        "The team's standing ranking of every 6★ operator on Global, rebuilt after each banner. Placements assume **E2 level 60**, module stage 2 and no potential above 1. Tiers describe how much of a stage an operator can solve on their own, not raw damage numbers — that is why several Supporters sit above Snipers here.",
    listType: "official" as const,
    createdBy: "u-myrtle",
    isListed: true,
    flair: { id: 1, code: "meta", label: "Meta", color: "#5aa9d9", displayOrder: 1, isActive: true },
    author: { id: "u-myrtle", uid: "myrtle", nickname: "myrtle.moe", avatarId: "char_151_myrtle" },
    stats: {
        viewCount: 214800,
        uniqueViewCount: 98400,
        favoriteCount: 6120,
        shareCount: 1240,
        isTrending: true,
        trendingScore: 88.6,
        viewsLast24h: 3180,
        viewsLast7d: 22400,
        lastViewedAt: "2024-05-15T11:40:00.000Z",
        statsUpdatedAt: "2024-05-15T12:00:00.000Z",
    },
    tiers: TIERS,
    createdAt: "2024-01-18T10:00:00.000Z",
    updatedAt: "2024-05-14T08:30:00.000Z",
};

const communityDetail = {
    ...officialDetail,
    id: "tl-cc13-risk18",
    slug: "cc13-fake-wave-risk-18",
    title: "CC#13 Fake Wave — Risk 18 core picks",
    description: "Ranked by how much risk each operator lets you skip on their own. Built around the Risk 18 clear that drops Ranged Restriction and Deployment Cost.",
    listType: "community" as const,
    flair: { id: 2, code: "cc", label: "Contingency Contract", color: "#e0834a", displayOrder: 2, isActive: true },
    author: { id: "u-kestrel", uid: "kestrel", nickname: "Dr. Kestrel", avatarId: "char_263_skadi" },
    stats: { ...officialDetail.stats, viewCount: 18420, uniqueViewCount: 9130, favoriteCount: 412, shareCount: 88, isTrending: false, trendingScore: 12.4, viewsLast24h: 240, viewsLast7d: 1830 },
    updatedAt: "2024-05-13T09:00:00.000Z",
};

const bareDetail = {
    ...officialDetail,
    id: "tl-h6-4",
    slug: "h6-4-low-end-clears",
    title: "H6-4 low-end clears",
    description: "",
    listType: "community" as const,
    flair: null,
    author: null,
    stats: null,
    createdAt: "2024-05-09T17:00:00.000Z",
    updatedAt: "2024-05-15T10:15:00.000Z",
};

export const OfficialTrending = () => <TierListHero detail={officialDetail} />;

export const CommunityList = () => <TierListHero detail={communityDetail} />;

export const NoAuthorOrStats = () => <TierListHero detail={bareDetail} />;
