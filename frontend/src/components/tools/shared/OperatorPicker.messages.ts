import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The picker is shared by the DPS and HPS calculators, which call the thing
 * being picked an "operator" and a "healer" respectively. That word is inside
 * the sentence, and the article in front of it ("a"/"an") depends on it, so
 * each string carries both wordings as an ICU `select` rather than pasting a
 * noun into a template.
 */
export const namespace = "tools";

export const messages = {
    "calc.picker.addFirst": {
        text: "{noun, select, healer {Add a healer} other {Add an operator}}",
        description: "Label over the picker before anything has been added.",
    },
    "calc.picker.addAnother": {
        text: "{noun, select, healer {Add another healer} other {Add another operator}}",
        description: "Label over the picker once at least one has been added.",
    },
    "calc.picker.loading": {
        text: "{noun, select, healer {Loading healers...} other {Loading operators...}}",
        description: "Placeholder in the picker's search box while the list is still loading. Three full stops, not an ellipsis character.",
    },
    "calc.picker.search": {
        text: "{noun, select, healer {Search {count, plural, one {# healer} other {# healers}}...} other {Search {count, plural, one {# operator} other {# operators}}...}}",
        description: "Placeholder in the picker's search box, e.g. 'Search 287 operators...'. Three full stops, not an ellipsis character.",
    },
    "calc.picker.empty": {
        text: "{noun, select, healer {No matching healers.} other {No matching operators.}}",
        description: "Shown inside the picker dropdown when the typed query matches nothing.",
    },
    "calc.picker.loadFailed": {
        text: "{noun, select, healer {Couldn't load healer list.} other {Couldn't load operator list.}}",
        description: "Error line under the picker when the list request failed. The server's own message follows after a space.",
    },
    "calc.picker.hint.first": {
        text: "{noun, select, healer {Pick any healer to start. You can add the same one multiple times to compare different builds.} other {Pick any operator to start. You can add the same one multiple times to compare different builds.}}",
        description: "Hint under an empty picker. 'Build' means one configuration of level, skill and module.",
    },
    "calc.picker.hint.second": {
        text: "{noun, select, healer {Tip: add the same healer again with the picker, then tweak skill/level/module to compare two setups.} other {Tip: add the same operator again with the picker, then tweak skill/level/module to compare two setups.}}",
        description: "Hint under the picker once exactly one entry has been added.",
    },
    "calc.picker.rarityPrefix": {
        text: "{rarity}★ · ",
        description: "Star rating in front of an operator's class in the picker dropdown, e.g. '6★ · '. Keep the trailing separator and space; the class name follows.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
