import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stats";

export const messages = {
    "header.kicker": {
        text: "Site · overview",
        description: "Eyebrow label above the site-statistics page heading. Rendered uppercase; the middle dot is a separator.",
    },
    "header.title": {
        text: "A count of {em} on myrtle.moe.",
        description: "Stats-page headline. {em} is the emphasised word, italicised in the accent colour; move it wherever the sentence needs it. 'myrtle.moe' is this site's name and is never translated.",
    },
    "header.title.em": {
        text: "everything",
        description: "The emphasised word substituted into header.title as {em}.",
    },
    "header.blurb": {
        text: "Live snapshot of the game catalog we index, the tier lists the community maintains, and the rosters players have synced with their Yostar accounts.",
        description: "Caption under the stats-page headline. A roster is a player's own operator collection; 'Yostar' is the game publisher's account system.",
    },
    "header.updated": {
        text: "updated {when}",
        description: "Freshness pill on the stats page. {when} is an already-formatted relative time such as '5m ago'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
