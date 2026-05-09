import { Calculator, EyeOff } from "lucide-react";
import * as React from "react";
import { CartesianGrid, Legend, Line, LineChart, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Spinner } from "#/components/ui/spinner";
import { formatLargeNumber, X_AXIS_LABELS, Y_METRIC_LABELS } from "../constants";
import type { ICurvePoint, IDpsInstance, XAxisKind, YMetric } from "../types";

interface DpsChartProps {
    instances: IDpsInstance[];
    rows: ICurvePoint[];
    xAxis: XAxisKind;
    yMetric: YMetric;
    snapshotX: number;
    isLoading: boolean;
    onLegendClick: (uid: string) => void;
}

export function DpsChart({ instances, rows, xAxis, yMetric, snapshotX, isLoading, onLegendClick }: DpsChartProps): React.ReactElement {
    const visible = React.useMemo(() => instances.filter((i) => i.visible), [instances]);
    const sameOpCounts = React.useMemo(() => {
        const counts = new Map<string, number>();
        for (const i of instances) counts.set(i.op.id, (counts.get(i.op.id) ?? 0) + 1);
        return counts;
    }, [instances]);

    if (instances.length === 0) {
        return (
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <Calculator />
                    </EmptyMedia>
                    <EmptyTitle>No operators yet</EmptyTitle>
                    <EmptyDescription>
                        Pick any operator from the picker to plot a DPS curve. The chart will show how DPS scales as you sweep across <span className="font-medium text-foreground">{X_AXIS_LABELS[xAxis]}</span>.
                    </EmptyDescription>
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
                    <EmptyTitle>All curves hidden</EmptyTitle>
                    <EmptyDescription>Toggle visibility on an operator card (or use the chart legend) to plot it again.</EmptyDescription>
                </EmptyHeader>
            </Empty>
        );
    }

    return (
        <div className="relative h-85 w-full sm:h-100 xl:h-110" role="img" aria-label={`${Y_METRIC_LABELS[yMetric]} by ${X_AXIS_LABELS[xAxis]} for ${visible.length} operator${visible.length === 1 ? "" : "s"}`}>
            {isLoading && (
                <div aria-live="polite" className="absolute right-2 top-2 z-10 inline-flex items-center gap-1.5 rounded-md bg-background/72 px-2 py-1 text-[11px] text-muted-foreground shadow-sm backdrop-blur-sm">
                    <Spinner className="size-3" />
                    Calculating
                </div>
            )}
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={rows} margin={{ top: 12, right: 24, left: 12, bottom: 28 }}>
                    <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" />
                    <XAxis
                        dataKey="x"
                        type="number"
                        domain={["dataMin", "dataMax"]}
                        stroke="var(--muted-foreground)"
                        tick={{ fill: "var(--muted-foreground)", fontSize: 11 }}
                        tickFormatter={(v) => formatLargeNumber(v as number)}
                        label={{ value: X_AXIS_LABELS[xAxis], position: "insideBottom", offset: -16, fill: "var(--muted-foreground)", fontSize: 12 }}
                    />
                    <YAxis stroke="var(--muted-foreground)" tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} tickFormatter={(v) => formatLargeNumber(v as number)} width={64} label={{ value: Y_METRIC_LABELS[yMetric], angle: -90, position: "insideLeft", fill: "var(--muted-foreground)", fontSize: 11, dy: 50 }} />
                    <Tooltip content={<DpsTooltip instances={instances} xAxis={xAxis} yMetric={yMetric} />} cursor={{ stroke: "var(--border)", strokeWidth: 1 }} />
                    {snapshotX >= (rows[0]?.x ?? 0) && snapshotX <= (rows[rows.length - 1]?.x ?? 0) && (
                        <ReferenceLine x={snapshotX} stroke="var(--muted-foreground)" strokeDasharray="2 4" strokeOpacity={0.5} label={{ value: "snapshot", position: "insideTopLeft", fill: "var(--muted-foreground)", fontSize: 9.5, dy: 4, dx: 4 }} />
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
                        <Line key={inst.uid} type="monotone" dataKey={inst.uid} name={`${inst.op.name}${(sameOpCounts.get(inst.op.id) ?? 1) > 1 ? ` #${idx + 1}` : ""}`} stroke={inst.color} strokeWidth={2} dot={false} connectNulls isAnimationActive={false} />
                    ))}
                </LineChart>
            </ResponsiveContainer>
        </div>
    );
}

interface TooltipPayload {
    active?: boolean;
    label?: number | string;
    payload?: { dataKey?: string; value?: number; color?: string }[];
}

function DpsTooltip({ instances, xAxis, yMetric, ...rest }: TooltipPayload & { instances: IDpsInstance[]; xAxis: XAxisKind; yMetric: YMetric }): React.ReactElement | null {
    if (!rest.active || !rest.payload || rest.payload.length === 0) return null;

    const xValue = rest.label;
    const xLabel = xAxis === "defense" ? `DEF ${formatLargeNumber(Number(xValue))}` : `RES ${Number(xValue).toFixed(1)}%`;

    return (
        <div className="rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-md">
            <div className="mb-1 font-mono text-[10.5px] text-muted-foreground">{xLabel}</div>
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
                            <span className="font-mono tabular-nums">{p.value == null ? "—" : formatLargeNumber(p.value)}</span>
                        </div>
                    );
                })}
            </div>
            <div className="mt-1 border-t border-border pt-1 text-[10px] text-muted-foreground">{Y_METRIC_LABELS[yMetric]}</div>
        </div>
    );
}
