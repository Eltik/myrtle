import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.header.hiddenMarker": {
        text: "Hidden from visitors",
        description: "Tooltip and accessible label of the small eye-off icon the profile owner sees on a header number (level, a date, a stat card) they hid. Visitors never see the number or the icon.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
