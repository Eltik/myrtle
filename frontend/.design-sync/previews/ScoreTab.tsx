import { ScoreTab } from "frontend";

const veteranDoctor = {
    user_id: "1000048871",
    total_score: 0.8642,
    operator_score: 0.9128,
    stage_score: 0.8814,
    roguelike_score: 0.7431,
    sandbox_score: 0.5127,
    medal_score: 0.8209,
    base_score: 0.9407,
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
    skin_score: 0.1204,
    grade: "B",
    calculated_at: "2024-05-11T22:40:00Z",
};

// The full tab: beta notice, the overall grade spanning both columns, then the
// six weighted subscore cards. Improvements load on their own query, so the
// cards render before that resolves.
export const GradedProfile = () => <ScoreTab score={veteranDoctor} isLoading={false} improvements={null} isImprovementsLoading={false} />;

export const MidGameProfile = () => <ScoreTab score={midGameDoctor} isLoading={false} improvements={null} isImprovementsLoading={true} />;

export const Loading = () => <ScoreTab score={null} isLoading={true} improvements={null} isImprovementsLoading={true} />;

export const NoScoreOnFile = () => <ScoreTab score={null} isLoading={false} improvements={null} isImprovementsLoading={false} />;
