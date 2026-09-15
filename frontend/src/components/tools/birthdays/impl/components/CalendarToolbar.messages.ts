import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.toolbar.today": {
        text: "Today",
        description: "Button that jumps the calendar back to the current date.",
    },
    "birthdays.toolbar.prev": {
        text: "{scale, select, day {Previous day} 3day {Previous period} week {Previous week} other {Previous month}}",
        description: "Accessible name of the back arrow; the wording follows the calendar's current zoom level. The three-day view is called a 'period' because it is not a named unit.",
    },
    "birthdays.toolbar.next": {
        text: "{scale, select, day {Next day} 3day {Next period} week {Next week} other {Next month}}",
        description: "Accessible name of the forward arrow; the wording follows the calendar's current zoom level.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
