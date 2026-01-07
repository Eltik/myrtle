"use client";

import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";
import type { ChartType, GridStyle, LegendPosition, LineType } from "./types";

const CHART_TYPE_OPTIONS: { value: ChartType; label: string }[] = [
    { value: "line", label: "Line Chart" },
    { value: "area", label: "Area Chart" },
    { value: "bar", label: "Bar Chart" },
];

const LINE_TYPE_OPTIONS: { value: LineType; label: string }[] = [
    { value: "monotone", label: "Smooth" },
    { value: "linear", label: "Linear" },
    { value: "natural", label: "Natural" },
    { value: "basis", label: "Basis" },
    { value: "step", label: "Step" },
    { value: "stepBefore", label: "Step Before" },
    { value: "stepAfter", label: "Step After" },
];

const LEGEND_POSITION_OPTIONS: { value: LegendPosition; label: string }[] = [
    { value: "bottom", label: "Bottom" },
    { value: "top", label: "Top" },
    { value: "left", label: "Left" },
    { value: "right", label: "Right" },
    { value: "none", label: "Hidden" },
];

const GRID_STYLE_OPTIONS: { value: GridStyle; label: string }[] = [
    { value: "dashed", label: "Dashed" },
    { value: "dotted", label: "Dotted" },
    { value: "solid", label: "Solid" },
];

export function VisualSettingsSection() {
    const { settings, updateSettings } = useDpsChartSettings();
    const { visual } = settings;

    const showLineOptions = visual.chartType === "line" || visual.chartType === "area";

    return (
        <div className="space-y-4">
            {/* Chart Type */}
            <div className="space-y-2">
                <Label className="text-xs">Chart Type</Label>
                <Select onValueChange={(value: ChartType) => updateSettings({ visual: { chartType: value } })} value={visual.chartType}>
                    <SelectTrigger className="h-8">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {CHART_TYPE_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {opt.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* Line Type - only for line/area charts */}
            {showLineOptions && (
                <div className="space-y-2">
                    <Label className="text-xs">Line Type</Label>
                    <Select onValueChange={(value: LineType) => updateSettings({ visual: { lineType: value } })} value={visual.lineType}>
                        <SelectTrigger className="h-8">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {LINE_TYPE_OPTIONS.map((opt) => (
                                <SelectItem key={opt.value} value={opt.value}>
                                    {opt.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            )}

            {/* Stroke Width - only for line/area charts */}
            {showLineOptions && (
                <div className="space-y-2">
                    <div className="flex justify-between">
                        <Label className="text-xs">Line Width</Label>
                        <span className="text-muted-foreground text-xs">{visual.strokeWidth}px</span>
                    </div>
                    <Slider max={5} min={1} onValueChange={([value]) => updateSettings({ visual: { strokeWidth: value } })} step={0.5} value={[visual.strokeWidth]} />
                </div>
            )}

            {/* Show Dots - only for line/area charts */}
            {showLineOptions && (
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Show Data Points</Label>
                    <Switch checked={visual.showDots} onCheckedChange={(checked) => updateSettings({ visual: { showDots: checked } })} />
                </div>
            )}

            {/* Dot Size - conditional */}
            {showLineOptions && visual.showDots && (
                <div className="space-y-2">
                    <div className="flex justify-between">
                        <Label className="text-xs">Point Size</Label>
                        <span className="text-muted-foreground text-xs">{visual.dotSize}px</span>
                    </div>
                    <Slider max={8} min={2} onValueChange={([value]) => updateSettings({ visual: { dotSize: value } })} step={1} value={[visual.dotSize]} />
                </div>
            )}

            {/* Area Fill - only for line chart type */}
            {visual.chartType === "line" && (
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Area Fill</Label>
                    <Switch checked={visual.showAreaFill} onCheckedChange={(checked) => updateSettings({ visual: { showAreaFill: checked } })} />
                </div>
            )}

            {/* Area Fill Opacity - conditional */}
            {(visual.chartType === "area" || (visual.chartType === "line" && visual.showAreaFill)) && (
                <div className="space-y-2">
                    <div className="flex justify-between">
                        <Label className="text-xs">Fill Opacity</Label>
                        <span className="text-muted-foreground text-xs">{Math.round(visual.areaFillOpacity * 100)}%</span>
                    </div>
                    <Slider max={0.8} min={0.05} onValueChange={([value]) => updateSettings({ visual: { areaFillOpacity: value } })} step={0.05} value={[visual.areaFillOpacity]} />
                </div>
            )}

            {/* Show Grid */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Show Grid</Label>
                <Switch checked={visual.showGrid} onCheckedChange={(checked) => updateSettings({ visual: { showGrid: checked } })} />
            </div>

            {/* Grid Style - conditional */}
            {visual.showGrid && (
                <div className="space-y-2">
                    <Label className="text-xs">Grid Style</Label>
                    <Select onValueChange={(value: GridStyle) => updateSettings({ visual: { gridStyle: value } })} value={visual.gridStyle}>
                        <SelectTrigger className="h-8">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {GRID_STYLE_OPTIONS.map((opt) => (
                                <SelectItem key={opt.value} value={opt.value}>
                                    {opt.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            )}

            {/* Legend Position */}
            <div className="space-y-2">
                <Label className="text-xs">Legend Position</Label>
                <Select onValueChange={(value: LegendPosition) => updateSettings({ visual: { legendPosition: value } })} value={visual.legendPosition}>
                    <SelectTrigger className="h-8">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {LEGEND_POSITION_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {opt.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* Chart Height */}
            <div className="space-y-2">
                <div className="flex justify-between">
                    <Label className="text-xs">Chart Height</Label>
                    <span className="text-muted-foreground text-xs">{visual.chartHeight}px</span>
                </div>
                <Slider max={800} min={200} onValueChange={([value]) => updateSettings({ visual: { chartHeight: value } })} step={50} value={[visual.chartHeight]} />
            </div>

            {/* Enable Animation */}
            <div className="flex items-center justify-between">
                <Label className="text-xs">Enable Animation</Label>
                <Switch checked={visual.enableAnimation} onCheckedChange={(checked) => updateSettings({ visual: { enableAnimation: checked } })} />
            </div>
        </div>
    );
}
