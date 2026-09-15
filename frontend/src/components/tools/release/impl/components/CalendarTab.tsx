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
const DAY_HEADER_PX = 26;
const LANE_PX = 22;
const MIN_WEEK_PX = DAY_HEADER_PX + 3 * LANE_PX;
const NAV_CLEARANCE_REM = 5.5;

interface ICalendarTabProps {
    today: Date;
}

export function CalendarTab({ today }: ICalendarTabProps): React.ReactElement {
    const schedule = useSchedule();
    const autoOn = useAutoTranslate();
    const [kinds, setKinds] = React.useState<Set<ScheduleKind>>(() => new Set(ALL_KINDS.filter((k) => k !== "rerun" && k !== "review")));
    const [stageOnly, setStageOnly] = React.useState(true);
    const [cursor, setCursor] = React.useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
    const [selected, setSelected] = React.useState<string | null>(null);

    const monthFrom = dayStart(cursor);
    const monthTo = dayStart(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 0));
    const inMonth = React.useMemo(() => schedule.items.filter((it) => (!stageOnly || it.kind !== "event" || it.hasStage) && overlaps(it, monthFrom, monthTo + DAY_SECS - 1)), [schedule.items, stageOnly, monthFrom, monthTo]);
    const counts = React.useMemo(() => {
        const c: Record<ScheduleKind, number> = { event: 0, banner: 0, skin: 0, rerun: 0, review: 0 };
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

    const month = cursor.toLocaleDateString("en-US", { month: "long" });
    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                <h3 className="m-0 font-sans text-[22px] text-foreground leading-none">
                    <span className="font-bold">{month}</span> {cursor.getFullYear()}
                </h3>
                <div className="flex items-center gap-1">
                    <Button size="sm" variant="ghost" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() - 1, 1))} aria-label="Previous month">
                        <ChevronLeft />
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => setCursor(new Date(today.getFullYear(), today.getMonth(), 1))}>
                        Today
                    </Button>
                    <Button size="sm" variant="ghost" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 1))} aria-label="Next month">
                        <ChevronRight />
                    </Button>
                </div>
            </div>
            <ScheduleControls kinds={kinds} onKindsChange={setKinds} stageOnly={stageOnly} onStageOnlyChange={setStageOnly} counts={counts} />
            {selectedItem && <ScheduleDetail item={selectedItem} lookup={schedule.lookup} today={today} onClose={() => setSelected(null)} />}
            {schedule.items.length === 0 ? (
                <ReleaseEmpty title="Nothing dated" description="The backend returned no rows with an EN date." />
            ) : (
                <Card className="flex gap-0 overflow-y-auto p-0" style={{ height: `max(${weeks.length * MIN_WEEK_PX + 32}px, calc(100dvh - ${NAV_CLEARANCE_REM}rem))` }}>
                    <div className="grid shrink-0 grid-cols-7 border-border border-b">
                        {WEEKDAYS.map((d) => (
                            <div key={d} className="px-1.5 py-1.5 text-right font-medium font-sans text-[11.5px] text-muted-foreground sm:px-2">
                                <span className="sm:hidden">{d.charAt(0)}</span>
                                <span className="max-sm:hidden">{d}</span>
                            </div>
                        ))}
                    </div>
                    {weeks.map((weekStart) => (
                        <CalendarWeek key={weekStart} weekStart={weekStart} monthFrom={monthFrom} monthTo={monthTo} todayDay={todayDay} items={shown} selected={selected} onSelect={setSelected} autoOn={autoOn} />
                    ))}
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

function useVisibleLanes(): [React.RefObject<HTMLDivElement | null>, number] {
    const ref = React.useRef<HTMLDivElement>(null);
    const [lanes, setLanes] = React.useState(3);
    React.useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const measure = () => setLanes(Math.max(1, Math.floor((el.clientHeight - DAY_HEADER_PX - 2) / LANE_PX)));
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(el);
        return () => observer.disconnect();
    }, []);
    return [ref, lanes];
}

function CalendarWeek({ weekStart, monthFrom, monthTo, todayDay, items, selected, onSelect, autoOn }: ICalendarWeekProps): React.ReactElement {
    const [ref, visibleLanes] = useVisibleLanes();
    const [expanded, setExpanded] = React.useState(false);
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
    const shownLanes = expanded || laneCount <= visibleLanes ? laneCount : visibleLanes - 1;
    const rows = Math.max(shownLanes, 1);
    const hiddenPerDay = Array.from({ length: 7 }, (_, i) => segments.filter((sg) => (lanes.get(sg.it.key) ?? 0) >= shownLanes && sg.col <= i + 1 && sg.col + sg.span - 1 >= i + 1).length);
    const days = Array.from({ length: 7 }, (_, i) => weekStart + i * DAY_SECS);
    return (
        <div ref={ref} className={cn("grid min-h-0 grid-cols-7 border-border border-b last:border-b-0", expanded ? "shrink-0" : "flex-1 overflow-hidden")} style={{ gridTemplateRows: `${DAY_HEADER_PX}px repeat(${rows}, ${LANE_PX}px) minmax(0, 1fr)` }}>
            {days.map((d, i) => {
                const outside = d < monthFrom || d > monthTo;
                const isToday = d === todayDay;
                const n = new Date(d * 1000).getDate();
                return (
                    <div key={d} className={cn("border-border/60 border-l pt-1 pr-1.5 text-right font-sans text-[12px] tabular-nums first:border-l-0 sm:pr-2", outside ? "text-muted-foreground/40" : "text-foreground")} style={{ gridColumn: i + 1, gridRow: `1 / span ${rows + 2}` }}>
                        <span className={cn("inline-flex size-5 items-center justify-center rounded-full", isToday && "bg-primary font-semibold text-primary-foreground")}>{n}</span>
                    </div>
                );
            })}
            {segments
                .filter((sg) => (lanes.get(sg.it.key) ?? 0) < shownLanes)
                .map(({ it, col, span, continuesLeft, continuesRight }) => {
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
                                "z-1 my-0.5 flex min-w-0 cursor-pointer items-center gap-1.5 px-1.5 text-left font-medium font-sans text-[11.5px] leading-[18px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
                                style.pill,
                                continuesLeft ? "ml-0 rounded-l-none" : "ml-1 rounded-l-md",
                                continuesRight ? "mr-0 rounded-r-none" : "mr-1 rounded-r-md",
                                isSel && "ring-2 ring-foreground/60",
                            )}
                            style={{ gridColumn: `${col} / span ${span}`, gridRow: lane + 2 }}
                        >
                            {!continuesLeft && <span aria-hidden="true" className={cn("size-2 shrink-0 rounded-full", it.estimated ? "border-[1.5px] border-current bg-transparent" : style.dot)} />}
                            <span className="truncate">{name}</span>
                        </button>
                    );
                })}
            {hiddenPerDay.map((hidden, i) =>
                hidden > 0 ? (
                    <button key={days[i]} type="button" onClick={() => setExpanded(true)} className="z-1 cursor-pointer px-1.5 text-left font-sans text-[10.5px] text-muted-foreground leading-[18px] hover:text-foreground" style={{ gridColumn: i + 1, gridRow: rows + 2 }}>
                        +{hidden} more
                    </button>
                ) : null,
            )}
        </div>
    );
}
