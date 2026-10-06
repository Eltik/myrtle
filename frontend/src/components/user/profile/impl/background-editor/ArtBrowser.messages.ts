import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.background.source": {
        text: "Source",
        description: "Heading of the background editor's list of art sources (character art, and the gallery's Archives, story CGs and scenes), and accessible label of the row of source buttons on a phone.",
    },
    "profile.background.source.characters": {
        text: "Character art",
        description: "Source of the background editor listing outfits and operator art.",
    },
    "profile.background.gallery.loading": {
        text: "Loading the gallery…",
        description: "Shown while the gallery pictures load in the background editor.",
    },
    "profile.background.gallery.error": {
        text: "Couldn't load the gallery.",
        description: "Shown when the gallery pictures failed to load in the background editor.",
    },
    "profile.background.gallery.retry": {
        text: "Retry",
        description: "Button that loads the gallery pictures again after a failure.",
    },
    "profile.background.gallery.allGroups": {
        text: "All stories",
        description: "First entry of the gallery's story list: shows the pictures of every story.",
    },
    "profile.background.gallery.storySearchPlaceholder": {
        text: "Find a story",
        description: "Placeholder of the search box above the gallery's story list.",
    },
    "profile.background.gallery.storySearchLabel": {
        text: "Find a story by name",
        description: "Accessible label of the search box that narrows the gallery's story list.",
    },
    "profile.background.gallery.storyEmpty": {
        text: "No story matches.",
        description: "Shown in the gallery's story list when its search or the chosen categories leave no story.",
    },
    "profile.background.gallery.categoriesLabel": {
        text: "Filter by story category",
        description: "Accessible label of the gallery's row of category toggles.",
    },
    "profile.background.gallery.categoryOption": {
        text: "{label}, {count, plural, one {# picture} other {# pictures}}",
        description: "Accessible label of one category toggle in the gallery. {label} is the category, {count} how many pictures it holds under the other filters.",
    },
    "profile.background.gallery.count": {
        text: "{count, plural, one {# picture} other {# pictures}}",
        description: "How many gallery pictures the current source, filters and search leave.",
    },
    "profile.background.gallery.removeFilter": {
        text: "Remove filter: {name}",
        description: "Accessible label of the x on one active-filter chip above the gallery pictures. {name} is the category or story it filters by.",
    },
    "profile.background.gallery.clearFilters": {
        text: "Clear all",
        description: "Button beside the active-filter chips that turns off every category, story and search filter of the gallery.",
    },
    "profile.background.gallery.grid": {
        text: "Gallery pictures. Arrow keys move between pictures, Enter picks one.",
        description: "Accessible label of the grid of gallery picture tiles.",
    },
    "profile.background.gallery.searchPlaceholder": {
        text: "Search gallery pictures",
        description: "Placeholder of the search box in the background editor's gallery.",
    },
    "profile.background.gallery.searchLabel": {
        text: "Search gallery pictures by title, event or story",
        description: "Accessible label of the gallery search box. It matches a picture's title and the name of the event or theme it belongs to.",
    },
    "profile.background.gallery.empty": {
        text: "No pictures match.",
        description: "Shown when the gallery search and filter leave no picture.",
    },
    "profile.background.gallery.tileLabel": {
        text: "{title}, {group}",
        description: "Accessible label and tooltip of one gallery picture tile. {title} is the picture's title, {group} the event or Integrated Strategies theme it belongs to.",
    },
    "profile.background.gallery.storyTileLabel": {
        text: "From {group}",
        description: "Accessible label and tooltip of one story CG or story scene tile, which has no title of its own. {group} is the story (main story chapter, event or vignette) whose scripts first show it.",
    },
    "profile.background.gallery.source.archive_pic": {
        text: "Archives",
        description: "Gallery source button: the Archives gallery pictures events and Integrated Strategies collect.",
    },
    "profile.background.gallery.source.story_cg": {
        text: "Story CGs",
        description: "Gallery source button: the full-screen illustrations (CGs) shown during story scenes.",
    },
    "profile.background.gallery.source.story_scene": {
        text: "Scenes",
        description: "Gallery source button: the background plates (locations) story scenes are set against.",
    },
    "profile.background.gallery.category.main": {
        text: "Main Story",
        description: "Category filter of the gallery's story sources: pictures from main story chapters.",
    },
    "profile.background.gallery.category.side": {
        text: "Side Stories",
        description: "Category filter of the gallery's story sources: pictures from side story events.",
    },
    "profile.background.gallery.category.vignette": {
        text: "Vignettes",
        description: "Category filter of the gallery's story sources: pictures from vignette (mini story) events.",
    },
    "profile.background.gallery.category.is": {
        text: "Integrated Strategies",
        description: "Category filter of the gallery's story sources: pictures from Integrated Strategies stories.",
    },
    "profile.background.gallery.category.reclamation": {
        text: "Reclamation Algorithm",
        description: "Category filter of the gallery's story sources: pictures from Reclamation Algorithm stories.",
    },
    "profile.background.gallery.category.sideContent": {
        text: "Other Stories",
        description: "Category filter of the gallery's story sources: pictures from other story content outside events.",
    },
    "profile.background.gallery.category.record": {
        text: "Operator Records",
        description: "Category filter of the gallery's story sources: pictures from operator record stories.",
    },
    "profile.background.source.gallery": {
        text: "Gallery",
        description: "Small label over the background editor's gallery sources in its side rail: the Archives pictures, the story CGs and the story scenes.",
    },
    "profile.background.gallery.categories": {
        text: "Category",
        description: "Small label over the category toggles (Main Story, Side Stories, Vignettes, Integrated Strategies...) in the background editor's side rail. Several can be on at once.",
    },
    "profile.background.gallery.stories": {
        text: "Story",
        description: "Small label over the background editor's searchable list of stories (main story chapters, events, vignettes, Integrated Strategies themes) that filters the pictures to one of them.",
    },
    "profile.background.tileSize": {
        text: "Tile size",
        description: "Accessible label of the Small / Medium / Large toggle that sets how big the gallery picture tiles are.",
    },
    "profile.background.tileSize.s": {
        text: "Small tiles",
        description: "Accessible label and tooltip of the toggle that shows the gallery pictures as small tiles. The button itself shows only the letter S.",
    },
    "profile.background.tileSize.m": {
        text: "Medium tiles",
        description: "Accessible label and tooltip of the toggle that shows the gallery pictures as medium tiles. The button itself shows only the letter M.",
    },
    "profile.background.tileSize.l": {
        text: "Large tiles",
        description: "Accessible label and tooltip of the toggle that shows the gallery pictures as large tiles. The button itself shows only the letter L.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
