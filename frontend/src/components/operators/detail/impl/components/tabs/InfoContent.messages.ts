import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "info.title": {
        text: "Operator Information",
        description: "Heading of the Information tab. The blurb under it is the operator's trait text from the game data.",
    },
    "info.profile": {
        text: "Profile",
        description: "Heading of the collapsible panel holding the operator's personal details.",
    },
    "info.profile.placeOfBirth": {
        text: "Place of Birth",
        description: "Profile field label; its value is a place name from the game data.",
    },
    "info.profile.race": {
        text: "Race",
        description: "Profile field label; its value comes from the game data.",
    },
    "info.profile.gender": {
        text: "Gender",
        description: "Profile field label; its value comes from the game data.",
    },
    "info.profile.height": {
        text: "Height",
        description: "Profile field label; its value comes from the game data.",
    },
    "info.profile.artist": {
        text: "Artist",
        description: "Profile field label for the illustrator. The names themselves are never translated.",
    },
    "info.profile.unknown": {
        text: "Unknown",
        description: "Stand-in for a profile field the game data leaves blank.",
    },
    "info.controls": {
        text: "Operator Controls",
        description: "Heading of the collapsible panel holding the level, promotion, potential, module and trust controls.",
    },
    "info.controls.desc": {
        text: "Adjust to see how stats change at different levels, promotions, potentials, modules, and trust.",
        description: "Caption under the Operator Controls heading. 'Potential' and 'trust' are the game's own progression systems.",
    },
    "info.promotion": {
        text: "Promotion:",
        description: "Label before the row of Elite promotion buttons. Keep the colon.",
    },
    "info.eliteAlt": {
        text: "Elite {elite}",
        description: "Alt text of one promotion icon. 'Elite' is the game's promotion tier and {elite} is 0, 1 or 2.",
    },
    "info.level": {
        text: "Level",
        description: "Label beside the slider that sets the operator's level.",
    },
    "info.trust": {
        text: "Trust",
        description: "Label beside the slider that sets trust. 'Trust' is the game's own affinity stat.",
    },
    "info.potential": {
        text: "Potential",
        description: "Label over the row of potential-rank buttons. 'Potential' is the game's duplicate-copy upgrade system.",
    },
    "info.potential.tip": {
        text: "Select a potential rank to see stat bonuses.",
        description: "Hover text on the info icon beside the Potential label.",
    },
    "info.potential.alt": {
        text: "Pot {rank}",
        description: "Alt text of one potential icon. 'Pot' is short for potential; {rank} counts from 0.",
    },
    "info.potential.rankTip": {
        text: "Pot {rank}: {description}",
        description: "Hover text on a potential button. {description} is the bonus text from the game data.",
    },
    "info.potential.baseTip": {
        text: "Pot {rank}: Base potential",
        description: "Hover text on the first potential button, which grants nothing.",
    },
    "info.module": {
        text: "Module",
        description: "Label over the module select. 'Module' is the game's name for the equipment system.",
    },
    "info.module.placeholder": {
        text: "Select module",
        description: "Prompt in the module select before a choice is made.",
    },
    "info.module.none": {
        text: "No Module",
        description: "Module option meaning no module is equipped.",
    },
    "info.module.cohort": {
        text: "owners with a module equipped use this",
        description: "Names the group behind the share pill on a module option, completing 'N of M ...'.",
    },
    "info.module.shareLine": {
        text: "{pct}% of the {total} owners with a module equipped use this one",
        description: "Restates the selected module's community share under the select. {pct} is a whole number and {total} a formatted count.",
    },
    "info.module.level": {
        text: "Module Level",
        description: "Label over the module-level select.",
    },
    "info.module.levelPlaceholder": {
        text: "Select level",
        description: "Prompt in the module-level select before a choice is made.",
    },
    "info.module.levelOption": {
        text: "Level {level}",
        description: "One module-level option, e.g. 'Level 3'.",
    },
    "info.module.breakdown.title": {
        text: "Community module level",
        description: "Heading of the bar chart showing how far owners have upgraded this module.",
    },
    "info.module.breakdown.notUnlocked": {
        text: "Not unlocked",
        description: "Bar label for owners who have not unlocked the module. Spelled out rather than 'Lv0', which would read as a level someone reached.",
    },
    "info.module.breakdown.lv1": {
        text: "Lv1",
        description: "Bar label for module level 1. The game's own shorthand; very tight space.",
    },
    "info.module.breakdown.lv2": {
        text: "Lv2",
        description: "Bar label for module level 2. The game's own shorthand; very tight space.",
    },
    "info.module.breakdown.lv3": {
        text: "Lv3",
        description: "Bar label for module level 3. The game's own shorthand; very tight space.",
    },
    "info.module.breakdown.summary": {
        text: "unlocked this module",
        description: "Completes the headline 'N% have ...' over the module-level bars.",
    },
    "info.cohort.e2Owners": {
        text: "E2 owners",
        description: "Names the group the module-level bars are drawn from. 'E2' is the game's second promotion, the only stage at which modules exist.",
    },
    "info.combatStats": {
        text: "Combat Stats",
        description: "Heading over the two-column table of the operator's stats.",
    },
    "info.tags": {
        text: "Tags",
        description: "Heading over the operator's recruitment tags. The tags themselves come from the game data.",
    },
    "info.attackRange": {
        text: "Attack Range",
        description: "Heading over the grid showing which tiles the operator can hit.",
    },
    "info.attackRange.empty": {
        text: "No range data available.",
        description: "Shown instead of the range grid when the game data carries none.",
    },
    "info.moduleDetails": {
        text: "Module Details",
        description: "Heading of the collapsible panel describing the selected module.",
    },
    "info.moduleDetails.levelStats": {
        text: "Level {level} Stats",
        description: "Heading over the stat bonuses a module grants at the chosen level.",
    },
    "info.moduleDetails.noBonuses": {
        text: "No stat bonuses at this level.",
        description: "Shown when the chosen module level grants no stat changes.",
    },
    "info.moduleDetails.pickLevel": {
        text: "Select a module level to see its effects.",
        description: "Shown in the module panel while no level is chosen.",
    },
    "info.traitChanges": {
        text: "Trait Changes",
        description: "Heading over what the module changes about the operator's trait. 'Trait' is the game's always-on class ability.",
    },
    "info.showDiff.tip": {
        text: "Show diff vs. base",
        description: "Hover text on the switch that highlights what changed against the unmodified text. 'Diff' is short for difference.",
    },
    "info.baseTrait": {
        text: "Base Trait",
        description: "Small uppercase caption over the operator's unmodified trait text.",
    },
    "info.baseTalent": {
        text: "Base Talent",
        description: "Small uppercase caption over a talent's unmodified text.",
    },
    "info.changes": {
        text: "Changes",
        description: "Small uppercase caption over the modified text, below the unmodified version.",
    },
    "info.talentChanges": {
        text: "Talent Changes",
        description: "Heading over what the module changes about the operator's talents.",
    },
    "info.talents": {
        text: "Talents",
        description: "Heading of the collapsible panel listing the operator's talents. The talent names and text come from the game data.",
    },
    "info.talents.empty": {
        text: "No talents unlocked at this configuration.",
        description: "Shown when the chosen level, promotion and potential unlock no talents.",
    },
    "info.talents.unnamed": {
        text: "Unnamed Talent",
        description: "Stand-in heading for a talent the game data does not name.",
    },
    "info.talents.requiresPotential": {
        text: "Requires Potential {rank}",
        description: "Hover text on the potential icon beside a talent that needs duplicates to unlock.",
    },
    "info.talents.moduleBadge": {
        text: "Module",
        description: "Badge marking a talent the selected module has altered. Very short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
