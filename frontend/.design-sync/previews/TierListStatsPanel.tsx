import { TierListStatsPanel } from "frontend";

const base = {
    id: "tl-global-meta",
    slug: "global-6-star-meta",
    title: "Global 6★ meta — May 2024",
    description: "",
    listType: "official" as const,
    createdBy: "u-myrtle",
    isListed: true,
    flair: null,
    author: { id: "u-myrtle", uid: "myrtle", nickname: "myrtle.moe", avatarId: "char_151_myrtle" },
    tiers: [],
    createdAt: "2024-01-18T10:00:00.000Z",
    updatedAt: "2024-05-14T08:30:00.000Z",
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
};

const steady = {
    ...base,
    id: "tl-cc13",
    slug: "cc13-fake-wave-risk-18",
    createdAt: "2024-03-02T10:00:00.000Z",
    updatedAt: "2024-05-13T09:00:00.000Z",
    stats: { ...base.stats, viewCount: 18420, uniqueViewCount: 9130, favoriteCount: 412, shareCount: 88, isTrending: false, trendingScore: 12.4, viewsLast24h: 0, viewsLast7d: 1830 },
};

const fresh = {
    ...base,
    id: "tl-draft",
    slug: "chapter-14-draft",
    createdAt: "2024-05-15T09:40:00.000Z",
    updatedAt: "2024-05-15T10:15:00.000Z",
    stats: null,
};

export const Trending = () => (
    <div className="w-72">
        <TierListStatsPanel detail={base} />
    </div>
);

export const Steady = () => (
    <div className="w-72">
        <TierListStatsPanel detail={steady} />
    </div>
);

export const NoStatsYet = () => (
    <div className="w-72">
        <TierListStatsPanel detail={fresh} />
    </div>
);
