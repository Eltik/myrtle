import { OptimizerTab } from "frontend";
import { type ReactNode, useEffect } from "react";

// OptimizerTab is the profile's Optimizer tab: an underline tab strip over the
// registry in `optimizers.tsx` - one tab per optimizer, each panel a blurb and
// either the optimizer's component or `ComingSoon`. The Base Optimizer is a
// data-coupled container whose server functions are stubbed in the design
// bundle, so its panel settles into the no-synced-base state; that is the
// honest preview of the default tab. The second story switches to the
// Account Optimizer tab, which is the registry's `ComingSoon` entry.

const PHASES_6 = [{ maxLevel: 50 }, { maxLevel: 80 }, { maxLevel: 90 }];

const op = (id: string, name: string, profession: string, subProfessionId: string) => ({
    id,
    name,
    rarity: "TIER_6",
    profession,
    subProfessionId,
    isNotObtainable: false,
    phases: PHASES_6,
    skills: [{ skillId: `${id}_s1` }, { skillId: `${id}_s2` }, { skillId: `${id}_s3` }],
    modules: [],
    potentialRanks: [{}, {}, {}, {}, {}],
    baseSkills: [],
});

const entry = (operatorId: string, elite: number, level: number) => ({
    user_id: "1000123456",
    operator_id: operatorId,
    elite,
    level,
    exp: 0,
    potential: 1,
    skill_level: 7,
    favor_point: 25_570,
    skin_id: null,
    default_skill: 0,
    voice_lan: "JP",
    current_equip: null,
    current_tmpl: null,
    obtained_at: 1_690_000_000,
    masteries: [0, 1, 2].map((index) => ({ index, mastery: 0 })),
    modules: [],
});

const OPERATORS_STATIC = [op("char_272_strong", "Jaye", "SPECIAL", "merchant"), op("char_254_vodfox", "Shamare", "SUPPORT", "bard"), op("char_190_clour", "Vermeil", "SNIPER", "fastshot"), op("char_002_amiya", "Amiya", "CASTER", "corecaster")];

const ROSTER = [entry("char_272_strong", 2, 60), entry("char_254_vodfox", 1, 55), entry("char_190_clour", 2, 40), entry("char_002_amiya", 2, 70)];

// The active tab is internal state; the tab strip is the only way to move it.
// Base UI activates a tab on click, so click the nth tab two frames after
// mount (an effect-time click is dropped) and blur it so the focus ring does
// not read as part of the design.
function Stage({ tab, children }: { tab?: number; children: ReactNode }) {
    useEffect(() => {
        if (tab === undefined) return;
        let f2 = 0;
        let f3 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                document.querySelectorAll<HTMLElement>('[role="tab"]')[tab]?.click();
                f3 = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur());
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            cancelAnimationFrame(f3);
        };
    }, [tab]);
    return <div className="mx-auto max-w-3xl">{children}</div>;
}

/** The default tab: the Base Optimizer, settled into its no-synced-base state. */
export const BaseOptimizerTab = () => (
    <Stage>
        <OptimizerTab operatorsStatic={OPERATORS_STATIC} roster={ROSTER} uid="1000123456" />
    </Stage>
);

/** The Account Optimizer tab: a registry entry with no component yet. */
export const AccountOptimizerTab = () => (
    <Stage tab={1}>
        <OptimizerTab operatorsStatic={OPERATORS_STATIC} roster={ROSTER} uid="1000123456" />
    </Stage>
);
