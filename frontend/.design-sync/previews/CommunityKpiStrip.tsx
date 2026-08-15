import { CommunityKpiStrip } from "frontend";

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
        <CommunityKpiStrip data={STATS} />
    </div>
);

/** An early corpus: the same layout, an order of magnitude fewer pulls. */
export const SmallCorpus = () => (
    <div className="px-8 pt-4">
        <CommunityKpiStrip
            data={{
                ...STATS,
                collectiveStats: { totalPulls: 9840, totalUsers: 37, totalSixStars: 281, totalFiveStars: 902, totalFourStars: 4913, totalThreeStars: 3744, firstPullAt: 1713398400 },
                pullRates: { sixStarRate: 0.028556, fiveStarRate: 0.091667 },
                averagePullsToSixStar: 35.02,
                averagePullsToFiveStar: 10.91,
            }}
        />
    </div>
);

export const Loading = () => (
    <div className="px-8 pt-4">
        <CommunityKpiStrip data={null} />
    </div>
);
