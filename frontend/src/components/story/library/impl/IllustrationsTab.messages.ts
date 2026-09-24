import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "illustrations.kind.illustrations": {
        text: "Illustrations",
        description: "Sub-tab of the Illustrations tab listing the backgrounds and CG artwork a chapter uses.",
    },
    "illustrations.kind.sprites": {
        text: "Sprites",
        description: "Sub-tab of the Illustrations tab listing the character sprites a chapter uses.",
    },
    "illustrations.category.record": {
        text: "Operator records",
        description: "Category of the Illustrations tab holding the per-operator record sets, as opposed to the story chapters.",
    },
    "illustrations.category.aria": {
        text: "Filter by category",
        description: "Accessible name of the row of category filters above the artwork grid.",
    },
    "illustrations.pane.aria": {
        text: "Artwork kind",
        description: "Accessible name of the row of sub-tabs inside the artwork panel.",
    },
    "illustrations.pane.backgrounds": {
        text: "Backgrounds",
        description: "Sub-tab inside the artwork panel listing the background plates a chapter uses.",
    },
    "illustrations.pane.cgs": {
        text: "CGs",
        description: "Sub-tab inside the artwork panel listing the CG illustrations a chapter uses. CG is the game's own word for a full illustration.",
    },
    "illustrations.pane.sprites": {
        text: "Sprites",
        description: "Sub-tab inside the artwork panel listing the character sprites a chapter uses.",
    },
    "illustrations.panel.split": {
        text: "The card counts {total} pieces, which is {backgrounds} backgrounds plus {cgs} CGs. The index carries only the sum, so the split is shown here.",
        description: "Line inside the artwork panel explaining why the card's count is larger than the CG count a reader may be comparing it against.",
    },
    "illustrations.total": {
        text: "{count} in this category",
        description: "Counter above the artwork grid: the artwork summed over every chapter shown.",
    },
    "illustrations.totalUnknown": {
        text: "Counts not on the wire yet",
        description: "Shown in place of the counter when the served data carries no artwork counts.",
    },
    "illustrations.card.unknown": {
        text: "?",
        description: "Stands in for an artwork count the served data does not carry. Keep it to one character.",
    },
    "illustrations.card.open": {
        text: "Show the artwork in {name}",
        description: "Accessible name of a chapter card in the artwork grid.",
    },
    "illustrations.card.illustrations": {
        text: "{count} pieces",
        description: "Artwork count on a chapter card. It is the backgrounds and the CGs added together, which is the only figure the index carries, so the word must cover both and must not read as the CG count alone. Keep it short, it sits in a 2-column grid.",
    },
    "illustrations.card.sprites": {
        text: "{count} sprites",
        description: "Sprite count on a chapter card. Keep it short, it sits in a 2-column grid.",
    },
    "illustrations.empty": {
        text: "No chapter in this category.",
        description: "Empty state under the artwork grid.",
    },
    "illustrations.panel.close": {
        text: "Close",
        description: "Button that closes the artwork panel.",
    },
    "illustrations.panel.loading": {
        text: "Loading artwork",
        description: "Accessible name of the loading state inside the artwork panel.",
    },
    "illustrations.panel.failed": {
        text: "The artwork could not be loaded.",
        description: "Error state inside the artwork panel when the request fails.",
    },
    "illustrations.panel.unavailable": {
        text: "This backend does not serve artwork yet.",
        description: "Error state inside the artwork panel when the endpoint answers 404 because the running backend predates it.",
    },
    "illustrations.panel.unavailableNote": {
        text: "The endpoint is GET /story/group/<id>/illustrations. Everything else on this page keeps working.",
        description: "Second line of the 404 state in the artwork panel. Keep the endpoint path as it is.",
    },
    "illustrations.panel.empty": {
        text: "No artwork in this chapter.",
        description: "Empty state inside the artwork panel when the chapter references none.",
    },
    "illustrations.panel.summary": {
        text: "{rows} over {stories, plural, one {# story} other {# stories}}",
        description: "Header line of the artwork panel: how many pieces, over how many stories.",
    },
    "illustrations.panel.missing": {
        text: "{count} not extracted",
        description: "Header line of the artwork panel: how many pieces resolve to no file on disk.",
    },
    "illustrations.item.usedIn": {
        text: "Used in {count, plural, one {# story} other {# stories}}",
        description: "Line under an artwork tile naming how many stories reference it.",
    },
    "illustrations.item.notExtracted": {
        text: "Not extracted",
        description: "Shown in place of an artwork thumbnail whose file is missing from the served data.",
    },
    "illustrations.item.faces": {
        text: "{count, plural, one {# face} other {# faces}}",
        description: "Line under a sprite tile naming how many face variants the chapter asks for.",
    },
    "illustrations.item.view": {
        text: "View {name} full size",
        description: "Accessible name of an artwork tile that opens the full-size viewer.",
    },
    "illustrations.viewer.close": {
        text: "Close the viewer",
        description: "Button that closes the full-size artwork viewer.",
    },
} satisfies MessageMap;

// `dynamic`: the sub-tab and count labels are keyed from an `ArtKind` value
// (`illustrations.kind.${kind}`, `illustrations.card.${kind}`), and the category
// labels from an `ArtCategory` (`stories.tab.${key}`, plus `.category.record`).
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
