import { StageFiltersPanel } from "frontend";

// The "Stages" tab of the randomizer settings sheet. It buckets the stage table
// into Main Story / Events / Other, and every row carries a tri-state checkbox
// that folds down to the individual stages. Stages, zones and activity windows
// below are real rows from /api/static/{stages,zones,activities}. Preview capture
// pins the clock to 2024-05-15, so "Zwillingstürme im Herbst" (Apr 30 – May 21
// 2024) is genuinely the open event and the two later ones read as locked.

type Row = [string, string, string, string, number, string, string];

const STAGE_ROWS: Row[] = [
    ["main_08-01", "main_8", "R8-1", "Yesterday, the Chaff Cracked", 18, "NORMAL", "Elite 2 Lv. 10"],
    ["main_08-02", "main_8", "R8-2", "Innocent Flesh and Blood", 18, "NORMAL", "Elite 2 Lv. 10"],
    ["main_08-03", "main_8", "R8-3", "Oatstalk, Easily Alight", 18, "NORMAL", "Elite 2 Lv. 10"],
    ["main_08-06", "main_8", "R8-6", "War, Sprawling Without Cease", 18, "NORMAL", "Elite 2 Lv.15"],
    ["main_08-07", "main_8", "R8-7", "Baptism by Tractor Fire", 18, "NORMAL", "Elite 2 Lv.15"],
    ["main_08-07#f#", "main_8", "R8-7", "Baptism by Tractor Fire", 18, "FOUR_STAR", "-"],
    ["main_11-01", "main_11", "11-1", "Keeping Up Honors", 21, "NORMAL", "Elite 2 Lv. 1"],
    ["main_11-02", "main_11", "11-2", "A Sliver of Light", 21, "NORMAL", "Elite 2 Lv. 1"],
    ["main_11-13", "main_11", "11-15", "The Earth Shakes", 24, "NORMAL", "Elite 2 Lv. 35"],
    ["main_11-14", "main_11", "11-16", "Glory's Hunting Grounds", 21, "NORMAL", "Elite 2 Lv. 30"],
    ["tough_11-13", "main_11", "11-15", "The Earth Shakes", 24, "NORMAL", "Elite 2 Lv. 45"],
    ["act29side_01", "act29sre_zone1", "ZT-1", "Chorale 'Lied des klaren Himmels'", 9, "NORMAL", "LV.20"],
    ["act29side_05", "act29sre_zone1", "ZT-5", "Sonata 'Autumn'", 15, "NORMAL", "Elite 1 Lv. 30"],
    ["act29side_09", "act29sre_zone1", "ZT-9", "Fantasia 'The Gift'", 21, "NORMAL", "Elite 2 Lv. 10"],
    ["act29side_ex01", "act29sre_zone2", "ZT-EX-1", "Presto", 10, "NORMAL", "Elite 1 Lv. 50"],
    ["act29side_ex07", "act29sre_zone2", "ZT-EX-7", "Atonality", 20, "NORMAL", "Elite 2 Lv. 35"],
    ["act29side_ex08#f#", "act29sre_zone2", "ZT-EX-8", "Perpetuum Mobile", 20, "FOUR_STAR", "-"],
    ["act32side_01", "act32side_zone1", "CR-1", "City Exhibit", 9, "NORMAL", "LV.20"],
    ["act32side_02", "act32side_zone1", "CR-2", "Asymmetrical Garden", 9, "NORMAL", "LV.35"],
    ["act32side_ex05", "act32side_zone2", "CR-EX-5", "Scattered Ruins", 15, "NORMAL", "Elite 2 Lv. 10"],
    ["act32side_ex08#f#", "act32side_zone2", "CR-EX-8", "Solid Ground", 20, "FOUR_STAR", "Elite 2 Lv. 40"],
    ["act17mini_01", "act17mini_zone1", "KR-1", "Yumen: Dune Carnival", 9, "NORMAL", "LV.20"],
    ["act17mini_03", "act17mini_zone1", "KR-3", "Sorrento: Family Gathering", 12, "NORMAL", "Elite 1 Lv.35"],
    ["act17mini_05", "act17mini_zone1", "KR-5", "Highbury: Militia Drill", 15, "NORMAL", "Elite 2 Lv.1"],
    ["act17mini_07", "act17mini_zone1", "KR-7", "Norport: Cleanup & Reconstruction", 21, "NORMAL", "Elite 2 Lv.15"],
];

