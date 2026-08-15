import { Calendar, Card } from "frontend";

const MARCH_2026 = new Date(2026, 2, 1);

export const SingleDate = () => <Calendar className="rounded-lg border border-border p-3" defaultMonth={MARCH_2026} mode="single" selected={new Date(2026, 2, 12)} />;

export const DateRange = () => (
    <div className="flex flex-col gap-2">
        <Calendar
            className="rounded-lg border border-border p-3"
            defaultMonth={MARCH_2026}
            mode="range"
            selected={{
                from: new Date(2026, 2, 9),
                to: new Date(2026, 2, 16),
            }}
        />
        <p className="text-muted-foreground text-xs">
            Event window: <span className="font-mono text-foreground">Mar 9 – Mar 16</span> · 8 days of Annihilation resets
        </p>
    </div>
);

export const OperatorBirthdays = () => (
    <Card className="w-fit p-3">
        <Calendar
            defaultMonth={MARCH_2026}
            mode="single"
            modifiers={{
                birthday: [new Date(2026, 2, 3), new Date(2026, 2, 14), new Date(2026, 2, 21), new Date(2026, 2, 28)],
            }}
            modifiersClassNames={{
                birthday: "rounded-lg bg-primary/12 font-semibold",
            }}
            selected={new Date(2026, 2, 14)}
        />
        <p className="mt-2 border-border border-t px-1 pt-2 text-muted-foreground text-xs">
            <span className="font-semibold text-primary">4</span> operator birthdays this month — Skadi is on the 14th.
        </p>
    </Card>
);

export const WithDropdownNavigation = () => <Calendar captionLayout="dropdown" className="rounded-lg border border-border p-3" defaultMonth={MARCH_2026} endMonth={new Date(2027, 11, 1)} mode="single" selected={new Date(2026, 2, 12)} startMonth={new Date(2019, 4, 1)} />;

export const DisabledDays = () => (
    <div className="flex flex-col gap-2">
        <Calendar
            className="rounded-lg border border-border p-3"
            defaultMonth={MARCH_2026}
            disabled={[{ before: new Date(2026, 2, 10) }, new Date(2026, 2, 18), new Date(2026, 2, 19)]}
            mode="single"
            selected={new Date(2026, 2, 24)}
        />
        <p className="text-muted-foreground text-xs">Banner archive only goes back to Mar 10; the 18th–19th are maintenance days.</p>
    </div>
);
