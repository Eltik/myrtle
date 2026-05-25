import type * as React from "react";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { STEP_OPTIONS } from "./constants";
import type { IAxisInput, ISweepRange } from "./types";

export interface IOption {
    value: string;
    label: string;
    /** Compact label for the axis tab; falls back to `label`. Keeps long names from overflowing on narrow screens. */
    short?: string;
}

interface IAxisControlsProps {
    axes: IOption[];
    xAxis: string;
    onChangeAxis: (axis: string) => void;
    metrics: IOption[];
    yMetric: string;
    onChangeMetric: (metric: string) => void;
    sweep: ISweepRange;
    /** Granularity/bounds for the currently-selected axis. */
    axisInput: IAxisInput;
    /** Number of plotted points for the current sweep (caller computes via buildSweepPoints). */
    pointCount: number;
    onChangeSweep: (patch: Partial<ISweepRange>) => void;
    /** Bumped after `localStorage` hydration so uncontrolled NumberFields remount with persisted values. */
    hydrationToken: number;
    /** Optional caption describing the selected metric. */
    metricHint?: string;
    /** Optional trailing element aligned with the sweep-range inputs (e.g. an export button). */
    rangeAction?: React.ReactNode;
}

export function AxisControls({ axes, xAxis, onChangeAxis, metrics, yMetric, onChangeMetric, sweep, axisInput, pointCount, onChangeSweep, hydrationToken, metricHint, rangeAction }: IAxisControlsProps): React.ReactElement {
    const stepValue = (sweep.max - sweep.min) / Math.max(1, pointCount - 1);
    const showPoints = !axisInput.integer;
    const metricLabel = metrics.find((m) => m.value === yMetric)?.label ?? yMetric;
    return (
        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end">
            <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end">
                <div className="w-full space-y-1.5 sm:w-auto">
                    <Label className="block font-medium text-[12px] text-muted-foreground leading-none">X axis</Label>
                    <Tabs value={xAxis} onValueChange={onChangeAxis}>
                        <TabsList className="w-full sm:w-fit">
                            {axes.map((a) => (
                                <TabsTrigger key={a.value} value={a.value} className="min-w-0">
                                    {a.short ?? a.label}
                                </TabsTrigger>
                            ))}
                        </TabsList>
                    </Tabs>
                </div>
                <div className="w-full space-y-1.5 sm:w-auto sm:flex-none">
                    <Label className="block font-medium text-[12px] text-muted-foreground leading-none">Y metric</Label>
                    <Select value={yMetric} onValueChange={(v) => v != null && onChangeMetric(v)}>
                        <SelectTrigger size="sm">
                            <SelectValue placeholder="Metric">{() => metricLabel}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {metrics.map((m) => (
                                <SelectItem key={m.value} value={m.value}>
                                    {m.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <div className="flex w-full flex-col items-stretch gap-1 sm:ml-auto sm:w-auto">
                <div className="grid grid-cols-2 items-end gap-2 sm:flex sm:items-end sm:gap-2">
                    <NumberField key={`${xAxis}-min-${hydrationToken}`} defaultValue={sweep.min} min={0} max={sweep.max - 1} step={axisInput.step} onValueChange={(v) => onChangeSweep({ min: v ?? 0 })} size="sm" className="w-full sm:w-24">
                        <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">From</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    <NumberField key={`${xAxis}-max-${hydrationToken}`} defaultValue={sweep.max} min={sweep.min + 1} max={axisInput.maxBound} step={axisInput.step} onValueChange={(v) => onChangeSweep({ max: v ?? 0 })} size="sm" className="w-full sm:w-24">
                        <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">To</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    {showPoints && (
                        <div className="space-y-1">
                            <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">Points</Label>
                            <Select value={String(sweep.steps)} onValueChange={(v) => onChangeSweep({ steps: Number(v) })}>
                                <SelectTrigger size="sm" className="w-full sm:min-w-16">
                                    <SelectValue placeholder="Points">{(v: string) => v}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {STEP_OPTIONS.map((s) => (
                                        <SelectItem key={s} value={String(s)}>
                                            {s}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    )}
                    {rangeAction && (
                        <div className="flex items-end space-y-1">
                            <span aria-hidden="true" className="hidden text-[10.5px] leading-none sm:block">
                                &nbsp;
                            </span>
                            {rangeAction}
                        </div>
                    )}
                </div>
                <p className="font-mono text-[10.5px] text-muted-foreground/80 sm:text-right">
                    {pointCount} points
                    {axisInput.integer ? " · whole numbers" : ` · ~${Math.round(stepValue)}${axisInput.unit} apart`}
                </p>
            </div>
            {metricHint && (
                <p className="order-last w-full text-[11px] text-muted-foreground leading-snug sm:basis-full">
                    <span className="font-medium text-foreground">{metricLabel}:</span> {metricHint}
                </p>
            )}
        </div>
    );
}
