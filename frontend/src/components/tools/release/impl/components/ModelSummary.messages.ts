import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.model.disclosure": {
        text: "How are these estimates calculated?",
        description: "Label on the collapsed control that reveals the methodology paragraph. Phrased as the reader's own question.",
    },
    "release.model.empty": {
        text: "Lag model: no EN releases in the window yet, so nothing is estimated.",
        description: "Shown instead of the model summary when there is nothing to fit. 'Lag' is how far behind the English server runs; 'EN' is the English game server.",
    },
    "release.model.days": {
        text: "{days} d",
        description: "A number of days, e.g. '182 d'. 'd' abbreviates days.",
    },
    "release.model.median": {
        text: "Lag model: median {days} over the last {count} EN releases (p25 {p25}, p75 {p75})",
        description: "The model summary. {days} is the median, highlighted and labelled by release.model.days, and may move wherever the sentence needs it. 'p25' and 'p75' are the 25th and 75th percentiles, kept as statistical shorthand. No closing full stop: the backtest clause and the stop follow.",
    },
    "release.model.backtest": {
        text: ", backtest median abs error {days} over {count}, band hit rate {rate}",
        description: "Clause appended to release.model.median saying how well the model did on past data. {days} is the error and {rate} the share of estimates that landed in the band; both are highlighted and may move. 'abs' abbreviates absolute, 'band' is the estimated date window. Keep the leading comma.",
    },
    "release.model.yearly": {
        text: "{types}: {days} over {count}, a year rather than a season.",
        description: "Second sentence, for content that repeats once a year rather than per season. {types} names that content (e.g. 'April Fools') and {days} is its highlighted median; both may move.",
    },
    "release.model.aprilFools": {
        text: "April Fools",
        description: "Name of the once-a-year April Fools' content the yearly model covers.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
