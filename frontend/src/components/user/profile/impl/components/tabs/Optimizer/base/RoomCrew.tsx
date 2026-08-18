import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITile } from "#/lib/base/layout";
import styles from "./Board.module.css";

export function RoomCrew({ tile }: { tile: ITile }) {
    if (tile.seats === 0) return null;

    const vacancies = Math.max(0, tile.seats - tile.operators.length);

    return (
        <span className={styles["riic-tile-ops"]}>
            {tile.operators.map((op) => (
                <span className={styles["riic-tile-op-chip"]} data-change={op.change} key={op.id} title={op.change === "added" ? `${op.name} - joins this shift` : op.change === "removed" ? `${op.name} - leaves this shift` : op.name}>
                    <OperatorAvatar charId={op.id} name={op.name} />
                </span>
            ))}
            {Array.from({ length: vacancies }, (_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: vacancies are positional, there is no other identity
                <span key={i} className={styles["riic-tile-op-seat"]} />
            ))}
        </span>
    );
}
