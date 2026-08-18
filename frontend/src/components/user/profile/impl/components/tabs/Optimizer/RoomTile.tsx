import type { ITile } from "#/lib/base/layout";
import styles from "./Board.module.css";
import { LevelPips } from "./LevelPips";
import { tileStyle } from "./tile-style";

export function RoomTile({ tile }: { tile: ITile }) {
    const label = tile.facility ? tile.name : "Empty";

    return (
        <button
            type="button"
            className={`${styles["riic-tile"]} ${styles[`riic-tile-${tile.kind}`]}${tile.facility ? "" : ` ${styles["riic-tile-empty"]}`}${tile.built ? "" : ` ${styles["riic-tile-unbuilt"]}`}`}
            data-slot-id={tile.slotId}
            data-facility-type={tile.facility ?? undefined}
            title={tile.built ? label : `${label} (Not Built)`}
            style={tileStyle(tile)}
        >
            <span className={styles["riic-tile-room-clip"]}>
                <span className={styles["riic-tile-room-body"]} />
                <span className={styles["riic-tile-room-grid"]} />
                <span className={styles["riic-tile-room-accent"]} />
                <span className={styles["riic-tile-room-icon"]} />
            </span>
            <span className={styles["riic-tile-info"]}>
                <span className={styles["riic-tile-name-row"]}>
                    <span className={styles["riic-tile-name"]}>
                        {label}
                        {!tile.built && <span className={styles["riic-tile-unbuilt-label"]}>Not Built</span>}
                    </span>
                    {tile.built && <LevelPips max={tile.maxPhase} current={tile.level} />}
                </span>
            </span>
        </button>
    );
}
