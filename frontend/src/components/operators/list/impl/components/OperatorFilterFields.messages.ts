import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.section.basic": {
        text: "Basic",
        description: "Small uppercase divider over the everyday filters (class, rarity).",
    },
    "filters.section.advanced": {
        text: "Advanced",
        description: "Small uppercase divider over the collapsible, rarely-used filters.",
    },
    "filters.rarity": {
        text: "Rarity",
        description: "Field label over the row of star-rating buttons.",
    },
    "filters.archetype": {
        text: "Archetype",
        description: "Field label for the subclass filter. The archetype names themselves are game vocabulary and are not in this catalog.",
    },
    "filters.archetype.placeholder": {
        text: "Select archetype",
        description: "Prompt inside the empty archetype multi-select.",
    },
    "filters.gender": {
        text: "Gender",
        description: "Field label over the gender buttons. The values come from the game data and are not translated here.",
    },
    "filters.nation": {
        text: "Nation",
        description: "Field label for the nation filter. The nation names are game vocabulary and are not in this catalog.",
    },
    "filters.nation.placeholder": {
        text: "Select nation",
        description: "Prompt inside the empty nation multi-select.",
    },
    "filters.faction": {
        text: "Faction",
        description: "Field label for the faction filter. The faction names are game vocabulary and are not in this catalog.",
    },
    "filters.faction.placeholder": {
        text: "Select faction",
        description: "Prompt inside the empty faction multi-select.",
    },
    "filters.race": {
        text: "Race",
        description: "Field label for the race filter. The values come from the game data.",
    },
    "filters.race.placeholder": {
        text: "Select race",
        description: "Prompt inside the empty race multi-select.",
    },
    "filters.birthPlace": {
        text: "Place of Birth",
        description: "Field label for the birthplace filter. The place names are game vocabulary.",
    },
    "filters.birthPlace.placeholder": {
        text: "Select birth place",
        description: "Prompt inside the empty birthplace multi-select.",
    },
    "filters.artist": {
        text: "Artist",
        description: "Field label for the illustrator filter. The names themselves are never translated.",
    },
    "filters.artist.placeholder": {
        text: "Select artist",
        description: "Prompt inside the empty artist multi-select.",
    },
    "filters.voiceActor": {
        text: "Voice Actor",
        description: "Field label for the voice-actor filter. The names themselves are never translated.",
    },
    "filters.voiceActor.placeholder": {
        text: "Select voice actor",
        description: "Prompt inside the empty voice-actor multi-select.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
