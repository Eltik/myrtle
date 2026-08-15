import { RarityPanel } from "frontend";

const STATS = {
    collectiveStats: { totalPulls: 200670, totalUsers: 592, totalSixStars: 5818, totalFiveStars: 18571, totalFourStars: 100025, totalThreeStars: 76256, firstPullAt: 1705363200 },
    pullRates: { sixStarRate: 0.02899287, fiveStarRate: 0.09254497 },
    mostCommonOperators: [],
    averagePullsToSixStar: 34.49,
    averagePullsToFiveStar: 10.81,
    computedAt: "2024-05-15T11:52:00Z",
    cached: false,
};

export const CommunityOnly = () => <RarityPanel data={STATS} personal={null} />;

/** Signed in with stored pulls: each row gains a "you" marker and two comparison chips. */
export const WithPersonalComparison = () => <RarityPanel data={STATS} personal={{ totalPulls: 512, totalSixStars: 12, totalFiveStars: 46, totalFourStars: 231, totalThreeStars: 223 }} />;

/** An unlucky doctor: the 6★ row reads well below both community and expected. */
export const PersonalBelowExpected = () => <RarityPanel data={STATS} personal={{ totalPulls: 340, totalSixStars: 5, totalFiveStars: 27, totalFourStars: 168, totalThreeStars: 140 }} />;

export const Loading = () => <RarityPanel data={null} personal={null} />;
