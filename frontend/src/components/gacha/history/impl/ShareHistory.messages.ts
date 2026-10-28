import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.public.label": {
        text: "Public pull history",
        description: "Label of the switch that lets anyone who opens the player's profile, or a link they shared, see their pull history.",
    },
    "history.public.on": {
        text: "Anyone with your profile link can see your pulls.",
        description: "Caption under the public pull history switch while it is on.",
    },
    "history.public.off": {
        text: "Only you can see your pulls.",
        description: "Caption under the public pull history switch while it is off, the default.",
    },
    "history.public.savedOn": {
        text: "Your pull history is public",
        description: "Toast title after the player turned the public pull history switch on.",
    },
    "history.public.savedOff": {
        text: "Your pull history is private",
        description: "Toast title after the player turned the public pull history switch off.",
    },
    "history.public.saveFailed": {
        text: "Couldn't change who can see your pulls",
        description: "Toast title when saving the public pull history switch failed. The toast body says why.",
    },
    "history.share.action": {
        text: "Share",
        description: "Button that copies a link to the player's pull history, limited to the banners chosen in the filter, for others to open.",
    },
    "history.share.copiedAll": {
        text: "Link to your pull history copied",
        description: "Toast title after the share link was copied with no banner filter, so it shows the whole history.",
    },
    "history.share.copiedSome": {
        text: "{count, plural, one {Link to # banner's pulls copied} other {Link to # banners' pulls copied}}",
        description: "Toast title after the share link was copied with a banner filter; {count} is how many banners the link shows.",
    },
    "history.share.copyFailed": {
        text: "Couldn't copy the link",
        description: "Toast title when the browser refused to put the share link on the clipboard.",
    },
    "history.share.private.title": {
        text: "Make your pull history public?",
        description: "Title of the dialog shown when the player presses Share while their pull history is private.",
    },
    "history.share.private.body": {
        text: "Your pull history is private, so a shared link would open only for you. Making it public lets anyone with the link, or anyone visiting your profile, see your pulls and the banners you pulled on. You can make it private again at any time, here or in Settings.",
        description: "Body of the dialog shown when the player presses Share while their pull history is private. 'Settings' names the site's settings page.",
    },
    "history.share.private.confirm": {
        text: "Make public and copy link",
        description: "Button in the share dialog that turns the public pull history switch on, then copies the link.",
    },
    "history.share.cancel": {
        text: "Cancel",
        description: "Button that closes the share dialog without changing anything.",
    },
    "history.share.profilePrivate.title": {
        text: "Your profile is private",
        description: "Title of the dialog shown when the player presses Share while their whole profile is private, so no shared link can open.",
    },
    "history.share.profilePrivate.body": {
        text: "Shared pull history opens only on a public profile. Make your profile public in Settings, under Privacy, then share again.",
        description: "Body of the dialog shown when the player presses Share while their profile is private. 'Settings' and 'Privacy' name the site's settings page and its section.",
    },
    "history.share.profilePrivate.action": {
        text: "Open Settings",
        description: "Link in the share dialog to the settings page, where the profile can be made public.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
