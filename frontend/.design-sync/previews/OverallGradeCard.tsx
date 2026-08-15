import { OverallGradeCard } from "frontend";

// Shape of `IUserScore` as the backend's grader returns it — every subscore is
// a 0..1 fraction, `grade` is the letter bucket the composite falls into.
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

const freshSync = {
    user_id: "1000330017",
    total_score: 0.0912,
    operator_score: 0.1244,
    stage_score: 0.0731,
    roguelike_score: 0,
    sandbox_score: 0,
    medal_score: 0.0188,
    base_score: 0.1602,
    skin_score: 0,
    grade: null,
    calculated_at: null,
};

export const TopGrade = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <OverallGradeCard score={veteranDoctor} />
    </div>
);

export const MidGame = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <OverallGradeCard score={midGameDoctor} />
    </div>
);

// No letter grade yet and no `calculated_at` — the card degrades to a neutral
// dash and drops the "Calculated ·" line rather than rendering a broken date.
export const Ungraded = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <OverallGradeCard score={freshSync} />
    </div>
);
