import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Font choice names, resolved from a `StoryFont` value (`settings.font.${font}`). */
export const namespace = "story";

export const messages = {
    "settings.font.preset": {
        text: "Reader default",
        description: "Font choice: the reader's own default reading face, which is the site's sans-serif.",
    },
    "settings.font.dyslexic": {
        text: "OpenDyslexic",
        description: "Font choice: a typeface drawn for readers with dyslexia. `OpenDyslexic` is the typeface name and stays as-is.",
    },
    "settings.font.sans": {
        text: "Inter (site default)",
        description: "Font choice: the site's own sans-serif family. `Inter` is the typeface name and stays as-is.",
    },
    "settings.font.heading": {
        text: "Inter (headings)",
        description: "Font choice: the family the site sets its headings in. `Inter` is the typeface name and stays as-is.",
    },
    "settings.font.display": {
        text: "Fraunces (serif)",
        description: "Font choice: the site's display serif. `Fraunces` is the typeface name and stays as-is.",
    },
    "settings.font.mono": {
        text: "Geist Mono (monospace)",
        description: "Font choice: the site's monospace family. `Geist Mono` is the typeface name and stays as-is.",
    },
    "settings.font.cjk": {
        text: "Pretendard (CJK)",
        description: "Font choice: the family that covers Chinese, Japanese and Korean. `Pretendard` is the typeface name and stays as-is.",
    },
    "settings.font.terraAegir": {
        text: "Terra Aegir",
        description: "Font choice: one of the game's own in-world scripts. `Terra Aegir` is the typeface name and stays as-is.",
    },
    "settings.font.terraSami": {
        text: "Terra Sami",
        description: "Font choice: one of the game's own in-world scripts. `Terra Sami` is the typeface name and stays as-is.",
    },
    "settings.font.terraSarkaz": {
        text: "Terra Sarkaz",
        description: "Font choice: one of the game's own in-world scripts. `Terra Sarkaz` is the typeface name and stays as-is.",
    },
    "settings.font.system": {
        text: "System font",
        description: "Font choice: whatever interface font the reader's operating system ships.",
    },
    "settings.font.custom": {
        text: "Custom font file…",
        description: "Font choice: a font file the reader uploads from their own machine. The ellipsis marks that choosing it opens a file picker.",
    },
    "settings.fontNote.preset": {
        text: "The face the reader ships with. Even and quiet at any size.",
        description: "One-line note under the font dropdown for the reader default choice.",
    },
    "settings.fontNote.dyslexic": {
        text: "Weighted letter bottoms and distinct shapes, drawn to be harder to flip or confuse.",
        description: "One-line note under the font dropdown for the OpenDyslexic choice.",
    },
    "settings.fontNote.sans": {
        text: "The face the rest of the site is set in. Even and quiet at any size.",
        description: "One-line note under the font dropdown for the site sans choice.",
    },
    "settings.fontNote.heading": {
        text: "The same family the site titles use.",
        description: "One-line note under the font dropdown for the heading choice.",
    },
    "settings.fontNote.display": {
        text: "A serif with real contrast. The easiest of these on a long read.",
        description: "One-line note under the font dropdown for the serif choice.",
    },
    "settings.fontNote.mono": {
        text: "Fixed width: every character takes the same room. Good for terminal-flavoured scenes.",
        description: "One-line note under the font dropdown for the monospace choice.",
    },
    "settings.fontNote.cjk": {
        text: "Covers Chinese, Japanese and Korean text as well as Latin.",
        description: "One-line note under the font dropdown for the CJK choice.",
    },
    "settings.fontNote.terraAegir": {
        text: "One of Terra's own scripts, from the game. A display face: anything it does not carry falls back to the reading style.",
        description: "One-line note under the font dropdown for the Terra Aegir choice.",
    },
    "settings.fontNote.terraSami": {
        text: "The other of Terra's own scripts. A display face: anything it does not carry falls back to the reading style.",
        description: "One-line note under the font dropdown for the Terra Sami choice.",
    },
    "settings.fontNote.terraSarkaz": {
        text: "The Sarkaz script. A display face: anything it does not carry falls back to the reading style.",
        description: "One-line note under the font dropdown for the Terra Sarkaz choice.",
    },
    "settings.fontNote.system": {
        text: "Your device's own interface font. Nothing is downloaded.",
        description: "One-line note under the font dropdown for the system font choice.",
    },
    "settings.fontNote.custom": {
        text: "A font file from your machine, kept in this browser only.",
        description: "One-line note under the font dropdown for the custom font choice.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
