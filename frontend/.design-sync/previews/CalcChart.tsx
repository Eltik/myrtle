import { CalcChart } from "frontend";
import { Calculator } from "lucide-react";

// The comparison chart shared by the DPS and HPS calculators. One recharts line
// per visible instance, a dashed "snapshot" marker at the enemy the KPI panel is
// reporting on, and two of its own empty states. Animation is off, so it renders
// fully on the first paint.

const op = (id: string, name: string) => ({ id, name, availableSkills: [1, 2, 3], availableModules: [0, 1, 2], defaultSkill: 3, defaultModule: 1, conditionals: [] });

const config = { potential: 1, trust: 100, skillIndex: 3, skillRank: 10, moduleIndex: 1, moduleLevel: 3, buffs: {}, conditionals: {}, allCond: true };

const instance = (uid: string, id: string, name: string, color: string, visible = true) => ({ uid, op: op(id, name), color, visible, collapsed: false, config });

const INSTANCES = [instance("i1", "char_4064_mlynar", "Młynar", "oklch(0.65 0.20 30)"), instance("i2", "char_1035_wisdel", "Wiš'adel", "oklch(0.62 0.18 220)"), instance("i3", "char_2012_typhon", "Typhon", "oklch(0.62 0.18 145)")];

// A physical-damage DPS curve decays roughly as atk-per-hit minus DEF; these
// three shapes are the familiar high-DEF crossover between a Guard and two
// snipers.
const ROWS = Array.from({ length: 21 }, (_, i) => {
    const x = i * 150;
    return {
        x,
        i1: Math.round(Math.max(620, 5120 - x * 1.32)),
        i2: Math.round(Math.max(1180, 4180 - x * 0.62)),
        i3: Math.round(Math.max(980, 3460 - x * 0.41)),
    };
});

const noop = () => {};

const EMPTY_DESCRIPTION = (
    <>
        Pick any operator from the picker to plot a DPS curve. The chart will show how DPS scales as you sweep across <span className="font-medium text-foreground">Enemy DEF</span>.
    </>
);

const common = {
    allowDecimals: true,
    xLabel: "Enemy DEF",
    yLabel: "Skill DPS",
    formatTooltipX: (x: number) => `DEF ${x.toLocaleString("en-US")}`,
    snapshotX: 1000,
    onLegendClick: noop,
    emptyIcon: <Calculator />,
    emptyTitle: "No operators yet",
    emptyDescription: EMPTY_DESCRIPTION,
};

export const ThreeCurves = () => <CalcChart {...common} instances={INSTANCES} isLoading={false} rows={ROWS} />;

// A single curve — the legend collapses to one entry and the snapshot marker
// still sits at the enemy preset's DEF.
export const SingleCurve = () => <CalcChart {...common} instances={[INSTANCES[0]]} isLoading={false} rows={ROWS} />;

// Recalculating after a config change: the curves stay on screen under the
// "Calculating" pill.
export const Calculating = () => <CalcChart {...common} instances={INSTANCES} isLoading rows={ROWS} />;

// Every instance toggled off from its card or the legend.
export const AllCurvesHidden = () => <CalcChart {...common} instances={INSTANCES.map((i) => ({ ...i, visible: false }))} isLoading={false} rows={ROWS} />;

// First run: nothing added yet.
export const NoOperatorsYet = () => <CalcChart {...common} instances={[]} isLoading={false} rows={[]} />;
