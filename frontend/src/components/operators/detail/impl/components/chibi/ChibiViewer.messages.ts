import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibi.title": {
        text: "Chibi Preview",
        description: "Heading of the animated sprite viewer. 'Chibi' is the fan term for the small in-battle sprite, used as-is in English.",
    },
    "chibi.view": {
        text: "View",
        description: "Prompt in the select that picks which sprite set is shown, before a choice is made.",
    },
    "chibi.view.front": {
        text: "Front",
        description: "Sprite set: the operator seen from the front, as when facing the player.",
    },
    "chibi.view.back": {
        text: "Back",
        description: "Sprite set: the operator seen from behind.",
    },
    "chibi.view.dorm": {
        text: "Dorm",
        description: "Sprite set: the base/dormitory sprite, which idles rather than fights. 'Dorm' is short for dormitory.",
    },
    "chibi.view.down": {
        text: "Down",
        description: "Sprite set: the knocked-out sprite.",
    },
    "chibi.meta": {
        text: "Spine {version} · {bones, plural, one {# bone} other {# bones}}",
        description: "Line under the viewer heading about the loaded animation file, shown in capitals, e.g. 'SPINE 3.8.99 · 259 BONES'. 'Spine' is the animation format's name; {version} its version number; bones are the skeleton's joints.",
    },
    "chibi.stage": {
        text: "Animation stage",
        description: "Accessible name of the area where the sprite plays, which can be dragged to pan and zoomed.",
    },
    "chibi.loop": {
        text: "Loop",
        description: "Toggle under the viewer: repeat the clip instead of stopping on its last frame.",
    },
    "chibi.loopHint": {
        text: "Repeat the clip",
        description: "Tooltip of the Loop toggle.",
    },
    "chibi.chain": {
        text: "Chain",
        description: "Toggle under the viewer: when a clip ends, play the next one in the list (useful for skill begin, loop and end clips).",
    },
    "chibi.chainHint": {
        text: "Play the next clip in the list when this one ends",
        description: "Tooltip of the Chain toggle.",
    },
    "chibi.speed": {
        text: "Playback speed",
        description: "Accessible name of the group of speed buttons under the viewer.",
    },
    "chibi.speedOption": {
        text: "{speed}×",
        description: "One playback speed, e.g. '0.5×' for half speed.",
    },
    "chibi.error.noData": {
        text: "No spine data available",
        description: "Shown in the viewer when the chosen sprite set ships no animation files. 'Spine' is the animation format's name and stays as-is.",
    },
    "chibi.error.failed": {
        text: "Failed to load chibi",
        description: "Shown in the viewer when the animation files could not be fetched or parsed.",
    },
    "chibi.error.webgl": {
        text: "This browser has WebGL turned off, so the animation can't play",
        description: "Shown in the viewer when the browser cannot create a WebGL renderer (hardware acceleration off or the GPU unavailable). 'WebGL' is a technical name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
