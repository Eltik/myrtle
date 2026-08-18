import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { type ITile, vacanciesOf } from "#/lib/base/board";
import { crewLabel } from "../logic/tile-labels";
import styles from "./Tile.module.css";
import { TileTooltip } from "./TileTooltip";

export function RoomCrew({ tile }: { tile: ITile }) {
    if (tile.seats === 0) return null;

    const vacancies = vacanciesOf(tile);

    return (
        <span className={styles["riic-tile-ops"]}>
            {tile.operators.map((op) => (
                <TileTooltip key={op.id} label={crewLabel(op)}>
                    <span className={styles["riic-tile-op-chip"]} data-change={op.change}>
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                </TileTooltip>
            ))}
            {vacancies > 0 && <span className={styles["riic-tile-op-seats"]} style={{ "--seats": vacancies } as React.CSSProperties} />}
        </span>
    );
}