const STAGES = STAGE_ROWS.map(([stageId, zoneId, code, name, apCost, difficulty, dangerLevel]) => ({
    stageId,
    zoneId,
    code,
    name,
    stageType: stageId.startsWith("main_") || stageId.startsWith("tough_") ? "MAIN" : "ACTIVITY",
    difficulty,
    apCost,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel,
    dangerPoint: -1,
    expGain: apCost * 10,
    goldGain: apCost * 10,
    unlockCondition: [],
    bossMark: false,
}));

const zone = (zoneId: string, type: string, second: string, first?: string, title?: string) => ({
    zoneId,
    zoneIndex: 0,
    type,
    zoneNameFirst: first ?? null,
    zoneNameSecond: second,
    zoneNameTitleCurrent: title ?? null,
    canPreview: false,
    hasAdditionalPanel: false,
});

const ZONES = [
    zone("main_8", "MAINLINE", "Roaring Flare", "Episode 8", "08"),
    zone("main_11", "MAINLINE", "Return To Mist", "Episode 11", "11"),
    zone("act29sre_zone1", "ACTIVITY", "Zwillingstürme anbeten"),
    zone("act29sre_zone2", "ACTIVITY", "Die Gnade ist ewig"),
    zone("act32side_zone1", "ACTIVITY", "Roam the Exhibition"),
    zone("act32side_zone2", "ACTIVITY", "Skydiving"),
    zone("act17mini_zone1", "ACTIVITY", "Enter Kazdel"),
];

const activity = (id: string, name: string, startTime: number, endTime: number) => ({ id, name, startTime, endTime, hasStage: true, isReplicate: false });

const ACTIVITY_LOOKUP = {
    activityById: new Map([
        ["act29side", activity("act29side", "Zwillingstürme im Herbst", 1714496400, 1716289199)],
        ["act32side", activity("act32side", "Operation Lucent Arrowhead", 1725555600, 1726808399)],
        ["act17mini", activity("act17mini", "A Kazdelian Rescue", 1738947600, 1739552399)],
    ]),
    retroByZonePrefix: new Map(),
    retroByActivityId: new Map(),
};

// Every stage the user has cleared, keyed the way /plans returns them.
const STAGE_CLEARS = Object.fromEntries(STAGES.slice(0, 12).map((s) => [s.stageId, { state: 3, completeTimes: 4, practiceTimes: 0 }]));

const settings = (over: Record<string, unknown> = {}) => ({
    allowedClasses: ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"],
    allowedRarities: [6, 5, 4, 3, 2, 1],
    allowedZoneTypes: ["MAINLINE", "ACTIVITY"],
    squadSize: 12,
    allowDuplicates: false,
    hideUnplayableOperators: true,
    onlyOwnedOperators: false,
    onlyCompletedStages: false,
    onlyAvailableStages: true,
    onlyE2Operators: false,
    deselectedStageIds: [],
    ...over,
});

const noop = () => {};

// Default: only currently-available content, so the two closed events drop out
// and the whole pool is selected.
export const AvailableOnly = () => (
    <div className="max-w-md">
        <StageFiltersPanel activityLookup={ACTIVITY_LOOKUP} hasProfile onChange={noop} settings={settings()} stageClears={STAGE_CLEARS} stages={STAGES} zones={ZONES} />
    </div>
);

// Availability filter off: Events and Other fill in, and the closed events carry
// the padlock next to their name.
export const AllEvents = () => (
    <div className="max-w-md">
        <StageFiltersPanel activityLookup={ACTIVITY_LOOKUP} hasProfile onChange={noop} settings={settings({ onlyAvailableStages: false })} stageClears={STAGE_CLEARS} stages={STAGES} zones={ZONES} />
    </div>
);

// A partly-pruned pool: the counters drop below the totals and the group
// checkboxes go indeterminate.
export const PartialSelection = () => (
    <div className="max-w-md">
        <StageFiltersPanel
            activityLookup={ACTIVITY_LOOKUP}
            hasProfile
            onChange={noop}
            settings={settings({ onlyAvailableStages: false, deselectedStageIds: ["main_08-01", "main_08-02", "main_08-07#f#", "act32side_01", "act32side_02", "act32side_ex05", "act32side_ex08#f#", "act17mini_01"] })}
            stageClears={STAGE_CLEARS}
            stages={STAGES}
            zones={ZONES}
        />
    </div>
);

// Signed out: "Only stages I've cleared" is locked because there is no profile
// to read clears from.
export const SignedOut = () => (
    <div className="max-w-md">
        <StageFiltersPanel activityLookup={ACTIVITY_LOOKUP} hasProfile={false} onChange={noop} settings={settings()} stageClears={null} stages={STAGES} zones={ZONES} />
    </div>
);
