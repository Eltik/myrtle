import * as React from "react";
import { Card } from "#/components/ui/card";
import { cn } from "#/lib/utils";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate, formatDateRange, isPast } from "../helpers";
import { DAY_SECS, dayOf, dayStart, type IScheduleItem, KIND_LABEL, packLanes, type ScheduleKind, useSchedule } from "../schedule";
import { ALL_KINDS, itemName, KIND_STYLE, ScheduleControls, ScheduleDetail } from "./ScheduleShared";
import { ReleaseEmpty, ReleaseError, ReleaseLoading, ToggleField } from "./shared";

const PX_PER_DAY = 7;
const LANE_PX = 26;
const LEAD_DAYS = 14;
const MAX_BAR_DAYS = 366;

function drawnEnd(it: IScheduleItem): { end: number; cut: boolean } {
    const end = it.end ?? it.start;
    const cap = it.start + MAX_BAR_DAYS * DAY_SECS;
    return end > cap ? { end: cap, cut: true } : { end, cut: false };
}

interface ITimelineTabProps {
    today: Date;
}

export function TimelineTab({ today }: ITimelineTabProps): React.ReactElement {
    const schedule = useSchedule();
    const autoOn = useAutoTranslate();
    const [kinds, setKinds] = React.useState<Set<ScheduleKind>>(() => new Set(ALL_KINDS.filter((k) => k !== "rerun")));
    const [stageOnly, setStageOnly] = React.useState(true);
    const [showPast, setShowPast] = React.useState(false);
    const [selected, setSelected] = React.useState<string | null>(null);

    const visible = React.useMemo(
        () =>
            schedule.items.filter((it) => {
                if (stageOnly && it.kind === "event" && !it.hasStage) return false;
                if (!showPast && isPast(it.end ?? it.start, today)) return false;
                return true;
            }),
        [schedule.items, stageOnly, showPast, today],
    );
    const counts = React.useMemo(() => {
        const c: Record<ScheduleKind, number> = { event: 0, banner: 0, skin: 0, rerun: 0 };
        for (const it of visible) c[it.kind] += 1;
        return c;
    }, [visible]);
    const shown = React.useMemo(() => visible.filter((it) => kinds.has(it.kind)), [visible, kinds]);

    const range = React.useMemo(() => {
        const todaySecs = dayStart(today);
        let lo = todaySecs - LEAD_DAYS * DAY_SECS;
        let hi = todaySecs + 90 * DAY_SECS;
        for (const it of shown) {
            lo = Math.min(lo, it.start);
            hi = Math.max(hi, drawnEnd(it).end);
        }
        const a = new Date(lo * 1000);
        const b = new Date(hi * 1000);
        const from = dayStart(new Date(a.getFullYear(), a.getMonth(), 1));
        const to = dayStart(new Date(b.getFullYear(), b.getMonth() + 1, 1));
        return { from, to, days: Math.round((to - from) / DAY_SECS) };
    }, [shown, today]);

    const groups = React.useMemo(
        () =>
            ALL_KINDS.filter((k) => kinds.has(k)).map((kind) => {
                const items = shown.filter((it) => it.kind === kind);
                const lanes = packLanes(
                    items.map((it) => ({ ...it, end: drawnEnd(it).end })),
                    DAY_SECS,
                );
                const laneCount = items.length === 0 ? 0 : Math.max(...items.map((it) => lanes.get(it.key) ?? 0)) + 1;
                return { kind, items, lanes, laneCount };
            }),
        [shown, kinds],
    );

    const months = React.useMemo(() => {
        const out: { label: string; x: number; width: number }[] = [];
        const d = new Date(range.from * 1000);
        while (dayStart(d) < range.to) {
            const start = dayStart(d);
            const next = dayStart(new Date(d.getFullYear(), d.getMonth() + 1, 1));
            out.push({ label: d.toLocaleDateString("en-US", { month: "short", year: "numeric" }), x: ((start - range.from) / DAY_SECS) * PX_PER_DAY, width: ((next - start) / DAY_SECS) * PX_PER_DAY });
            d.setMonth(d.getMonth() + 1);
        }
        return out;
    }, [range]);

    const scrollRef = React.useRef<HTMLDivElement>(null);
    const todayX = ((dayStart(today) - range.from) / DAY_SECS) * PX_PER_DAY;
    React.useEffect(() => {
        const el = scrollRef.current;
        if (el) el.scrollLeft = Math.max(0, todayX - Math.min(160, el.clientWidth * 0.25));
    }, [todayX]);

    const selectedItem = selected ? shown.find((it) => it.key === selected) : undefined;

    if (schedule.isPending) return <ReleaseLoading />;
    if (schedule.error) return <ReleaseError error={schedule.error} onRetry={schedule.refetch} />;

    const width = range.days * PX_PER_DAY;
    return (
        <div className="flex flex-col gap-3">
            <ScheduleControls kinds={kinds} onKindsChange={setKinds} stageOnly={stageOnly} onStageOnlyChange={setStageOnly} counts={counts}>
                <ToggleField id="timeline-show-past" label="Show past" checked={showPast} onChange={setShowPast} />
            </ScheduleControls>
            {selectedItem && <ScheduleDetail item={selectedItem} lookup={schedule.lookup} today={today} onClose={() => setSelected(null)} />}
            {shown.length === 0 ? (
                <ReleaseEmpty title="Nothing to draw" description="Every row is filtered out. Turn a kind back on, or Show past." />
            ) : (
                <Card className="gap-0 overflow-hidden p-0">
                    <div ref={scrollRef} className="overflow-x-auto">
                        <div className="relative w-max min-w-full pl-16 sm:pl-28">
                            <div className="sticky top-0 z-2 flex h-7 border-border border-b bg-card" style={{ width }}>
                                {months.map((m) => (
                                    <div key={m.label} className="shrink-0 border-border border-l px-1.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-7 tracking-[0.06em]" style={{ width: m.width }}>
                                        {m.label}
                                    </div>
                                ))}
                            </div>
                            <div className="pointer-events-none absolute inset-y-7 right-0 left-16 sm:left-28">
                                {months.map((m) => (
                                    <div key={m.label} className="absolute inset-y-0 border-border/60 border-l" style={{ left: m.x }} />
                                ))}
                                <div className="absolute inset-y-0 z-1 border-primary/70 border-l-2" style={{ left: todayX }} title={`Today, ${formatDate(dayStart(today))}`} />
                            </div>
                            {groups.map((g) => (
                                <TimelineGroup key={g.kind} kind={g.kind} items={g.items} lanes={g.lanes} laneCount={g.laneCount} rangeFrom={range.from} width={width} selected={selected} onSelect={setSelected} autoOn={autoOn} />
                            ))}
                        </div>
                    </div>
                </Card>
            )}
        </div>
    );
}

