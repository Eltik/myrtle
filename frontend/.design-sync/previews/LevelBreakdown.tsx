import { LevelBreakdown } from "frontend";

// Where E2 owners stop investing in one skill or module: four rows, one per
// level, each with its own bar as a share of the WHOLE cohort. Level 0 is a
// real row ("No mastery" / "Not unlocked"), not missing data - it is usually
// the largest. `buckets` is always the zero-filled 0..3 histogram the backend
// returns (`LevelBucket[]`), `total` the cohort size, and `ownLevel` marks the
// viewer's own row with a "you" tag when they are signed in and at E2.

const MASTERY_LABELS = ["No mastery", "M1", "M2", "M3"] as const;
const MODULE_LABELS = ["Not unlocked", "Lv1", "Lv2", "Lv3"] as const;

// Młynar S3: a skill most E2 owners take to M3. 12,418 E2 owners.
const MLYNAR_S3 = [
    { level: 0, users: 3_102 },
    { level: 1, users: 611 },
    { level: 2, users: 1_407 },
    { level: 3, users: 7_298 },
];

// The Skills tab strip under the skill selector, tracking the open skill.
export const SkillMastery = () => (
    <div className="max-w-md">
        <LevelBreakdown buckets={MLYNAR_S3} total={12_418} title="Community mastery" labels={MASTERY_LABELS} summary="mastered this skill" cohort="E2 owners" ownLevel={null} />
    </div>
);

// Signed in, roster synced, and the viewer sits at M3 with the majority.
export const SkillMasteryOwnRow = () => (
    <div className="max-w-md">
        <LevelBreakdown buckets={MLYNAR_S3} total={12_418} title="Community mastery" labels={MASTERY_LABELS} summary="mastered this skill" cohort="E2 owners" ownLevel={3} />
    </div>
);

// Młynar S1: almost nobody masters it. The invested headline drops to single
// digits and the M1/M2 rows shrink to the pixel floor rather than vanishing.
const MLYNAR_S1 = [
    { level: 0, users: 11_566 },
    { level: 1, users: 289 },
    { level: 2, users: 173 },
    { level: 3, users: 390 },
];

export const RarelyMastered = () => (
    <div className="max-w-md">
        <LevelBreakdown buckets={MLYNAR_S1} total={12_418} title="Community mastery" labels={MASTERY_LABELS} summary="mastered this skill" cohort="E2 owners" ownLevel={1} />
    </div>
);

// The Information tab's module strip, spanning both columns of the module
// picker (`md:col-span-2`). Levels are 0..3 with 0 meaning not yet unlocked.
const MLYNAR_LIB_X = [
    { level: 0, users: 4_020 },
    { level: 1, users: 1_133 },
    { level: 2, users: 1_866 },
    { level: 3, users: 5_399 },
];

export const ModuleLevels = () => (
    <div className="max-w-md">
        <LevelBreakdown buckets={MLYNAR_LIB_X} total={12_418} title="Community module level" labels={MODULE_LABELS} summary="unlocked this module" cohort="E2 owners" ownLevel={0} />
    </div>
);

// A newer operator, just over the 50-owner reporting floor: small counts, and
// the whole-number percentages give way to one decimal under 10%.
const RECENT_S2 = [
    { level: 0, users: 41 },
    { level: 1, users: 3 },
    { level: 2, users: 6 },
    { level: 3, users: 12 },
];

export const SmallCohort = () => (
    <div className="max-w-md">
        <LevelBreakdown buckets={RECENT_S2} total={62} title="Community mastery" labels={MASTERY_LABELS} summary="mastered this skill" cohort="E2 owners" ownLevel={null} />
    </div>
);
