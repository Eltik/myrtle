import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.account.maxLevel.title": {
        text: "Max-level cost",
        description: "Title of the card that totals what it takes to bring every operator to its level cap.",
    },
    "profile.account.maxLevel.blurb": {
        text: "The EXP and LMD still needed to take every operator you own to its final promotion and level cap, against what you hold.",
        description: "One-line explanation under the max-level cost card title. Promotion is the game's elite upgrade; LMD is the game's currency.",
    },
    "profile.account.maxLevel.calculate": {
        text: "Calculate",
        description: "Button that runs the max-level cost calculation.",
    },
    "profile.account.maxLevel.recalculate": {
        text: "Recalculate",
        description: "Button label once a result is shown; runs the calculation again.",
    },
    "profile.account.maxLevel.calculating": {
        text: "Calculating…",
        description: "Button label while the calculation runs.",
    },
    "profile.account.maxLevel.error": {
        text: "The calculation failed. Try again.",
        description: "Shown when the max-level cost request fails.",
    },
    "profile.account.maxLevel.remaining": {
        text: "Operators left",
        description: "Label over the count of operators not yet at their cap, shown as 'left / total'.",
    },
    "profile.account.maxLevel.expNeeded": {
        text: "EXP needed",
        description: "Label over the total EXP still needed.",
    },
    "profile.account.maxLevel.lmdNeeded": {
        text: "LMD needed",
        description: "Label over the total LMD still needed (levels plus promotions).",
    },
    "profile.account.maxLevel.lmdSplit": {
        text: "{levels} for levels · {promotions} for promotions",
        description: "Hint under the LMD total splitting it into level-ups and promotions. {levels} and {promotions} are formatted numbers.",
    },
    "profile.account.maxLevel.owned": {
        text: "{owned} owned",
        description: "Hint under a total showing what the account currently holds. {owned} is a formatted number.",
    },
    "profile.account.maxLevel.expMissing": {
        text: "EXP short",
        description: "Label over the EXP still missing after what the account holds.",
    },
    "profile.account.maxLevel.lmdMissing": {
        text: "LMD short",
        description: "Label over the LMD still missing after what the account holds.",
    },
    "profile.account.maxLevel.incomePerDay": {
        text: "LMD per day",
        description: "Label over the LMD the account earns in a day without farming: its base plus daily and weekly mission chests.",
    },
    "profile.account.maxLevel.incomeSplit": {
        text: "{base} from the base · {dailies} from dailies",
        description: "Hint under the LMD-per-day figure splitting it. {base} and {dailies} are formatted numbers.",
    },
    "profile.account.maxLevel.incomeNoBase": {
        text: "{dailies} from dailies · no synced base",
        description: "Hint under the LMD-per-day figure when the account has no synced base, so only mission chests count. {dailies} is a formatted number.",
    },
    "profile.account.maxLevel.daysToAfford": {
        text: "Days to afford",
        description: "Label over the number of days until the account has earned the LMD it is short, without farming.",
    },
    "profile.account.maxLevel.daysWithFarming": {
        text: "{days} with {stage} farming ({perDay} more per day)",
        description: "Hint under the days figure: the same count if every day's natural sanity is spent on the LMD stage. {days} is a formatted day count, {stage} the stage code (e.g. CE-6), {perDay} a formatted number.",
    },
    "profile.account.maxLevel.days": {
        text: "{count, plural, one {# day} other {# days}}",
        description: "A day count, e.g. '12 days'. {count} is the number of days.",
    },
    "profile.account.maxLevel.noIncome": {
        text: "Nothing earned yet",
        description: "Shown in place of the day count when the account earns no LMD at all (no base, no chests).",
    },
    "profile.account.maxLevel.allMaxed": {
        text: "Every operator you own is already at its cap.",
        description: "Shown when no operator is left to level.",
    },
    "profile.account.maxLevel.col.operator": {
        text: "Operator",
        description: "Table column header: the operator.",
    },
    "profile.account.maxLevel.col.now": {
        text: "Now",
        description: "Table column header: the operator's current promotion and level.",
    },
    "profile.account.maxLevel.col.target": {
        text: "Cap",
        description: "Table column header: the operator's final promotion and level.",
    },
    "profile.account.maxLevel.col.exp": {
        text: "EXP",
        description: "Table column header: EXP still needed for this operator.",
    },
    "profile.account.maxLevel.col.lmd": {
        text: "LMD",
        description: "Table column header: LMD still needed for this operator (levels plus promotions).",
    },
    "profile.account.maxLevel.elite": {
        text: "E{elite} Lv{level}",
        description: "An operator's promotion and level, e.g. 'E2 Lv90'. {elite} is 0-2, {level} the level.",
    },
    "profile.account.maxLevel.showAll": {
        text: "Show all {count}",
        description: "Button that expands the operator list. {count} is the number of operators.",
    },
    "profile.account.maxLevel.showFewer": {
        text: "Show fewer",
        description: "Button that collapses the operator list back to the first few.",
    },
    "profile.account.rest": {
        text: "Promotion, mastery and module rankings are not built yet.",
        description: "Note under the max-level cost card: the rest of the Account Optimizer is still to come.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
