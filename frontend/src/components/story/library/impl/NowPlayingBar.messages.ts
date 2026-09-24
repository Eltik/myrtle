import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "nowPlaying.region": {
        text: "Now playing",
        description: "Accessible name of the bar at the foot of the story library that carries whatever chapter theme is sounding.",
    },
    "nowPlaying.playing": {
        text: "{title} from {chapter}",
        description: "Line the now-playing bar reads out when a track starts. {title} is the track's name or the words for a chapter theme; {chapter} is the chapter it belongs to.",
    },
    "nowPlaying.resume": {
        text: "Resume {name}",
        description: "Accessible name of the play button in the now-playing bar while its track is paused. Resuming continues from the second the track was paused on.",
    },
    "nowPlaying.pause": {
        text: "Pause {name}",
        description: "Accessible name of the pause button in the now-playing bar while its track is sounding.",
    },
    "nowPlaying.stop": {
        text: "Stop {name}",
        description: "Accessible name of the button in the now-playing bar that stops the track and dismisses the bar.",
    },
    "nowPlaying.volume": {
        text: "Music volume",
        description: "Accessible name of the volume slider in the now-playing bar. It is the same setting the reader's own Settings dialog writes.",
    },
    "nowPlaying.seek": {
        text: "Position in track",
        description: "Accessible name of the slider in the now-playing bar that moves playback to any point in the track.",
    },
    "nowPlaying.seekValue": {
        text: "{elapsed} of {total}",
        description: "What a screen reader announces for the position slider in the now-playing bar. Both arguments are times written as minutes and seconds, such as 1:04.",
    },
    "nowPlaying.loopPoint": {
        text: "Loop starts here",
        description: "Tooltip on the small mark drawn across the position slider in the now-playing bar, at the point where the track's introduction ends and its looping part begins.",
    },
    "nowPlaying.paused": {
        text: "Paused",
        description: "Word shown in place of the chapter name in the now-playing bar while its track is paused.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
