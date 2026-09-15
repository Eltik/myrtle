import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "mobileNav.openMenu": {
        text: "Open navigation menu",
        description: "Accessible name of the hamburger button that opens the mobile navigation drawer.",
    },
    "mobileNav.navigation": {
        text: "Navigation",
        description: "Heading over the site links in the mobile drawer.",
    },
    "mobileNav.external": {
        text: "External",
        description: "Heading over the links that leave the site (repository, donations) in the mobile drawer.",
    },
    "mobileNav.github": {
        text: "GitHub",
        description: "Link to the source repository. Product name, stays as-is.",
    },
    "mobileNav.donate": {
        text: "Donate",
        description: "Link to the donation page in the mobile drawer.",
    },
    "mobileNav.account": {
        text: "Account",
        description: "Heading over the sign-in and account links in the mobile drawer.",
    },
    "mobileNav.userAvatar": {
        text: "User avatar",
        description: "Alt text on the signed-in visitor's avatar image.",
    },
    "mobileNav.level": {
        text: "Level {level}",
        description: "The signed-in visitor's in-game account level, under their nickname. {level} is the number, unformatted.",
    },
    "mobileNav.myTierLists": {
        text: "My Tier Lists",
        description: "Link to the tier lists the visitor has created.",
    },
    "mobileNav.settings": {
        text: "Settings",
        description: "Link to the settings page.",
    },
    "mobileNav.logout": {
        text: "Logout",
        description: "Action that signs the visitor out.",
    },
    "mobileNav.login": {
        text: "Login",
        description: "Action that opens the sign-in dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
