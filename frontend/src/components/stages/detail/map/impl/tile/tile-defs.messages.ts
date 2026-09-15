import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The glyphs painted onto a map tile. `tile-defs.ts` has no React, so the
 * table carries message KEYS and `Tile.tsx` resolves them with `t()`.
 *
 * Every one of these sits inside a 48px tile, so they are abbreviations by
 * necessity - a translation has the same room and no more. The tile's longer
 * name is not a message: the board shows only the glyph.
 */
export const namespace = "stages";

export const messages = {
    "tile.flyStart": {
        text: "air",
        description: "Glyph on a flying-enemy spawn tile, short for 'airborne'. Lowercase, and at most four characters wide.",
    },
    "tile.teleportIn": {
        text: "in",
        description: "Glyph on a teleporter entrance tile. Lowercase, and at most four characters wide.",
    },
    "tile.teleportOut": {
        text: "out",
        description: "Glyph on a teleporter exit tile. Lowercase, and at most four characters wide.",
    },
    "tile.hole": {
        text: "hole",
        description: "Glyph on a pit tile that ground units fall into. Lowercase, and at most four characters wide.",
    },
    "tile.bigForce": {
        text: "F+",
        description: "Glyph on a tile that boosts an operator's force / knockback strength. The '+' means 'increased'; at most four characters wide.",
    },
    "tile.defUp": {
        text: "def+",
        description: "Glyph on a tile that raises the defense of whatever stands on it. The '+' means 'increased'; at most four characters wide.",
    },
    "tile.gazebo": {
        text: "air+",
        description: "Glyph on a gazebo tile, which boosts airborne units. The '+' means 'increased'; at most four characters wide.",
    },
    "tile.woodenWall": {
        text: "WW",
        description: "Glyph on a wooden-wall tile, initials of 'wooden wall'. At most four characters wide.",
    },
    "tile.stairs": {
        text: "↕",
        description: "Glyph on a stairs tile: an up/down arrow. Already a symbol, so it normally needs no translation.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on the TILE_TYPES table and resolved by
// `Tile.tsx` as `t(type.contentKey)`, so the extractor has no literal call site.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
