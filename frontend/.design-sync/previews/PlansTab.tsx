import { PlansTab } from "frontend";

// Plans come from `/planner/public?uid=…`. Server functions are stubbed in the
// design-system bundle, so the honest render of this tab is its empty state:
// no plans pinned to the profile.
const roster = [
    { user_id: "1000048871", operator_id: "char_4064_mlynar", elite: 2, level: 90, exp: 0, potential: 5, skill_level: 7, favor_point: 25570, skin_id: null, default_skill: 2, voice_lan: "JP", current_equip: "uniequip_002_mlynar", current_tmpl: null, obtained_at: 1671840000, masteries: [{ index: 0, mastery: 3 }], modules: [{ id: "uniequip_002_mlynar", level: 3, locked: false }] },
    { user_id: "1000048871", operator_id: "char_263_skadi", elite: 2, level: 82, exp: 0, potential: 2, skill_level: 7, favor_point: 18400, skin_id: null, default_skill: 2, voice_lan: "CN_MANDARIN", current_equip: null, current_tmpl: null, obtained_at: 1580515200, masteries: [], modules: [] },
];

export const NoPublicPlans = () => (
    <div className="mx-auto max-w-2xl">
        <PlansTab uid="1000048871" roster={roster} />
    </div>
);
