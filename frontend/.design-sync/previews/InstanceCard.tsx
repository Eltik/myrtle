import { InstanceCard } from "frontend";

// One configured operator in the DPS/HPS calculators: promotion, level, potential,
// skill, skill rank, module, trust, the operator's own conditionals, and (when
// `onUpdateBuffs` is passed) an External buffs drawer.
//
// Skill and module *names* come from the global operator-list query, which is
// stubbed in previews, so the chips fall back to their S1/S2/S3 and M1/M2
// placeholders. Elite, potential and mastery icons are static assets and load
// normally.

type Conditional = { conditionalType: string; name: string; default: boolean; skills: number[]; modules: number[] };

const op = (id: string, name: string, conditionals: Conditional[] = []) => ({
    id,
    name,
    availableSkills: [1, 2, 3],
    availableModules: [0, 1, 2],
    defaultSkill: 3,
    defaultModule: 1,
    conditionals,
});

const config = (over: Record<string, unknown> = {}) => ({
    promotion: 2,
    level: 90,
    potential: 1,
    trust: 100,
    skillIndex: 3,
    skillRank: 10,
    moduleIndex: 1,
    moduleLevel: 3,
    buffs: {},
    conditionals: { talentDamage: true, moduleDamage: false },
    allCond: true,
    ...over,
});

const MLYNAR = op("char_4064_mlynar", "Młynar", [
    { conditionalType: "talent", name: "Molten Aegis fully stacked", default: true, skills: [2, 3], modules: [] },
    { conditionalType: "module", name: "LIB-X target below 50% HP", default: false, skills: [], modules: [1] },
]);

const EYJA = op("char_180_amgoat", "Eyjafjalla", [{ conditionalType: "talent", name: "Volcano — target has Arts Weakness", default: true, skills: [], modules: [] }]);

const noop = () => {};

const handlers = {
    onUpdate: noop,
    onToggleConditional: noop,
    onToggleVisibility: noop,
    onToggleCollapsed: noop,
    onMoveUp: noop,
    onMoveDown: noop,
    onDuplicate: noop,
    onRemove: noop,
};

export const MaxedBuild = () => (
    <div className="max-w-md">
        <InstanceCard {...handlers} index={0} inst={{ uid: "i1", op: MLYNAR, color: "oklch(0.65 0.20 30)", visible: true, collapsed: false, config: config() }} isFirst isLast={false} />
    </div>
);

// The DPS calculator passes `onUpdateBuffs`, which adds the External buffs
// disclosure at the bottom of the card.
export const WithExternalBuffs = () => (
    <div className="max-w-md">
        <InstanceCard
            {...handlers}
            index={1}
            inst={{ uid: "i2", op: EYJA, color: "oklch(0.62 0.18 220)", visible: true, collapsed: false, config: config({ promotion: 1, level: 70, potential: 3, trust: 50, skillIndex: 2, skillRank: 7, moduleIndex: 0, buffs: { atk: 0.4, flatAtk: 0, aspd: 55, fragile: 0.3 } }) }}
            isFirst={false}
            isLast
            onUpdateBuffs={noop}
        />
    </div>
);

// Collapsed to the summary line — how a long comparison list is kept readable.
export const Collapsed = () => (
    <div className="max-w-md">
        <InstanceCard {...handlers} index={2} inst={{ uid: "i3", op: MLYNAR, color: "oklch(0.62 0.18 145)", visible: true, collapsed: true, config: config() }} isFirst={false} isLast={false} />
    </div>
);

// An E1 build: masteries are locked out and the card explains why.
export const EliteOneLocked = () => (
    <div className="max-w-md">
        <InstanceCard {...handlers} index={3} inst={{ uid: "i4", op: EYJA, color: "oklch(0.70 0.16 65)", visible: false, collapsed: false, config: config({ promotion: 1, level: 70, skillIndex: 1, skillRank: 7, moduleIndex: 0, potential: 6, trust: 75 }) }} isFirst={false} isLast />
    </div>
);
