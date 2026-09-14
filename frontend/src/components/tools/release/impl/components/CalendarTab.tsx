import { ChevronLeft, ChevronRight } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { cn } from "#/lib/utils";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange } from "../helpers";
import { DAY_SECS, dayOf, dayStart, type IScheduleItem, overlaps, packLanes, type ScheduleKind, useSchedule } from "../schedule";
import { ALL_KINDS, itemName, KIND_STYLE, ScheduleControls, ScheduleDetail } from "./ScheduleShared";
import { ReleaseEmpty, ReleaseError, ReleaseLoading } from "./shared";

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

interface ICalendarTabProps {
    today: Date;
}

export function CalendarTab({ today }: ICalendarTabProps): React.ReactElement {
    const schedule = useSchedule();
    const autoOn = useAutoTranslate();
    const [kinds, setKinds] = React.useState<Set<ScheduleKind>>(() => new Set(ALL_KINDS.filter((k) => k !== "rerun")));
    const [stageOnly, setStageOnly] = React.useState(true);
    const [cursor, setCursor] = React.useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
    const [selected, setSelected] = React.useState<string | null>(null);

    const monthFrom = dayStart(cursor);
    const monthTo = dayStart(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 0));
    const inMonth = React.useMemo(() => schedule.items.filter((it) => (!stageOnly || it.kind !== "event" || it.hasStage) && overlaps(it, monthFrom, monthTo + DAY_SECS - 1)), [schedule.items, stageOnly, monthFrom, monthTo]);
    const counts = React.useMemo(() => {
        const c: Record<ScheduleKind, number> = { event: 0, banner: 0, skin: 0, rerun: 0 };
        for (const it of inMonth) c[it.kind] += 1;
        return c;
    }, [inMonth]);
    const shown = React.useMemo(() => inMonth.filter((it) => kinds.has(it.kind)), [inMonth, kinds]);

    const weeks = React.useMemo(() => {
        const first = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
        const gridStart = dayStart(new Date(first.getFullYear(), first.getMonth(), 1 - first.getDay()));
        const out: number[] = [];
        for (let w = gridStart; w <= monthTo; w += 7 * DAY_SECS) out.push(w);
        return out;
    }, [cursor, monthTo]);

    const todayDay = dayStart(today);
    const selectedItem = selected ? shown.find((it) => it.key === selected) : undefined;

    if (schedule.isPending) return <ReleaseLoading />;
    if (schedule.error) return <ReleaseError error={schedule.error} onRetry={schedule.refetch} />;

    const monthLabel = cursor.toLocaleDateString("en-US", { month: "long", year: "numeric" });
    return (
        <div className="flex flex-col gap-3">
            <ScheduleControls kinds={kinds} onKindsChange={setKinds} stageOnly={stageOnly} onStageOnlyChange={setStageOnly} counts={counts} />
            <div className="flex items-center gap-2">
                <Button size="sm" variant="outline" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() - 1, 1))} aria-label="Previous month">
                    <ChevronLeft />
                </Button>
                <Button size="sm" variant="outline" onClick={() => setCursor(new Date(today.getFullYear(), today.getMonth(), 1))}>
                    Today
                </Button>
                <Button size="sm" variant="outline" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 1))} aria-label="Next month">
                    <ChevronRight />
                </Button>
                <h3 className="m-0 ml-1 font-sans font-semibold text-[16px] text-foreground">{monthLabel}</h3>
                <span className="font-medium font-mono text-[11px] text-muted-foreground">{shown.length} on EN</span>
            </div>
            {selectedItem && <ScheduleDetail item={selectedItem} lookup={schedule.lookup} today={today} onClose={() => setSelected(null)} />}
            {schedule.items.length === 0 ? (
                <ReleaseEmpty title="Nothing dated" description="The backend returned no rows with an EN date." />
            ) : (
                <Card className="gap-0 overflow-hidden p-0">
                    <div>
                        <div>
                            <div className="grid grid-cols-7 border-border border-b">
                                {WEEKDAYS.map((d) => (
                                    <div key={d} className="px-1.5 py-1.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.06em] sm:px-2">
                                        <span className="sm:hidden">{d.charAt(0)}</span>
                                        <span className="max-sm:hidden">{d}</span>
                                    </div>
                                ))}
                            </div>
                            {weeks.map((weekStart) => (
                                <CalendarWeek key={weekStart} weekStart={weekStart} monthFrom={monthFrom} monthTo={monthTo} todayDay={todayDay} items={shown} selected={selected} onSelect={setSelected} autoOn={autoOn} />
                            ))}
                        </div>
                    </div>
                </Card>
            )}
        </div>
    );
}

