import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.breakdown.kicker": {
        text: "Banner breakdown",
        description: "Small uppercase label above the table of the player's pulls grouped by banner.",
    },
    "history.breakdown.title": {
        text: "Where your pulls went.",
        description: "Heading of the panel grouping the player's pulls by banner.",
    },
    "history.breakdown.empty": {
        text: "No banner data yet.",
        description: "Empty state when the player has no pulls to group by banner.",
    },
    "history.breakdown.col.banner": {
        text: "Banner",
        description: "Column heading for the banner's name. Rendered uppercase.",
    },
    "history.breakdown.col.lastPull": {
        text: "Last pull",
        description: "Column heading for the date of the player's most recent pull on that banner. Rendered uppercase.",
    },
    "history.breakdown.col.pulls": {
        text: "Pulls",
        description: "Column heading for how many pulls the player made on that banner. Rendered uppercase.",
    },
    "history.breakdown.col.sixStars": {
        text: "6★",
        description: "Column heading for how many 6-star operators that banner gave the player. Rendered uppercase in a very narrow column.",
    },
    "history.breakdown.col.distribution": {
        text: "Distribution",
        description: "Column heading over the bar comparing each banner's pull count with the largest. Rendered uppercase.",
    },
    "history.breakdown.stat.pulls": {
        text: "Pulls",
        description: "Label over the pull count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.stat.sixStars": {
        text: "6★",
        description: "Label over the 6-star count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.stat.fiveStars": {
        text: "5★",
        description: "Label over the 5-star count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.featured": {
        text: "Featured {rarity}★",
        description: "Label over the row of operators a banner boosts, e.g. 'Featured 6★'. Rendered uppercase.",
    },
    "history.breakdown.moreOperators": {
        text: "{count} more",
        description: "Tooltip on the '+3' chip that stands in for featured operators the card has no room to show.",
    },
    "history.breakdown.guarantee": {
        text: "{name} within {count} pulls",
        description: "A banner's guarantee, e.g. 'Guaranteed 5★ within 10 pulls'. {name} is the guarantee's own name from the game data where it has one.",
    },
    "history.breakdown.guaranteeDefault": {
        text: "Guaranteed 5★",
        description: "Stand-in name for a banner's guarantee when the game data leaves it blank.",
    },
    "history.breakdown.dateRange": {
        text: "{open} → {close}",
        description: "A banner's run window in its hover card, e.g. 'Mar 3, 2025 → Mar 17, 2025'. Keep the arrow.",
    },
    "history.breakdown.showSixStars": {
        text: "Show the {count} 6★ operators from {banner}",
        description: "Screen-reader label on the button that expands a banner row to list the 6-star operators the player got from it.",
    },
    "history.breakdown.hideSixStars": {
        text: "Hide the {count} 6★ operators from {banner}",
        description: "Screen-reader label on the same button once the banner row is expanded.",
    },
    "history.breakdown.rateUp": {
        text: "Rate-up",
        description: "Tag on a 6-star pull that was one of the banner's featured operators. Rendered uppercase, very short.",
    },
    "history.breakdown.offBanner": {
        text: "Off-banner",
        description: "Tag on a 6-star pull that was NOT one of the banner's featured operators (lost the rate-up). Rendered uppercase, very short.",
    },
    "history.breakdown.rateUpCount": {
        text: "{count} rate-up",
        description: "Summary above a banner's expanded 6-star list: how many of them were the banner's featured operators. Rendered uppercase.",
    },
    "history.breakdown.offBannerCount": {
        text: "{count} off-banner",
        description: "Summary next to the rate-up count: how many of the banner's 6-star pulls were NOT featured operators. Rendered uppercase.",
    },
    "history.breakdown.pityCaption": {
        text: "Pity",
        description: "Tiny caption under the number on a 6-star pull's card: how many pulls it took since the previous 6-star on the same pity counter. Rendered uppercase, very short.",
    },
    "history.breakdown.pityLowerBound": {
        text: "{count}+",
        description: "The pity number when earlier pulls are missing from the history, so the real count is at least this, e.g. '42+'. Very short.",
    },
    "history.breakdown.pityAt": {
        text: "6★ at pull {count}",
        description: "Tooltip and screen-reader text for a 6-star pull's pity number: it came on the {count}th pull since the previous 6-star on the same pity counter.",
    },
    "history.breakdown.pityAtLeast": {
        text: "6★ at pull {count} or later. Earlier pulls are not in this history.",
        description: "Same as the pity tooltip, for a pull whose earlier pulls are missing from the history, so {count} is only a lower bound.",
    },
    "history.breakdown.pull.pulled": {
        text: "Pulled",
        description: "Label over the date and time a 6-star operator was pulled, in the details popover of a 6-star pull. Rendered uppercase, very short.",
    },
    "history.breakdown.pull.banner": {
        text: "Banner",
        description: "Label over the name of the banner a 6-star pull came from, in the pull's details popover. Rendered uppercase, very short.",
    },
    "history.breakdown.pull.pity": {
        text: "Pity",
        description: "Label over a 6-star pull's pity count (pulls since the previous 6-star on the same counter), in the pull's details popover. Rendered uppercase, very short.",
    },
    "history.breakdown.pull.counter.standard": {
        text: "Standard counter, carries across banners",
        description: "Which pity counter a 6-star pull rolled on: the shared Standard Headhunting counter, which keeps counting from one Standard banner to the next.",
    },
    "history.breakdown.pull.counter.kernel": {
        text: "Kernel counter, carries across banners",
        description: "Which pity counter a 6-star pull rolled on: the shared Kernel Headhunting counter, which keeps counting from one Kernel banner to the next.",
    },
    "history.breakdown.pull.counter.isolated": {
        text: "This banner only",
        description: "Which pity counter a 6-star pull rolled on: the banner's own counter, which does not carry to or from any other banner.",
    },
    "history.breakdown.pull.lowerBound": {
        text: "Earlier pulls are not in this history, so the real count may be higher.",
        description: "Note under a 6-star pull's pity count when the player's history starts after this counter began, so the shown number is a lower bound.",
    },
    "history.breakdown.pull.previous": {
        text: "Previous 6★",
        description: "Label over the previous 6-star operator the player got on the same pity counter, in a 6-star pull's details popover. Rendered uppercase.",
    },
    "history.breakdown.pull.previousNone": {
        text: "None in history",
        description: "Shown under the 'Previous 6★' label when the player's history has no earlier 6-star on the same pity counter.",
    },
    "history.breakdown.pull.stretch": {
        text: "In those {count, plural, one {# pull} other {# pulls}}",
        description: "Label over the rarity counts of the pulls that make up a 6-star pull's pity count, e.g. 'In those 48 pulls'. Rendered uppercase.",
    },
    "history.breakdown.pull.stretchStar": {
        text: "{count} × {rarity}★",
        description: "One rarity count in a 6-star pull's details popover, e.g. '3 × 5★' for three 5-star operators. Very short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
