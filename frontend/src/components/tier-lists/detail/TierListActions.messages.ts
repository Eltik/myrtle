import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.actions.favorite": {
        text: "Favorite",
        description: "Button that saves this tier list to the visitor's favourites.",
    },
    "detail.actions.favorited": {
        text: "Favorited",
        description: "The same button's label once the list is saved, so pressing it again removes it.",
    },
    "detail.actions.favorite.add": {
        text: "Add to favorites",
        description: "Accessible name of the favourite button while the list is not saved.",
    },
    "detail.actions.favorite.remove": {
        text: "Remove from favorites",
        description: "Accessible name of the favourite button while the list is saved.",
    },
    "detail.actions.favorite.errTitle": {
        text: "Couldn't update favorite",
        description: "Toast title when saving or unsaving the list failed.",
    },
    "detail.actions.favorite.errBody": {
        text: "Something went wrong. Try again in a moment.",
        description: "Toast body when saving or unsaving the list failed.",
    },
    "detail.actions.favorite.authTitle": {
        text: "Sign in to favorite",
        description: "Toast title when a signed-out visitor tries to save the list.",
    },
    "detail.actions.favorite.authBody": {
        text: "Connect your account to save lists for later.",
        description: "Toast body telling a signed-out visitor to sign in before saving a list.",
    },
    "detail.actions.download": {
        text: "Download",
        description: "Button that downloads the tier list as an image.",
    },
    "detail.actions.downloading": {
        text: "Preparing…",
        description: "The download button's label while the image is being rendered. Keep the single-character ellipsis.",
    },
    "detail.actions.download.label": {
        text: "Download tier list as image",
        description: "Accessible name of the download button.",
    },
    "detail.actions.download.errTitle": {
        text: "Couldn't download image",
        description: "Toast title when rendering the tier-list image failed.",
    },
    "detail.actions.download.errBody": {
        text: "Something went wrong while preparing the image. Try again in a moment.",
        description: "Toast body when rendering the tier-list image failed.",
    },
    "detail.actions.share": {
        text: "Share",
        description: "Button that shares the list, or copies its link when the device cannot share.",
    },
    "detail.actions.share.label": {
        text: "Share this tier list",
        description: "Accessible name of the share button.",
    },
    "detail.actions.share.text": {
        text: "{title} - tier list on myrtle.moe",
        description: "Text handed to the device's share sheet when the list has no description of its own. myrtle.moe is the site's name and stays as-is.",
    },
    "detail.actions.share.okTitle": {
        text: "Link copied",
        description: "Toast title after the list's link was copied to the clipboard.",
    },
    "detail.actions.share.okBody": {
        text: "Share it anywhere.",
        description: "Toast body after the list's link was copied to the clipboard.",
    },
    "detail.actions.share.errTitle": {
        text: "Couldn't copy link",
        description: "Toast title when copying the list's link failed.",
    },
    "detail.actions.share.errBody": {
        text: "Clipboard access was denied.",
        description: "Toast body when the browser refused clipboard access.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
