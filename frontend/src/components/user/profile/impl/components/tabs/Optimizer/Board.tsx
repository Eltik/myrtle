import type { IBoard } from "#/lib/base/layout";
import styles from "./Board.module.css";
import { RiicTile } from "./RiicTile";

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
