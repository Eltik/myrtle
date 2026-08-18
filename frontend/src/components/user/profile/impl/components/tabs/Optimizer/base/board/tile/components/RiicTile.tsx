import { memo } from "react";
import type { ITile } from "#/lib/base/board";
import { tileStyle } from "../logic/tile-style";
import { ControlCenterTile } from "./ControlCenterTile";
import { RoomTile } from "./RoomTile";
import styles from "./Tile.module.css";

function RiicTileImpl({ tile }: { tile: ITile }) {
    if (tile.kind === "elevator" || tile.kind === "path") {
        return <div className={`${styles["riic-tile"]} ${styles[`riic-tile-${tile.kind}`]}`} data-facility-type={tile.facility ?? undefined} data-slot-id={tile.slotId} style={tileStyle(tile)} />;
    }

    if (tile.facility === "CONTROL") return <ControlCenterTile tile={tile} />;

    return <RoomTile tile={tile} />;
}

export const RiicTile = memo(RiicTileImpl);
