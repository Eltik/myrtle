import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.hero.share": {
        text: "Share",
        description: "Button that copies a link to this profile. Hidden on the narrowest screens, so keep it short.",
    },
    "profile.hero.copyUid": {
        text: "Copy UID",
        description: "Copies the account number: accessible name and tooltip of the small copy button. 'UID' is the in-game account number.",
    },
    "profile.hero.copyUsername": {
        text: "Copy username",
        description: "Copies the player's nickname: accessible name and tooltip on the nickname itself.",
    },
    "profile.hero.copied.desc": {
        text: "{value} copied to clipboard.",
        description: "Toast body after a copy succeeded. {value} is the text that was copied.",
    },
    "profile.hero.copyFailed.desc": {
        text: "Clipboard access was denied.",
        description: "Toast body when the browser refused clipboard access.",
    },
    "profile.hero.share.copied.title": {
        text: "Link copied",
        description: "Toast title after the profile link is copied.",
    },
    "profile.hero.share.copied.desc": {
        text: "Profile link copied to clipboard.",
        description: "Toast body after the profile link is copied.",
    },
    "profile.hero.share.failed.title": {
        text: "Couldn't copy link",
        description: "Toast title when copying the profile link failed. Keep the apostrophe.",
    },
    "profile.hero.uid.copied.title": {
        text: "UID copied",
        description: "Toast title after the account number is copied. 'UID' is the in-game account number.",
    },
    "profile.hero.uid.failed.title": {
        text: "Couldn't copy UID",
        description: "Toast title when copying the account number failed. Keep the apostrophe.",
    },
    "profile.hero.username.copied.title": {
        text: "Username copied",
        description: "Toast title after the nickname is copied.",
    },
    "profile.hero.username.failed.title": {
        text: "Couldn't copy username",
        description: "Toast title when copying the nickname failed. Keep the apostrophe.",
    },
    "profile.hero.serverTag": {
        text: "{server} server",
        description: "Pill naming the game server this account is on. {server} is the game's own region code (EN, JP, CN…) and is not translated.",
    },
    "profile.hero.registered": {
        text: "Registered ·",
        description: "Label before the account's creation date, which follows in bold after a space. Keep the separator dot.",
    },
    "profile.hero.signedIn": {
        text: "Signed in ·",
        description: "Label before the sign-in tally, which follows in bold as 'signed/total' and then the word for days. Keep the separator dot.",
    },
    "profile.hero.days": {
        text: "days",
        description: "Unit after the 'signed-in / total' pair, e.g. '1,024/1,200 days'. Always plural: it follows a ratio, not a single count.",
    },
    "profile.hero.level.lv": {
        text: "Lv",
        description: "Abbreviation before the account level over the experience bar. Very little room.",
    },
    "profile.hero.level.max": {
        text: "Max · {max}",
        description: "Shown at the right of the experience bar once the account is at the level cap. {max} is that cap.",
    },
    "profile.hero.level.exp": {
        text: "{current} / {required} EXP",
        description: "Progress toward the next account level, at the right of the experience bar. 'EXP' is the game's own abbreviation for experience.",
    },
    "profile.hero.level.ariaMax": {
        text: "Level {level} (max)",
        description: "Accessible name of the experience bar once the account is at the level cap.",
    },
    "profile.hero.level.aria": {
        text: "Level {level}, {current} of {required} EXP to level {next}",
        description: "Accessible name of the experience bar. {next} is the level being worked toward.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
