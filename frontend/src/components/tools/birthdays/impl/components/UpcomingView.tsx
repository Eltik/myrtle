import { SearchX } from "lucide-react";
import * as React from "react";
import { Card } from "#/components/ui/card";
import { cn, parseOperatorName } from "#/lib/utils";
import { MONTHS_SHORT } from "../constants";
import { compareUpcoming, dayKey, daysUntil, isTodayMonthDay, operatorRarity, rarityVar } from "../helpers";
import type { IOperatorBirthday } from "../types";
import { BirthdayEmpty } from "./BirthdayEmpty";
import { OpChip } from "./OpChip";

/** How many upcoming dates (not operators) to surface. */
const MAX_DATES = 24;

interface IUpcomingViewProps {
    items: IOperatorBirthday[];
    today: Date;
}

/** The next ~24 birthday dates from today, soonest first. */
export function UpcomingView({ items, today }: IUpcomingViewProps): React.ReactElement {
    const dates = React.useMemo(() => {
        const known = items.filter((b): b is Extract<IOperatorBirthday, { known: true }> => b.known);
        known.sort((a, b) => compareUpcoming(a, b, today) || operatorRarity(b.operator) - operatorRarity(a.operator));

        const groups = new Map<string, Extract<IOperatorBirthday, { known: true }>[]>();
        for (const b of known) {
            const key = dayKey(b.month, b.day);
            const bucket = groups.get(key);
            if (bucket) bucket.push(b);
            else groups.set(key, [b]);
        }
        return [...groups.values()].slice(0, MAX_DATES);
    }, [items, today]);

    if (dates.length === 0) {
        return (
            <Card>
                <BirthdayEmpty icon={<SearchX />} title="No results" description="No operators match your filters. Try clearing rarity or class chips, or reset everything." />
            </Card>
        );
    }

    return (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
            {dates.map((ops) => {
                const { month, day } = ops[0];
                const isToday = isTodayMonthDay(today, month, day);
                const until = daysUntil(month, day, today);
                const when = isToday ? "Today" : until === 1 ? "Tomorrow" : `In ${until} days`;
                const weekday = new Date(today.getFullYear(), month - 1, day).toLocaleDateString("en-US", { weekday: "long" });

                return (
                    <div key={dayKey(month, day)} className={cn("flex flex-col gap-2.5 rounded-[14px] border bg-card p-4 shadow-xs/5", isToday ? "border-primary/50 bg-primary/6" : "border-border")}>
                        <div className="flex items-baseline justify-between">
                            <div>
                                <div className={cn("font-medium font-mono text-[11px] uppercase tracking-[0.08em]", isToday ? "text-primary" : "text-muted-foreground")}>{when}</div>
                                <div className="font-bold font-sans text-[18px] text-foreground tracking-tight">
                                    {MONTHS_SHORT[month - 1]} {day}
                                </div>
                            </div>
                            <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{weekday}</span>
                        </div>
                        <div className="flex flex-col gap-1.5">
                            {ops.map((b) => {
                                const { displayName } = parseOperatorName(b.operator.name);
                                const rarity = operatorRarity(b.operator);
                                return (
                                    <div key={b.operator.id} className="grid grid-cols-[auto_1fr_auto] items-center gap-2.5">
                                        <OpChip operator={b.operator} />
                                        <span className="truncate font-medium font-sans text-[13px] text-foreground">{displayName}</span>
                                        <span className="font-sans font-semibold text-[12px]" style={{ color: rarityVar(rarity) }}>
                                            {rarity}★
                                        </span>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
