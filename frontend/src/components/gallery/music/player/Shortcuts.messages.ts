import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gallery";

/**
 * One label per binding in `SHORTCUTS` (engine.ts), looked up by its `id`, so
 * the dialog renders from the same list the keydown handler implements.
 */
export const messages = {
    "music.shortcut.playPause": { text: "Play or pause", description: "Keyboard shortcuts table row." },
    "music.shortcut.next": { text: "Next track", description: "Keyboard shortcuts table row." },
    "music.shortcut.previous": { text: "Previous track (restarts the track after 3 seconds)", description: "Keyboard shortcuts table row." },
    "music.shortcut.back": { text: "Back 10 seconds", description: "Keyboard shortcuts table row." },
    "music.shortcut.forward": { text: "Forward 10 seconds", description: "Keyboard shortcuts table row." },
    "music.shortcut.mute": { text: "Mute or unmute", description: "Keyboard shortcuts table row." },
    "music.shortcut.shuffle": { text: "Shuffle", description: "Keyboard shortcuts table row." },
    "music.shortcut.repeat": { text: "Repeat (off, all, one)", description: "Keyboard shortcuts table row." },
    "music.shortcut.lyrics": { text: "Show or hide lyrics", description: "Keyboard shortcuts table row." },
    "music.shortcut.queue": { text: "Show or hide the queue", description: "Keyboard shortcuts table row." },
    "music.shortcut.like": { text: "Like the track", description: "Keyboard shortcuts table row." },
    "music.shortcut.help": { text: "This list", description: "Keyboard shortcuts table row: the key that opens this dialog." },
    "music.shortcut.close": { text: "Close a panel or dialog", description: "Keyboard shortcuts table row." },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
