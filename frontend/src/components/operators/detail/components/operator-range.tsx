"use client";

import { useMemo } from "react";
import type { Range } from "~/types/api";
import { cn } from "~/lib/utils";

interface OperatorRangeProps {
    range: Range;
}

enum GridCell {
    Operator = "operator",
    Empty = "empty",
    Active = "active",
}

function normalizeRange(range: Range): {
    rows: number;
    cols: number;
    grid: GridCell[][];
} {
    if (!range.grids || range.grids.length === 0) {
        return { rows: 1, cols: 1, grid: [[GridCell.Operator]] };
    }

    // Find bounds including operator position at (0, 0)
    let minRow = 0;
    let maxRow = 0;
    let minCol = 0;
    let maxCol = 0;

    for (const grid of range.grids) {
        minRow = Math.min(minRow, grid.row);
        maxRow = Math.max(maxRow, grid.row);
        minCol = Math.min(minCol, grid.col);
        maxCol = Math.max(maxCol, grid.col);
    }

    const rows = maxRow - minRow + 1;
    const cols = maxCol - minCol + 1;

    // Create grid filled with empty cells
    const grid: GridCell[][] = Array.from({ length: rows }, () => Array.from({ length: cols }, () => GridCell.Empty));

    // Mark operator position
    const operatorRow = 0 - minRow;
    const operatorCol = 0 - minCol;
    if (grid[operatorRow]) {
        grid[operatorRow][operatorCol] = GridCell.Operator;
    }

    // Mark active cells
    for (const cell of range.grids) {
        const row = cell.row - minRow;
        const col = cell.col - minCol;
        if (grid[row] && grid[row][col] !== GridCell.Operator) {
            grid[row][col] = GridCell.Active;
        }
    }

    return { rows, cols, grid };
}

export function OperatorRange({ range }: OperatorRangeProps) {
    const { rows, cols, grid } = useMemo(() => normalizeRange(range), [range]);

    return (
        <div className="inline-block">
            <table className="border-collapse">
                <tbody>
                    {Array.from({ length: rows }).map((_, rowIndex) => (
                        <tr key={rowIndex}>
                            {Array.from({ length: cols }).map((_, colIndex) => {
                                const cellType = grid[rowIndex]?.[colIndex];
                                return <td key={colIndex} className={cn("h-6 w-6 box-border", cellType === GridCell.Active && "border-2 border-border", cellType === GridCell.Operator && "bg-primary rounded-sm")} />;
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
