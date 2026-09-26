import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.events.shop": {
        text: "Event shop",
        description: "Collapsed heading for an event's shop.",
    },
    "release.events.shop.meta": {
        text: "{name} · {server} {dates}",
        description: "Shop details: its name, source server, and date range. Keep the middle dots.",
    },
    "release.events.shop.buyEverything": {
        text: "Buy everything",
        description: "Row label for the cost of clearing the shop's limited stock.",
    },
    "release.events.shop.limitedGoods": {
        text: "· {count} limited goods",
        description: "Note after the buyout cost saying how many goods have limited stock. Keep the leading middle dot.",
    },
    "release.events.shop.missions": {
        text: "Points awarded by clearing missions",
        description: "Row label for shop tokens awarded by event missions.",
    },
    "release.events.shop.missionsValue": {
        text: "-{count}",
        description: "Mission tokens shown as a deduction. Keep the leading minus sign.",
    },
    "release.events.shop.toFarm": {
        text: "To farm",
        description: "Row label for tokens still needed after mission rewards.",
    },
    "release.events.shop.toFarmValue": {
        text: "{sanity} sanity",
        description: "Energy needed to earn the remaining shop tokens. 'Sanity' is the game's name for energy.",
    },
    "release.events.shop.sanityNote": {
        text: "Upper bound: first-clear token rewards are not in the data.",
        description: "Caveat below the shop calculation. 'First-clear' means the one-off reward for completing a stage.",
    },
    "release.events.shop.noShop": {
        text: "Missions pay {count} tokens. The shop is fetched from the game server once it opens on a server this site has an account on.",
        description: "Shown when mission rewards are known but the shop contents are not available yet.",
    },
    "release.events.shop.groupCount": {
        text: "· {count}",
        description: "How many goods are in one shop category. Keep the leading middle dot.",
    },
    "release.events.shop.unlimited": {
        text: "Unlimited",
        description: "Heading over shop goods with no stock limit.",
    },
    "release.events.shop.unlimitedNote": {
        text: "· not in the total, {token} left over goes here",
        description: "Note under the unlimited heading. {token} is the shop currency name. Keep the leading middle dot.",
    },
    "release.events.shop.goodCount": {
        text: " ×{count}",
        description: "How many of an item one purchase gives. Keep the leading space and multiplication sign.",
    },
    "release.events.shop.stock": {
        text: " × {count}",
        description: "How many times an item can be bought after its unit price. Keep the leading space and multiplication sign.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
