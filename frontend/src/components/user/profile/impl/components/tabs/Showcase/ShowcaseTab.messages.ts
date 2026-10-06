import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.showcase.aria": {
        text: "Showcase",
        description: "Accessible name of the Showcase tab's content: the blocks a player chose to show first on their profile.",
    },
    "profile.showcase.edit": {
        text: "Edit showcase",
        description: "Button on the owner's own Showcase tab that opens the showcase editor.",
    },
    "profile.showcase.tabHidden": {
        text: "Your Showcase tab is hidden, so visitors don't see it. Show it again from Customize.",
        description: "Notice on the owner's showcase when they hid the Showcase tab in the tab editor. 'Customize' is the tab editor button's label.",
    },
    "profile.showcase.empty.kicker": {
        text: "Showcase",
        description: "Small uppercase label on the owner's empty showcase.",
    },
    "profile.showcase.empty.title": {
        text: "Make this profile yours",
        description: "Heading of the owner's empty showcase.",
    },
    "profile.showcase.empty.desc": {
        text: "Pick favourite operators, skins, events or story chapters, and pin your grids, tier lists and plans. Visitors see your showcase first.",
        description: "Body of the owner's empty showcase, listing what a showcase can hold.",
    },
    "profile.showcase.empty.cta": {
        text: "Set up your showcase",
        description: "Button on the owner's empty showcase that opens the editor.",
    },
    "profile.showcase.error": {
        text: "The showcase couldn't be loaded.",
        description: "Shown in place of the showcase when it fails to load.",
    },
    "profile.showcase.retry": {
        text: "Try again",
        description: "Button that reloads the showcase after a failure.",
    },
    "profile.showcase.block.favourites.count": {
        text: "{count, plural, one {# pick} other {# picks}}",
        description: "Count of entities in a favourites block of the showcase.",
    },
    "profile.showcase.block.grid": {
        text: "Grid",
        description: "Small uppercase label on a showcase block that shows one of the player's grids.",
    },
    "profile.showcase.block.tierList": {
        text: "Tier list",
        description: "Small uppercase label on a showcase block that shows a tier list.",
    },
    "profile.showcase.block.tierList.viewAll": {
        text: "View full tier list",
        description: "Link under a showcase tier list block that is too tall to show whole; opens the tier list's own page.",
    },
    "profile.showcase.block.plan": {
        text: "Plan",
        description: "Small uppercase label on a showcase block that shows one of the player's operator plans.",
    },
    "profile.showcase.block.open": {
        text: "Open",
        description: "Link in a showcase grid or tier list block to its full page.",
    },
    "profile.showcase.block.openAria": {
        text: "Open {title}",
        description: "Accessible name of the link to a grid's or tier list's full page. {title} is its title.",
    },
    "profile.showcase.block.plansHidden": {
        text: "Only you: your Plans tab is hidden",
        description: "Chip on a plan block, shown to the owner when their Plans tab is hidden, so visitors don't see this block.",
    },
    "profile.showcase.removed.badge": {
        text: "Removed",
        description: "Chip on a showcase block (or entity) whose grid, tier list, plan or game entry no longer exists. Shown only to the owner.",
    },
    "profile.showcase.removed.grid": {
        text: "This grid was deleted. Visitors don't see this block; remove it in the editor.",
        description: "Owner-only note on a showcase block whose grid was deleted.",
    },
    "profile.showcase.removed.tierList": {
        text: "This tier list was deleted. Visitors don't see this block; remove it in the editor.",
        description: "Owner-only note on a showcase block whose tier list was deleted.",
    },
    "profile.showcase.removed.plan": {
        text: "This plan was deleted or is no longer shown on your profile. Visitors don't see this block.",
        description: "Owner-only note on a showcase block whose plan was deleted or had 'display on profile' turned off.",
    },
    "profile.showcase.removed.favourites": {
        text: "The game data no longer has these entries. Visitors don't see this block.",
        description: "Owner-only note on a favourites block none of whose entries the game data knows any more.",
    },
    "profile.showcase.removed.entity": {
        text: "Unknown entry {id}",
        description: "Accessible name and tooltip of a favourite the game data no longer knows. {id} is its raw game id. Shown only to the owner.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
