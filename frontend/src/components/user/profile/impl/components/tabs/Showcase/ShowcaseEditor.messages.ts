import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.showcase.editor.kicker": {
        text: "Edit showcase",
        description: "Small uppercase label above the showcase editor.",
    },
    "profile.showcase.editor.help": {
        text: "Add blocks, then drag them or use the arrows to set the order. Visitors see them top to bottom, before any other tab.",
        description: "Explanation under the showcase editor's label.",
    },
    "profile.showcase.editor.list": {
        text: "Showcase blocks",
        description: "Accessible name of the reorderable list of showcase blocks in the editor.",
    },
    "profile.showcase.editor.empty": {
        text: "No blocks yet. Add one to start your showcase.",
        description: "Shown in the editor when the showcase has no block.",
    },
    "profile.showcase.editor.add": {
        text: "Add block",
        description: "Button in the showcase editor that opens the dialog for adding a block.",
    },
    "profile.showcase.editor.full": {
        text: "A showcase holds up to {max} blocks.",
        description: "Shown when the showcase already holds the most blocks it can. {max} is that number (12).",
    },
    "profile.showcase.editor.count": {
        text: "{count} / {max} blocks",
        description: "How many blocks the showcase holds, out of the most it can.",
    },
    "profile.showcase.editor.cancel": {
        text: "Cancel",
        description: "Button that leaves the showcase editor without saving.",
    },
    "profile.showcase.editor.save": {
        text: "Save",
        description: "Button that saves the showcase.",
    },
    "profile.showcase.editor.dropped": {
        text: "{count, plural, one {# removed or empty item will be dropped when you save.} other {# removed or empty items will be dropped when you save.}}",
        description: "Notice in the showcase editor: blocks or favourites whose grid, tier list, plan or game entry no longer exists, and favourites blocks with nothing picked, are dropped on save.",
    },
    "profile.showcase.editor.plansHidden": {
        text: "Your Plans tab is hidden, so visitors don't see plan blocks.",
        description: "Notice in the showcase editor when the showcase has a plan block but the owner hid their Plans tab.",
    },
    "profile.showcase.editor.moved": {
        text: "{item} moved to position {position} of {total}",
        description: "Screen-reader announcement after a block or a favourite is moved. {item} names it.",
    },
    "profile.showcase.editor.moveUp": {
        text: "Move {item} up",
        description: "Accessible name of the button that moves a showcase block up one place.",
    },
    "profile.showcase.editor.moveDown": {
        text: "Move {item} down",
        description: "Accessible name of the button that moves a showcase block down one place.",
    },
    "profile.showcase.editor.remove": {
        text: "Remove {item}",
        description: "Accessible name of the button that removes a showcase block, or one favourite from a block.",
    },
    "profile.showcase.editor.type.favourites": {
        text: "Favourites",
        description: "Showcase block type: a row of favourite operators, skins, events or anything else of one type.",
    },
    "profile.showcase.editor.type.favouritesDesc": {
        text: "Up to 24 operators, skins, events, enemies, chapters or more, of one type.",
        description: "Description of the favourites block type in the add-block dialog.",
    },
    "profile.showcase.editor.type.grid": {
        text: "Grid",
        description: "Showcase block type: one of the player's grids.",
    },
    "profile.showcase.editor.type.gridDesc": {
        text: "One of your grids, or any grid by its link.",
        description: "Description of the grid block type in the add-block dialog.",
    },
    "profile.showcase.editor.type.tierList": {
        text: "Tier list",
        description: "Showcase block type: a tier list.",
    },
    "profile.showcase.editor.type.tierListDesc": {
        text: "One of your tier lists, or any tier list by its link.",
        description: "Description of the tier list block type in the add-block dialog.",
    },
    "profile.showcase.editor.type.plan": {
        text: "Plan",
        description: "Showcase block type: one of the player's operator plans shown on their profile.",
    },
    "profile.showcase.editor.type.planDesc": {
        text: "An operator plan you show on your profile.",
        description: "Description of the plan block type in the add-block dialog.",
    },
    "profile.showcase.editor.favouritesOf": {
        text: "Favourite {kind}",
        description: "Name of a favourites block in the editor when it has no title. {kind} is the plural type name, e.g. 'Operators'.",
    },
    "profile.showcase.editor.titleLabel": {
        text: "Block title",
        description: "Label of the favourites block's optional title field.",
    },
    "profile.showcase.editor.titlePlaceholder": {
        text: "{kind} (optional title)",
        description: "Placeholder of the favourites block's title field. {kind} is the plural type name shown when no title is set.",
    },
    "profile.showcase.editor.entities": {
        text: "{count} / {max}",
        description: "How many favourites a block holds, out of the most it can (24).",
    },
    "profile.showcase.editor.entitiesList": {
        text: "Favourites in this block",
        description: "Accessible name of the reorderable list of favourites inside one block.",
    },
    "profile.showcase.editor.entityHint": {
        text: "Drag to reorder, or focus a pick and use the arrow keys. Delete removes it.",
        description: "Hint under a favourites block's picks in the editor, explaining keyboard reordering.",
    },
    "profile.showcase.editor.pick": {
        text: "Choose",
        description: "Button that opens the picker to add or remove favourites in a block.",
    },
    "profile.showcase.editor.emptyFavourites": {
        text: "Nothing picked yet. An empty block is not saved.",
        description: "Shown inside a favourites block with no favourite yet.",
    },
    "profile.showcase.editor.unknownGrid": {
        text: "Grid {slug}",
        description: "Editor name of a grid block whose title could not be loaded. {slug} is the grid's slug.",
    },
    "profile.showcase.editor.unknownTierList": {
        text: "Tier list {slug}",
        description: "Editor name of a tier list block whose title could not be loaded. {slug} is its slug.",
    },
    "profile.showcase.editor.unknownPlan": {
        text: "A plan",
        description: "Editor name of a plan block whose plan could not be loaded.",
    },
    "profile.showcase.editor.saved.title": {
        text: "Showcase saved",
        description: "Toast title after the owner saves their showcase.",
    },
    "profile.showcase.editor.saved.body": {
        text: "Visitors now see these blocks first.",
        description: "Toast body after the owner saves their showcase.",
    },
    "profile.showcase.editor.saveFailed.title": {
        text: "Couldn't save your showcase",
        description: "Toast title when saving the showcase fails. The body explains why.",
    },
    "profile.showcase.add.title": {
        text: "Add a block",
        description: "Title of the add-block dialog in the showcase editor.",
    },
    "profile.showcase.add.description": {
        text: "Choose what the block shows.",
        description: "Line under the add-block dialog's title, on its first step.",
    },
    "profile.showcase.add.back": {
        text: "Back",
        description: "Button in the add-block dialog that returns to the choice of block type.",
    },
    "profile.showcase.add.close": {
        text: "Close",
        description: "Button that closes the add-block dialog.",
    },
    "profile.showcase.add.kindTitle": {
        text: "Favourites of which type?",
        description: "Heading of the add-block dialog's step where the owner chooses the type of entity a favourites block holds.",
    },
    "profile.showcase.add.gridTitle": {
        text: "Add a grid",
        description: "Heading of the add-block dialog's grid step.",
    },
    "profile.showcase.add.tierListTitle": {
        text: "Add a tier list",
        description: "Heading of the add-block dialog's tier list step.",
    },
    "profile.showcase.add.planTitle": {
        text: "Add a plan",
        description: "Heading of the add-block dialog's plan step.",
    },
    "profile.showcase.add.yours": {
        text: "Yours",
        description: "Label above the list of the owner's own grids or tier lists in the add-block dialog.",
    },
    "profile.showcase.add.noGrids": {
        text: "You have no grids yet.",
        description: "Shown in the add-block dialog's grid step when the owner has made no grid.",
    },
    "profile.showcase.add.noTierLists": {
        text: "You have no tier lists yet.",
        description: "Shown in the add-block dialog's tier list step when the owner has made no tier list.",
    },
    "profile.showcase.add.noPlans": {
        text: "No plan is shown on your profile. Turn on 'Show on profile' for a plan in the planner first.",
        description: "Shown in the add-block dialog's plan step when the owner shows no plan on their profile. 'Show on profile' names the planner's own option.",
    },
    "profile.showcase.add.added": {
        text: "Added",
        description: "Marks a grid, tier list or plan the showcase already holds, in the add-block dialog.",
    },
    "profile.showcase.add.pasteLabel": {
        text: "Or paste a link",
        description: "Label of the field where the owner pastes a link to, or the slug of, any grid or tier list.",
    },
    "profile.showcase.add.pastePlaceholder": {
        text: "Link or slug",
        description: "Placeholder of the paste-a-link field in the add-block dialog.",
    },
    "profile.showcase.add.pasteAdd": {
        text: "Add",
        description: "Button that adds the grid or tier list named by the pasted link.",
    },
    "profile.showcase.add.notALink": {
        text: "That isn't a link to a grid.",
        description: "Error under the paste field on the grid step when the text is not a grid link or slug.",
    },
    "profile.showcase.add.notATierListLink": {
        text: "That isn't a link to a tier list.",
        description: "Error under the paste field on the tier list step when the text is not a tier list link or slug.",
    },
    "profile.showcase.add.notFound": {
        text: "Nothing found at that link.",
        description: "Error under the paste field when the grid or tier list it names does not exist.",
    },
    "profile.showcase.add.checking": {
        text: "Checking…",
        description: "Shown while the add-block dialog checks that a pasted grid or tier list exists.",
    },
    "profile.showcase.add.size": {
        text: "{rows} × {cols}",
        description: "A grid's size in the add-block dialog's list of the owner's grids.",
    },
    "profile.showcase.picker.title": {
        text: "Choose {kind}",
        description: "Title of the picker for a favourites block. {kind} is the plural type name, e.g. 'Operators'.",
    },
    "profile.showcase.picker.description": {
        text: "Pick up to {max}. Pick one again to take it out.",
        description: "Line under the favourites picker's title. {max} is the most a block holds (24).",
    },
    "profile.showcase.picker.count": {
        text: "{count} of {max} picked",
        description: "How many favourites the block holds, in the picker's footer.",
    },
    "profile.showcase.picker.done": {
        text: "Done",
        description: "Button that closes the favourites picker.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
