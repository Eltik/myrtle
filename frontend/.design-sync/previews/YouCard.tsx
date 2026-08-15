import { YouCard } from "frontend";

const player = (over: Record<string, unknown>) => ({
    id: "18220561",
    uid: "18220561",
    nickname: "Eltik",
    nick_number: "1024",
    level: 114,
    avatar_id: "char_102_texas",
    secretary: null,
    secretary_skin_id: null,
    server: "EN",
    total_score: 8422,
    grade: "A",
    operator_score: 0.869,
    stage_score: 0.772,
    roguelike_score: 0.652,
    sandbox_score: 0.601,
    medal_score: 0.732,
    base_score: 0.792,
    skin_score: 0.531,
    rank_global: 214,
    rank_server: 88,
    rank_delta: 12,
    ...over,
});

const CLIMBING = { player: player({}), neighbors: [], percentile: 0.0116, rank_delta: 12 };

const TOP_TIER = {
    player: player({ nickname: "Kyostinv", uid: "10000817", id: "10000817", avatar_id: "char_4064_mlynar", nick_number: null, grade: "SS+", total_score: 9418, rank_global: 1, rank_server: 1, level: 120, rank_delta: 0 }),
    neighbors: [],
    percentile: 0.00005,
    rank_delta: 0,
};

const SLIPPED = {
    player: player({ nickname: "Rhodes Admin", uid: "44012908", id: "44012908", avatar_id: "char_4045_heidi", nick_number: null, grade: "B", total_score: 6104, rank_global: 3871, rank_server: 1902, level: 109, rank_delta: -46 }),
    neighbors: [],
    percentile: 0.21,
    rank_delta: -46,
};

export const ClimbingRanks = () => (
    <div className="w-80">
        <YouCard rankedDoctors={18432} standing={CLIMBING} />
    </div>
);

export const TopOfTheBoard = () => (
    <div className="w-80">
        <YouCard rankedDoctors={18432} standing={TOP_TIER} />
    </div>
);

export const SlippedDown = () => (
    <div className="w-80">
        <YouCard rankedDoctors={18432} standing={SLIPPED} />
    </div>
);
