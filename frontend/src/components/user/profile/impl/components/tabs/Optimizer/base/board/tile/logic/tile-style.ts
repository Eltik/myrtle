import type { CSSProperties } from "react";
import { riicRoomBackground, riicRoomIcon } from "#/components/operators/detail/impl/assets";
import type { ITile } from "#/lib/base/board";

type ICssVars = CSSProperties & Record<`--${string}`, string>;

export function placement(tile: ITile): CSSProperties {
    return {
        gridColumn: `${tile.col} / span ${tile.w}`,
        gridRow: `${tile.row} / span ${tile.h}`,
    };
}

const cssURL = (href: string): string => (href ? `url("${href}")` : "none");

export function tileStyle(tile: ITile): ICssVars {
    return {
        ...placement(tile),
        "--tile-icon": cssURL(tile.facility ? riicRoomIcon(tile.facility) : ""),
        "--tile-plate": cssURL(tile.kind === "path" && tile.facility ? riicRoomBackground(tile.facility) : ""),
    };
}
