import type * as React from "react";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { cn } from "#/lib/utils";
import { STEP_OPTIONS, X_AXIS_LABELS, Y_METRIC_LABELS } from "../constants";
import type { ISweepRange, XAxisKind, YMetric } from "../types";

interface IAxisControlsProps {
    xAxis: XAxisKind;
    yMetric: YMetric;
    sweep: ISweepRange;
    /** Bumped once after `localStorage` hydration so uncontrolled NumberFields remount with the persisted values. */
    hydrationToken: number;
    onChangeAxis: (axis: XAxisKind) => void;
    onChangeMetric: (metric: YMetric) => void;
    onChangeSweep: (patch: Partial<ISweepRange>) => void;
    /** Optional trailing element rendered in the same baseline as the sweep-range inputs (e.g. an export button). */
    rangeAction?: React.ReactNode;
}

export function AxisControls({ xAxis, yMetric, sweep, hydrationToken, onChangeAxis, onChangeMetric, onChangeSweep, rangeAction }: IAxisControlsProps): React.ReactElement {
    const stepValue = (sweep.max - sweep.min) / Math.max(1, sweep.steps - 1);
    const sweepStep = xAxis === "defense" ? 100 : 5;
    const sweepMaxBound = xAxis === "defense" ? 5000 : 100;
    return (
        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end">
            <div className="flex flex-wrap items-end gap-3">
                <div className="space-y-1.5">
                    <Label className="block font-medium text-[12px] text-muted-foreground leading-none">X axis</Label>
                    <Tabs value={xAxis} onValueChange={(v) => onChangeAxis(v as XAxisKind)}>
                        <TabsList>
                            <TabsTrigger value="defense">{X_AXIS_LABELS.defense}</TabsTrigger>
                            <TabsTrigger value="res">{X_AXIS_LABELS.res}</TabsTrigger>
                        </TabsList>
                    </Tabs>
                </div>
                <div className="min-w-32 flex-1 space-y-1.5 sm:flex-none">
                    <Label className="block font-medium text-[12px] text-muted-foreground leading-none">Y metric</Label>
                    <Select value={yMetric} onValueChange={(v) => onChangeMetric(v as YMetric)}>
                        <SelectTrigger size="sm">
                            <SelectValue placeholder="Metric">{(v: string) => Y_METRIC_LABELS[v as YMetric]}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {(Object.keys(Y_METRIC_LABELS) as YMetric[]).map((k) => (
                                <SelectItem key={k} value={k}>
                                    {Y_METRIC_LABELS[k]}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <div className="flex w-full flex-col items-stretch gap-1 sm:ml-auto sm:w-auto">
                <div className={cn("grid items-end gap-2 sm:flex sm:items-end sm:gap-2", rangeAction ? "grid-cols-[1fr_1fr_auto_auto]" : "grid-cols-[1fr_1fr_auto]")}>
                    <NumberField key={`${xAxis}-min-${hydrationToken}`} defaultValue={sweep.min} min={0} max={sweep.max - 1} step={sweepStep} onValueChange={(v) => onChangeSweep({ min: v ?? 0 })} size="sm" className="sm:w-24">
                        <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">From</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    <NumberField key={`${xAxis}-max-${hydrationToken}`} defaultValue={sweep.max} min={sweep.min + 1} max={sweepMaxBound} step={sweepStep} onValueChange={(v) => onChangeSweep({ max: v ?? 0 })} size="sm" className="sm:w-24">
                        <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">To</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    <div className="space-y-1">
                        <Label className="block text-[10.5px] text-muted-foreground/80 leading-none">Points</Label>
                        <Select value={String(sweep.steps)} onValueChange={(v) => onChangeSweep({ steps: Number(v) })}>
                            <SelectTrigger size="sm" className="min-w-16">
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
                    {rangeAction && (
                        <div className="space-y-1">
                            {/* invisible spacer so the button shares its column height with the labelled inputs above */}
                            <span aria-hidden="true" className="block text-[10.5px] leading-none">
                                &nbsp;
                            </span>
                            {rangeAction}
                        </div>
                    )}
                </div>
                <p className="font-mono text-[10.5px] text-muted-foreground/80 sm:text-right">
                    {sweep.steps} points · ~{Math.round(stepValue)}
                    {xAxis === "res" ? "%" : ""} apart
                </p>
            </div>
        </div>
    );
}
