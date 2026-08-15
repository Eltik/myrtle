import { AxisControls, Button } from "frontend";
import { Download } from "lucide-react";

// The chart's axis header, shared by the DPS and HPS calculators: X-axis tabs,
// Y-metric select, and the sweep range. `axisInput.integer` is the interesting
// switch — an integer axis hides the "Points" select and captions the range as
// "whole numbers" instead of a step size.

const noop = () => {};

const DPS_AXES = [
    { value: "defense", label: "Enemy DEF", short: "DEF" },
    { value: "res", label: "Enemy RES (%)", short: "RES %" },
];
const DPS_METRICS = [
    { value: "skill_dps", label: "Skill DPS" },
    { value: "average_dps", label: "Average DPS" },
    { value: "total_damage", label: "Total Damage" },
];

const HPS_AXES = [
    { value: "targets", label: "Targets healed", short: "Targets" },
    { value: "atk", label: "ATK buff (%)", short: "ATK %" },
    { value: "aspd", label: "ASPD buff", short: "ASPD" },
];
const HPS_METRICS = [
    { value: "skill_hps", label: "Skill HPS" },
    { value: "base_hps", label: "Base HPS" },
    { value: "avg_hps", label: "Average HPS" },
];

// DPS default: sweep enemy DEF from 0 to 3,000 across 21 plotted points.
export const DefenceSweep = () => (
    <AxisControls
        axes={DPS_AXES}
        axisInput={{ step: 100, maxBound: 5000, integer: false, unit: "" }}
        hydrationToken={1}
        metrics={DPS_METRICS}
        onChangeAxis={noop}
        onChangeMetric={noop}
        onChangeSweep={noop}
        pointCount={21}
        sweep={{ min: 0, max: 3000, steps: 21 }}
        xAxis="defense"
        yMetric="skill_dps"
    />
);

// Resistance sweep at a finer granularity — 41 points across 0–100%.
export const ResistanceSweep = () => (
    <AxisControls
        axes={DPS_AXES}
        axisInput={{ step: 5, maxBound: 100, integer: false, unit: "%" }}
        hydrationToken={1}
        metrics={DPS_METRICS}
        onChangeAxis={noop}
        onChangeMetric={noop}
        onChangeSweep={noop}
        pointCount={41}
        sweep={{ min: 0, max: 100, steps: 41 }}
        xAxis="res"
        yMetric="average_dps"
    />
);

// HPS target-count axis: integer, so the Points select disappears and the
// caption reads "whole numbers".
export const IntegerAxis = () => (
    <AxisControls
        axes={HPS_AXES}
        axisInput={{ step: 1, maxBound: 12, integer: true, unit: "" }}
        hydrationToken={1}
        metricHint="Sustained HPS averaged over a full skill + recharge cycle."
        metrics={HPS_METRICS}
        onChangeAxis={noop}
        onChangeMetric={noop}
        onChangeSweep={noop}
        pointCount={12}
        sweep={{ min: 1, max: 12, steps: 12 }}
        xAxis="targets"
        yMetric="avg_hps"
    />
);

// With a metric caption and the chart's PNG-export button in the range slot.
export const WithHintAndAction = () => (
    <AxisControls
        axes={HPS_AXES}
        axisInput={{ step: 10, maxBound: 400, integer: false, unit: "%" }}
        hydrationToken={1}
        metricHint="Always-on healing per second during SP recharge (0 for burst healers)."
        metrics={HPS_METRICS}
        onChangeAxis={noop}
        onChangeMetric={noop}
        onChangeSweep={noop}
        pointCount={21}
        rangeAction={
            <Button aria-label="Download chart as PNG" className="mb-1" size="icon-sm" variant="outline">
                <Download />
            </Button>
        }
        sweep={{ min: 0, max: 200, steps: 21 }}
        xAxis="atk"
        yMetric="base_hps"
    />
);
