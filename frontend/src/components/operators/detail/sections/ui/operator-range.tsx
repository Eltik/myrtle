"use client";

import { useMemo } from "react";
import type { Range } from "~/types/api/impl/range";
import { cn } from "~/lib/utils";

interface OperatorRangeProps {
    range: Range;
    className?: string;
}

enum GridCell {
    empty = 0,
    active = 1,
    Operator = 2,
}

export function OperatorRange({ range, className }: OperatorRangeProps) {
    const { rows, cols, grid } = useMemo(() => normalizeRange(range), [range]);

    return (
        <div className={cn("inline-block", className)}>
            <table className="border-collapse">
                <tbody>
                    {Array.from({ length: rows }).map((_, rowIndex) => (
                        <tr key={rowIndex}>
                            {Array.from({ length: cols }).map((_, colIndex) => {
                                const gridType = grid[rowIndex]?.[colIndex];
                                return <td className={cn("h-6 w-6 border border-border/30", gridType === GridCell.active && "border-2 border-primary/60 bg-primary/20", gridType === GridCell.Operator && "bg-primary", gridType === GridCell.empty && "bg-transparent")} key={colIndex} />;
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

function normalizeRange(range: Range): { rows: number; cols: number; grid: GridCell[][] } {
    if (!range.grids || range.grids.length === 0) {
        return { rows: 1, cols: 1, grid: [[GridCell.Operator]] };
    }

    // Find min and max row/col
    let minRow = 0;
    let maxRow = 0;
    let minCol = 0;
    let maxCol = 0;

    range.grids.forEach((g) => {
        minRow = Math.min(minRow, g.row);
        maxRow = Math.max(maxRow, g.row);
        minCol = Math.min(minCol, g.col);
        maxCol = Math.max(maxCol, g.col);
    });

    // Normalize to include operator position (0,0)
    minRow = Math.min(minRow, 0);
    maxRow = Math.max(maxRow, 0);
    minCol = Math.min(minCol, 0);
    maxCol = Math.max(maxCol, 0);

    const rows = maxRow - minRow + 1;
    const cols = maxCol - minCol + 1;

    // Initialize grid
    const grid: GridCell[][] = Array.from({ length: rows }, () => Array.from({ length: cols }, () => GridCell.empty));

    // Place active cells
    range.grids.forEach((g) => {
        const row = g.row - minRow;
        const col = g.col - minCol;
        if (grid[row]) {
            grid[row][col] = GridCell.active;
        }
    });

    // Place operator at (0,0) normalized
    const operatorRow = 0 - minRow;
    const operatorCol = 0 - minCol;
    if (grid[operatorRow]) {
        grid[operatorRow][operatorCol] = GridCell.Operator;
    }

    return { rows, cols, grid };
}
