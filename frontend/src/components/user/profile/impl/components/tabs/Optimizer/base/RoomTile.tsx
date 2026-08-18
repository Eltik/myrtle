import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import type { ITile } from "#/lib/base/layout";
import styles from "./Board.module.css";
import { LevelPips } from "./LevelPips";
import { RoomCrew } from "./RoomCrew";
import { RoomPopover } from "./RoomPopover";
import { tileStyle } from "./tile-style";

export function RoomTile({ tile }: { tile: ITile }) {
    const label = tile.facility ? tile.name : "Empty";
    const className = `${styles["riic-tile"]} ${styles[`riic-tile-${tile.kind}`]}${tile.facility ? "" : ` ${styles["riic-tile-empty"]}`}${tile.built ? "" : ` ${styles["riic-tile-unbuilt"]}`}`;

    const body = (
        <>
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
            <RoomCrew tile={tile} />
        </>
    );

    if (!tile.built || !tile.facility) {
        return (
            <div className={className} data-slot-id={tile.slotId} data-facility-type={tile.facility ?? undefined} title={label} style={tileStyle(tile)}>
                {body}
            </div>
        );
    }

    return (
        <Popover>
            <PopoverTrigger
                render={(props) => (
                    <button {...props} type="button" className={className} data-slot-id={tile.slotId} data-facility-type={tile.facility ?? undefined} title={tile.built ? label : `${label} (Not Built)`} style={tileStyle(tile)}>
                        {body}
                    </button>
                )}
            />
            <PopoverPopup align="center" side="bottom">
                <RoomPopover tile={tile} />
            </PopoverPopup>
        </Popover>
    );
}
