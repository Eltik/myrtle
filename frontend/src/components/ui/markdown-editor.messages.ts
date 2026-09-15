import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "markdownEditor.write": {
        text: "Write",
        description: "Tab that shows the editable textarea. A verb, paired with the 'Preview' tab; it sits in a narrow tab so keep it short.",
    },
    "markdownEditor.preview": {
        text: "Preview",
        description: "Tab that shows the rendered Markdown. A verb, paired with the 'Write' tab; it sits in a narrow tab so keep it short.",
    },
    "markdownEditor.formatting": {
        text: "Formatting",
        description: "Accessible name of the toolbar row holding the bold/italic/list buttons.",
    },
    "markdownEditor.nothingToPreview": {
        text: "Nothing to preview yet.",
        description: "Shown on the Preview tab while the editor is still empty.",
    },
    "markdownEditor.bold": {
        text: "Bold (⌘B)",
        description: "Tooltip and accessible name of the bold button. The parenthesised part is the keyboard shortcut and stays as-is.",
    },
    "markdownEditor.italic": {
        text: "Italic (⌘I)",
        description: "Tooltip and accessible name of the italic button. The parenthesised part is the keyboard shortcut and stays as-is.",
    },
    "markdownEditor.inlineCode": {
        text: "Inline code",
        description: "Tooltip and accessible name of the button that wraps the selection in backticks.",
    },
    "markdownEditor.link": {
        text: "Link (⌘K)",
        description: "Tooltip and accessible name of the insert-link button. The parenthesised part is the keyboard shortcut and stays as-is.",
    },
    "markdownEditor.bulletList": {
        text: "Bullet list",
        description: "Tooltip and accessible name of the button that turns the selected lines into an unordered list.",
    },
    "markdownEditor.numberedList": {
        text: "Numbered list",
        description: "Tooltip and accessible name of the button that turns the selected lines into an ordered list.",
    },
    "markdownEditor.quote": {
        text: "Quote",
        description: "Tooltip and accessible name of the button that turns the selected lines into a block quote. A verb.",
    },
    "markdownEditor.markdownHint": {
        text: "Markdown",
        description: "Narrow-screen hint under the editor, standing in for the desktop syntax cheatsheet. 'Markdown' is the name of the markup language and normally stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
