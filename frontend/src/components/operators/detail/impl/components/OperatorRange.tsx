import { useMemo } from "react";
import type { IRange } from "#/lib/api/ranges";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./OperatorRange.messages";

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
    const t: TypedT<typeof messages> = useT("operators");
    const { rows, cols, grid } = useMemo(() => normalize(range), [range]);
    const reach = useMemo(() => describe(range), [range]);

    return (
        <div className={cn("max-w-full overflow-x-auto", className)}>
            {/* A grid of empty cells has nothing to read, so the table is one image with its reach as
                the name. The edge colour is darker in light mode: the game's #e8973c measured 2.35 on
                the white card, oklch(0.6 0.14 65) measures 4.09 on the card, 3.92 on the page and 3.44
                on muted. Dark keeps the original, 8.13 on the card. */}
            <table aria-label={t("range.aria", reach)} className="border-separate border-spacing-1 [--range-edge:oklch(0.6_0.14_65)] dark:[--range-edge:#e8973c]" role="img">
                <tbody>
                    {Array.from({ length: rows }).map((_, r) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: grid cells are positionally stable
                        <tr key={r}>
                            {Array.from({ length: cols }).map((_, c) => {
                                const cell = grid[r]?.[c];
                                return (
                                    <td
                                        // biome-ignore lint/suspicious/noArrayIndexKey: grid cells are positionally stable
                                        key={c}
                                        className={cn(
                                            "h-5 w-5 sm:h-6 sm:w-6",
                                            cell === GridCell.empty && "border border-(--range-edge)/25 border-dashed",
                                            cell === GridCell.active && "border border-(--range-edge) bg-[repeating-linear-gradient(135deg,var(--range-edge)_0_3px,transparent_3px_7px)]",
                                            cell === GridCell.operator && "border border-foreground bg-foreground",
                                        )}
                                    />
                                );
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

/** Tiles hit and how far they reach from the operator, who faces +col. */
function describe(range: IRange): { tiles: number; ahead: number; behind: number; side: number } {
    const grids = range.grids ?? [];
    let ahead = 0;
    let behind = 0;
    let side = 0;
    for (const g of grids) {
        ahead = Math.max(ahead, g.col);
        behind = Math.max(behind, -g.col);
        side = Math.max(side, Math.abs(g.row));
    }
    return { tiles: grids.length, ahead, behind, side };
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
