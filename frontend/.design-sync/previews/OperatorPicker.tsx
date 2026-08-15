import { OperatorPicker } from "frontend";

// The "Add an operator" combobox that seeds a calculator instance. It fuzzy
// matches on name, appellation, class, sub-class, rarity, tags and nation, and
// its footer copy changes with how many instances already exist. The popup only
// opens on focus/typing, so the card captures the resting trigger.

const entry = (id: string, name: string, availableSkills: number[], availableModules: number[], defaultSkill: number, defaultModule: number) => ({
    id,
    name,
    availableSkills,
    availableModules,
    defaultSkill,
    defaultModule,
    conditionals: [],
});

const DPS_OPERATORS = [
    entry("char_4064_mlynar", "Młynar", [1, 2, 3], [0, 1, 2], 3, 1),
    entry("char_1035_wisdel", "Wiš'adel", [1, 2, 3], [0, 1], 3, 1),
    entry("char_2012_typhon", "Typhon", [1, 2, 3], [0, 1], 3, 1),
    entry("char_180_amgoat", "Eyjafjalla", [1, 2, 3], [0, 1, 2], 3, 1),
    entry("char_103_angel", "Exusiai", [1, 2, 3], [0, 1, 2], 3, 1),
    entry("char_172_svrash", "SilverAsh", [1, 2, 3], [0, 1, 2], 3, 1),
    entry("char_350_surtr", "Surtr", [1, 2, 3], [0, 1], 3, 1),
    entry("char_263_skadi", "Skadi", [1, 2, 3], [0, 1, 2], 3, 1),
    entry("char_102_texas", "Texas", [1, 2], [0, 1, 2], 2, 1),
    entry("char_140_whitew", "Lappland", [1, 2], [0, 1, 2], 2, 1),
];

const HPS_OPERATORS = [entry("char_179_cgbird", "Nightingale", [1, 2, 3], [0, 1], 3, 1), entry("char_128_plosis", "Ptilopsis", [1, 2], [0, 1], 2, 1), entry("char_171_bldsk", "Warfarin", [1, 2], [0, 1], 2, 1), entry("char_187_ccheal", "Gavial", [1, 2], [0, 1], 2, 1)];

const noop = () => {};

// First run: the label reads "Add an operator" and the hint explains that the
// same operator can be added twice to compare builds.
export const FirstOperator = () => (
    <div className="max-w-sm">
        <OperatorPicker error={undefined} existingCount={0} isError={false} isLoading={false} noun="operator" onAdd={noop} operators={DPS_OPERATORS} />
    </div>
);

// One instance in: the label flips to "Add another operator" and the hint
// becomes the duplicate-build tip.
export const AddAnother = () => (
    <div className="max-w-sm">
        <OperatorPicker error={undefined} existingCount={1} isError={false} isLoading={false} noun="operator" onAdd={noop} operators={DPS_OPERATORS} />
    </div>
);

// The HPS calculator reuses it with `noun="healer"`, which rewrites every label.
export const HealerVariant = () => (
    <div className="max-w-sm">
        <OperatorPicker error={undefined} existingCount={2} isError={false} isLoading={false} noun="healer" onAdd={noop} operators={HPS_OPERATORS} />
    </div>
);

// The list endpoint is down: the picker stays usable but explains itself.
export const ListUnavailable = () => (
    <div className="max-w-sm">
        <OperatorPicker error={new Error("Failed to load DPS operators: 503")} existingCount={0} isError isLoading={false} noun="operator" onAdd={noop} operators={[]} />
    </div>
);
