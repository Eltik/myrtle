import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.stages.eligibility": {
        text: "Eligibility",
        description: "Heading over the switches deciding which stages may be drawn at all.",
    },
    "randomizer.stages.onlyAvailable": {
        text: "Only currently available",
        description: "Switch label: skip content that is not running right now.",
    },
    "randomizer.stages.onlyAvailable.desc": {
        text: "Hide events that aren't open right now (permanent events stay).",
        description: "Explanation under the 'Only currently available' switch.",
    },
    "randomizer.stages.onlyCleared": {
        text: "Only stages I've cleared",
        description: "Switch label: draw only from stages the player has finished.",
    },
    "randomizer.stages.onlyCleared.desc": {
        text: "Deselects every stage you haven't cleared in your profile.",
        description: "Explanation under the 'Only stages I've cleared' switch; the profile is the game account linked to this site.",
    },
    "randomizer.stages.pool": {
        text: "Stage pool",
        description: "Heading over the searchable list of stages that may be drawn.",
    },
    "randomizer.stages.search": {
        text: "Search events or stages…",
        description: "Placeholder in the stage search box. Keep the single ellipsis character.",
    },
    "randomizer.stages.clearSearch": {
        text: "Clear search",
        description: "Accessible name of the small cross that empties the search box.",
    },
    "randomizer.stages.count": {
        text: "stages",
        description: "Follows a 'selected / total' count, e.g. '812 / 1204 stages'. Lowercase; set in small caps.",
    },
    "randomizer.stages.all": {
        text: "All",
        description: "Button that selects every stage. Very narrow button.",
    },
    "randomizer.stages.none": {
        text: "None",
        description: "Button that deselects every stage. Very narrow button.",
    },
    "randomizer.stages.noMatch": {
        text: "No events match.",
        description: "Shown in place of the stage list when the search matches nothing.",
    },
    "randomizer.stages.sectionCount": {
        text: "{selected}/{total}",
        description: "How many stages in one section are selected, e.g. '48/60'.",
    },
    "randomizer.stages.groupCount": {
        text: "{selected}/{total}",
        description: "How many stages in one event or chapter are selected, e.g. '9/12'.",
    },
    "randomizer.stages.toggleGroup": {
        text: "Toggle {group}",
        description: "Accessible name of the checkbox that selects or clears a whole event or chapter. {group} is its name.",
    },
    "randomizer.stages.toggleStage": {
        text: "Toggle {stage}",
        description: "Accessible name of one stage's checkbox. {stage} is the stage code, optionally followed by a dash and its title.",
    },
    "randomizer.stages.stageLabel": {
        text: "{code} - {name}",
        description: "Names a stage by its code and title for the checkbox label. Both halves come from the game data; the dash is a plain hyphen.",
    },
    "randomizer.stages.tag.mainline": {
        text: "Mainline",
        description: "Sub-label under a main-story chapter. Community jargon for the main campaign.",
    },
    "randomizer.stages.tag.mini": {
        text: "Mini",
        description: "Sub-label under a small side event, shown when nothing more specific applies.",
    },
    "randomizer.stages.tag.event": {
        text: "Event",
        description: "Sub-label under a side story, shown when nothing more specific applies.",
    },
    "randomizer.stages.badge.adverse": {
        text: "ADV",
        description: "Badge marking the harder variant of a main-story stage. 'Adverse' is the game's own name for it; keep it to a few letters.",
    },
    "randomizer.stages.badge.story": {
        text: "STR",
        description: "Badge marking a story-only stage with no combat. Abbreviation of 'story'; keep it to a few letters.",
    },
    "randomizer.stages.badge.cm": {
        text: "CM",
        description: "Badge marking a Challenge Mode stage. 'CM' is the game's own abbreviation and normally stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
