import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "role.superAdmin": {
        text: "super_admin",
        description: "Badge label for the highest global role. It is deliberately spelled like the API value the column stores, so leave it exactly as it is.",
    },
    "role.tierListAdmin": {
        text: "tier_list_admin",
        description: "Badge label for the role that manages every tier list. It is deliberately spelled like the API value the column stores, so leave it exactly as it is.",
    },
    "role.tierListEditor": {
        text: "tier_list_editor",
        description: "Badge label for the role that may edit tier lists. It is deliberately spelled like the API value the column stores, so leave it exactly as it is.",
    },
    "role.translator": {
        text: "translator",
        description: "Badge label for the role that may write translations. It is deliberately spelled like the API value the column stores, so leave it exactly as it is.",
    },
    "role.user": {
        text: "user",
        description: "Badge label for an ordinary account with no admin role. It is deliberately spelled like the API value the column stores, so leave it exactly as it is.",
    },
    "level.view": {
        text: "View",
        description: "Lowest rung of the permission ladder: read a tier list or locale but change nothing. A noun-ish button label, not the verb.",
    },
    "level.edit": {
        text: "Edit",
        description: "Second rung of the permission ladder: change content without publishing it. A noun-ish button label, not the verb.",
    },
    "level.publish": {
        text: "Publish",
        description: "Third rung of the permission ladder: make a new version live. A noun-ish button label, not the verb.",
    },
    "level.admin": {
        text: "Admin",
        description: "Top rung of the permission ladder: also hand out grants to other people.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
