import { CalendarDays, Clock, ListIcon } from "lucide-react";
import * as React from "react";
import { PageHeader } from "#/components/ui/page-header";
import { Tabs, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorListItem } from "#/types/operators";
import type { messages } from "./Birthdays.messages";
import { calculateBirthdays } from "./impl/calculate";
import { CalendarView } from "./impl/components/CalendarView";
import { DayDialog } from "./impl/components/DayDialog";
import { FilterSheet } from "./impl/components/FilterSheet";
import { FilterSidebar } from "./impl/components/FilterSidebar";
import { ListView } from "./impl/components/ListView";
import { TodayCallout } from "./impl/components/TodayCallout";
import { UpcomingView } from "./impl/components/UpcomingView";
import { applyFilters, countActiveFilters, countKnown, createEmptyFilters, deriveNations, groupByDay, isCalendarOperator, opsOn } from "./impl/helpers";
import type { BirthdayView, CalendarScale, IBirthdayFilters, ISelectedDay } from "./impl/types";

interface IBirthdaysProps {
    operators: IOperatorListItem[];
}

export function Birthdays({ operators }: IBirthdaysProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const today = React.useMemo(() => new Date(), []);

    const allBirthdays = React.useMemo(() => calculateBirthdays(operators.filter(isCalendarOperator)), [operators]);
    const nations = React.useMemo(() => deriveNations(allBirthdays.map((b) => b.operator)), [allBirthdays]);

    const [view, setView] = React.useState<BirthdayView>("calendar");
    const [scale, setScale] = React.useState<CalendarScale>("month");
    const [anchor, setAnchor] = React.useState<Date>(today);
    const [selected, setSelected] = React.useState<ISelectedDay | null>({ year: today.getFullYear(), month: today.getMonth() + 1, day: today.getDate() });
    const [dayDialogOpen, setDayDialogOpen] = React.useState(false);
    const [filters, setFilters] = React.useState<IBirthdayFilters>(createEmptyFilters);

    const filtered = React.useMemo(() => applyFilters(allBirthdays, filters), [allBirthdays, filters]);
    const byDay = React.useMemo(() => groupByDay(filtered), [filtered]);

    const knownTotal = React.useMemo(() => countKnown(allBirthdays), [allBirthdays]);
    const matchedCount = React.useMemo(() => countKnown(filtered), [filtered]);
    const todayOps = opsOn(byDay, today.getMonth() + 1, today.getDate());
    const selectedOps = selected ? opsOn(byDay, selected.month, selected.day) : [];

    const onSelectDay = React.useCallback((day: ISelectedDay) => {
        setSelected(day);
        setDayDialogOpen(true);
    }, []);

    const onResetFilters = React.useCallback(() => setFilters(createEmptyFilters()), []);
    const activeFilterCount = React.useMemo(() => countActiveFilters(filters), [filters]);

    return (
        <div className="page-shell [--page-max:1400px]">
            <PageHeader breadcrumbLabel="breadcrumb" breadcrumb={[t("birthdays.breadcrumb.tools"), t("birthdays.title")]} title={t("birthdays.title")} description={t("birthdays.intro")} />

            <TodayCallout ops={todayOps} today={today} />

            <div className="mt-7 mb-4.5 flex flex-col items-stretch gap-3 sm:flex-row sm:items-center sm:justify-between">
                <Tabs value={view} onValueChange={(v) => setView(v as BirthdayView)}>
                    <TabsList className="max-sm:w-full">
                        <TabsTrigger value="calendar" className="max-sm:flex-1">
                            <CalendarDays />
                            {t("birthdays.tab.calendar")}
                        </TabsTrigger>
                        <TabsTrigger value="list" className="max-sm:flex-1">
                            <ListIcon />
                            {t("birthdays.tab.list")}
                            <span className="font-medium font-mono text-[11px] text-muted-foreground">{matchedCount}</span>
                        </TabsTrigger>
                        <TabsTrigger value="upcoming" className="max-sm:flex-1">
                            <Clock />
                            {t("birthdays.tab.upcoming")}
                        </TabsTrigger>
                    </TabsList>
                </Tabs>
                <FilterSheet filters={filters} onChange={setFilters} nations={nations} matched={matchedCount} total={knownTotal} activeCount={activeFilterCount} onReset={onResetFilters} />
            </div>

            <div className="grid grid-cols-1 items-start gap-4 lg:grid-cols-[300px_1fr]">
                <FilterSidebar filters={filters} onChange={setFilters} nations={nations} matched={matchedCount} total={knownTotal} onReset={onResetFilters} />
                <main className="min-w-0">
                    {view === "calendar" && <CalendarView scale={scale} onScaleChange={setScale} anchor={anchor} onAnchorChange={setAnchor} byDay={byDay} onSelect={onSelectDay} today={today} />}
                    {view === "list" && <ListView items={filtered} today={today} />}
                    {view === "upcoming" && <UpcomingView items={filtered} today={today} />}
                </main>
            </div>

            <DayDialog open={dayDialogOpen} onOpenChange={setDayDialogOpen} date={selected} ops={selectedOps} today={today} />
        </div>
    );
}
