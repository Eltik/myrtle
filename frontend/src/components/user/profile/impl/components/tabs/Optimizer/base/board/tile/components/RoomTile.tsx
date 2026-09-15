import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import type { ITile } from "#/lib/base/board";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { RoomPopover } from "../../RoomPopover";
import { tileAriaLabel, tileLabel } from "../logic/tile-labels";
import type { messages as labelMessages } from "../logic/tile-labels.messages";
import { tileStyle } from "../logic/tile-style";
import { LevelPips } from "./LevelPips";
import { RoomCrew } from "./RoomCrew";
import type { messages } from "./RoomTile.messages";
import styles from "./Tile.module.css";
import { TileHint } from "./TileHint";
import { TileTooltip } from "./TileTooltip";

export function RoomTile({ tile }: { tile: ITile }) {
    /** The tile's own name and accessible name come from `tile-labels.messages.ts`. */
    const t: TypedT<typeof messages & typeof labelMessages> = useT("user");
    const label = tileLabel(tile, t);
    const className = `${styles["riic-tile"]} ${styles[`riic-tile-${tile.kind}`]}${tile.facility ? "" : ` ${styles["riic-tile-empty"]}`}${tile.built ? "" : ` ${styles["riic-tile-unbuilt"]}`}`;

    const body = (
        <>
            <TileTooltip label={<TileHint tile={tile} />}>
                <span className={styles["riic-tile-room-clip"]}>
                    <span className={styles["riic-tile-room-body"]} />
                    <span className={styles["riic-tile-room-grid"]} />
                    <span className={styles["riic-tile-room-accent"]} />
                    <span className={styles["riic-tile-room-icon"]} />
                </span>
            </TileTooltip>
            <span className={styles["riic-tile-info"]}>
                <span className={styles["riic-tile-name-row"]}>
                    <span className={styles["riic-tile-name"]}>
                        {label}
                        {!tile.built && <span className={styles["riic-tile-unbuilt-label"]}>{t("profile.base.tile.notBuilt")}</span>}
                    </span>
                    {tile.built && <LevelPips max={tile.maxPhase} current={tile.level} />}
                </span>
            </span>
            <RoomCrew tile={tile} />
        </>
    );

    if (!tile.built || !tile.facility) {
        return (
            <div className={className} data-facility-type={tile.facility ?? undefined} data-slot-id={tile.slotId} style={tileStyle(tile)}>
                {body}
            </div>
        );
    }

    return (
        <Popover>
            <TileTooltip label={<TileHint tile={tile} />}>
                <PopoverTrigger
                    render={(props) => (
                        <button {...props} aria-label={tileAriaLabel(tile, t)} className={className} data-facility-type={tile.facility ?? undefined} data-slot-id={tile.slotId} style={tileStyle(tile)} type="button">
                            {body}
                        </button>
                    )}
                />
            </TileTooltip>
            <PopoverPopup align="center" side="bottom">
                <RoomPopover tile={tile} />
            </PopoverPopup>
        </Popover>
    );
}
