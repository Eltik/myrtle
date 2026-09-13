import { ScoreHistoryCard } from "frontend";

// Total score over time. `history` is the leaderboard job's snapshot series
// (`IScoreHistoryPoint[]`: taken_at, total_score 0..1, rank_global,
// rank_server) and the live `score.calculated_at` point is appended as the
// freshest sample. The line takes the grade's colour; the chip on the right
// is first-to-last delta in points. Snapshots here are the weekly Sunday
// runs leading up to 2024-05-15.

const veteranDoctor = {
    user_id: "1000048871",
    total_score: 0.8642,
    operator_score: 0.9128,
    stage_score: 0.8814,
    roguelike_score: 0.7431,
    sandbox_score: 0.5127,
    medal_score: 0.8209,
    base_score: 0.9407,
    base_utilization: 0.9216,
    base_infrastructure: 0.9583,
    skin_score: 0.4412,
    grade: "S",
    calculated_at: "2024-05-14T09:12:00Z",
};

const midGameDoctor = {
    user_id: "1000112904",
    total_score: 0.5183,
    operator_score: 0.6042,
    stage_score: 0.5771,
    roguelike_score: 0.2914,
    sandbox_score: 0.1108,
    medal_score: 0.3376,
    base_score: 0.6688,
    base_utilization: 0.7021,
    base_infrastructure: 0.6375,
    skin_score: 0.1204,
    grade: "B",
    calculated_at: "2024-05-11T22:40:00Z",
};

// Sixteen Sunday snapshots, 2024-01-28 -> 2024-05-12: a steady climb from
// A into S, global rank tightening as the account catches up.
const climbing = [
    { taken_at: "2024-01-28T04:00:00Z", total_score: 0.7818, rank_global: 2412, rank_server: 1180 },
    { taken_at: "2024-02-04T04:00:00Z", total_score: 0.7864, rank_global: 2371, rank_server: 1162 },
    { taken_at: "2024-02-11T04:00:00Z", total_score: 0.7931, rank_global: 2288, rank_server: 1119 },
    { taken_at: "2024-02-18T04:00:00Z", total_score: 0.7952, rank_global: 2254, rank_server: 1107 },
    { taken_at: "2024-02-25T04:00:00Z", total_score: 0.8047, rank_global: 2103, rank_server: 1034 },
    { taken_at: "2024-03-03T04:00:00Z", total_score: 0.8102, rank_global: 2011, rank_server: 992 },
    { taken_at: "2024-03-10T04:00:00Z", total_score: 0.8119, rank_global: 1986, rank_server: 981 },
    { taken_at: "2024-03-17T04:00:00Z", total_score: 0.8188, rank_global: 1874, rank_server: 926 },
    { taken_at: "2024-03-24T04:00:00Z", total_score: 0.8276, rank_global: 1729, rank_server: 855 },
    { taken_at: "2024-03-31T04:00:00Z", total_score: 0.8301, rank_global: 1690, rank_server: 837 },
    { taken_at: "2024-04-07T04:00:00Z", total_score: 0.8344, rank_global: 1612, rank_server: 799 },
    { taken_at: "2024-04-14T04:00:00Z", total_score: 0.8412, rank_global: 1498, rank_server: 742 },
    { taken_at: "2024-04-21T04:00:00Z", total_score: 0.8455, rank_global: 1421, rank_server: 706 },
    { taken_at: "2024-04-28T04:00:00Z", total_score: 0.8503, rank_global: 1340, rank_server: 668 },
    { taken_at: "2024-05-05T04:00:00Z", total_score: 0.8571, rank_global: 1246, rank_server: 623 },
    { taken_at: "2024-05-12T04:00:00Z", total_score: 0.8609, rank_global: 1187, rank_server: 594 },
];

// A B-grade account drifting down as the leaderboard around it grows: the
// chip flips to the red "▼" form.
const slipping = [
    { taken_at: "2024-03-10T04:00:00Z", total_score: 0.5466, rank_global: 9874, rank_server: 4611 },
    { taken_at: "2024-03-17T04:00:00Z", total_score: 0.5471, rank_global: 9902, rank_server: 4630 },
    { taken_at: "2024-03-24T04:00:00Z", total_score: 0.5432, rank_global: 10115, rank_server: 4718 },
    { taken_at: "2024-03-31T04:00:00Z", total_score: 0.5398, rank_global: 10402, rank_server: 4855 },
    { taken_at: "2024-04-07T04:00:00Z", total_score: 0.5381, rank_global: 10566, rank_server: 4931 },
    { taken_at: "2024-04-14T04:00:00Z", total_score: 0.5329, rank_global: 10988, rank_server: 5140 },
    { taken_at: "2024-04-21T04:00:00Z", total_score: 0.5277, rank_global: 11431, rank_server: 5352 },
    { taken_at: "2024-04-28T04:00:00Z", total_score: 0.5240, rank_global: 11790, rank_server: 5519 },
    { taken_at: "2024-05-05T04:00:00Z", total_score: 0.5211, rank_global: 12034, rank_server: 5637 },
];

/** The canonical card: sixteen weekly snapshots plus today's live score, climbing through S. */
export const Climbing = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ScoreHistoryCard history={climbing} isLoading={false} score={veteranDoctor} />
    </div>
);

/** A falling series: the trend chip turns red and the line takes the B grade's colour. */
export const Slipping = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ScoreHistoryCard history={slipping} isLoading={false} score={midGameDoctor} />
    </div>
);

/** One snapshot on file: with the live score that is two points, the shortest line the card will draw. */
export const FirstSnapshot = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ScoreHistoryCard history={[climbing[climbing.length - 1]]} isLoading={false} score={veteranDoctor} />
    </div>
);

/** No snapshots yet - the account has not appeared in a leaderboard run, so the card explains itself instead of drawing. */
export const NoHistoryYet = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ScoreHistoryCard history={[]} isLoading={false} score={midGameDoctor} />
    </div>
);

/** While the history query is in flight the same slot reads "Loading history…". */
export const Loading = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ScoreHistoryCard history={undefined} isLoading score={veteranDoctor} />
    </div>
);
