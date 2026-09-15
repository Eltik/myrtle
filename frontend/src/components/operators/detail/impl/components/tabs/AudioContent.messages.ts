import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "audio.title": {
        text: "Audio / SFX",
        description: "Heading of the Audio tab. 'SFX' is the usual shorthand for sound effects.",
    },
    "audio.subtitle": {
        text: "Operator voice lines and battle sound effects.",
        description: "Blurb under the Audio tab heading.",
    },
    "audio.tab.voice": {
        text: "Voice Lines",
        description: "Tab switching to the operator's recorded dialogue.",
    },
    "audio.tab.sfx": {
        text: "Battle SFX",
        description: "Tab switching to the in-combat sound effects. 'SFX' is the usual shorthand for sound effects.",
    },
    "audio.mute": {
        text: "Mute",
        description: "Accessible name of the speaker button while sound is on.",
    },
    "audio.unmute": {
        text: "Unmute",
        description: "Accessible name of the speaker button while sound is muted.",
    },
    "audio.volume": {
        text: "Volume",
        description: "Accessible name of the volume slider.",
    },
    "audio.play": {
        text: "Play",
        description: "Accessible name of the play button on an audio row.",
    },
    "audio.pause": {
        text: "Pause",
        description: "Accessible name of the play button while that clip is playing.",
    },
    "audio.unavailable": {
        text: "Unavailable",
        description: "Accessible name of the play button, and the badge on the row, when the clip cannot be fetched.",
    },
    "audio.download": {
        text: "Download",
        description: "Tooltip on the download button of an audio row.",
    },
    "audio.download.voice": {
        text: "Download voice line",
        description: "Accessible name of the download button on a voice-line row.",
    },
    "audio.download.sfx": {
        text: "Download sound effect",
        description: "Accessible name of the download button on a sound-effect row.",
    },
    "audio.cv": {
        text: "CV:",
        description: "Label before the voice actor's name. 'CV' is the standard credit abbreviation for character voice. Keep the colon.",
    },
    "audio.language": {
        text: "Language",
        description: "Prompt in the voice-language select before a choice is made.",
    },
    "audio.category.all": {
        text: "All",
        description: "First tab in both audio lists: every clip, unfiltered.",
    },
    "audio.search.voice.placeholder": {
        text: "Search voice lines...",
        description: "Prompt inside the empty voice-line search box. Keep the three dots as written.",
    },
    "audio.search.voice.label": {
        text: "Search voice lines",
        description: "Accessible name of the voice-line search box.",
    },
    "audio.search.sfx.placeholder": {
        text: "Search sound effects...",
        description: "Prompt inside the empty sound-effect search box. Keep the three dots as written.",
    },
    "audio.search.sfx.label": {
        text: "Search sound effects",
        description: "Accessible name of the sound-effect search box.",
    },
    "audio.search.clear": {
        text: "Clear search",
        description: "Accessible name of the x that empties a search box.",
    },
    "audio.empty.voiceSearch": {
        text: "No voice lines match “{query}”.",
        description: "Empty state when a voice-line search matches nothing. Keep the typographic quotation marks, or use this language's own quote pair.",
    },
    "audio.empty.voice": {
        text: "No voice data available for this operator.",
        description: "Empty state when the game data carries no voice lines at all.",
    },
    "audio.empty.sfxSearch": {
        text: "No sound effects match “{query}”.",
        description: "Empty state when a sound-effect search matches nothing. Keep the typographic quotation marks, or use this language's own quote pair.",
    },
    "audio.empty.sfx": {
        text: "No battle audio available for this operator.",
        description: "Empty state when the game data carries no battle sounds at all.",
    },
    "audio.showMore": {
        text: "Show more",
        description: "Button that reveals the rest of a clipped voice line.",
    },
    "audio.showLess": {
        text: "Show less",
        description: "Button that re-clips an expanded voice line.",
    },
    "audio.altSkin": {
        text: "Alt. skin",
        description: "Badge marking a sound that belongs to an alternate outfit rather than the default one. Abbreviated; the badge is tiny.",
    },
    "audio.variants": {
        text: "{count, plural, one {# variant} other {# variants}}",
        description: "Toggle showing how many interchangeable takes of one sound the game ships.",
    },
    "audio.playVariant": {
        text: "Play variant {index}",
        description: "Accessible name of one numbered take button. {index} counts from 1.",
    },
    "audio.sfx.skillSlot": {
        text: "Skill {slot}",
        description: "Row label for a numbered skill's sound, e.g. 'Skill 2'.",
    },
    "audio.sfx.skill": {
        text: "Skill",
        description: "Row label for a skill sound the game data does not number.",
    },
    "audio.sfx.deployment": {
        text: "Deployment",
        description: "Row label for the sound of placing the operator on the map.",
    },
    "audio.sfx.skillSlotVoice": {
        text: "Skill {slot} Voice",
        description: "Row label for the line the operator says when a numbered skill fires.",
    },
    "audio.sfx.battleVoice": {
        text: "Battle Voice",
        description: "Row label for in-combat lines not tied to a numbered skill.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
