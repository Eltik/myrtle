import { ReleaseInfoHint, Stat } from "frontend";

// The pull planner's headline figure: a small label, a bold mono value, an
// optional sub line, currency icon and hint.

// The budget summary row. 45,600 Orundum (76 pulls) + 38 permits + 2 ten-roll
// permits (20) = 134 pulls; 120 Originite Prime converts to 21,600 Orundum, 36 more.
// Weekly: (100 daily missions + 200 monthly card) x 7 + 500 weekly missions +
// 1,800 Annihilation = 4,400 Orundum, 7 whole pulls.
export const BudgetSummary = () => (
    <div className="flex flex-wrap gap-x-8 gap-y-3 p-4">
        <Stat label="Pulls now" value="134" sub="170 with Originite Prime" />
        <Stat label="Earned per week" value="4,400" icon="orundum" sub="7 pulls per week" />
        <Stat
            label="Free on banners"
            value="24"
            hint={
                <ReleaseInfoHint label="About free pulls">
                    Pulls the banners in this window hand out: a Limited banner's free ten-roll plus one a day, a collab's two ten-rolls. They expire with their banner, so they are counted on it rather than added to your bank.
                </ReleaseInfoHint>
            }
        />
    </div>
);

// Simulator readout: 87 pulls cost 87 x 600 = 52,200 Orundum.
export const SimulatorReadout = () => (
    <div className="flex flex-wrap gap-x-8 gap-y-3 p-4">
        <Stat label="Pulls spent" value="87" sub="52,200 Orundum" />
        <Stat label="Since last 6★" value="23" />
        <Stat label="Rate-up copies" value="1" />
    </div>
);

// Plan totals with the overrun figure in amber.
export const PlanTotals = () => (
    <div className="flex flex-wrap gap-x-8 gap-y-3 p-4">
        <Stat label="Committed" value="160" />
        <Stat label="Left over" value="0" />
        <Stat label="Short on 1 banner" value="26" className="text-amber-500" />
    </div>
);

export const Single = () => (
    <div className="p-4">
        <Stat label="Earned per week" value="4,400" icon="orundum" />
    </div>
);