interface ITimelineGroupProps {
    kind: ScheduleKind;
    items: IScheduleItem[];
    lanes: Map<string, number>;
    laneCount: number;
    rangeFrom: number;
    width: number;
    selected: string | null;
    onSelect: (key: string | null) => void;
    autoOn: boolean;
}

function xOf(rangeFrom: number, secs: number): number {
    return ((dayOf(secs) - rangeFrom) / DAY_SECS) * PX_PER_DAY;
}

function spanOf(rangeFrom: number, from: number, to: number): { x: number; w: number } {
    const x = xOf(rangeFrom, from);
    return { x, w: Math.max(PX_PER_DAY, xOf(rangeFrom, to) + PX_PER_DAY - x) };
}

function TimelineGroup({ kind, items, lanes, laneCount, rangeFrom, width, selected, onSelect, autoOn }: ITimelineGroupProps): React.ReactElement {
    const height = Math.max(1, laneCount) * LANE_PX + 8;
    const style = KIND_STYLE[kind];
    return (
        <div className="relative flex border-border border-b last:border-b-0" style={{ height }}>
            <div className="sticky left-0 z-2 -ml-16 w-16 shrink-0 border-border border-r bg-card px-2 py-2 font-sans font-semibold text-[11px] text-foreground leading-tight sm:-ml-28 sm:w-28 sm:px-3 sm:text-[12px]">
                {KIND_LABEL[kind]}
                <span className="ml-1 font-medium font-mono text-[10.5px] text-muted-foreground max-sm:hidden sm:ml-1.5">{items.length}</span>
            </div>
            <div className="relative" style={{ width }}>
                {items.map((it) => {
                    const lane = lanes.get(it.key) ?? 0;
                    const top = 4 + lane * LANE_PX;
                    const drawn = drawnEnd(it);
                    const bar = spanOf(rangeFrom, it.start, drawn.end);
                    const name = itemName(it, autoOn);
                    const isSel = it.key === selected;
                    return (
                        <React.Fragment key={it.key}>
                            <button
                                type="button"
                                onClick={() => onSelect(isSel ? null : it.key)}
                                title={`${name}\n${formatDateRange(it.start, it.end)}`}
                                className={cn(
                                    "absolute cursor-pointer truncate rounded-sm px-1.5 text-left font-medium font-sans text-[11px] leading-[20px] outline-none transition-[box-shadow] hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring",
                                    it.estimated ? style.estimated : style.solid,
                                    drawn.cut && "rounded-r-none",
                                    isSel && "ring-2 ring-foreground/80",
                                )}
                                style={{ left: bar.x, width: bar.w, top, height: LANE_PX - 6 }}
                            >
                                {bar.w >= 40 ? name : ""}
                                {drawn.cut ? " \u2192" : ""}
                            </button>
                        </React.Fragment>
                    );
                })}
            </div>
        </div>
    );
}
