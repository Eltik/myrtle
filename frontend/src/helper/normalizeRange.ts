import type { Range } from "~/types/impl/api/static/ranges";
import { GridCell, type NormalizedRange } from "~/types/impl/frontend/impl/operators";

/**
 * @author Credit to
 * https://github.com/iansjk/sanity-gone/blob/main/src/components/CharacterRange/CharacterRange.tsx
 */
export const normalizeRange = (rangeObject: Range): NormalizedRange => {
    const rangeGrids = [...rangeObject.grids, { row: 0, col: 0 }];
    // for each of rows and cols,
    // find the minimum value and the maximum value
    // then return max-min to get number of rows/cols
    const rowIndices = rangeGrids.map((cell) => cell.row);
    const colIndices = rangeGrids.map((cell) => cell.col);
    const minRowIndex = Math.min(...rowIndices);
    const maxRowIndex = Math.max(...rowIndices);
    const minColIndex = Math.min(...colIndices);
    const maxColIndex = Math.max(...colIndices);

    // create a 2d-array of size [rows, cols]
    const rows = maxRowIndex - minRowIndex + 1;
    const cols = maxColIndex - minColIndex + 1;
    const grid = Array<GridCell>(rows)
        .fill(GridCell.empty)
        .map(() => Array<GridCell>(cols).fill(GridCell.empty));
    rangeGrids.forEach((cell) => {
        const type = cell.row === 0 && cell.col === 0 ? GridCell.Operator : GridCell.active;
        Object.assign(grid[cell.row - minRowIndex] ?? {}, {
            [cell.col - minColIndex]: type,
        });
    });

    return {
        rows,
        cols,
        grid,
    };
};
