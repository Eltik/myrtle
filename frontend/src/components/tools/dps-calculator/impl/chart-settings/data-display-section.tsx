"use client";

import { Plus, X } from "lucide-react";
import { useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";
import type { AxisScale, NumberFormat } from "~/types/frontend/impl/tools/dps-chart-settings";

const NUMBER_FORMAT_OPTIONS: { value: NumberFormat; label: string }[] = [
    { value: "compact", label: "Compact (1.5K)" },
    { value: "standard", label: "Standard (1,500)" },
    { value: "scientific", label: "Scientific (1.5e3)" },
];

const AXIS_SCALE_OPTIONS: { value: AxisScale; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "linear", label: "Linear" },
    { value: "log", label: "Logarithmic" },
];

export function DataDisplaySection() {
    const { settings, updateSettings } = useDpsChartSettings();
    const { dataDisplay } = settings;
    const [newRefLine, setNewRefLine] = useState("");

    const handleAddReferenceLine = () => {
        const value = Number.parseInt(newRefLine, 10);
        if (!Number.isNaN(value) && value > 0 && !dataDisplay.referenceLineValues.includes(value)) {
            updateSettings({
                dataDisplay: {
                    referenceLineValues: [...dataDisplay.referenceLineValues, value].sort((a, b) => a - b),
                },
            });
            setNewRefLine("");
        }
    };

    const handleRemoveReferenceLine = (value: number) => {
        updateSettings({
            dataDisplay: {
                referenceLineValues: dataDisplay.referenceLineValues.filter((v) => v !== value),
            },
        });
    };

    return (
        <div className="space-y-4">
            {/* Show Tooltip */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Show Tooltip</Label>
                <Switch checked={dataDisplay.showTooltip} onCheckedChange={(checked) => updateSettings({ dataDisplay: { showTooltip: checked } })} />
            </div>

            {/* Show Axis Labels */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Show Axis Labels</Label>
                <Switch checked={dataDisplay.showAxisLabels} onCheckedChange={(checked) => updateSettings({ dataDisplay: { showAxisLabels: checked } })} />
            </div>

            {/* Show Data Labels */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Show Values on Chart</Label>
                <Switch checked={dataDisplay.showDataLabels} onCheckedChange={(checked) => updateSettings({ dataDisplay: { showDataLabels: checked } })} />
            </div>

            {/* Data Label Interval - conditional */}
            {dataDisplay.showDataLabels && (
                <div className="space-y-2">
                    <div className="flex justify-between">
                        <Label className="text-xs">Label Interval</Label>
                        <span className="text-muted-foreground text-xs">Every {dataDisplay.dataLabelInterval} points</span>
                    </div>
                    <Slider max={20} min={1} onValueChange={([value]) => updateSettings({ dataDisplay: { dataLabelInterval: value } })} step={1} value={[dataDisplay.dataLabelInterval]} />
                </div>
            )}

            {/* Number Format */}
            <div className="space-y-2">
                <Label className="text-xs">Number Format</Label>
                <Select onValueChange={(value: NumberFormat) => updateSettings({ dataDisplay: { numberFormat: value } })} value={dataDisplay.numberFormat}>
                    <SelectTrigger className="h-8">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {NUMBER_FORMAT_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {opt.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* Decimal Places */}
            <div className="space-y-2">
                <div className="flex justify-between">
                    <Label className="text-xs">Decimal Places</Label>
                    <span className="text-muted-foreground text-xs">{dataDisplay.decimalPlaces}</span>
                </div>
                <Slider max={3} min={0} onValueChange={([value]) => updateSettings({ dataDisplay: { decimalPlaces: value } })} step={1} value={[dataDisplay.decimalPlaces]} />
            </div>

            {/* Y-Axis Scale */}
            <div className="space-y-2">
                <Label className="text-xs">Y-Axis Scale</Label>
                <Select onValueChange={(value: AxisScale) => updateSettings({ dataDisplay: { yAxisScale: value } })} value={dataDisplay.yAxisScale}>
                    <SelectTrigger className="h-8">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {AXIS_SCALE_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {opt.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* Show Reference Lines */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Reference Lines</Label>
                <Switch checked={dataDisplay.showReferenceLines} onCheckedChange={(checked) => updateSettings({ dataDisplay: { showReferenceLines: checked } })} />
            </div>

            {/* Reference Line Values - conditional */}
            {dataDisplay.showReferenceLines && (
                <div className="space-y-2">
                    <Label className="text-xs">DPS Thresholds</Label>
                    <div className="flex gap-2">
                        <Input
                            className="h-8"
                            onChange={(e) => setNewRefLine(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === "Enter") {
                                    e.preventDefault();
                                    handleAddReferenceLine();
                                }
                            }}
                            placeholder="Enter DPS value"
                            type="number"
                            value={newRefLine}
                        />
                        <Button className="h-8 w-8 shrink-0" onClick={handleAddReferenceLine} size="icon" variant="outline">
                            <Plus className="h-4 w-4" />
                        </Button>
                    </div>
                    {dataDisplay.referenceLineValues.length > 0 && (
                        <div className="flex flex-wrap gap-1">
                            {dataDisplay.referenceLineValues.map((value) => (
                                <span className="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-xs" key={value}>
                                    {value.toLocaleString()}
                                    <button className="hover:text-destructive" onClick={() => handleRemoveReferenceLine(value)} type="button">
                                        <X className="h-3 w-3" />
                                    </button>
                                </span>
                            ))}
                        </div>
                    )}
                    <p className="text-muted-foreground text-xs">Add horizontal lines at specific DPS values for easy comparison.</p>
                </div>
            )}
        </div>
    );
}
