import { CalendarX } from "lucide-react";
import type * as React from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogPanel, DialogTitle } from "#/components/ui/dialog";
import { MONTHS } from "../constants";
import { isToday } from "../helpers";
import type { IOperatorBirthday, ISelectedDay } from "../types";
import { BirthdayEmpty } from "./BirthdayEmpty";
import { OperatorRow } from "./OperatorRow";

interface IDayDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    date: ISelectedDay | null;
    ops: IOperatorBirthday[];
    today: Date;
}

/** Modal listing every operator whose birthday lands on the clicked calendar day. */
export function DayDialog({ open, onOpenChange, date, ops, today }: IDayDialogProps): React.ReactElement | null {
    if (!date) return null;

    const weekday = new Date(date.year, date.month - 1, date.day).toLocaleDateString("en-US", { weekday: "long" });

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle className="tracking-tight">
                        {MONTHS[date.month - 1]} {date.day}
                    </DialogTitle>
                    <DialogDescription className="font-mono text-[11.5px] uppercase tracking-[0.08em]">
                        {weekday}
                        {isToday(today, date.year, date.month, date.day) && " · today"} · {ops.length} {ops.length === 1 ? "operator" : "operators"}
                    </DialogDescription>
                </DialogHeader>

                {ops.length === 0 ? (
                    <BirthdayEmpty className="py-10" icon={<CalendarX />} title="No birthdays" description="No operators in your current filter celebrate on this day." />
                ) : (
                    <DialogPanel className="pt-1">
                        <div className="flex flex-col gap-0.5">
                            {ops.map((b) => (
                                <OperatorRow key={b.operator.id} birthday={b} />
                            ))}
                        </div>
                    </DialogPanel>
                )}
            </DialogContent>
        </Dialog>
    );
}
