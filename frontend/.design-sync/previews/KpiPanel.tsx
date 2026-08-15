import { KpiPanel } from "frontend";

// The snapshot strip under the chart: one row per configured operator with the
// three result columns, a crown on the leader, and a signed percentage against
// the leader for everyone else. Skill/module names come from the operator-list
// query, which is stubbed in previews, so the summary line falls back to
// "S3 · M3 · Mod 1".

const op = (id: string, name: string) => ({ id, name, availableSkills: [1, 2, 3], availableModules: [0, 1, 2], defaultSkill: 3, defaultModule: 1, conditionals: [] });

const config = { promotion: 2, level: 90, potential: 1, trust: 100, skillIndex: 3, skillRank: 10, moduleIndex: 1, moduleLevel: 3, buffs: {}, conditionals: {}, allCond: true };

const instance = (uid: string, id: string, name: string, color: string, visible = true) => ({ uid, op: op(id, name), color, visible, collapsed: false, config });

const DPS_INSTANCES = [instance("i1", "char_4064_mlynar", "Młynar", "oklch(0.65 0.20 30)"), instance("i2", "char_1035_wisdel", "Wiš'adel", "oklch(0.62 0.18 220)"), instance("i3", "char_2012_typhon", "Typhon", "oklch(0.62 0.18 145)", false)];

const DPS_COLUMNS = [
    { key: "skill_dps", label: "Skill DPS" },
    { key: "average_dps", label: "Avg DPS" },
    { key: "total_damage", label: "Total" },
];

const snap = (uid: string, data: Record<string, number> | undefined, over: Record<string, unknown> = {}) => ({ uid, data, isPending: false, error: null, ...over });

// Wiš'adel leads on Skill DPS against a DEF 1,000 elite; the other two carry
// their deficit against her.
export const OperatorComparison = () => (
    <KpiPanel
        columns={DPS_COLUMNS}
        instances={DPS_INSTANCES}
        leaderKey="skill_dps"
        leaderLabel="Skill DPS"
        snapshots={[snap("i1", { skill_dps: 4182, average_dps: 2410, total_damage: 125_460 }), snap("i2", { skill_dps: 5037, average_dps: 3186, total_damage: 201_480 }), snap("i3", { skill_dps: 3164, average_dps: 2098, total_damage: 158_200 })]}
    />
);

// HPS columns, with the `hint` callback firing on a burst healer's 0 base HPS.
export const HealerComparison = () => (
    <KpiPanel
        columns={[
            { key: "skill_hps", label: "Skill HPS" },
            { key: "base_hps", label: "Base HPS", hint: (d: Record<string, number>) => (d.base_hps === 0 ? "Burst healer - heals only on skill activation, so there's no off-skill (base) healing." : undefined) },
            { key: "avg_hps", label: "Avg HPS" },
        ]}
        instances={[instance("h1", "char_179_cgbird", "Nightingale", "oklch(0.65 0.20 30)"), instance("h2", "char_128_plosis", "Ptilopsis", "oklch(0.62 0.18 220)")]}
        leaderKey="avg_hps"
        leaderLabel="Average HPS"
        snapshots={[snap("h1", { skill_hps: 1864, base_hps: 0, avg_hps: 742 }), snap("h2", { skill_hps: 1120, base_hps: 686, avg_hps: 894 })]}
    />
);

// Results still in flight: each value cell holds a spinner.
export const Calculating = () => <KpiPanel columns={DPS_COLUMNS} instances={DPS_INSTANCES.slice(0, 2)} leaderKey="skill_dps" leaderLabel="Skill DPS" snapshots={[snap("i1", undefined, { isPending: true }), snap("i2", undefined, { isPending: true })]} />;

// One operator's request failed — its name picks up a destructive warning icon
// while the rest of the panel keeps reporting.
export const OneCalculationFailed = () => (
    <KpiPanel
        columns={DPS_COLUMNS}
        instances={DPS_INSTANCES.slice(0, 2)}
        leaderKey="skill_dps"
        leaderLabel="Skill DPS"
        snapshots={[snap("i1", { skill_dps: 4182, average_dps: 2410, total_damage: 125_460 }), snap("i2", undefined, { error: new Error("Module 2 is not supported for this operator yet.") })]}
    />
);
