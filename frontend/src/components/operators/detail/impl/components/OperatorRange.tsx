import { useMemo } from "react";
import type { IRange } from "#/lib/api/ranges";
import { cn } from "#/lib/utils";

enum GridCell {
    empty = 0,
    active = 1,
    operator = 2,
}

interface IOperatorRangeProps {
    range: IRange;
    className?: string;
}

export function OperatorRange({ range, className }: IOperatorRangeProps) {
    const { rows, cols, grid } = useMemo(() => normalize(range), [range]);

    return (
        <div className={cn("max-w-full overflow-x-auto", className)}>
            <table className="border-collapse">
                <tbody>
                    {Array.from({ length: rows }).map((_, r) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: grid cells are positionally stable
                        <tr key={r}>
                            {Array.from({ length: cols }).map((_, c) => {
                                const cell = grid[r]?.[c];
                                // biome-ignore lint/suspicious/noArrayIndexKey: grid cells are positionally stable
                                return <td key={c} className={cn("h-5 w-5 border border-border/30 sm:h-6 sm:w-6", cell === GridCell.active && "border-2 border-primary/60 bg-primary/20", cell === GridCell.operator && "bg-primary")} />;
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

function normalize(range: IRange): { rows: number; cols: number; grid: GridCell[][] } {
    if (!range.grids || range.grids.length === 0) return { rows: 1, cols: 1, grid: [[GridCell.operator]] };

    let minRow = 0;
    let maxRow = 0;
    let minCol = 0;
    let maxCol = 0;
    for (const g of range.grids) {
        minRow = Math.min(minRow, g.row);
        maxRow = Math.max(maxRow, g.row);
        minCol = Math.min(minCol, g.col);
        maxCol = Math.max(maxCol, g.col);
    }

    const rows = maxRow - minRow + 1;
    const cols = maxCol - minCol + 1;
    const grid: GridCell[][] = Array.from({ length: rows }, () => Array.from({ length: cols }, () => GridCell.empty));

    for (const g of range.grids) {
        const r = g.row - minRow;
        const c = g.col - minCol;
        if (grid[r]) grid[r][c] = GridCell.active;
    }
    const opR = -minRow;
    const opC = -minCol;
    if (grid[opR]) grid[opR][opC] = GridCell.operator;

    return { rows, cols, grid };
}
