import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The side-card copy for the outfit list. `skins.ts` has no React, so
 * `buildOperatorSkinList` takes a `t` and resolves these while it builds the
 * rows.
 *
 * Real outfit names, group names and obtain descriptions come from the game
 * data; only the stand-ins this site supplies when that data is missing live
 * here.
 */
export const namespace = "operators";

export const messages = {
    "skins.default.name": {
        text: "Default",
        description: "Name of the entry for an operator's standard, unskinned art.",
    },
    "skins.evolvedArt.name": {
        text: "Evolved Art",
        description: "Name of the entry for the alternate illustration an operator gains at Elite 2.",
    },
    "skins.kicker.elite0": {
        text: "Elite 0",
        description: "Small caption over the default art, naming the promotion stage it is shown at. 'Elite' is the game's promotion tier.",
    },
    "skins.kicker.elite0or1": {
        text: "Elite 0 / Elite 1",
        description: "Small caption over the default art when it covers both of the first two promotion stages.",
    },
    "skins.kicker.elite2": {
        text: "Elite 2",
        description: "Small caption over art that is only shown at the second promotion.",
    },
    "skins.artist": {
        text: "Artist · {artist}",
        description: "Sub line crediting the illustrator. {artist} is a person's name and is never translated. Keep the middle dot separator.",
    },
    "skins.unlockedByDefault": {
        text: "Unlocked by default",
        description: "Sub line for art every player already has, shown where an outfit would say how it was obtained.",
    },
    "skins.elite2Promotion": {
        text: "Elite 2 Promotion",
        description: "Sub line for the Evolved Art entry: it is unlocked by promoting the operator to Elite 2.",
    },
    "skins.fallback.name": {
        text: "Outfit",
        description: "Stand-in name for an outfit the game data gives no name for.",
    },
    "skins.fallback.kicker": {
        text: "Skin",
        description: "Stand-in caption for an outfit whose collection the game data does not name. 'Skin' is the game's word for a cosmetic outfit.",
    },
    "skins.fallback.sub": {
        text: "Special Outfit",
        description: "Stand-in sub line for an outfit with no credited artist and no obtain description.",
    },
    "skins.defaultDescription": {
        text: "Standard operator outfit.",
        description: "Description shown for the default art, which the game data does not describe.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
