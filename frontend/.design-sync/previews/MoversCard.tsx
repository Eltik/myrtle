import { MoversCard } from "frontend";

const CLIMBERS = [
    { uid: "18220561", nickname: "Eltik", nick_number: null, avatar_id: "char_102_texas", server: "en", current_rank: 4, previous_rank: 16, rank_delta: 12, current_score: 0.842, score_delta: 0.031 },
    { uid: "21748302", nickname: "Doktah", nick_number: null, avatar_id: "char_263_skadi", server: "jp", current_rank: 2, previous_rank: 5, rank_delta: 3, current_score: 0.961, score_delta: 0.009 },
    { uid: "52310884", nickname: "Ceylonade", nick_number: null, avatar_id: "char_180_amgoat", server: "cn", current_rank: 9, previous_rank: 11, rank_delta: 2, current_score: 0.884, score_delta: 0.004 },
];

const FALLERS = [
    { uid: "44012908", nickname: "Rhodes Admin", nick_number: null, avatar_id: "char_4045_heidi", server: "kr", current_rank: 27, previous_rank: 12, rank_delta: -15, current_score: 0.701, score_delta: -0.018 },
    { uid: "60338125", nickname: "Nine Lives", nick_number: null, avatar_id: "char_1028_texas2", server: "tw", current_rank: 41, previous_rank: 33, rank_delta: -8, current_score: 0.664, score_delta: -0.007 },
    { uid: "70115552", nickname: "Amiya Enjoyer", nick_number: null, avatar_id: "char_002_amiya", server: "en", current_rank: 58, previous_rank: 58, rank_delta: 0, current_score: 0.612, score_delta: 0 },
];

export const Climbers = () => (
    <div className="w-80">
        <MoversCard intervalLabel="today" movers={CLIMBERS} />
    </div>
);

export const Fallers = () => (
    <div className="w-80">
        <MoversCard intervalLabel="7 days" movers={FALLERS} />
    </div>
);

export const Loading = () => (
    <div className="w-80">
        <MoversCard isLoading intervalLabel="today" movers={[]} />
    </div>
);

export const NoMovementYet = () => (
    <div className="w-80">
        <MoversCard intervalLabel="30 days" movers={[]} />
    </div>
);
