import type { IBoard } from "#/lib/base/board";
import styles from "./Board.module.css";
import { RiicTile } from "./tile/components/RiicTile";

export function Board({ board }: { board: IBoard }) {
    return (
        <div className={styles["riic-board-fit"]}>
            <div
                className={styles["riic-board"]}
                style={{
                    gridTemplateColumns: board.templateColumns,
                    gridTemplateRows: board.templateRows,
                }}
            >
                {board.tiles.map((tile) => (
                    <RiicTile key={tile.slotId} tile={tile} />
                ))}
            </div>
        </div>
    );
}
