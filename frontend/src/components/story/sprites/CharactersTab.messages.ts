import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "sprites.tab": {
        text: "Characters",
        description: "Story library tab listing every story sprite, operators and NPCs, with their expressions.",
    },
    "sprites.subtitle": {
        text: "{count} story sprites: {operators} operator sets and {npcs} other characters, named by who speaks while they are on stage.",
        description: "Line under the character gallery heading. The counts are preformatted numbers.",
    },
    "sprites.search.placeholder": {
        text: "Search a name, an operator or a sprite id",
        description: "Placeholder of the search box above the character gallery grid.",
    },
    "sprites.search.aria": {
        text: "Search characters",
        description: "Accessible name of the character gallery's search box.",
    },
    "sprites.kind.aria": {
        text: "Show",
        description: "Accessible name of the row of kind filters (All, Operators, NPCs) on the character gallery.",
    },
    "sprites.kind.all": {
        text: "All",
        description: "Kind filter showing every story sprite.",
    },
    "sprites.kind.operator": {
        text: "Operators",
        description: "Kind filter showing only sprites of playable operators.",
    },
    "sprites.kind.npc": {
        text: "NPCs",
        description: "Kind filter showing only non-playable story characters.",
    },
    "sprites.sort.aria": {
        text: "Sort by",
        description: "Accessible name of the sort control on the character gallery.",
    },
    "sprites.sort.appearances": {
        text: "Most stories",
        description: "Sort option: characters that appear in the most stories first.",
    },
    "sprites.sort.name": {
        text: "Name",
        description: "Sort option: alphabetical by the character's display name.",
    },
    "sprites.sort.firstSeen": {
        text: "First seen",
        description: "Sort option: characters in the order their first event released; characters only in main chapters come last.",
    },
    "sprites.results": {
        text: "{count, plural, one {# character} other {# characters}}",
        description: "Count of characters the current search and filter show.",
    },
    "sprites.empty": {
        text: "No character matches that search.",
        description: "Empty state of the character gallery grid.",
    },
    "sprites.showMore": {
        text: "Show {count} more ({remaining} left)",
        description: "Button under the character grid that mounts the next page of cards.",
    },
    "sprites.loading": {
        text: "Loading characters",
        description: "Accessible label of the character gallery's loading placeholders.",
    },
    "sprites.failed": {
        text: "The character list could not be loaded.",
        description: "Error state of the character gallery.",
    },
    "sprites.unavailable": {
        text: "The character gallery is not available yet.",
        description: "Shown when the backend does not serve the character gallery route.",
    },
    "sprites.unavailableNote": {
        text: "The server answering this page predates it; it appears after the next backend restart.",
        description: "Second line under the gallery's unavailable state, for a backend that predates the route.",
    },
    "sprites.card.open": {
        text: "Open {name}'s expressions",
        description: "Accessible name of a character card, which opens its expression sheet.",
    },
    "sprites.card.aliases": {
        text: "{count, plural, one {+# name} other {+# names}}",
        description: "On a character card: how many other names the scripts give this character.",
    },
    "sprites.card.stories": {
        text: "{count, plural, one {# story} other {# stories}}",
        description: "On a character card: how many stories this sprite appears in.",
    },
    "sprites.badge.operator": {
        text: "Operator",
        description: "Badge on a character card whose sprite belongs to a playable operator.",
    },
    "sprites.badge.npc": {
        text: "NPC",
        description: "Badge on a character card whose sprite is a non-playable character.",
    },
    "sprites.sheet.expressions": {
        text: "{count, plural, one {# expression} other {# expressions}}",
        description: "Heading of the expression grid on a character's sheet.",
    },
    "sprites.sheet.body": {
        text: "Body {body}",
        description: "Heading of one group of expressions that share a body pose (the $M index).",
    },
    "sprites.sheet.names": {
        text: "Named in scripts",
        description: "Heading of the list of display names the scripts give a character, primary name first, with line counts.",
    },
    "sprites.name.open": {
        text: "Where {name} is spoken",
        description: "Accessible name of a name chip on a character's sheet, which opens the stories and example lines for that name.",
    },
    "sprites.name.count": {
        text: "{count, plural, one {# line} other {# lines}}",
        description: "In a name's popover: how many lines the scripts speak under this name while the character is lit.",
    },
    "sprites.name.share": {
        text: "{share} of {owner}'s lines",
        description: "In a name's popover: this name's share of the character's named lines; share is a formatted percentage, owner the character's primary name.",
    },
    "sprites.name.stories": {
        text: "Stories",
        description: "Heading of the stories that use this name, in a name's popover.",
    },
    "sprites.name.more": {
        text: "+{count} more",
        description: "Under the five stories listed in a name's popover: how many more stories use the name.",
    },
    "sprites.name.examples": {
        text: "Lines",
        description: "Heading of up to three example lines spoken under this name, each linking into the reader at that line.",
    },
    "sprites.stray.toggle": {
        text: "{lines, plural, one {# stray line} other {# stray lines}} under {names, plural, one {# other name} other {# other names}}",
        description: "Disclosure under a character's names: lines spoken under names too rare to be aliases, mostly someone else talking while this character stays highlighted. Opens the list of those names.",
    },
    "sprites.stray.more": {
        text: "+{count} more",
        description: "At the end of the first 30 stray names: shows the rest.",
    },
    "sprites.sheet.noNames": {
        text: "No named line is spoken while this sprite is lit.",
        description: "Shown instead of the names list when no line was attributed to the sprite.",
    },
    "sprites.sheet.lineCount": {
        text: "{count, plural, one {# line} other {# lines}}",
        description: "Line count beside a name or a story on a character's sheet. Counts can be fractional when two characters were lit at once.",
    },
    "sprites.sheet.stories": {
        text: "Appears in",
        description: "Heading of the list of stories a character appears in.",
    },
    "sprites.sheet.onStage": {
        text: "on stage",
        description: "Beside a story in which the character appears but never speaks.",
    },
    "sprites.sheet.folder": {
        text: "Sprite {base}",
        description: "The sprite folder id shown on a character's sheet.",
    },
    "sprites.sheet.variant": {
        text: "Set {variant}",
        description: "The operator's sprite set or outfit tag (for example 1, 2 or ex) shown on a character's sheet.",
    },
    "sprites.sheet.operator": {
        text: "Operator page",
        description: "Link from a character's sheet to the operator's own page.",
    },
    "sprites.sheet.firstSeen": {
        text: "First seen {date}",
        description: "The release date of the first event that shows this character.",
    },
    "sprites.sheet.uses": {
        text: "{count, plural, =0 {unused} one {used once} other {used # times}}",
        description: "How often the scripts put this expression on stage.",
    },
    "sprites.sheet.wholeBody": {
        text: "whole body",
        description: "Marks an expression drawn as its own full plate rather than a face patch on a shared body.",
    },
    "sprites.sheet.variantAria": {
        text: "Expression {key}",
        description: "Accessible name of one expression in the sheet's grid; key is the script code such as #3$1.",
    },
    "sprites.sheet.gridAria": {
        text: "Expressions. Arrow keys move between them.",
        description: "Accessible name of the expression grid, which arrow keys navigate.",
    },
    "sprites.sheet.download": {
        text: "Download sheet",
        description: "Button that saves every expression of the character, at full resolution, as one PNG.",
    },
    "sprites.sheet.scaled": {
        text: "This character has too many expressions for a full-size image, so the sheet was saved smaller.",
        description: "Note after a download whose sheet was scaled down to stay inside the browser's image size limit.",
    },
    "sprites.sheet.downloading": {
        text: "Composing…",
        description: "Label of the download button while the PNG is being drawn.",
    },
    "sprites.sheet.downloadFailed": {
        text: "The sheet could not be drawn.",
        description: "Shown when composing the expression sheet PNG fails.",
    },
    "sprites.sheet.close": {
        text: "Close",
        description: "Button that closes the expression sheet dialog.",
    },
    "sprites.sheet.notFound": {
        text: "There is no story sprite called {base}.",
        description: "Shown on a character page whose sprite id is unknown.",
    },
    "sprites.sheet.loading": {
        text: "Loading expressions",
        description: "Accessible label of the expression sheet's loading placeholders.",
    },
} satisfies MessageMap;

// `dynamic`: the kind, sort and badge labels are keyed from union values
// (`sprites.kind.${kind}`, `sprites.sort.${sort}`, `sprites.badge.${kind}`).
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
