import { EyeOff } from "lucide-react";
import * as React from "react";
import { CartesianGrid, Legend, Line, LineChart, ReferenceDot, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Spinner } from "#/components/ui/spinner";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./CalcChart.messages";
import { formatLargeNumber } from "./constants";
import { curveMarkersFor, type ICurveMarkers } from "./curveMarkers";
import type { ICurvePoint, IInstance } from "./types";

interface ICalcChartProps {
    instances: IInstance[];
    rows: ICurvePoint[];
    /** Axis + metric labels for the current view. */
    xLabel: string;
    yLabel: string;
    /** Whether the X axis can show fractional ticks (false for integer counts). */
    allowDecimals: boolean;
    /** Tooltip header text for an X value (e.g. "DEF 1,000" / "3 targets"). */
    formatTooltipX: (x: number) => string;
    snapshotX: number;
    isLoading: boolean;
    onLegendClick: (uid: string) => void;
    /** Empty state shown when no instances exist yet. */
    emptyIcon: React.ReactNode;
    emptyTitle: string;
    emptyDescription: React.ReactNode;
    containerRef?: React.RefObject<HTMLDivElement | null>;
    /** Draw break-point and half-value markers on each visible curve, with their values under the chart. */
    showMarkers?: boolean;
}

export function CalcChart({ instances, rows, xLabel, yLabel, allowDecimals, formatTooltipX, snapshotX, isLoading, onLegendClick, emptyIcon, emptyTitle, emptyDescription, containerRef, showMarkers = false }: ICalcChartProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const locale = useLocale();
    const visible = React.useMemo(() => instances.filter((i) => i.visible), [instances]);
    const sameOpCounts = React.useMemo(() => {
        const counts = new Map<string, number>();
        for (const i of instances) counts.set(i.op.id, (counts.get(i.op.id) ?? 0) + 1);
        return counts;
    }, [instances]);
    // Only from a settled sweep: a marker read off a half-loaded curve would be wrong.
    const markers = React.useMemo(() => {
        if (!showMarkers || isLoading) return undefined;
        return new Map(visible.map((inst) => [inst.uid, curveMarkersFor(rows, inst.uid)]));
    }, [showMarkers, isLoading, visible, rows]);
    const lineName = (inst: IInstance, idx: number) => `${inst.op.name}${(sameOpCounts.get(inst.op.id) ?? 1) > 1 ? ` #${idx + 1}` : ""}`;

    if (instances.length === 0) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">{emptyIcon}</EmptyMedia>
                    <EmptyTitle>{emptyTitle}</EmptyTitle>
                    <EmptyDescription>{emptyDescription}</EmptyDescription>
                </EmptyHeader>
            </Empty>
        );
    }

    if (visible.length === 0) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <EyeOff />
                    </EmptyMedia>
                    <EmptyTitle>{t("calc.chart.allHidden.title")}</EmptyTitle>
                    <EmptyDescription>{t("calc.chart.allHidden.desc")}</EmptyDescription>
                </EmptyHeader>
            </Empty>
        );
    }

    return (
        <>
            <div ref={containerRef} className="relative h-85 w-full sm:h-100 xl:h-110" role="img" aria-label={t("calc.chart.ariaLabel", { yLabel, xLabel, count: visible.length })}>
                {isLoading && (
                    <span aria-live="polite" className="absolute top-2 left-1/2 z-10 inline-flex -translate-x-1/2 items-center gap-1.5 rounded-md bg-background/72 px-2 py-1 text-[11px] text-muted-foreground shadow-sm backdrop-blur-sm">
                        <Spinner className="size-3" />
                        {t("calc.chart.calculating")}
                    </span>
                )}
                <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={rows} margin={{ top: 12, right: 24, left: 12, bottom: 28 }}>
                        <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" />
                        <XAxis
                            dataKey="x"
                            type="number"
                            domain={["dataMin", "dataMax"]}
                            allowDecimals={allowDecimals}
                            stroke="var(--muted-foreground)"
                            tick={{ fill: "var(--muted-foreground)", fontSize: 11 }}
                            tickFormatter={(v) => formatLargeNumber(v as number, locale)}
                            label={{ value: xLabel, position: "insideBottom", offset: -16, fill: "var(--muted-foreground)", fontSize: 12 }}
                        />
                        <YAxis stroke="var(--muted-foreground)" tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} tickFormatter={(v) => formatLargeNumber(v as number, locale)} width={64} label={{ value: yLabel, angle: -90, position: "insideLeft", fill: "var(--muted-foreground)", fontSize: 11, dy: 50 }} />
                        <Tooltip content={<CalcTooltip instances={instances} yLabel={yLabel} formatTooltipX={formatTooltipX} locale={locale} />} cursor={{ stroke: "var(--border)", strokeWidth: 1 }} />
                        {snapshotX >= (rows[0]?.x ?? 0) && snapshotX <= (rows[rows.length - 1]?.x ?? 0) && (
                            <ReferenceLine x={snapshotX} stroke="var(--muted-foreground)" strokeDasharray="2 4" strokeOpacity={0.5} label={{ value: t("calc.chart.snapshot"), position: "insideTopLeft", fill: "var(--muted-foreground)", fontSize: 9.5, dy: 4, dx: 4 }} />
                        )}
                        <Legend
                            verticalAlign="top"
                            align="right"
                            iconType="circle"
                            wrapperStyle={{ paddingBottom: 8 }}
                            formatter={(value: string) => <span className="text-[12px] text-foreground">{value}</span>}
                            onClick={(payload) => {
                                const id = (payload as { dataKey?: string }).dataKey;
                                if (typeof id === "string") onLegendClick(id);
                            }}
                        />
                        {visible.map((inst, idx) => (
                            <Line key={inst.uid} type="monotone" dataKey={inst.uid} name={lineName(inst, idx)} stroke={inst.color} strokeWidth={2} dot={false} connectNulls isAnimationActive={false} />
                        ))}
                        {/* After the lines, so a marker sits on top of every curve. */}
                        {markers &&
                            visible.flatMap((inst, idx) => {
                                const m = markers.get(inst.uid);
                                if (!m) return [];
                                const name = lineName(inst, idx);
                                const dots: React.ReactElement[] = [];
                                if (m.breakPoint) {
                                    const title = `${name}: ${t("calc.chart.marker.break", { x: formatTooltipX(m.breakPoint.x) })}`;
                                    dots.push(<ReferenceDot key={`${inst.uid}-break`} x={m.breakPoint.x} y={m.breakPoint.y} shape={(p: IMarkerShapeProps) => <BreakMarker {...p} color={inst.color} title={title} />} />);
                                }
                                if (m.halfPoint) {
                                    const title = `${name}: ${t("calc.chart.marker.half", { x: formatTooltipX(m.halfPoint.x) })}`;
                                    dots.push(<ReferenceDot key={`${inst.uid}-half`} x={m.halfPoint.x} y={m.halfPoint.y} shape={(p: IMarkerShapeProps) => <HalfMarker {...p} color={inst.color} title={title} />} />);
                                }
                                return dots;
                            })}
                    </LineChart>
                </ResponsiveContainer>
            </div>
            {showMarkers && <MarkerDetail visible={visible} markers={markers} lineName={lineName} formatX={formatTooltipX} />}
        </>
    );
}

