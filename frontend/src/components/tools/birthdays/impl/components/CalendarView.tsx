import type * as React from "react";
import { Card } from "#/components/ui/card";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { buildAgendaDays, formatRangeTitle, stepAnchor } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import type { CalendarScale, IOperatorBirthday, ISelectedDay } from "../types";
import { AgendaColumns } from "./AgendaColumns";
import { CalendarToolbar } from "./CalendarToolbar";
import { MonthGrid } from "./MonthGrid";

interface ICalendarViewProps {
    scale: CalendarScale;
    onScaleChange: (scale: CalendarScale) => void;
    /** The date the view is centred on; navigation moves this by one page per scale. */
    anchor: Date;
    onAnchorChange: (date: Date) => void;
    byDay: Map<string, IOperatorBirthday[]>;
    onSelect: (day: ISelectedDay) => void;
    today: Date;
}

/** The calendar surface: a toolbar over a month grid or an agenda (day/3day/week). */
export function CalendarView({ scale, onScaleChange, anchor, onAnchorChange, byDay, onSelect, today }: ICalendarViewProps): React.ReactElement {
    const t: TypedT<typeof helperMessages> = useT("tools");
    const locale = useLocale();
    return (
        <Card className="overflow-hidden">
            <CalendarToolbar
                title={formatRangeTitle(scale, anchor, locale, t)}
                titleShort={formatRangeTitle(scale, anchor, locale, t, true)}
                scale={scale}
                onScaleChange={onScaleChange}
                onPrev={() => onAnchorChange(stepAnchor(scale, anchor, -1))}
                onNext={() => onAnchorChange(stepAnchor(scale, anchor, 1))}
                onToday={() => onAnchorChange(new Date())}
            />
            {scale === "month" ? <MonthGrid anchor={anchor} byDay={byDay} onSelect={onSelect} today={today} /> : <AgendaColumns scale={scale} days={buildAgendaDays(scale, anchor)} byDay={byDay} today={today} onSelect={onSelect} />}
        </Card>
    );
}
