import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "userMenu.userAvatar": {
        text: "User avatar",
        description: "Alt text on the signed-in visitor's avatar image in the header.",
    },
    "userMenu.openMenu": {
        text: "Open user menu",
        description: "Accessible name of the chevron button that opens the account dropdown.",
    },
    "userMenu.level": {
        text: "Level {level}",
        description: "The signed-in visitor's in-game account level, under their nickname. {level} is the number, unformatted.",
    },
    "userMenu.myTierLists": {
        text: "My Tier Lists",
        description: "Menu item linking to the tier lists the visitor has created.",
    },
    "userMenu.settings": {
        text: "Settings",
        description: "Menu item linking to the settings page.",
    },
    "userMenu.github": {
        text: "GitHub",
        description: "Menu item linking to the source repository. Product name, stays as-is.",
    },
    "userMenu.support": {
        text: "Support",
        description: "Menu item linking to the donation page. 'Support' here means financially supporting the site.",
    },
    "userMenu.adminPanel": {
        text: "Admin panel",
        description: "Menu item shown only to staff, linking to the moderation and translation tools.",
    },
    "userMenu.logout": {
        text: "Logout",
        description: "Menu item that signs the visitor out.",
    },
    "userMenu.login": {
        text: "Login",
        description: "Button shown to signed-out visitors, opening the sign-in dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
