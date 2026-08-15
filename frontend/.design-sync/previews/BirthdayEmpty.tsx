import { BirthdayEmpty, Card } from "frontend";
import { CalendarX, SearchX, UserX } from "lucide-react";

// The two empty states the birthday tool actually ships: a filtered-out list
// (SearchX) and a calendar day nobody celebrates on (CalendarX). `className`
// only overrides padding for the tighter dialog/agenda placements.
export const NoResults = () => (
    <Card className="max-w-2xl">
        <BirthdayEmpty icon={<SearchX />} title="No results" description="No operators match your filters. Try clearing rarity or class chips, or reset everything." />
    </Card>
);

export const NoBirthdaysOnDay = () => (
    <Card className="max-w-2xl">
        <BirthdayEmpty className="py-14" icon={<CalendarX />} title="No birthdays" description="No operators in your current filter celebrate on this day." />
    </Card>
);

export const CompactInDialog = () => (
    <div className="max-w-md rounded-[14px] border border-border bg-card">
        <div className="px-6 pt-6">
            <h2 className="m-0 font-bold font-sans text-[18px] text-foreground tracking-tight">May 12</h2>
            <p className="mt-1 font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.08em]">Sunday · 0 operators</p>
        </div>
        <BirthdayEmpty className="py-10" icon={<CalendarX />} title="No birthdays" description="No operators in your current filter celebrate on this day." />
    </div>
);

export const UnknownBirthdays = () => (
    <Card className="max-w-2xl">
        <BirthdayEmpty icon={<UserX />} title="No dated operators" description="Kal'tsit, Blaze and 14 others keep their date of birth undisclosed, so they never land on the calendar." />
    </Card>
);
