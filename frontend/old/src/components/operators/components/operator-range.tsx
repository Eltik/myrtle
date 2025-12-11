"use client";

import React, { useMemo } from "react";
import { normalizeRange } from "~/helper/normalizeRange";
import type { Range } from "~/types/impl/api/static/ranges";
import { GridCell } from "~/types/impl/frontend/impl/operators";

const OperatorRange: React.FC<{ range: Range } & React.HTMLAttributes<HTMLTableElement>> = (props) => {
    const { range, ...rest } = props;

    const { rows, cols, grid } = useMemo(() => normalizeRange(range), [range]);

    const width = "w-6";
    const height = "h-6";

    return (
        <table className="flex-shrink-0" {...rest}>
            <thead>
                <tr>
                    <th></th>
                    {[...Array(cols).keys()].map((i) => (
                        <th
                            className="absolute h-[1px] w-[1px] overflow-hidden whitespace-nowrap"
                            key={i}
                            scope="col"
                            style={{
                                clip: "rect(0 0 0 0)",
                                clipPath: "inset(50%)",
                            }}
                        >{`Y${i + 1}`}</th>
                    ))}
                </tr>
            </thead>
            <tbody>
                {[...Array(rows).keys()].map((rowIndex) => (
                    <tr key={rowIndex}>
                        <th
                            className="absolute h-[1px] w-[1px] overflow-hidden whitespace-nowrap"
                            scope="row"
                            style={{
                                clip: "rect(0 0 0 0)",
                                clipPath: "inset(50%)",
                            }}
                        >{`X${rowIndex + 1}`}</th>
                        {[...Array(cols).keys()].map((colIndex) => {
                            const gridType = grid[rowIndex]?.[colIndex];
                            const className = gridType === GridCell.active ? "border-2 border-border border-solid" : gridType === GridCell.Operator ? "bg-primary" : gridType === GridCell.empty ? "" : "";
                            return (
                                <td className={`${width} ${height} box-border ${className}`} key={colIndex}>
                                    <span
                                        className="absolute h-[1px] w-[1px] overflow-hidden whitespace-nowrap"
                                        style={{
                                            clip: "rect(0 0 0 0)",
                                            clipPath: "inset(50%)",
                                        }}
                                    >{`${grid[rowIndex]?.[colIndex]} cell`}</span>
                                </td>
                            );
                        })}
                    </tr>
                ))}
            </tbody>
        </table>
    );
};

export default React.memo(OperatorRange);
