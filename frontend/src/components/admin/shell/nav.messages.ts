import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "shell.nav.inbox": {
        text: "Inbox",
        description: "Admin section tab for staff: the landing page listing everything that needs an admin's attention.",
    },
    "shell.nav.myWork": {
        text: "My work",
        description: "Admin section tab for translators and tier list editors: their landing page, scoped to what they can edit.",
    },
    "shell.nav.people": {
        text: "People",
        description: "Admin section tab: the list of players, where staff see and change roles and access.",
    },
    "shell.nav.tierLists": {
        text: "Tier lists",
        description: "Admin section tab for staff: the official tier lists and who may edit them.",
    },
    "shell.nav.myTierLists": {
        text: "My tier lists",
        description: "Admin section tab for non-staff: only the tier lists this person has been granted access to.",
    },
    "shell.nav.notes": {
        text: "Operator notes",
        description: "Admin section tab: the editor for the guidance text shown on each operator's page.",
    },
    "shell.nav.translations": {
        text: "Translations",
        description: "Admin section tab: the editor for the site's interface text in each language.",
    },
    "shell.nav.system": {
        text: "System",
        description: "Admin section tab: service health, the edit history and site languages.",
    },
    "shell.role.user": {
        text: "Player",
        description: "Human-readable name of the plain account role (no admin access). Shown next to a person's name.",
    },
    "shell.role.translator": {
        text: "Translator",
        description: "Human-readable name of the role that edits the languages granted to it.",
    },
    "shell.role.tierListEditor": {
        text: "Tier list editor",
        description: "Human-readable name of the role that edits operator notes and the tier lists granted to it.",
    },
    "shell.role.tierListAdmin": {
        text: "Tier list admin",
        description: "Human-readable name of the role that edits and publishes every tier list and operator note.",
    },
    "shell.role.superAdmin": {
        text: "Super-admin",
        description: "Human-readable name of the highest role: full access, including roles, grants and site languages.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
