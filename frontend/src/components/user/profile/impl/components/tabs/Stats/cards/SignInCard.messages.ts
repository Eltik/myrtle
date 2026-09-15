import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.signin.record.title": {
        text: "Sign-In Record",
        description: "Heading of the card covering lifetime daily sign-ins. 'Sign-in' is the game's daily login reward.",
    },
    "profile.stats.signin.total": {
        text: "total sign-ins",
        description: "Tile caption under the lifetime sign-in count. Rendered uppercase by CSS.",
    },
    "profile.stats.signin.total.tooltip": {
        text: "Lifetime cumulative sign-in days",
        description: "Tooltip on the lifetime sign-in tile.",
    },
    "profile.stats.signin.age": {
        text: "days since joining",
        description: "Tile caption under the account's age in days. Rendered uppercase by CSS.",
    },
    "profile.stats.signin.rate": {
        text: "Sign-in rate",
        description: "Label over the bar comparing sign-ins to days since joining.",
    },
    "profile.stats.signin.memberSince": {
        text: "Member since",
        description: "Row label: the date the account was created.",
    },
    "profile.stats.signin.accountAge": {
        text: "Account age",
        description: "Row label: how long the account has existed.",
    },
    "profile.stats.signin.thisMonth": {
        text: "This month",
        description: "Row label: sign-ins claimed in the current month.",
    },
    "profile.stats.signin.thisMonth.tooltip": {
        text: "Sign-ins claimed this month, out of the days elapsed",
        description: "Tooltip on the 'This month' row.",
    },
    "profile.stats.signin.missed": {
        text: "Days missed",
        description: "Row label: days since joining with no sign-in.",
    },
    "profile.stats.signin.missed.tooltip": {
        text: "Days since joining without a sign-in",
        description: "Tooltip on the 'Days missed' row.",
    },
    "profile.stats.signin.lastOnline": {
        text: "Last online",
        description: "Row label: when the account was last seen.",
    },
    "profile.stats.signin.age.days": {
        text: "{n}d",
        description: "Account age under a month. 'd' abbreviates days; there is room for two or three characters.",
    },
    "profile.stats.signin.age.months": {
        text: "{months}mo",
        description: "Account age under a year. 'mo' abbreviates months; very little room.",
    },
    "profile.stats.signin.age.years": {
        text: "{years}y",
        description: "Account age of whole years. 'y' abbreviates years; very little room.",
    },
    "profile.stats.signin.age.yearsMonths": {
        text: "{years}y {months}mo",
        description: "Account age in years and months, e.g. '2y 7mo'. Very little room.",
    },
    "profile.stats.signin.calendar.title": {
        text: "Daily Sign-In",
        description: "Heading of the card showing this month's sign-in slots.",
    },
    "profile.stats.signin.syncedTitle": {
        text: "Last synced {date}",
        description: "Tooltip on the sync line, giving the exact date of the last import.",
    },
    "profile.stats.signin.synced": {
        text: "Synced {ago}",
        description: "Sync line on the calendar card. {ago} is a relative time such as '3 days ago'.",
    },
    "profile.stats.signin.claimed": {
        text: "claimed",
        description: "Follows the 'claimed / elapsed' figures over the sign-in grid.",
    },
    "profile.stats.signin.summary": {
        text: "{month}: {claimed} of {elapsed} days claimed, as of {date}",
        description: "Accessible summary of the sign-in grid, read instead of its thirty-odd cells. {month} is a month and year, {date} the last sync date.",
    },
    "profile.stats.signin.cell.day": {
        text: "Day {day}",
        description: "Opens a sign-in cell's tooltip: which reward slot it is. Further clauses may follow, each after a middle dot.",
    },
    "profile.stats.signin.state.claimed": {
        text: "Claimed",
        description: "State of a sign-in slot that has been taken. Used in the grid legend and in a cell's tooltip.",
    },
    "profile.stats.signin.state.unclaimed": {
        text: "Unclaimed",
        description: "State of a sign-in slot whose day has passed but which is still open. Used in the grid legend and in a cell's tooltip.",
    },
    "profile.stats.signin.state.upcoming": {
        text: "Upcoming",
        description: "State of a sign-in slot not yet reachable. Used in the grid legend.",
    },
    "profile.stats.signin.cell.monthlyCard": {
        text: "Monthly card",
        description: "Clause in a sign-in cell's tooltip: that day's claim included the game's paid monthly card bonus.",
    },
    "profile.stats.signin.cell.nextUp": {
        text: "Next up",
        description: "Clause in a sign-in cell's tooltip: this is the slot the player can claim next.",
    },
    "profile.stats.signin.behind": {
        text: "Days behind",
        description: "Row label: how many sign-in slots the player has fallen behind by.",
    },
    "profile.stats.signin.behind.days": {
        text: "{count, plural, one {day} other {days}}",
        description: "The unit after the days-behind count, which is rendered separately.",
    },
    "profile.stats.signin.caughtUp": {
        text: "All caught up",
        description: "Replaces the days-behind count when no slot is outstanding.",
    },
    "profile.stats.signin.claimReady": {
        text: "· claim ready",
        description: "Appended to the days-behind row when today's sign-in is still waiting. Keep the leading middle dot.",
    },
    "profile.stats.signin.cardDays.tooltip": {
        text: "{cardDays} of this month's {claimed} claims came with the monthly card",
        description: "Tooltip on the days-behind row. The 'monthly card' is the game's paid subscription item.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
