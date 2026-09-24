import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "archive.view.aria": {
        text: "What the chapter sheet shows",
        description: "Accessible name of the two-way switch under a chapter's meta row, between its list of stories and the archive the event kept.",
    },
    "archive.view.entries": {
        text: "Entries",
        description: "Switch option showing the chapter's own stories, the sheet's default view.",
    },
    "archive.view.archive": {
        text: "From the archive",
        description: "Switch option showing the extra material an event kept beside its stories: logs, landmarks, news, files, recordings, pictures and music. The game's own screen is called the archive.",
    },
    "archive.total": {
        text: "{count} entries in this archive",
        description: "Line above the archive's sections counting every row in them.",
    },
    "archive.section.logs": {
        text: "Logs",
        description: "Archive section holding the short written entries an event files under its chapters.",
    },
    "archive.section.landmarks": {
        text: "Landmarks",
        description: "Archive section holding the places an event describes, each with a picture and a name in the setting's own language.",
    },
    "archive.section.news": {
        text: "News",
        description: "Archive section holding the in-setting newspaper and broadcast stories an event ran.",
    },
    "archive.section.files": {
        text: "Files",
        description: "Archive section holding the in-setting documents an event collected, each dated.",
    },
    "archive.section.recordings": {
        text: "Recordings",
        description: "Archive section holding voice clips, grouped into the nodes the game unlocks them in.",
    },
    "archive.section.gallery": {
        text: "Gallery",
        description: "Archive section holding the event's pictures.",
    },
    "archive.section.music": {
        text: "Music",
        description: "Archive section holding the event's tracks, each playable here.",
    },
    "archive.logs.icon.normal": {
        text: "Normal",
        description: "Tag on a log chapter the game marks as ordinary, as opposed to a special one. Keep it short, it sits in a chip.",
    },
    "archive.logs.icon.special": {
        text: "Special",
        description: "Tag on a log chapter the game marks as special. Keep it short, it sits in a chip.",
    },
    "archive.news.author": {
        text: "By {author}",
        description: "Byline over an in-setting news story in the archive.",
    },
    "archive.theme.label": {
        text: "Chapter theme",
        description: "Label of the play button on a chapter's hero picture, for chapters whose theme has no title of its own.",
    },
    "archive.theme.play": {
        text: "Play {name}",
        description: "Accessible name of the play button on a chapter's hero picture. The name is the track's title, or the words for a chapter theme when it has none.",
    },
    "archive.theme.pause": {
        text: "Pause {name}",
        description: "Accessible name of the same button while its track is sounding. Pausing keeps the track in the now-playing bar; the bar's own stop button dismisses it.",
    },
    "archive.music.untitled": {
        text: "Untitled track",
        description: "Stands in for the name of an archive track the game's music table leaves blank.",
    },
    "archive.clip.language": {
        text: "Language",
        description: "Accessible name of the language picker beside a voice clip in the archive.",
    },
    "archive.clip.play": {
        text: "Play {name}",
        description: "Accessible name of the play button beside a voice clip. The name is the speaking operator.",
    },
    "archive.clip.pause": {
        text: "Stop {name}",
        description: "Accessible name of the same button while the clip is sounding.",
    },
    "archive.recordings.hidden": {
        text: "Not on the shelf",
        description: "Heading over the recordings the game hides until they are unlocked, which belong to no node.",
    },
    "archive.recordings.hiddenNote": {
        text: "The game keeps these back until they are unlocked, so its own shelf does not list them.",
        description: "Line under the heading for the hidden recordings, saying why they are set apart.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
