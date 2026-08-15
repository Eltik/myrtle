import { RandomizerStagePreview } from "frontend";

// The map thumbnail inside a rolled stage slab (exported as
// `RandomizerStagePreview`; it is `StagePreview` in the randomizer source). It
// probes the `stage_mappreview_h2_<family>*_0/<stageId>.png` candidates in order
// and renders the first that loads, or a placeholder if none do. Clicking it
// opens a zoom/pan dialog — interaction only, not captured here.

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
    unlockCondition: [],
    bossMark: false,
    ...over,
});

export const MainlineMap = () => (
    <div className="max-w-sm">
        <RandomizerStagePreview stage={stage({})} />
    </div>
);

export const BossMap = () => (
    <div className="max-w-sm">
        <RandomizerStagePreview stage={stage({ stageId: "main_10-15", zoneId: "main_10", code: "10-17", name: "A Citadel and Its Walls", apCost: 24, bossMark: true })} />
    </div>
);

// Event stages ship no map preview asset, so every candidate 404s and the
// component settles on its own placeholder — the real, common outcome.
export const NoPreviewAvailable = () => (
    <div className="max-w-sm">
        <RandomizerStagePreview stage={stage({ stageId: "act32side_ex08", zoneId: "act32side_zone2", code: "CR-EX-8", name: "Solid Ground", stageType: "ACTIVITY", apCost: 20, dangerLevel: "Elite 2 Lv. 40" })} />
    </div>
);
