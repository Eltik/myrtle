import { ChevronLeft, ChevronRight } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { CALENDAR_SCALES } from "../constants";
import type { CalendarScale } from "../types";

const UNIT: Record<CalendarScale, string> = { day: "day", "3day": "period", week: "week", month: "month" };

interface ICalendarToolbarProps {
    title: string;
    /** Condensed title (weekday dropped, months abbreviated). */
    titleShort: string;
    scale: CalendarScale;
    onScaleChange: (scale: CalendarScale) => void;
    onPrev: () => void;
    onNext: () => void;
    onToday: () => void;
}

/** Calendar header: Today + range stepper on the left, scale switcher on the right. */
export function CalendarToolbar({ title, titleShort, scale, onScaleChange, onPrev, onNext, onToday }: ICalendarToolbarProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-3 border-border border-b px-4 py-3.5 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between sm:px-5">
            <div className="flex min-w-0 items-center gap-2">
                <Button variant="outline" size="sm" onClick={onToday}>
                    Today
                </Button>
                <div className="flex items-center gap-1">
                    <Button variant="outline" size="icon-sm" aria-label={`Previous ${UNIT[scale]}`} onClick={onPrev}>
                        <ChevronLeft />
                    </Button>
                    <Button variant="outline" size="icon-sm" aria-label={`Next ${UNIT[scale]}`} onClick={onNext}>
                        <ChevronRight />
                    </Button>
                </div>
                <h2 className="m-0 ml-1 min-w-0 truncate font-bold font-sans text-[17px] text-foreground leading-tight tracking-tight sm:text-[19px]">
                    <span className="sm:hidden">{titleShort}</span>
                    <span className="hidden sm:inline">{title}</span>
                </h2>
            </div>
            <Tabs value={scale} onValueChange={(v) => v && onScaleChange(v as CalendarScale)} className="w-full sm:w-auto">
                <TabsList className="w-full sm:w-fit">
                    {CALENDAR_SCALES.map((s) => (
                        <TabsTrigger key={s.id} value={s.id} className="flex-1 sm:flex-none">
                            {s.label}
                        </TabsTrigger>
                    ))}
                </TabsList>
            </Tabs>
        </div>
    );
}
