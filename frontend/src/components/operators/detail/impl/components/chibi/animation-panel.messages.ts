import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibiClips.title": {
        text: "Animations",
        description: "Heading of the sprite viewer's list of animation clips.",
    },
    "chibiClips.count": {
        text: "{count}",
        description: "Number of clips, shown as a small badge beside the list heading.",
    },
    "chibiClips.group.idle": {
        text: "Idle",
        description: "Clip group: standing, resting and dormitory loops.",
    },
    "chibiClips.group.move": {
        text: "Move & entry",
        description: "Clip group: walking clips and the deployment entrance ('Start').",
    },
    "chibiClips.group.attack": {
        text: "Attack",
        description: "Clip group: normal attacks.",
    },
    "chibiClips.group.skill": {
        text: "Skill",
        description: "Clip group: skill clips, in play order (begin, loop, idle, end).",
    },
    "chibiClips.group.other": {
        text: "Other",
        description: "Clip group: everything else, such as the defeat clip.",
    },
    "chibiClips.frames": {
        text: "{count} F",
        description: "A clip's length in frames, e.g. '180 F'. 'F' abbreviates frames; use this language's short form.",
    },
    "chibiClips.events": {
        text: "{count, plural, one {# event} other {# events}}",
        description: "Accessible name of the marker showing how many timed events (hit, sound or effect cues) a clip fires.",
    },
    "chibiClips.event": {
        text: "{name} · {seconds} s",
        description: "One event in the marker's tooltip: its name from the animation file and when it fires, e.g. 'OnAttack · 0.33 s'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
