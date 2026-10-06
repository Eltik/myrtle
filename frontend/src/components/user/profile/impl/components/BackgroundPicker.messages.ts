import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.background.title": {
        text: "Profile background",
        description: "Title of the dialog where the profile's owner picks the art shown behind their profile header.",
    },
    "profile.background.description": {
        text: "Pick any outfit, operator art or gallery picture. It shows behind your header for everyone who can see your profile.",
        description: "Explains the background picker. 'Outfit' is the site's word for a skin; a gallery picture is a scene from an event's or Integrated Strategies' Archives gallery.",
    },
    "profile.background.source": {
        text: "Art source",
        description: "Accessible label of the tab row that switches the background picker between character art and gallery pictures.",
    },
    "profile.background.source.characters": {
        text: "Character art",
        description: "Tab of the background picker listing outfits and operator art.",
    },
    "profile.background.source.gallery": {
        text: "Gallery",
        description: "Tab of the background picker listing the Archives gallery pictures: the scene artwork events and Integrated Strategies collect in their archive.",
    },
    "profile.background.gallery.loading": {
        text: "Loading the gallery…",
        description: "Shown while the gallery pictures load in the background picker.",
    },
    "profile.background.gallery.error": {
        text: "Couldn't load the gallery.",
        description: "Shown when the gallery pictures failed to load in the background picker.",
    },
    "profile.background.gallery.retry": {
        text: "Retry",
        description: "Button that loads the gallery pictures again after a failure.",
    },
    "profile.background.gallery.stories": {
        text: "Story",
        description: "Heading of the Gallery tab's list of stories (main story chapters, events, vignettes, Integrated Strategies themes) that filters the pictures to one of them.",
    },
    "profile.background.gallery.allGroups": {
        text: "All stories",
        description: "First entry of the Gallery tab's story list: shows the pictures of every story.",
    },
    "profile.background.gallery.storyButton": {
        text: "Story: {name}",
        description: "Button on a narrow background picker that opens the story list. {name} is the chosen story, or 'All stories'.",
    },
    "profile.background.gallery.storySearchPlaceholder": {
        text: "Find a story",
        description: "Placeholder of the search box above the Gallery tab's story list.",
    },
    "profile.background.gallery.storySearchLabel": {
        text: "Find a story by name",
        description: "Accessible label of the search box that narrows the Gallery tab's story list.",
    },
    "profile.background.gallery.storyEmpty": {
        text: "No story matches.",
        description: "Shown in the Gallery tab's story list when its search or the chosen categories leave no story.",
    },
    "profile.background.gallery.categories": {
        text: "Category",
        description: "Heading of the Gallery tab's category filter (Main Story, Side Stories, Vignettes, Integrated Strategies...). Several can be on at once.",
    },
    "profile.background.gallery.categoriesLabel": {
        text: "Filter by story category",
        description: "Accessible label of the Gallery tab's row of category toggles.",
    },
    "profile.background.gallery.categoryOption": {
        text: "{label}, {count, plural, one {# picture} other {# pictures}}",
        description: "Accessible label of one category toggle in the Gallery tab. {label} is the category, {count} how many pictures it holds under the other filters.",
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
        description: "Button beside the active-filter chips that turns off every category, story and search filter of the Gallery tab.",
    },
    "profile.background.gallery.grid": {
        text: "Gallery pictures. Arrow keys move between pictures, Enter picks one.",
        description: "Accessible label of the grid of gallery picture tiles.",
    },
    "profile.background.gallery.searchPlaceholder": {
        text: "Search gallery pictures",
        description: "Placeholder of the search box in the background picker's Gallery tab.",
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
    "profile.background.gallery.sources": {
        text: "Picture source",
        description: "Accessible label of the row of buttons that switch the Gallery tab between Archives pictures, story CGs and story scenes.",
    },
    "profile.background.gallery.source.archive_pic": {
        text: "Archives",
        description: "Source button of the Gallery tab: the Archives gallery pictures events and Integrated Strategies collect.",
    },
    "profile.background.gallery.source.story_cg": {
        text: "Story CGs",
        description: "Source button of the Gallery tab: the full-screen illustrations (CGs) shown during story scenes.",
    },
    "profile.background.gallery.source.story_scene": {
        text: "Scenes",
        description: "Source button of the Gallery tab: the background plates (locations) story scenes are set against.",
    },
    "profile.background.gallery.category.main": {
        text: "Main Story",
        description: "Filter button of the Gallery tab's story sources: pictures from main story chapters.",
    },
    "profile.background.gallery.category.side": {
        text: "Side Stories",
        description: "Filter button of the Gallery tab's story sources: pictures from side story events.",
    },
    "profile.background.gallery.category.vignette": {
        text: "Vignettes",
        description: "Filter button of the Gallery tab's story sources: pictures from vignette (mini story) events.",
    },
    "profile.background.gallery.category.is": {
        text: "Integrated Strategies",
        description: "Filter button of the Gallery tab's story sources: pictures from Integrated Strategies stories.",
    },
    "profile.background.gallery.category.reclamation": {
        text: "Reclamation Algorithm",
        description: "Filter button of the Gallery tab's story sources: pictures from Reclamation Algorithm stories.",
    },
    "profile.background.gallery.category.sideContent": {
        text: "Other Stories",
        description: "Filter button of the Gallery tab's story sources: pictures from other story content outside events.",
    },
    "profile.background.gallery.category.record": {
        text: "Operator Records",
        description: "Filter button of the Gallery tab's story sources: pictures from operator record stories.",
    },
    "profile.background.cropX": {
        text: "Horizontal crop",
        description: "Label of the slider that moves which part of the chosen art shows in the header, from its left edge to its right. Disabled while the art is no wider than the header.",
    },
    "profile.background.cropY": {
        text: "Vertical crop",
        description: "Label of the slider that moves which part of the chosen art shows in the header, from its top to its bottom.",
    },
    "profile.background.cropFitsShort": {
        text: "Fits, zoom in",
        description: "Short note beside a disabled crop slider's label: the art already fits the header on that axis, so zooming in is what makes it croppable. Keep it to a few words.",
    },
    "profile.background.cropXFits": {
        text: "The art fits the header's width at this zoom, so there is nothing to crop sideways. Zoom in to crop it.",
        description: "Tooltip of the short note beside a disabled horizontal crop slider: on this screen the chosen art is exactly as wide as the header, so moving it sideways would change nothing until it is zoomed in.",
    },
    "profile.background.cropYFits": {
        text: "The art fits the header's height at this zoom, so there is nothing to crop vertically. Zoom in to crop it.",
        description: "Tooltip of the short note beside a disabled vertical crop slider: on this screen the chosen art is exactly as tall as the header, so moving it up or down would change nothing until it is zoomed in.",
    },
    "profile.background.zoom": {
        text: "Zoom",
        description: "Label of the slider that zooms the chosen art in the profile header, from just filling it (100%) to three times that.",
    },
    "profile.background.zoomValue": {
        text: "{scale}%",
        description: "The zoom slider's current value. {scale} is a whole number from 100 to 300.",
    },
    "profile.background.panHint": {
        text: "Drag the picture in the header to position it, and scroll over it to zoom. The sliders do the same.",
        description: "Hint under the zoom and crop sliders. 'The header' is the profile header above the dialog, which previews the background live and can be dragged and zoomed directly.",
    },
    "profile.background.move": {
        text: "Move dialog",
        description: "Accessible label and tooltip of the grip in the dialog's title bar. Drag it, or focus it and press the arrow keys, to move the dialog.",
    },
    "profile.background.resize": {
        text: "Resize dialog",
        description: "Accessible label and tooltip of the grip in the dialog's bottom-right corner. Drag it, or focus it and press the arrow keys, to resize the dialog.",
    },
    "profile.background.none": {
        text: "No background. Your header shows its usual look.",
        description: "Shown in place of the crop slider when no art is chosen.",
    },
    "profile.background.remove": {
        text: "Remove background",
        description: "Footer button that clears the chosen art, back to the header's usual look. Takes effect on Save.",
    },
    "profile.background.cancel": {
        text: "Cancel",
        description: "Footer button that closes the dialog and discards the unsaved pick.",
    },
    "profile.background.save": {
        text: "Save",
        description: "Footer button that saves the chosen background.",
    },
    "profile.background.saved.title": {
        text: "Background saved",
        description: "Toast title after the background was saved.",
    },
    "profile.background.removed.title": {
        text: "Background removed",
        description: "Toast title after the background was removed and saved.",
    },
    "profile.background.saveFailed.title": {
        text: "Couldn't save the background",
        description: "Toast title when saving the background fails. The body explains why.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
