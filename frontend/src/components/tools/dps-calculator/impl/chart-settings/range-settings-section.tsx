"use client";

import { Input } from "~/components/ui/shadcn/input";
import { Label } from "~/components/ui/shadcn/label";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";

export function RangeSettingsSection() {
    const { settings, updateSettings } = useDpsChartSettings();
    const { range } = settings;

    const handleNumberChange = (key: keyof typeof range, value: string) => {
        const num = Number.parseInt(value, 10);
        if (!Number.isNaN(num) && num >= 0) {
            updateSettings({ range: { [key]: num } });
        }
    };

    return (
        <div className="space-y-4">
            {/* Defense Range */}
            <div className="space-y-3">
                <Label className="font-medium text-xs">Defense Range</Label>
                <div className="grid grid-cols-3 gap-2">
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Min</Label>
                        <Input className="h-8" max={range.maxDef - 1} min={0} onChange={(e) => handleNumberChange("minDef", e.target.value)} type="number" value={range.minDef} />
                    </div>
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Max</Label>
                        <Input className="h-8" max={10000} min={range.minDef + 1} onChange={(e) => handleNumberChange("maxDef", e.target.value)} type="number" value={range.maxDef} />
                    </div>
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Step</Label>
                        <Input className="h-8" max={500} min={1} onChange={(e) => handleNumberChange("defStep", e.target.value)} type="number" value={range.defStep} />
                    </div>
                </div>
                <p className="text-muted-foreground text-xs">{Math.ceil((range.maxDef - range.minDef) / range.defStep) + 1} data points</p>
            </div>

            {/* Resistance Range */}
            <div className="space-y-3">
                <Label className="font-medium text-xs">Resistance Range</Label>
                <div className="grid grid-cols-3 gap-2">
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Min</Label>
                        <Input className="h-8" max={range.maxRes - 1} min={0} onChange={(e) => handleNumberChange("minRes", e.target.value)} type="number" value={range.minRes} />
                    </div>
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Max</Label>
                        <Input className="h-8" max={200} min={range.minRes + 1} onChange={(e) => handleNumberChange("maxRes", e.target.value)} type="number" value={range.maxRes} />
                    </div>
                    <div className="space-y-1">
                        <Label className="text-muted-foreground text-xs">Step</Label>
                        <Input className="h-8" max={50} min={1} onChange={(e) => handleNumberChange("resStep", e.target.value)} type="number" value={range.resStep} />
                    </div>
                </div>
                <p className="text-muted-foreground text-xs">{Math.ceil((range.maxRes - range.minRes) / range.resStep) + 1} data points</p>
            </div>

            {/* Note */}
            <p className="text-muted-foreground text-xs">Changing range settings will refetch DPS data from the server.</p>
        </div>
    );
}
