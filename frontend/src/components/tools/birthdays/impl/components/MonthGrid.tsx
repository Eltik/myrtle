import * as React from "react";
import { cn } from "#/lib/utils";
import { WEEKDAYS } from "../constants";
import { buildCalendarCells, isToday, opsOn } from "../helpers";
import type { IOperatorBirthday, ISelectedDay } from "../types";
import { OpChip } from "./OpChip";

/** Operator chips shown per month cell before collapsing to "+N". */
const MAX_CHIPS = 9;

interface IMonthGridProps {
    anchor: Date;
    byDay: Map<string, IOperatorBirthday[]>;
    onSelect: (day: ISelectedDay) => void;
    today: Date;
}

/** Classic 6-week month grid; each day is a chip-preview cell that opens the day dialog. */
export function MonthGrid({ anchor, byDay, onSelect, today }: IMonthGridProps): React.ReactElement {
    const year = anchor.getFullYear();
    const month = anchor.getMonth() + 1;
    const cells = React.useMemo(() => buildCalendarCells(year, month), [year, month]);

    return (
        <>
            <div className="grid grid-cols-7 px-2 pt-2">
                {WEEKDAYS.map((w) => (
                    <span key={w} className="px-1.5 py-2 text-center font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">
                        {w}
                    </span>
                ))}
            </div>

            <div className="grid grid-cols-7 gap-px p-2">
                {cells.map((c) => {
                    const ops = c.outside ? [] : opsOn(byDay, c.month, c.day);
                    const cellIsToday = isToday(today, c.year, c.month, c.day);
                    const visible = ops.length > MAX_CHIPS + 1 ? ops.slice(0, MAX_CHIPS) : ops;
                    const overflow = ops.length - visible.length;

                    return (
                        <button
                            key={`${c.year}-${c.month}-${c.day}`}
                            type="button"
                            onClick={() => onSelect({ year: c.year, month: c.month, day: c.day })}
                            className={cn(
                                "flex min-h-13 cursor-pointer flex-col gap-1 rounded-[10px] border p-1.5 text-left transition-colors sm:aspect-[1.05/1] sm:min-h-24 sm:gap-1.5 sm:p-2",
                                cellIsToday ? "border-primary/40 bg-primary/8" : ops.length > 0 ? "border-border bg-muted/50 hover:bg-accent" : "border-transparent hover:border-border hover:bg-accent",
                                c.outside && "opacity-40",
                            )}
                        >
                            <div className="flex items-center justify-between">
                                <span className={cn("font-sans font-semibold text-[13px] leading-none sm:text-[14px]", c.outside ? "text-muted-foreground" : "text-foreground")}>{c.day}</span>
                                {ops.length > 0 && <span className="hidden size-1.25 rounded-full bg-primary sm:block" />}
                            </div>
                            {ops.length > 0 && <span className="mt-auto font-medium font-mono text-[11px] text-muted-foreground tabular-nums sm:hidden">{ops.length}</span>}
                            {visible.length > 0 && (
                                <div className="mt-auto hidden flex-wrap gap-1 sm:flex">
                                    {visible.map((b) => (
                                        <OpChip key={b.operator.id} operator={b.operator} />
                                    ))}
                                    {overflow > 0 && <span className="inline-flex h-5.5 items-center justify-center rounded-md border border-border bg-card px-1.5 font-medium font-mono text-[10.5px] text-muted-foreground">+{overflow}</span>}
                                </div>
                            )}
                        </button>
                    );
                })}
            </div>
        </>
    );
}
