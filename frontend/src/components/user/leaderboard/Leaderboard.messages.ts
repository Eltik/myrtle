import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.breadcrumb.label": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> above the leaderboard.",
    },
    "leaderboard.breadcrumb.doctors": {
        text: "Players",
        description: "First breadcrumb crumb, the section that lists players. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "leaderboard.breadcrumb.current": {
        text: "Leaderboard",
        description: "Last breadcrumb crumb, naming the current page.",
    },
    "leaderboard.showing": {
        text: "Showing {start}-{end} of {count, number} {count, plural, one {player} other {players}}",
        description: "Range summary in the table footer. {start} and {end} are the first and last row numbers on this page, {count} the number of Doctors matching the filters.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
