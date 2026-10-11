import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** The maintenance countdown in the top bar and on Home: site chrome, so `common`. */
export const namespace = "common";

export const messages = {
    "maintenance.short.upcoming": {
        text: "Maintenance in {time}",
        description: "Top-bar pill before a game maintenance starts. {time} is a short span like '2h 14m' or '3d 4h'.",
    },
    "maintenance.short.ongoing": {
        text: "Maintenance, back in {time}",
        description: "Top-bar pill while the game is down for maintenance. {time} is a short span like '42m' until the announced end.",
    },
    "maintenance.short.extended": {
        text: "Maintenance extended, back in {time}",
        description: "Top-bar pill when Yostar has moved the end of a running maintenance later. {time} is a short span until the new end.",
    },
    "maintenance.short.overrun": {
        text: "Maintenance running long",
        description: "Top-bar pill when the announced end has passed but the game is still down.",
    },
    "maintenance.short.openEnded": {
        text: "Down for maintenance",
        description: "Top-bar pill when the game is down for maintenance and no end time has been announced.",
    },
    "maintenance.pillLabel": {
        text: "Open the official maintenance notice",
        description: "Accessible label added to the maintenance pill, which links to Yostar's own notice.",
    },
    "maintenance.detail.upcoming": {
        text: "The game goes down {start} and should be back by {end}.",
        description: "Home banner line before a maintenance. {start} and {end} are dates with times, like 'Oct 14, 4:00 AM'.",
    },
    "maintenance.detail.upcomingNoEnd": {
        text: "The game goes down {start}. No end time yet.",
        description: "Home banner line before a maintenance whose notice gives no end time. {start} is a date with a time.",
    },
    "maintenance.detail.ongoing": {
        text: "Down since {start}. Should be back by {end}.",
        description: "Home banner line during a maintenance. {start} and {end} are dates with times.",
    },
    "maintenance.detail.extended": {
        text: "Yostar pushed the end back. Should be back by {end}.",
        description: "Home banner line when a newer notice moved the end of a running maintenance later. 'Yostar' is the game's publisher. {end} is a date with a time.",
    },
    "maintenance.detail.overrun": {
        text: "It was planned to end {end}, but the game is still down. We'll keep checking.",
        description: "Home banner line when the announced end has passed and the game is still down. {end} is a date with a time.",
    },
    "maintenance.detail.openEnded": {
        text: "Down since {start}. No end time yet.",
        description: "Home banner line during a maintenance with no announced end. {start} is a date with a time.",
    },
    "maintenance.notice": {
        text: "Official notice",
        description: "Link on the Home maintenance banner to Yostar's own announcement page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
