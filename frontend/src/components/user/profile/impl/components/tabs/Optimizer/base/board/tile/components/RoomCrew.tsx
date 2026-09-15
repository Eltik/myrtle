import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { type ITile, vacanciesOf } from "#/lib/base/board";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { crewLabel } from "../logic/tile-labels";
import type { messages as labelMessages } from "../logic/tile-labels.messages";
import styles from "./Tile.module.css";
import { TileTooltip } from "./TileTooltip";

export function RoomCrew({ tile }: { tile: ITile }) {
    /** The crew tooltips are declared in `tile-labels.messages.ts`. */
    const t: TypedT<typeof labelMessages> = useT("user");
    if (tile.seats === 0) return null;

    const vacancies = vacanciesOf(tile);

    return (
        <span className={styles["riic-tile-ops"]}>
            {tile.operators.map((op) => (
                <TileTooltip key={op.id} label={crewLabel(op, t)}>
                    <span className={styles["riic-tile-op-chip"]} data-change={op.change}>
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                </TileTooltip>
            ))}
            {vacancies > 0 && <span className={styles["riic-tile-op-seats"]} style={{ "--seats": vacancies } as React.CSSProperties} />}
        </span>
    );
}
