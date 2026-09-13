import { ContributionCard, OverallGradeCard } from "frontend";

// `IUserScore` as the grader returns it: every subscore is a 0..1 fraction,
// `grade` the letter bucket the composite falls into. The card turns the six
// section weights (Operator 0.85, Stages 0.6, Base 0.35, Roguelike 0.3,
// Sandbox 0.2, Medals 0.2) into one segmented bar - each segment is a section's
// share of the grade, filled by how much of it the account has earned - so the
// legend reads "earned / potential" per section.

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

const freshDoctor = {
    user_id: "1000330017",
    total_score: 0.1712,
    operator_score: 0.2244,
    stage_score: 0.1531,
    roguelike_score: 0,
    sandbox_score: 0,
    medal_score: 0.0488,
    base_score: 0.2602,
    base_utilization: null,
    base_infrastructure: null,
    skin_score: 0,
    grade: "D",
    calculated_at: "2024-05-15T06:03:00Z",
};

/** An S-grade account: every segment is mostly filled, Sandbox is the visible gap. */
export const VeteranAccount = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ContributionCard score={veteranDoctor} />
    </div>
);

/** Mid-game: the heavy Operator and Stages segments are half full, the small ones barely started. */
export const MidGameAccount = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ContributionCard score={midGameDoctor} />
    </div>
);

/** A fresh sync - Roguelike and Sandbox at zero leave their segments empty, so the untapped pools stand out. */
export const FreshAccount = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 sm:grid-cols-2">
        <ContributionCard score={freshDoctor} />
    </div>
);

/** As the Score tab stacks it: directly under the overall grade, both spanning the two-column grid. */
export const UnderOverallGrade = () => (
    <div className="mx-auto grid max-w-3xl grid-cols-1 items-start gap-3 sm:grid-cols-2">
        <OverallGradeCard score={veteranDoctor} standing={null} />
        <ContributionCard score={veteranDoctor} />
    </div>
);
