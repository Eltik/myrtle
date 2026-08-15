import { CommunityPageHeader } from "frontend";

// Live /gacha/stats/enhanced payload, re-stamped to the capture clock (2024-05-15T12:00Z).
const STATS = {
    collectiveStats: { totalPulls: 200670, totalUsers: 592, totalSixStars: 5818, totalFiveStars: 18571, totalFourStars: 100025, totalThreeStars: 76256, firstPullAt: 1705363200 },
    pullRates: { sixStarRate: 0.02899287, fiveStarRate: 0.09254497 },
    mostCommonOperators: [],
    averagePullsToSixStar: 34.49,
    averagePullsToFiveStar: 10.81,
    computedAt: "2024-05-15T11:52:00Z",
    cached: false,
};

export const Default = () => (
    <div className="px-8 pt-4">
        <CommunityPageHeader data={STATS} isLoading={false} />
    </div>
);

/** The backend answered from its 5-minute cache, so the header adds a `cached` chip. */
export const ServedFromCache = () => (
    <div className="px-8 pt-4">
        <CommunityPageHeader data={{ ...STATS, cached: true, computedAt: "2024-05-15T09:30:00Z" }} isLoading={false} />
    </div>
);

export const Loading = () => (
    <div className="px-8 pt-4">
        <CommunityPageHeader data={null} isLoading={true} />
    </div>
);
