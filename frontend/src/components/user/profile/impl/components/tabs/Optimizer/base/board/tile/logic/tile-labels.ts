import type { ITile, ITileOperator } from "#/lib/base/board";
import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages as labelMessages } from "./tile-labels.messages";

/** The `t` these labels need, narrowed to the keys they can render. */
export type TileT = TypedT<typeof labelMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the board renders and
 * this file carries no second copy of the text.
 */
const sourceT: TileT = (key, values) => formatMessage(sourceMessage(fullMessageKey("user", key)) ?? key, DEFAULT_LOCALE, values);

export function tileLabel(tile: ITile, t: TileT = sourceT): string {
    return tile.facility ? tile.name : t("profile.base.tile.empty");
}

export function tileAriaLabel(tile: ITile, t: TileT = sourceT): string {
    return t("profile.base.tile.aria", { name: tileLabel(tile, t), level: tile.level, staffed: tile.operators.length, seats: tile.seats });
}

const CHANGE_KEY = {
    added: "profile.base.tile.crew.added",
    removed: "profile.base.tile.crew.removed",
} as const satisfies Record<string, keyof typeof labelMessages>;

export function crewLabel(operator: ITileOperator, t: TileT = sourceT): string {
    return operator.change ? t(CHANGE_KEY[operator.change], { name: operator.name }) : operator.name;
}
