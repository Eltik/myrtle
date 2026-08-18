import type { ITile } from "#/lib/base/layout";
import styles from "./Board.module.css";
import { tileStyle } from "./tile-style";

export function ControlCenterTile({ tile }: { tile: ITile }) {
    return (
        <button type="button" className={`${styles["riic-tile"]} ${styles["riic-tile-fixed"]}`} data-slot-id={tile.slotId} data-facility-type={tile.facility ?? undefined} title={tile.name} style={tileStyle(tile)}>
            <span className={styles["riic-tile-room-clip"]}>
                <span className={styles["riic-tile-room-body"]} />
                <span className={styles["riic-tile-room-grid"]} />
            </span>
            <span className={styles["riic-cc-badge"]}>
                <span className={styles["riic-cc-bracket"]}>&#10094;</span>
                <span className={styles["riic-cc-badge-inner"]}>
                    <span className={styles["riic-cc-title"]}>{tile.name}</span>
                    <span className={styles["riic-cc-subtitle"]}>Level of Control_Interface</span>
                    <span className={styles["riic-cc-ver-row"]}>
                        <span className={styles["riic-cc-ver-icon-col"]}>
                            <span className={styles["riic-cc-stripe-icon"]} />
                            <span className={styles["riic-cc-ver-label"]}>Ver</span>
                        </span>
                        <span className={styles["riic-cc-ver-num"]}>
                            {tile.level}
                            <span className={styles["riic-cc-ver-dec"]}>.0</span>
                        </span>
                    </span>
                </span>
                <span className={styles["riic-cc-bracket"]}>&#10095;</span>
            </span>
        </button>
    );
}
