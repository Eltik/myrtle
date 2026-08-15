import { StageSlab } from "frontend";

// Slot 01 of a randomizer roll. Stage/zone rows are copied verbatim out of
// /api/static/stages and /api/static/zones, so the map thumbnail on the right
// resolves against the live asset API.

const EMPTY_LOOKUP = {
    activityById: new Map(),
    retroByZonePrefix: new Map(),
    retroByActivityId: new Map(),
};

const stage = (over: Record<string, unknown>) => ({
    stageId: "main_08-07",
    levelId: "Obt/Main/level_main_08-07",
    zoneId: "main_8",
    code: "R8-7",
    name: "Baptism by Tractor Fire",
    stageType: "MAIN",
    difficulty: "NORMAL",
    apCost: 18,
    canPractice: true,
    canBattleReplay: true,
    canMultipleBattle: true,
    isStoryOnly: false,
    isPredefined: false,
    dangerLevel: "Elite 2 Lv.15",
    dangerPoint: -1,
    expGain: 180,
    goldGain: 180,
    appearanceStyle: "MAIN_NORMAL",
    unlockCondition: [],
    bossMark: false,
    ...over,
});

const EPISODE_8 = {
    zoneId: "main_8",
    zoneIndex: 4,
    type: "MAINLINE",
    zoneNameFirst: "Episode 8",
    zoneNameSecond: "Roaring Flare",
    zoneNameTitleCurrent: "08",
    canPreview: true,
    hasAdditionalPanel: false,
};

const EPISODE_10 = {
    zoneId: "main_10",
    zoneIndex: 1,
    type: "MAINLINE",
    zoneNameFirst: "Episode 10",
    zoneNameSecond: "Shatterpoint",
    zoneNameTitleCurrent: "10",
    canPreview: true,
    hasAdditionalPanel: true,
};

const noop = () => {};

export const MainlineStage = () => <StageSlab lookup={EMPTY_LOOKUP} onReroll={noop} stage={stage({})} zone={EPISODE_8} />;

// `difficulty: "FOUR_STAR"` is Challenge Mode — the CM badge appears next to the
// code and the preview falls back to the base stage's map.
export const ChallengeMode = () => <StageSlab lookup={EMPTY_LOOKUP} onReroll={noop} stage={stage({ stageId: "main_08-07#f#", difficulty: "FOUR_STAR", dangerLevel: "-", mainStageId: "main_08-07#f#" })} zone={EPISODE_8} />;

// `bossMark` adds the red Boss badge alongside the AP and danger-level chips.
export const BossStage = () => (
    <StageSlab
        lookup={EMPTY_LOOKUP}
        onReroll={noop}
        stage={stage({
            stageId: "main_10-15",
            levelId: "Obt/Main/level_main_10-15",
            zoneId: "main_10",
            code: "10-17",
            name: "A Citadel and Its Walls",
            apCost: 24,
            dangerLevel: "Elite 2 Lv. 30",
            expGain: 240,
            goldGain: 240,
            bossMark: true,
        })}
        zone={EPISODE_10}
    />
);
