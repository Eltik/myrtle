"use client";

import { useMemo } from "react";
import { motion } from "motion/react";
import type { Range } from "~/types/api";
import { normalizeRange, GridCell } from "~/lib/operator-helpers";

interface OperatorRangeProps {
    range: Range;
}

export function OperatorRange({ range }: OperatorRangeProps) {
    const { rows, cols, grid } = useMemo(() => normalizeRange(range), [range]);

    return (
        <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="inline-block rounded-lg bg-muted/30 p-3">
            <table className="border-separate border-spacing-0.5">
                <tbody>
                    {[...Array(rows).keys()].map((rowIndex) => (
                        <tr key={rowIndex}>
                            {[...Array(cols).keys()].map((colIndex) => {
                                const gridType = grid[rowIndex]?.[colIndex];
                                return <td key={colIndex} className={`h-6 w-6 rounded-sm transition-colors ${gridType === GridCell.Operator ? "bg-primary" : gridType === GridCell.active ? "border-2 border-border bg-card" : "bg-transparent"}`} />;
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </motion.div>
    );
}
