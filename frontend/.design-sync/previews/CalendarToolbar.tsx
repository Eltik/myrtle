import { CalendarToolbar, Card } from "frontend";

// The toolbar always sits at the top of the calendar Card, so every story wraps
// it in one. Titles are what `formatRangeTitle` produces for each scale.
// Prev/next/today/scale changes are interaction-only and can't be photographed.
const noop = () => {};

export const MonthScale = () => (
    <Card className="max-w-2xl overflow-hidden">
        <CalendarToolbar title="May 2024" titleShort="May 2024" scale="month" onScaleChange={noop} onPrev={noop} onNext={noop} onToday={noop} />
    </Card>
);

export const WeekScale = () => (
    <Card className="max-w-2xl overflow-hidden">
        <CalendarToolbar title="May 12 - 18, 2024" titleShort="May 12 - 18, 2024" scale="week" onScaleChange={noop} onPrev={noop} onNext={noop} onToday={noop} />
    </Card>
);

export const ThreeDayScale = () => (
    <Card className="max-w-2xl overflow-hidden">
        <CalendarToolbar title="May 14 - 16, 2024" titleShort="May 14 - 16, 2024" scale="3day" onScaleChange={noop} onPrev={noop} onNext={noop} onToday={noop} />
    </Card>
);

export const DayScale = () => (
    <Card className="max-w-2xl overflow-hidden">
        <CalendarToolbar title="Wednesday, May 15, 2024" titleShort="May 15, 2024" scale="day" onScaleChange={noop} onPrev={noop} onNext={noop} onToday={noop} />
    </Card>
);