interface ICalendarWeekProps {
    weekStart: number;
    monthFrom: number;
    monthTo: number;
    todayDay: number;
    items: IScheduleItem[];
    selected: string | null;
    onSelect: (key: string | null) => void;
    autoOn: boolean;
}

function CalendarWeek({ weekStart, monthFrom, monthTo, todayDay, items, selected, onSelect, autoOn }: ICalendarWeekProps): React.ReactElement {
    const weekEnd = weekStart + 7 * DAY_SECS - 1;
    const segments = items
        .filter((it) => overlaps(it, weekStart, weekEnd))
        .map((it) => {
            const s = Math.max(dayOf(it.start), weekStart);
            const e = Math.min(dayOf(it.end ?? it.start), weekStart + 6 * DAY_SECS);
            const col = Math.round((s - weekStart) / DAY_SECS) + 1;
            const span = Math.round((e - s) / DAY_SECS) + 1;
            return { it, col, span, continuesLeft: dayOf(it.start) < weekStart, continuesRight: dayOf(it.end ?? it.start) > weekEnd };
        });
    const lanes = packLanes(segments.map(({ it, col, span }) => ({ ...it, start: col, end: col + span - 1 })));
    const laneCount = segments.length === 0 ? 0 : Math.max(...segments.map((sg) => lanes.get(sg.it.key) ?? 0)) + 1;
    const days = Array.from({ length: 7 }, (_, i) => weekStart + i * DAY_SECS);
    return (
        <div className="grid grid-cols-7 border-border border-b last:border-b-0" style={{ gridTemplateRows: `28px repeat(${laneCount}, 22px) 6px` }}>
            {days.map((d, i) => {
                const outside = d < monthFrom || d > monthTo;
                const isToday = d === todayDay;
                const n = new Date(d * 1000).getDate();
                return (
                    <div key={d} className={cn("border-border/60 border-l px-1 pt-1 font-mono text-[11px] tabular-nums first:border-l-0 sm:px-2", outside ? "text-muted-foreground/50" : "text-muted-foreground")} style={{ gridColumn: i + 1, gridRow: `1 / span ${laneCount + 2}` }}>
                        <span className={cn("inline-flex size-5 items-center justify-center rounded-full", isToday && "bg-primary font-semibold text-primary-foreground")}>{n}</span>
                    </div>
                );
            })}
            {segments.map(({ it, col, span, continuesLeft, continuesRight }) => {
                const lane = lanes.get(it.key) ?? 0;
                const style = KIND_STYLE[it.kind];
                const name = itemName(it, autoOn);
                const isSel = it.key === selected;
                return (
                    <button
                        key={it.key}
                        type="button"
                        onClick={() => onSelect(isSel ? null : it.key)}
                        title={`${name}\n${formatDateRange(it.start, it.end)}`}
                        className={cn(
                            "z-1 mx-0.5 h-4.5 cursor-pointer truncate px-1.5 text-left font-medium font-sans text-[11px] leading-[18px] outline-none hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring",
                            it.estimated ? style.estimated : style.solid,
                            continuesLeft ? "rounded-l-none" : "rounded-l-sm",
                            continuesRight ? "rounded-r-none" : "rounded-r-sm",
                            isSel && "ring-2 ring-foreground/80",
                        )}
                        style={{ gridColumn: `${col} / span ${span}`, gridRow: lane + 2 }}
                    >
                        {name}
                    </button>
                );
            })}
        </div>
    );
}
