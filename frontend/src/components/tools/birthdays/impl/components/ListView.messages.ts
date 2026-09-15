import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.list.empty.title": {
        text: "No results",
        description: "Empty-state heading in the list view when the filters keep nothing.",
    },
    "birthdays.list.empty.desc": {
        text: "No operators match your filters. Try clearing rarity or class chips, or reset everything.",
        description: "Empty-state body in the list view; 'chips' are the small filter buttons in the rail.",
    },
    "birthdays.list.monthCount": {
        text: "· {count}",
        description: "How many operators celebrate in a month, shown after the month name, e.g. '· 27'. Keep the leading middle dot.",
    },
    "birthdays.list.weekdayToday": {
        text: "{weekday} · today",
        description: "Weekday label of a day row when that day is today. {weekday} comes from Intl; 'today' is lowercase because the line is set in small caps.",
    },
    "birthdays.list.rarityClass": {
        text: "{rarity}★ · {class}",
        description: "Trailing detail on an operator line: star rating then class, e.g. '5★ · Sniper'. {class} is game vocabulary and comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
