import { CalendarX } from "lucide-react";
import type * as React from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogPanel, DialogTitle } from "#/components/ui/dialog";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { isToday, monthNames } from "../helpers";
import type { IOperatorBirthday, ISelectedDay } from "../types";
import { BirthdayEmpty } from "./BirthdayEmpty";
import type { messages } from "./DayDialog.messages";
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
    const t: TypedT<typeof messages> = useT("tools");
    const locale = useLocale();
    const f = useFormatters();
    if (!date) return null;

    const weekday = f.date(new Date(date.year, date.month - 1, date.day), { weekday: "long" });
    const isTodayDate = isToday(today, date.year, date.month, date.day);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle className="tracking-tight">{t("birthdays.day.title", { month: monthNames(locale)[date.month - 1], day: date.day })}</DialogTitle>
                    <DialogDescription className="font-mono text-[11.5px] uppercase tracking-[0.08em]">{isTodayDate ? t("birthdays.day.metaToday", { weekday, count: ops.length }) : t("birthdays.day.meta", { weekday, count: ops.length })}</DialogDescription>
                </DialogHeader>

                {ops.length === 0 ? (
                    <BirthdayEmpty className="py-10" icon={<CalendarX />} title={t("birthdays.day.empty.title")} description={t("birthdays.day.empty.desc")} />
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
