import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "primitives.moreInfo": {
        text: "More information",
        description: "Accessible name of the small (i) button that opens an explanatory tooltip, when the caller gives no better label.",
    },
    "primitives.flag.yes": {
        text: "Yes",
        description: "Value pill on a stage property row when the flag is on. Short uppercase label in a narrow pill.",
    },
    "primitives.flag.no": {
        text: "No",
        description: "Value pill on a stage property row when the flag is off. Short uppercase label in a narrow pill.",
    },
    "legend.title": {
        text: "Legend",
        description: "Kicker over the row of colour swatches explaining the map's tile colours.",
    },
    "legend.start": {
        text: "Start",
        description: "Map legend swatch: the tiles enemies spawn on.",
    },
    "legend.end": {
        text: "End",
        description: "Map legend swatch: the tiles enemies are trying to reach.",
    },
    "legend.ground": {
        text: "Ground",
        description: "Map legend swatch: ordinary low-ground tiles.",
    },
    "legend.highGround": {
        text: "High Ground",
        description: "Map legend swatch: raised tiles, where ranged operators deploy.",
    },
    "legend.forbidden": {
        text: "Forbidden",
        description: "Map legend swatch: tiles nothing can be deployed on.",
    },
} satisfies MessageMap;

// `dynamic`: the legend swatch keys are stored on a module-level table and
// resolved as `t(swatch.labelKey)`, so the extractor has no literal call site.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
