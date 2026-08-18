import type { ITile, ITileOperator } from "#/lib/base/board";

export function tileLabel(tile: ITile): string {
    return tile.facility ? tile.name : "Empty";
}

export function tileAriaLabel(tile: ITile): string {
    return `${tileLabel(tile)}, level ${tile.level}, ${tile.operators.length} of ${tile.seats} staffed`;
}

const CHANGE_LABEL = {
    added: "joins this shift",
    removed: "leaves this shift",
} as const;

export function crewLabel(operator: ITileOperator): string {
    return operator.change ? `${operator.name} - ${CHANGE_LABEL[operator.change]}` : operator.name;
}
