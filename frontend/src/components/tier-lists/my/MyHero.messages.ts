import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.hero.kicker": {
        text: "My Tier Lists",
        description: "Small uppercase label above the heading of the player's own tier lists.",
    },
    "my.hero.title": {
        text: "Your workshop.",
        description: "Heading of the page holding the player's own tier lists.",
    },
    "my.hero.blurb": {
        text: "Manage, refine, and share {total} {count, plural, one {list} other {lists}} you've built. Edits sync to anyone who visits.",
        description: "Sentence under the heading when the player has at least one list. {total} is the formatted count, rendered in a monospace box, and may move wherever the sentence needs it; {count} is the same number again, used only to pick the plural form.",
    },
    "my.hero.blurbEmpty": {
        text: "You haven't created any tier lists yet. Spin up your first one - drafts publish instantly to your share link.",
        description: "Sentence under the heading when the player has no lists yet.",
    },
    "my.hero.create": {
        text: "New tier list",
        description: "Button that opens the dialog for creating a tier list.",
    },
    "my.hero.quotaReached": {
        text: "You've reached the community list cap ({max})",
        description: "Tooltip on the disabled create button, saying how many community lists one player may keep.",
    },
    "my.hero.stat.total": {
        text: "Total lists",
        description: "Tile label: how many lists the player has, official ones included. Rendered uppercase.",
    },
    "my.hero.stat.quota": {
        text: "Community quota",
        description: "Tile label over 'used / allowed' for the player's own community lists. Rendered uppercase.",
    },
    "my.hero.stat.views": {
        text: "Total views",
        description: "Tile label: views across every list the player has. Rendered uppercase.",
    },
    "my.hero.stat.favorites": {
        text: "Total favorites",
        description: "Tile label: favourites across every list the player has. Rendered uppercase.",
    },
    "my.hero.officialIncluded": {
        text: "{count, plural, one {# official list included} other {# official lists included}}",
        description: "Note under the tiles when some of the player's lists are official ones they maintain for the team. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
