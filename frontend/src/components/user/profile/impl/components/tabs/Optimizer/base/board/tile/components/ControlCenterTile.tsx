import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import type { ITile } from "#/lib/base/board";
import { RoomPopover } from "../../RoomPopover";
import { tileAriaLabel } from "../logic/tile-labels";
import { tileStyle } from "../logic/tile-style";
import { RoomCrew } from "./RoomCrew";
import styles from "./Tile.module.css";
import { TileHint } from "./TileHint";
import { TileTooltip } from "./TileTooltip";

export function ControlCenterTile({ tile }: { tile: ITile }) {
    return (
        <Popover>
            <PopoverTrigger
                render={(props) => (
                    <button {...props} type="button" className={`${styles["riic-tile"]} ${styles["riic-tile-fixed"]}`} data-slot-id={tile.slotId} data-facility-type={tile.facility ?? undefined} aria-label={tileAriaLabel(tile)} style={tileStyle(tile)}>
                        <TileTooltip label={<TileHint tile={tile} />}>
                            <span className={styles["riic-tile-room-clip"]}>
                                <span className={styles["riic-tile-room-body"]} />
                                <span className={styles["riic-tile-room-grid"]} />
                            </span>
                        </TileTooltip>
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
                        <RoomCrew tile={tile} />
                    </button>
                )}
            />
            <PopoverPopup align="center" side="bottom">
                <RoomPopover tile={tile} />
            </PopoverPopup>
        </Popover>
    );
}
