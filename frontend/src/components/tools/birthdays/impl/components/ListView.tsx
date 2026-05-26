import { SearchX } from "lucide-react";
import * as React from "react";
import { Card } from "#/components/ui/card";
import { cn, formatProfession, parseOperatorName } from "#/lib/utils";
import { MONTHS } from "../constants";
import { isTodayMonthDay, operatorRarity } from "../helpers";
import type { IOperatorBirthday } from "../types";
import { BirthdayEmpty } from "./BirthdayEmpty";
import { OpChip } from "./OpChip";

interface IListViewProps {
    items: IOperatorBirthday[];
    today: Date;
}

/** Chronological list of every known birthday, months ordered starting from the current one. */
export function ListView({ items, today }: IListViewProps): React.ReactElement {
    const months = React.useMemo(() => {
        const byMonth = new Map<number, IOperatorBirthday[]>();
        for (const b of items) {
            if (!b.known) continue;
            const bucket = byMonth.get(b.month);
            if (bucket) bucket.push(b);
            else byMonth.set(b.month, [b]);
        }

        const currentMonth = today.getMonth() + 1;
        const ordered: { month: number; entries: IOperatorBirthday[] }[] = [];
        for (let i = 0; i < 12; i++) {
            const month = ((currentMonth - 1 + i) % 12) + 1;
            const entries = byMonth.get(month);
            if (entries) ordered.push({ month, entries });
        }
        return ordered;
    }, [items, today]);

    if (months.length === 0) {
        return (
            <Card>
                <BirthdayEmpty icon={<SearchX />} title="No results" description="No operators match your filters. Try clearing rarity or class chips, or reset everything." />
            </Card>
        );
    }

    return (
        <Card>
            {months.map(({ month, entries }) => {
                const byDay = new Map<number, IOperatorBirthday[]>();
                for (const b of entries) {
                    if (!b.known) continue;
                    const bucket = byDay.get(b.day);
                    if (bucket) bucket.push(b);
                    else byDay.set(b.day, [b]);
                }
                const days = [...byDay.entries()].sort((a, b) => a[0] - b[0]);

                return (
                    <section key={month} className="border-border border-t px-4 py-4 first:border-t-0 sm:px-5">
                        <h3 className="m-0 mb-2.5 flex items-baseline gap-2 font-sans font-semibold text-[16px] text-foreground">
                            {MONTHS[month - 1]}
                            <span className="font-medium font-mono text-[11px] text-muted-foreground">· {entries.length}</span>
                        </h3>
                        {days.map(([day, ops]) => {
                            const dayIsToday = isTodayMonthDay(today, month, day);
                            const weekday = new Date(today.getFullYear(), month - 1, day).toLocaleDateString("en-US", { weekday: "short" });
                            return (
                                <div key={day} className="grid grid-cols-[64px_1fr] gap-4 border-border border-t py-2 first-of-type:border-t-0">
                                    <div className="flex flex-col items-start">
                                        <span className={cn("font-bold font-sans text-[20px] leading-none tracking-tight", dayIsToday ? "text-primary" : "text-foreground")}>{day}</span>
                                        <span className="mt-1 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {weekday}
                                            {dayIsToday && " · today"}
                                        </span>
                                    </div>
                                    <div className="flex flex-col gap-0.5">
                                        {ops.map((b) => {
                                            const { displayName } = parseOperatorName(b.operator.name);
                                            return (
                                                <div key={b.operator.id} className="grid grid-cols-[auto_1fr_auto] items-center gap-3 py-1">
                                                    <OpChip operator={b.operator} />
                                                    <span className="truncate font-medium font-sans text-[13.5px] text-foreground">{displayName}</span>
                                                    <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.06em]">
                                                        {operatorRarity(b.operator)}★ · {formatProfession(b.operator.profession)}
                                                    </span>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            );
                        })}
                    </section>
                );
            })}
        </Card>
    );
}
