import type { ITile } from "#/lib/base/layout";
import styles from "./Board.module.css";
import { ControlCenterTile } from "./ControlCenterTile";
import { RoomTile } from "./RoomTile";
import { tileStyle } from "./tile-style";

export function RiicTile({ tile }: { tile: ITile }) {
    if (tile.kind === "elevator" || tile.kind === "path") {
        return <div className={`${styles["riic-tile"]} ${styles[`riic-tile-${tile.kind}`]}`} data-facility-type={tile.facility ?? undefined} data-slot-id={tile.slotId} title={tile.name || undefined} style={tileStyle(tile)} />;
    }

    if (tile.facility === "CONTROL") return <ControlCenterTile tile={tile} />;

    return <RoomTile tile={tile} />;
}
