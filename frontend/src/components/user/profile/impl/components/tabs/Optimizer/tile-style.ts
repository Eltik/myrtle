import type { CSSProperties } from "react";
import { riicRoomBackground, riicRoomIcon } from "#/components/operators/detail/impl/assets";
import type { ITile } from "#/lib/base/layout";

export function placement(tile: ITile): CSSProperties {
    return {
        gridColumn: `${tile.col} / span ${tile.w}`,
        gridRow: `${tile.row} / span ${tile.h}`,
    };
}

const cssUrl = (href: string): string => (href ? `url("${href}")` : "none");

export function tileStyle(tile: ITile): CSSProperties {
    return {
        ...placement(tile),
        "--tile-icon": cssUrl(tile.facility ? riicRoomIcon(tile.facility) : ""),
        "--tile-plate": cssUrl(tile.kind === "path" && tile.facility ? riicRoomBackground(tile.facility) : ""),
    } as CSSProperties;
}
