import { TagSelector } from "frontend";

// The tag grid of the recruitment calculator, grouped Qualification → Position →
// Class → Affix. Every tag below is a real `gachaTags` row from
// /api/static/gacha (Male/Female are filtered out upstream).

const tag = (id: number, name: string, type: string) => ({ id, name, type });

const GROUPS = {
    qualification: [tag(11, "Top Operator", "qualification"), tag(14, "Senior Operator", "qualification"), tag(17, "Starter", "qualification"), tag(28, "Robot", "qualification")],
    position: [tag(9, "Melee", "position"), tag(10, "Ranged", "position")],
    class: [
        tag(1, "Guard", "class"),
        tag(2, "Sniper", "class"),
        tag(3, "Defender", "class"),
        tag(4, "Medic", "class"),
        tag(5, "Supporter", "class"),
        tag(6, "Caster", "class"),
        tag(7, "Specialist", "class"),
        tag(8, "Vanguard", "class"),
    ],
    affix: [
        tag(12, "Crowd-Control", "affix"),
        tag(13, "Nuker", "affix"),
        tag(15, "Healing", "affix"),
        tag(16, "Support", "affix"),
        tag(18, "DP-Recovery", "affix"),
        tag(19, "DPS", "affix"),
        tag(20, "Survival", "affix"),
        tag(21, "AoE", "affix"),
        tag(22, "Defense", "affix"),
        tag(23, "Slow", "affix"),
        tag(24, "Debuff", "affix"),
        tag(25, "Fast-Redeploy", "affix"),
        tag(26, "Shift", "affix"),
        tag(27, "Summon", "affix"),
        tag(29, "Elemental", "affix"),
    ],
};

const noop = () => {};

export const NoneSelected = () => (
    <div className="max-w-sm">
        <TagSelector groups={GROUPS} maxReached={false} onToggle={noop} selectedTagIds={new Set<number>()} />
    </div>
);

// A typical mid-roll board: three of the five tags picked.
export const PartiallySelected = () => (
    <div className="max-w-sm">
        <TagSelector groups={GROUPS} maxReached={false} onToggle={noop} selectedTagIds={new Set([14, 4, 15])} />
    </div>
);

// Five tags in: every unselected tag is disabled until one is removed.
export const MaxReached = () => (
    <div className="max-w-sm">
        <TagSelector groups={GROUPS} maxReached onToggle={noop} selectedTagIds={new Set([11, 2, 10, 19, 25])} />
    </div>
);
