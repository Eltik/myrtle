import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.background.title": {
        text: "Profile background",
        description: "Title of the full-screen editor where the profile's owner picks the art shown behind their profile header and positions it.",
    },
    "profile.background.description": {
        text: "Pick an art below and position it in the header preview. It shows behind your header for everyone who can see your profile.",
        description: "Screen-reader description of the background editor.",
    },
    "profile.background.mode": {
        text: "Preview as",
        description: "Accessible label of the Desktop / Phone toggle that switches the header preview between the two layouts visitors see.",
    },
    "profile.background.mode.desktop": {
        text: "Desktop",
        description: "Toggle of the background editor: preview the header as it shows on this desktop screen.",
    },
    "profile.background.mode.phone": {
        text: "Phone",
        description: "Toggle of the background editor: preview the header as it shows on a phone.",
    },
    "profile.background.preview.role": {
        text: "header preview",
        description: "Screen-reader role description of the draggable header preview in the background editor. Lower case: it is read after the label.",
    },
    "profile.background.preview.label": {
        text: "Background position",
        description: "Accessible label of the header preview in the background editor, which is dragged or moved with the arrow keys to position the art.",
    },
    "profile.background.preview.keys": {
        text: "Arrow keys move the picture, Shift with an arrow moves it further, plus and minus zoom.",
        description: "Screen-reader instructions of the focused header preview in the background editor.",
    },
    "profile.background.zoom": {
        text: "Zoom",
        description: "Label of the slider that zooms the chosen art in the profile header, from just filling it (100%) to three times that.",
    },
    "profile.background.zoomValue": {
        text: "{scale}%",
        description: "The zoom slider's current value. {scale} is a whole number from 100 to 300.",
    },
    "profile.background.fit": {
        text: "Fit",
        description: "Button under the header preview that sets the zoom back to 100%, the art just filling the header, keeping its position.",
    },
    "profile.background.reset": {
        text: "Reset",
        description: "Button under the header preview that puts the art back where a fresh pick starts: default position and no zoom.",
    },
    "profile.background.panHint": {
        text: "Drag the picture to position it. Scroll or pinch to zoom.",
        description: "Hint beside the zoom controls of the background editor, about the header preview above them.",
    },
    "profile.background.deadX": {
        text: "Zoom in to move it sideways",
        description: "Shown for a moment when the owner drags the picture sideways but it already fills the header's width exactly, so nothing can move that way until it is zoomed in.",
    },
    "profile.background.deadY": {
        text: "Zoom in to move it up or down",
        description: "Shown for a moment when the owner drags the picture up or down but it already fills the header's height exactly, so nothing can move that way until it is zoomed in.",
    },
    "profile.background.elite": {
        text: "Art",
        description: "Accessible label of the Elite 1 / Elite 2 toggle under the header preview, shown when the background is an operator's art: which promotion's illustration it draws.",
    },
    "profile.background.elite.e1": {
        text: "Elite 1",
        description: "Toggle under the header preview: draw the operator's Elite 1 (first promotion) illustration.",
    },
    "profile.background.elite.e2": {
        text: "Elite 2",
        description: "Toggle under the header preview: draw the operator's Elite 2 (second promotion) illustration.",
    },
    "profile.background.elite.noE2": {
        text: "This operator has no Elite 2 art.",
        description: "Tooltip of the disabled Elite 2 toggle: three-star and lower operators stop at Elite 1, so there is no second illustration.",
    },
    "profile.background.expand": {
        text: "Full preview",
        description: "Button in the compact preview strip that shows while the owner scrolls through the art: scrolls back up to the full-size header preview.",
    },
    "profile.background.none": {
        text: "No background. Your header shows its usual look.",
        description: "Shown in place of the zoom controls when no art is chosen.",
    },
    "profile.background.remove": {
        text: "Remove background",
        description: "Button under the header preview that clears the chosen art, back to the header's usual look. Takes effect on Save.",
    },
    "profile.background.cancel": {
        text: "Cancel",
        description: "Top-bar button that closes the background editor without saving. Asks first when there are unsaved changes.",
    },
    "profile.background.save": {
        text: "Save",
        description: "Top-bar button that saves the chosen background.",
    },
    "profile.background.discard.title": {
        text: "Discard changes?",
        description: "Title of the confirmation shown when the owner closes the background editor with unsaved changes.",
    },
    "profile.background.discard.description": {
        text: "Your new background isn't saved. Closing the editor drops it.",
        description: "Body of the confirmation shown when the owner closes the background editor with unsaved changes.",
    },
    "profile.background.discard.confirm": {
        text: "Discard",
        description: "Button of the confirmation that closes the background editor and drops the unsaved changes.",
    },
    "profile.background.discard.keep": {
        text: "Keep editing",
        description: "Button of the confirmation that goes back to the background editor with the changes kept.",
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
