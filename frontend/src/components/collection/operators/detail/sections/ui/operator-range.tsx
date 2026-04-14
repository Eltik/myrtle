"use client";

import { useMemo } from "react";
import { cn } from "~/lib/utils";
import type { Range } from "~/types/api/impl/range";

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
        <div className={cn("max-w-full overflow-x-auto", className)}>
            <table className="border-collapse">
                <tbody>
                    {Array.from({ length: rows }).map((_, rowIndex) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: Static grid rows
                        <tr key={rowIndex}>
                            {Array.from({ length: cols }).map((_, colIndex) => {
                                const gridType = grid[rowIndex]?.[colIndex];
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static grid cells
                                return <td className={cn("h-5 w-5 border border-border/30 sm:h-6 sm:w-6", gridType === GridCell.active && "border-2 border-primary/60 bg-primary/20", gridType === GridCell.Operator && "bg-primary", gridType === GridCell.empty && "bg-transparent")} key={colIndex} />;
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

    minRow = Math.min(minRow, 0);
    maxRow = Math.max(maxRow, 0);
    minCol = Math.min(minCol, 0);
    maxCol = Math.max(maxCol, 0);

    const rows = maxRow - minRow + 1;
    const cols = maxCol - minCol + 1;

    const grid: GridCell[][] = Array.from({ length: rows }, () => Array.from({ length: cols }, () => GridCell.empty));

    range.grids.forEach((g) => {
        const row = g.row - minRow;
        const col = g.col - minCol;
        if (grid[row]) {
            grid[row][col] = GridCell.active;
        }
    });

    const operatorRow = 0 - minRow;
    const operatorCol = 0 - minCol;
    if (grid[operatorRow]) {
        grid[operatorRow][operatorCol] = GridCell.Operator;
    }

    return { rows, cols, grid };
}
