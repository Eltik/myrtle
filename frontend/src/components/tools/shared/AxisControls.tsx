import type * as React from "react";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./AxisControls.messages";
import { STEP_OPTIONS } from "./constants";
import type { IAxisInput, ISweepRange } from "./types";

export interface IOption {
    value: string;
    label: string;
    /** Tab label for narrow screens; falls back to `label`. */
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
    axisInput: IAxisInput;
    /** From the caller's `buildSweepPoints`. */
    pointCount: number;
    onChangeSweep: (patch: Partial<ISweepRange>) => void;
    /** Bumped after `localStorage` hydration so uncontrolled NumberFields remount with persisted values. */
    hydrationToken: number;
    metricHint?: string;
    /** Sits beside the sweep-range inputs, e.g. an export button. */
    rangeAction?: React.ReactNode;
}

export function AxisControls({ axes, xAxis, onChangeAxis, metrics, yMetric, onChangeMetric, sweep, axisInput, pointCount, onChangeSweep, hydrationToken, metricHint, rangeAction }: IAxisControlsProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const stepValue = (sweep.max - sweep.min) / Math.max(1, pointCount - 1);
    const showPoints = !axisInput.integer;
    const metricLabel = metrics.find((m) => m.value === yMetric)?.label ?? yMetric;
    return (
        // Every field is the same stack, label then control 6px below, and the rows align at
        // the TOP: aligned at the bottom, controls of different heights (a 28px toggle beside a
        // taller select) and the point summary under the range put the labels 8px apart.
        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-start">
            <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-start">
                <div className="flex w-full flex-col gap-1.5 sm:w-auto">
                    <Label className="block text-subtle-foreground text-xs leading-none">{t("calc.axis.xAxis")}</Label>
                    <ToggleGroup aria-label={t("calc.axis.xAxis")} variant="outline" size="sm" value={[xAxis]} onValueChange={(v) => v[0] && onChangeAxis(v[0])} className="grid w-full auto-cols-fr grid-flow-col sm:w-fit">
                        {axes.map((a) => (
                            <ToggleGroupItem key={a.value} value={a.value} className="min-w-0 text-[12px]">
                                {a.short ?? a.label}
                            </ToggleGroupItem>
                        ))}
                    </ToggleGroup>
                </div>
                <div className="flex w-full flex-col gap-1.5 sm:w-auto sm:flex-none">
                    <Label className="block text-subtle-foreground text-xs leading-none">{t("calc.axis.yMetric")}</Label>
                    <Select value={yMetric} onValueChange={(v) => v != null && onChangeMetric(v)}>
                        <SelectTrigger size="sm">
                            <SelectValue placeholder={t("calc.axis.metricPlaceholder")}>{() => metricLabel}</SelectValue>
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
                <div className="grid grid-cols-2 items-start gap-2 sm:flex sm:items-start sm:gap-2">
                    <NumberField key={`${xAxis}-min-${hydrationToken}`} defaultValue={sweep.min} min={0} max={sweep.max - 1} step={axisInput.step} onValueChange={(v) => onChangeSweep({ min: v ?? 0 })} size="sm" className="w-full gap-1.5 sm:w-24">
                        <Label className="block text-subtle-foreground text-xs leading-none">{t("calc.axis.from")}</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    <NumberField key={`${xAxis}-max-${hydrationToken}`} defaultValue={sweep.max} min={sweep.min + 1} max={axisInput.maxBound} step={axisInput.step} onValueChange={(v) => onChangeSweep({ max: v ?? 0 })} size="sm" className="w-full gap-1.5 sm:w-24">
                        <Label className="block text-subtle-foreground text-xs leading-none">{t("calc.axis.to")}</Label>
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                    {showPoints && (
                        <div className="flex flex-col gap-1.5">
                            <Label className="block text-subtle-foreground text-xs leading-none">{t("calc.axis.points")}</Label>
                            <Select value={String(sweep.steps)} onValueChange={(v) => onChangeSweep({ steps: Number(v) })}>
                                <SelectTrigger size="sm" className="w-full sm:min-w-16">
                                    <SelectValue placeholder={t("calc.axis.pointsPlaceholder")}>{(v: string) => v}</SelectValue>
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
                        <div className="flex flex-col gap-1.5">
                            <span aria-hidden="true" className="hidden text-xs leading-none sm:block">
                                &nbsp;
                            </span>
                            {rangeAction}
                        </div>
                    )}
                </div>
                <p className="font-mono text-[10.5px] text-subtle-foreground sm:text-right">{t("calc.axis.pointSummary", { count: pointCount, mode: axisInput.integer ? "whole" : "spacing", step: Math.round(stepValue), unit: axisInput.unit })}</p>
            </div>
            {metricHint && (
                <p className="order-last w-full text-[11px] text-muted-foreground leading-snug sm:basis-full">
                    <span className="font-medium text-foreground">{t("calc.axis.metricHint", { metric: metricLabel })}</span> {metricHint}
                </p>
            )}
        </div>
    );
}