interface IMarkerShapeProps {
    cx?: number;
    cy?: number;
}

/** Diamond: the curve has hit its floor and stays flat from here. */
function BreakMarker({ cx, cy, color, title }: IMarkerShapeProps & { color: string; title: string }): React.ReactElement | null {
    if (cx == null || cy == null) return null;
    return (
        <path d={`M${cx},${cy - 5}L${cx + 5},${cy}L${cx},${cy + 5}L${cx - 5},${cy}Z`} fill={color} stroke="var(--background)" strokeWidth={1.5}>
            <title>{title}</title>
        </path>
    );
}

/** Ring: the value is half of what it was at the start of the sweep. */
function HalfMarker({ cx, cy, color, title }: IMarkerShapeProps & { color: string; title: string }): React.ReactElement | null {
    if (cx == null || cy == null) return null;
    return (
        <circle cx={cx} cy={cy} r={4} fill="var(--background)" stroke={color} strokeWidth={2}>
            <title>{title}</title>
        </circle>
    );
}

/** The marker values in words, one entry per visible curve: the dots alone carry no numbers. */
function MarkerDetail({ visible, markers, lineName, formatX }: { visible: IInstance[]; markers: ReadonlyMap<string, ICurveMarkers> | undefined; lineName: (inst: IInstance, idx: number) => string; formatX: (x: number) => string }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="mt-1 space-y-1.5 text-[11px] leading-snug">
            <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-muted-foreground">
                <span className="inline-flex items-center gap-1.5">
                    <svg aria-hidden="true" viewBox="0 0 10 10" className="size-2.5">
                        <path d="M5,0L10,5L5,10L0,5Z" fill="currentColor" />
                    </svg>
                    {t("calc.chart.marker.breakKey")}
                </span>
                <span className="inline-flex items-center gap-1.5">
                    <svg aria-hidden="true" viewBox="0 0 10 10" className="size-2.5">
                        <circle cx="5" cy="5" r="3.5" fill="none" stroke="currentColor" strokeWidth="2" />
                    </svg>
                    {t("calc.chart.marker.halfKey")}
                </span>
            </p>
            <ul className="flex flex-wrap gap-x-4 gap-y-1">
                {visible.map((inst, idx) => {
                    const m = markers?.get(inst.uid);
                    const value = (p: ICurveMarkers["breakPoint"] | undefined) => (!markers ? "…" : p ? formatX(p.x) : t("calc.chart.marker.notInRange"));
                    return (
                        <li key={inst.uid} className="inline-flex items-center gap-1.5">
                            <span aria-hidden="true" className="inline-block size-2 rounded-full" style={{ backgroundColor: inst.color }} />
                            <span className="font-medium text-foreground">{lineName(inst, idx)}</span>
                            <span className="font-mono text-muted-foreground tabular-nums">
                                {t("calc.chart.marker.break", { x: value(m?.breakPoint) })}
                                {" · "}
                                {t("calc.chart.marker.half", { x: value(m?.halfPoint) })}
                            </span>
                        </li>
                    );
                })}
            </ul>
        </div>
    );
}

