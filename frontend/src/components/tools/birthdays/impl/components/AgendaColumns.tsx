import { CalendarX } from "lucide-react";
import type * as React from "react";
import { cn, parseOperatorName } from "#/lib/utils";
import { WEEKDAYS } from "../constants";
import { isSameDate, opsOn, toSelectedDay } from "../helpers";
import type { CalendarScale, IOperatorBirthday, ISelectedDay } from "../types";
import { BirthdayEmpty } from "./BirthdayEmpty";
import { OpChip } from "./OpChip";
import { OperatorRow } from "./OperatorRow";

/** Compact preview rows shown per agenda column before collapsing to "+N". */
const MAX_PREVIEW = 6;

interface IAgendaColumnsProps {
    scale: Exclude<CalendarScale, "month">;
    days: Date[];
    byDay: Map<string, IOperatorBirthday[]>;
    today: Date;
    onSelect: (day: ISelectedDay) => void;
}

const dayOps = (byDay: Map<string, IOperatorBirthday[]>, date: Date) => opsOn(byDay, date.getMonth() + 1, date.getDate());

/** Day / 3 Day / Week agenda. Day shows a full inline list; the wider scales show clickable day columns. */
export function AgendaColumns({ scale, days, byDay, today, onSelect }: IAgendaColumnsProps): React.ReactElement {
    if (scale === "day") {
        const ops = dayOps(byDay, days[0]);
        if (ops.length === 0) {
            return <BirthdayEmpty className="py-14" icon={<CalendarX />} title="No birthdays" description="No operators in your current filter celebrate on this day." />;
        }
        return (
            <div className="flex flex-col gap-0.5 p-2">
                {ops.map((b) => (
                    <OperatorRow key={b.operator.id} birthday={b} />
                ))}
            </div>
        );
    }

    const columnsClass = scale === "week" ? "lg:grid-cols-7" : "sm:grid-cols-3";
    const columnMinH = scale === "week" ? "lg:min-h-38" : "sm:min-h-38";

    return (
        <div className="p-2">
            <div className={cn("grid grid-cols-1 gap-1.5", columnsClass)}>
                {days.map((date) => {
                    const ops = dayOps(byDay, date);
                    const isToday = isSameDate(date, today);
                    const visible = ops.slice(0, MAX_PREVIEW);
                    const overflow = ops.length - visible.length;

                    return (
                        <button
                            key={date.toISOString()}
                            type="button"
                            onClick={() => onSelect(toSelectedDay(date))}
                            className={cn("flex cursor-pointer flex-col gap-2.5 rounded-[10px] border p-2.5 text-left transition-colors", columnMinH, isToday ? "border-primary/40 bg-primary/8" : ops.length > 0 ? "border-border bg-muted/40 hover:bg-accent" : "border-transparent hover:border-border hover:bg-accent")}
                        >
                            <div className="flex items-baseline justify-between gap-1">
                                <div className="flex min-w-0 flex-col gap-0.5">
                                    <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.08em]">{WEEKDAYS[date.getDay()]}</span>
                                    <span className={cn("font-bold font-sans text-[20px] leading-none tracking-tight", isToday ? "text-primary" : "text-foreground")}>{date.getDate()}</span>
                                </div>
                                {ops.length > 0 && <span className="font-medium font-mono text-[10.5px] text-muted-foreground">{ops.length}</span>}
                            </div>
                            {visible.length > 0 && (
                                <div className="flex flex-col gap-1">
                                    {visible.map((b) => {
                                        const { displayName } = parseOperatorName(b.operator.name);
                                        return (
                                            <div key={b.operator.id} className="grid grid-cols-[auto_1fr] items-center gap-2">
                                                <OpChip operator={b.operator} />
                                                <span className="truncate font-medium font-sans text-[12px] text-foreground">{displayName}</span>
                                            </div>
                                        );
                                    })}
                                    {overflow > 0 && <span className="ps-0.5 font-medium font-mono text-[10.5px] text-muted-foreground">+{overflow} more</span>}
                                </div>
                            )}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
