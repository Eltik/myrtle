import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.skins.showPast": {
        text: "Show past",
        description: "Switch label: also list skins that have already arrived on the English server.",
    },
    "release.skins.showReviews": {
        text: "Fashion Reviews",
        description: "Switch label: also show the periodic sale the game calls the Rhodes Fashion Review.",
    },
    "release.skins.count": {
        text: "{shown} of {total}",
        description: "How many skins the filters keep out of all of them, e.g. '18 of 240'.",
    },
    "release.skins.newTitle": {
        text: "New skins",
        description: "Section heading over the outfits arriving for the first time.",
    },
    "release.skins.new.empty.title": {
        text: "No new skins",
        description: "Empty-state heading when there are no unreleased skins to list.",
    },
    "release.skins.new.empty.none": {
        text: "Every CN skin with a release time is already on EN.",
        description: "Empty-state body when nothing is pending. 'CN' and 'EN' are the Chinese and English game servers.",
    },
    "release.skins.new.empty.filtered": {
        text: "Every new skin is filtered out. Turn on Show past.",
        description: "Empty-state body when the filter hid everything. 'Show past' names the switch above, so keep it matching.",
    },
    "release.skins.rerunsTitle": {
        text: "Reruns",
        description: "Section heading over the outfits going back on sale.",
    },
    "release.skins.reruns.empty.title": {
        text: "No reruns pending",
        description: "Empty-state heading when no rerun is expected.",
    },
    "release.skins.reruns.empty.desc": {
        text: "CN has re-listed no skin group in the lookback that EN has not shown.",
        description: "Empty-state body when no rerun is expected. 'Lookback' is the recent window this tool examines.",
    },
    "release.skins.reviews.intro": {
        text: "A Fashion Review puts every 18-prime brand outfit that is at least two years old back on sale for four weeks; each edition keeps the last one's stock and adds the outfits that came of age since{cadence}{latest}. EN follows each one about six months later; an unmatched CN listing takes the lag estimate.",
        description:
            "The Fashion Review explanation. {cadence} and {latest} are the optional clauses release.skins.reviews.intro.cadence and .latest, each empty when the figure is unknown; both may move wherever the sentence needs them. '18-prime' is the price in Originite Prime; 'brand' is the game's own grouping of outfits.",
    },
    "release.skins.reviews.intro.cadence": {
        text: ", every {min} to {max} days on CN",
        description: "Optional clause continuing the Fashion Review explanation with how often it recurs. Keep the leading comma and space.",
    },
    "release.skins.reviews.intro.latest": {
        text: "; the newest CN listing opened {date}",
        description: "Optional clause continuing the Fashion Review explanation with the most recent Chinese listing. Keep the leading semicolon and space.",
    },
    "release.skins.reviewsTitle": {
        text: "Fashion Reviews",
        description: "Section heading over the listed Fashion Review windows.",
    },
    "release.skins.reviews.empty.title": {
        text: "No Fashion Review listed",
        description: "Empty-state heading when no Fashion Review window is known.",
    },
    "release.skins.reviews.empty.desc": {
        text: "CN has listed none.",
        description: "Empty-state body when no Fashion Review window is known. 'CN' is the Chinese game server.",
    },
    "release.skins.review.tag": {
        text: "Fashion Review",
        description: "Tag on a Fashion Review row. The game's own name for the sale.",
    },
    "release.skins.review.cn": {
        text: "CN",
        description: "Small heading before a Fashion Review's Chinese-server dates. The game server's usual abbreviation.",
    },
    "release.skins.review.outfits": {
        text: " · {count} outfits · ",
        description: "Sits between a Fashion Review's dates and its show/hide link, e.g. ' · 118 outfits · '. Keep the spaces and middle dots.",
    },
    "release.skins.review.show": {
        text: "show",
        description: "Link that reveals a Fashion Review's outfits. Lowercase, mid-sentence.",
    },
    "release.skins.review.hide": {
        text: "hide",
        description: "Link that folds a Fashion Review's outfits away. Lowercase, mid-sentence.",
    },
    "release.skins.rerunSummary": {
        text: "Reruns follow CN's re-listings: EN repeats them under the same event (129 of 139 within 7 d of its EN start) or at the lag (48 of 48 within 31 d), confirmed once EN game data lists the group. One EN has already shown estimates its next by EN's own re-listing cadence{batch}. Fashion Reviews, the quarterly sale of every outfit two years old or more, sit behind their own toggle above.",
        description: "The rerun explanation. {batch} is either release.skins.rerunSummary.batch or release.skins.rerunSummary.noBatch, and may move wherever the sentence needs it. 'CN' and 'EN' are the game servers; 'd' abbreviates days; 'lag' is how far behind EN runs.",
    },
    "release.skins.rerunSummary.batch": {
        text: '. The next rotation batch ("Multi-theme Outfit", groups not in game data) is {name} {resolution}',
        description:
            "Clause naming the next batch of rotating outfits, substituted into release.skins.rerunSummary as {batch}. {name} is the batch name in a brighter colour and {resolution} its confirmed or estimated date; both may move. 'Multi-theme Outfit' is the game's own label, in its own quotes. Keep the leading full stop.",
    },
    "release.skins.rerunSummary.noBatch": {
        text: ". No rotation batch is pending",
        description: "Alternative clause when no batch of rotating outfits is expected. Keep the leading full stop.",
    },
    "release.skins.inline.confirmed": {
        text: "(confirmed, {date})",
        description: "Inline date for a batch whose English date is in the game data. Keep the parentheses.",
    },
    "release.skins.inline.estimated": {
        text: "(estimated {date}, {relative})",
        description: "Inline date for a batch whose English date is projected, e.g. '(estimated Nov 3, 2026, in 48 days)'. Keep the parentheses.",
    },
    "release.skins.inline.none": {
        text: "(no date)",
        description: "Inline stand-in for a batch with no English date at all. Keep the parentheses.",
    },
    "release.skins.basis.cnListing": {
        text: "CN re-listed {dates}",
        description: "Caption explaining that a rerun is expected because China put the group back on sale. 'CN' is the Chinese game server.",
    },
    "release.skins.basis.cadence": {
        text: "next re-listing: last on EN {date}; groups re-list every {median} d (p25 {p25}, p75 {p75}, n={count})",
        description: "Caption explaining that a rerun is projected from how often groups come back. 'd' abbreviates days; 'p25'/'p75' are percentiles and 'n' the sample size, kept as statistical shorthand.",
    },
    "release.skins.stat.new": {
        text: "New skins",
        description: "Label of the tile counting skins not yet confirmed on the English server. Very narrow tile.",
    },
    "release.skins.stat.new.sub": {
        text: "of {total} CN skins not yet confirmed on EN",
        description: "Caption under the new-skins tile. 'CN' and 'EN' are the game servers.",
    },
    "release.skins.stat.next": {
        text: "Next confirmed",
        description: "Label of the tile counting skins with a confirmed English date still ahead. Very narrow tile.",
    },
    "release.skins.stat.next.none": {
        text: "none scheduled in EN gamedata",
        description: "Caption under the next-confirmed tile when nothing is scheduled.",
    },
    "release.skins.stat.next.first": {
        text: "first on {date}",
        description: "Caption under the next-confirmed tile naming the soonest date.",
    },
    "release.skins.stat.reruns": {
        text: "Reruns",
        description: "Label of the tile counting re-listings found in the lookback window. Very narrow tile.",
    },
    "release.skins.stat.reruns.some": {
        text: "CN re-listings in the lookback, {count} already in EN gamedata",
        description: "Caption under the reruns tile when some are already confirmed on the English server.",
    },
    "release.skins.stat.reruns.none": {
        text: "CN re-listings in the lookback, none in EN gamedata yet",
        description: "Caption under the reruns tile when none are confirmed yet.",
    },
    "release.skins.stat.anniversary": {
        text: "Anniversary reruns",
        description: "Label of the tile showing how often a group comes back on its anniversary. Very narrow tile.",
    },
    "release.skins.stat.anniversary.sub": {
        text: "re-listed by name at year 1 ({observed} of {eligible} EN groups)",
        description: "Caption under the anniversary tile. An optional second-year clause may follow.",
    },
    "release.skins.stat.anniversary.year2": {
        text: ", year 2 {observed} of {eligible}",
        description: "Optional clause adding the second-year figures to the anniversary caption. Keep the leading comma and space.",
    },
    "release.skins.stat.anniversary.none": {
        text: "no EN group has reached its first anniversary",
        description: "Caption under the anniversary tile when there is nothing to measure yet.",
    },
    "release.skins.groupCount": {
        text: "{count, plural, one {# skin} other {# skins}}",
        description: "How many outfits are in one group card.",
    },
    "release.skins.groupCn": {
        text: "CN",
        description: "Small heading before a group's Chinese-server date. The game server's usual abbreviation.",
    },
    "release.skins.tileTitle": {
        text: "CN {date}, {how}",
        description: "Tooltip on a skin tile: when it arrived in China and how it is obtained.",
    },
    "release.skins.tile.shop": {
        text: "shop",
        description: "How a skin is obtained: bought in the store. Lowercase, mid-sentence.",
    },
    "release.skins.tile.notForSale": {
        text: "not for sale",
        description: "How a skin is obtained when it is not sold at all. Lowercase, mid-sentence.",
    },
    "release.skins.noEnWindow": {
        text: "No EN window yet",
        description: "Shown on a rerun card that has never been on sale on the English server.",
    },
    "release.skins.en": {
        text: "EN",
        description: "Small heading before a group's English-server dates. The game server's usual abbreviation.",
    },
    "release.skins.window": {
        text: "{dates}",
        description: "One past sale window on a rerun card; the dates are already formatted.",
    },
    "release.skins.window.review": {
        text: "{dates} (review)",
        description: "One past sale window that was part of a Fashion Review rather than its own listing.",
    },
    "release.skins.windows.title": {
        text: "Every time this group was on sale on EN: its own listings (debut and reruns) and the Fashion Reviews it was old enough for",
        description: "Tooltip on the sale-count text of a rerun card.",
    },
    "release.skins.windows.debutOnly": {
        text: "debut only",
        description: "Shown when a group has only ever been on sale once. 'Debut' is its first listing.",
    },
    "release.skins.windows.rerunCount": {
        text: "{count, plural, one {# rerun} other {# reruns}}",
        description: "How many times a group has come back on sale after its debut.",
    },
    "release.skins.windows.inReviews": {
        text: " ({count} in reviews)",
        description: "Optional clause saying how many of those sales were Fashion Reviews. Keep the leading space and the parentheses.",
    },
    "release.skins.overdue": {
        text: "expected date passed, not yet listed on EN",
        description: "Warning on a rerun whose estimated date has already gone by.",
    },
    "release.skins.basisAnchorJoin": {
        text: ", ",
        description: "Separator between a rerun's basis line and the event it is expected alongside. Keep the trailing space.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