interface ITooltipPayload {
    active?: boolean;
    label?: number | string;
    payload?: { dataKey?: string; value?: number; color?: string }[];
}

function CalcTooltip({ instances, yLabel, formatTooltipX, locale, ...rest }: ITooltipPayload & { instances: IInstance[]; yLabel: string; formatTooltipX: (x: number) => string; locale: string }): React.ReactElement | null {
    if (!rest.active || !rest.payload || rest.payload.length === 0) return null;

    return (
        <div className="rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-md">
            <div className="mb-1 font-mono text-[10.5px] text-muted-foreground">{formatTooltipX(Number(rest.label))}</div>
            <div className="space-y-1">
                {rest.payload.map((p) => {
                    const inst = instances.find((i) => i.uid === p.dataKey);
                    if (!inst) return null;
                    return (
                        <div key={p.dataKey} className="flex items-center justify-between gap-3 text-[12px]">
                            <span className="flex items-center gap-1.5">
                                <span aria-hidden="true" className="inline-block size-2 rounded-full" style={{ backgroundColor: inst.color }} />
                                <span className="font-medium">{inst.op.name}</span>
                            </span>
                            <span className="font-mono tabular-nums">{p.value == null ? "-" : formatLargeNumber(p.value, locale)}</span>
                        </div>
                    );
                })}
            </div>
            <div className="mt-1 border-border border-t pt-1 text-[10px] text-muted-foreground">{yLabel}</div>
        </div>
    );
}
