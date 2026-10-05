import { describe, expect, it } from "vitest";
import { COMPACT_BOARD_PAD_PX, COMPACT_CELL_MAX, COMPACT_CELL_MIN, COMPACT_GAP_PX, COMPACT_GRID_HEIGHT, COMPACT_STRIP_PX, compactGridSize } from "./compact";

/** The cells' total height at a size: each row is art plus strip, with gaps between rows. */
function gridHeight(rows: number, cellPx: number): number {
    return rows * (cellPx + COMPACT_STRIP_PX) + (rows - 1) * COMPACT_GAP_PX;
}

describe("compactGridSize fits a grid to the showcase height budget", () => {
    it("draws a 6 x 6 at 68 px cells, inside the budget (the full board drew 180 px)", () => {
        const size = compactGridSize(6, 6);
        expect(size.cellPx).toBe(68);
        expect(gridHeight(6, size.cellPx)).toBeLessThanOrEqual(COMPACT_GRID_HEIGHT);
        expect(gridHeight(6, size.cellPx + 1)).toBeGreaterThan(COMPACT_GRID_HEIGHT);
        expect(size.boardMaxPx).toBe(6 * 68 + 5 * COMPACT_GAP_PX + 2 * COMPACT_BOARD_PAD_PX);
    });

    it("caps a short grid's cells at the max", () => {
        expect(compactGridSize(1, 3).cellPx).toBe(COMPACT_CELL_MAX);
        expect(compactGridSize(3, 3).cellPx).toBe(COMPACT_CELL_MAX);
        expect(compactGridSize(4, 4).cellPx).toBe(COMPACT_CELL_MAX);
    });

    it("floors a tall grid's cells at the min and lets it grow past the budget", () => {
        const size = compactGridSize(10, 10);
        expect(size.cellPx).toBe(COMPACT_CELL_MIN);
        expect(gridHeight(10, size.cellPx)).toBeGreaterThan(COMPACT_GRID_HEIGHT);
    });

    it("never grows a cell as rows are added", () => {
        let last = Number.POSITIVE_INFINITY;
        for (let rows = 1; rows <= 10; rows++) {
            const { cellPx } = compactGridSize(rows, 6);
            expect(cellPx).toBeLessThanOrEqual(last);
            last = cellPx;
        }
    });

    it("sizes cells by rows only; columns set the width", () => {
        expect(compactGridSize(5, 2).cellPx).toBe(compactGridSize(5, 9).cellPx);
        expect(compactGridSize(5, 9).boardMaxPx).toBeGreaterThan(compactGridSize(5, 2).boardMaxPx);
    });

    it("treats a nonsense size as one row and one column", () => {
        expect(compactGridSize(0, 0)).toEqual(compactGridSize(1, 1));
    });
});
