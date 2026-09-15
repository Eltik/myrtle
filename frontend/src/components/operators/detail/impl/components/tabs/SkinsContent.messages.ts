import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "outfits.title": {
        text: "Outfits",
        description: "Heading of the Skins tab.",
    },
    "outfits.subtitle": {
        text: "Alternate skins, E2 art variants, and collaboration outfits",
        description: "Blurb under the Skins tab heading. 'Skin' is the game's word for a cosmetic outfit and 'E2' is its second promotion. No full stop, by design.",
    },
    "outfits.fullscreen": {
        text: "Fullscreen",
        description: "Accessible name of the button that opens the artwork full screen.",
    },
    "outfits.field.description": {
        text: "Description",
        description: "Small uppercase caption over the outfit's flavour text.",
    },
    "outfits.field.obtained": {
        text: "Obtained",
        description: "Small uppercase caption over how the outfit is acquired.",
    },
    "outfits.field.usage": {
        text: "Usage",
        description: "Small uppercase caption over the outfit's in-world note about who wears it and when.",
    },
    "outfits.field.colors": {
        text: "Colors",
        description: "Small uppercase caption over the outfit's colour swatches.",
    },
    "outfits.field.dialog": {
        text: "Dialog",
        description: "Small uppercase caption over the line the operator says about the outfit.",
    },
    "outfits.zoomOut": {
        text: "Zoom out",
        description: "Accessible name and tooltip of the zoom-out button in the full-screen viewer.",
    },
    "outfits.zoomIn": {
        text: "Zoom in",
        description: "Accessible name and tooltip of the zoom-in button in the full-screen viewer.",
    },
    "outfits.resetView": {
        text: "Reset view",
        description: "Accessible name and tooltip of the button that returns the viewer to its opening zoom and position.",
    },
    "outfits.interact": {
        text: "Play the interact animation",
        description: "Accessible name and tooltip of the button that plays the animated artwork's tap reaction.",
    },
    "outfits.record": {
        text: "Record {seconds} s of the animation",
        description: "Accessible name and tooltip of the record button. {seconds} is how long a recording lasts; 's' abbreviates seconds.",
    },
    "outfits.recordStop": {
        text: "Stop recording ({seconds} s left)",
        description: "Accessible name and tooltip of the record button while recording. {seconds} counts down; 's' abbreviates seconds.",
    },
    "outfits.download": {
        text: "Download",
        description: "Accessible name and tooltip of the button that saves the static artwork.",
    },
    "outfits.zoomTitle": {
        text: "100% frames the composition: camera {camera}, {width} by {height} px",
        description: "Hover text on the zoom readout, explaining what 100% means for animated artwork. {camera} is a raw zoom factor and {width}/{height} are pixel sizes.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
