import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "commit.breaking": {
        text: "Breaking",
        description: "Badge on a commit that changes behaviour incompatibly. Short for 'breaking change'. Rendered uppercase in a small pill.",
    },
    "commit.details": {
        text: "Details",
        description: "Toggle that expands a long commit message. A noun.",
    },
    "commit.viewOnGitHub": {
        text: "View {sha} on GitHub",
        description: "Hover title of the link to a commit on GitHub. {sha} is the commit's short hash; 'GitHub' is a product name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
