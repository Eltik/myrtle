import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.strip.operators.kicker": {
        text: "Operators",
        description: "Label of the tile counting the operators this account owns.",
    },
    "profile.strip.skins.kicker": {
        text: "Skins",
        description: "Label of the tile counting the outfits this account owns.",
    },
    "profile.strip.items.kicker": {
        text: "Items",
        description: "Label of the tile counting the distinct items in this account's inventory.",
    },
    "profile.strip.orundum.kicker": {
        text: "Orundum",
        description: "Label of the tile showing the account's Orundum. The game's own currency name: use the localized client's term.",
    },
    "profile.strip.originite.kicker": {
        text: "Originite Prime",
        description: "Label of the tile showing the account's Originite Prime. The game's own currency name: use the localized client's term.",
    },
    "profile.strip.lmd.kicker": {
        text: "LMD",
        description: "Label of the tile showing the account's LMD (the game's money). The game's own currency name: use the localized client's term.",
    },
    "profile.strip.headhuntingPermits.kicker": {
        text: "Headhunting Permits",
        description: "Label of the tile showing the account's single-pull Headhunting Permits. The game's own currency name: use the localized client's term.",
    },
    "profile.strip.tenRollPermits.kicker": {
        text: "Ten-roll Permits",
        description: "Label of the tile showing the account's Ten-roll Headhunting Permits. The game's own currency name: use the localized client's term.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
