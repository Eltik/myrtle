import { FarmStages } from "frontend";

// An event's farmable stages: code, AP cost, and each drop with its rate band.
// Live rows from Adventure That Cannot Wait for the Sun - Rerun.
const STAGES = [
    { stageId: "act35side_07", code: "AS-7", apCost: 21, drops: [{ itemId: "30043", name: "异铁组", nameEn: "Oriron Cluster", iconId: "MTL_SL_IRON3", tier: 3, occ: "USUAL" }] },
    { stageId: "act35side_08", code: "AS-8", apCost: 21, drops: [{ itemId: "30093", name: "研磨石", nameEn: "Grindstone", iconId: "MTL_SL_PG1", tier: 3, occ: "USUAL" }] },
    { stageId: "act35side_09", code: "AS-9", apCost: 21, drops: [{ itemId: "31083", name: "环烃聚质", nameEn: "Aggregate Cyclicene", iconId: "MTL_SL_HT", tier: 3, occ: "USUAL" }] },
];

export const Default = () => (
    <div className="w-full max-w-2xl p-4">
        <FarmStages stages={STAGES} />
    </div>
);

// Compact: icons only, names moved into the title, as the calendar pill detail uses it.
export const Compact = () => (
    <div className="w-full max-w-md p-4">
        <FarmStages stages={STAGES} compact />
    </div>
);
