import { CommunitySharePill } from "frontend";

// The share of E2 owners who picked one skill (or, for modules, the share of
// owners with a module equipped). `IChoiceShare` comes from
// `useCommunityDefaults`: `rank` 0 is the community's most-picked option and
// gets the primary tint; everything else is muted. Under 10% the label keeps one
// decimal so 0.4% and 4.4% stay distinguishable. `share: undefined` renders
// nothing at all - never "0%" - because a missing figure means "cannot say"
// (cohort under the 50-owner reporting floor), not "nobody picked this".

const SKILL_ICON = (id: string) => `https://api.myrtle.moe/api/assets/textures/spritepack/skill_icons_0/skill_icon_${id}.png`;

// Młynar's E2 cohort. Shares sum to 1 across the three skills.
const E2_OWNERS = 12_418;
const MLYNAR_SKILLS = [
    { skillId: "skchr_mlynar_1", name: "Unvoiced Anger", share: { users: 571, share: 0.046, rank: 2 } },
    { skillId: "skchr_mlynar_2", name: "Unresolved Sorrow", share: { users: 2_980, share: 0.24, rank: 1 } },
    { skillId: "skchr_mlynar_3", name: "Unbrilliant Glory", share: { users: 8_867, share: 0.714, rank: 0 } },
];

// The skill selector on the operator page's Skills tab: one chip per skill,
// the open one in primary, each carrying its pill. This is the pill's home.
const SkillChip = ({ name, image, selected, children }: { name: string; image: string; selected?: boolean; children: React.ReactNode }) => (
    <button type="button" className={selected ? "flex w-full items-center gap-2 rounded-lg border border-primary bg-primary/10 px-4 py-2 font-medium text-primary text-sm transition-colors sm:w-auto" : "flex w-full items-center gap-2 rounded-lg border border-border bg-secondary/30 px-4 py-2 font-medium text-muted-foreground text-sm transition-colors hover:border-primary/50 hover:text-foreground sm:w-auto"}>
        <img alt="" className="h-5 w-5 shrink-0 object-contain" src={image} />
        {name}
        {children}
    </button>
);

export const SkillSelector = () => (
    <div className="flex max-w-2xl flex-wrap gap-2">
        {MLYNAR_SKILLS.map((skill, idx) => (
            <SkillChip key={skill.skillId} name={skill.name} image={SKILL_ICON(skill.skillId)} selected={idx === 2}>
                <CommunitySharePill className="ms-auto sm:ms-0" share={skill.share} total={E2_OWNERS} cohort="E2 owners use this skill" />
            </SkillChip>
        ))}
    </div>
);

// The module dropdown on the Information tab renders one pill per module
// option, over a different denominator: owners with an ADVANCED module equipped
// (unequipped and ORIGINAL are not a preference). The selected module's share is
// restated in a line under the trigger, since the collapsed trigger only shows
// the designator.
const MODULE_OWNERS = 9_306;
const MLYNAR_MODULES = [
    { id: "uniequip_002_mlynar", label: "LIB-X", share: { users: 7_249, share: 0.779, rank: 0 } },
    { id: "uniequip_003_mlynar", label: "LIB-Y", share: { users: 2_057, share: 0.221, rank: 1 } },
];

export const ModuleOptions = () => (
    <div className="flex max-w-xs flex-col gap-3">
        <ul className="m-0 flex list-none flex-col gap-1 rounded-lg border border-border bg-popover p-1">
            <li className="rounded-md px-2 py-1.5 text-sm">No Module</li>
            {MLYNAR_MODULES.map((mod, idx) => (
                <li key={mod.id} className={idx === 0 ? "rounded-md bg-accent px-2 py-1.5 text-sm" : "rounded-md px-2 py-1.5 text-sm"}>
                    <span className="flex items-center gap-2">
                        {mod.label}
                        <CommunitySharePill share={mod.share} total={MODULE_OWNERS} cohort="owners with a module equipped use this" />
                    </span>
                </li>
            ))}
        </ul>
        <p className="text-[11px] text-muted-foreground">78% of the {MODULE_OWNERS.toLocaleString()} owners with a module equipped use this one</p>
    </div>
);

// The three faces of the pill side by side: the top pick in primary, a
// runner-up in the muted secondary, and a sub-10% share keeping its decimal.
export const Variants = () => (
    <div className="flex flex-col gap-3 text-sm">
        <div className="flex items-center gap-3">
            <span className="w-32 text-muted-foreground">Most common</span>
            <CommunitySharePill share={{ users: 8_867, share: 0.714, rank: 0 }} total={E2_OWNERS} cohort="E2 owners use this skill" />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-32 text-muted-foreground">Runner-up</span>
            <CommunitySharePill share={{ users: 2_980, share: 0.24, rank: 1 }} total={E2_OWNERS} cohort="E2 owners use this skill" />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-32 text-muted-foreground">Under 10%</span>
            <CommunitySharePill share={{ users: 571, share: 0.046, rank: 2 }} total={E2_OWNERS} cohort="E2 owners use this skill" />
        </div>
        <div className="flex items-center gap-3">
            <span className="w-32 text-muted-foreground">Rare pick</span>
            <CommunitySharePill share={{ users: 37, share: 0.003, rank: 3 }} total={E2_OWNERS} cohort="E2 owners use this skill" />
        </div>
    </div>
);

// An operator whose E2 cohort is under the reporting floor: every share is
// `undefined`, so the chips render with no pill at all. Nothing says "0%".
export const BelowReportingFloor = () => (
    <div className="flex max-w-2xl flex-wrap gap-2">
        {MLYNAR_SKILLS.map((skill, idx) => (
            <SkillChip key={skill.skillId} name={skill.name} image={SKILL_ICON(skill.skillId)} selected={idx === 2}>
                <CommunitySharePill className="ms-auto sm:ms-0" share={undefined} total={0} cohort="E2 owners use this skill" />
            </SkillChip>
        ))}
    </div>
);
