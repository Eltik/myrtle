import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The schedule row builders in `schedule.ts` are plain functions in a module
 * with no React, so the kind labels are message KEYS the rendering component
 * resolves, and the two derived detail lines take an optional `t`.
 */
export const namespace = "tools";

export const messages = {
    "release.kind.event": {
        text: "Events",
        description: "Schedule row kind: a limited-time story event. Also the label of its filter checkbox, so it is plural.",
    },
    "release.kind.banner": {
        text: "Banners",
        description: "Schedule row kind: a gacha recruitment banner. Also the label of its filter checkbox, so it is plural.",
    },
    "release.kind.skin": {
        text: "New skins",
        description: "Schedule row kind: outfits arriving for the first time. Also the label of its filter checkbox.",
    },
    "release.kind.rerun": {
        text: "Skin reruns",
        description: "Schedule row kind: outfits going back on sale. Also the label of its filter checkbox.",
    },
    "release.kind.review": {
        text: "Fashion Reviews",
        description: "Schedule row kind: the periodic sale the game calls the Rhodes Fashion Review. Also the label of its filter checkbox.",
    },
    "release.detail.skinCount": {
        text: "{count, plural, one {# skin} other {# skins}}",
        description: "Stands in for the operator list on a rerun row when no operator is known.",
    },
    "release.detail.reviewOutfits": {
        text: "{count} outfits at least two years old, every brand",
        description: "Detail line on a Fashion Review row, describing the stock on sale. 'Brand' is the game's own grouping of outfits.",
    },
} satisfies MessageMap;

// `dynamic`: the kind keys are stored in a lookup table and resolved by the
// consuming component as `t(KIND_LABEL_KEYS[kind])`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
