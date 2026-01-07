"use client";

import { FileSpreadsheet, Image } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";

interface ExportSectionProps {
    onExportPng: () => void;
    onExportCsv: () => void;
    isExporting?: boolean;
}

export function ExportSection({ onExportPng, onExportCsv, isExporting = false }: ExportSectionProps) {
    const { settings, updateSettings } = useDpsChartSettings();

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2">
                <Button className="flex-1 gap-2" disabled={isExporting} onClick={onExportPng} size="sm" variant="outline">
                    <Image className="h-4 w-4" />
                    Export PNG
                </Button>
                <Button className="flex-1 gap-2" disabled={isExporting} onClick={onExportCsv} size="sm" variant="outline">
                    <FileSpreadsheet className="h-4 w-4" />
                    Export CSV
                </Button>
            </div>

            <div className="flex items-center gap-4">
                <div className="flex-1 space-y-1">
                    <Label className="text-xs">Scale</Label>
                    <Select onValueChange={(value) => updateSettings({ export: { exportScale: Number(value) } })} value={String(settings.export.exportScale)}>
                        <SelectTrigger className="h-8">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="1">1x</SelectItem>
                            <SelectItem value="2">2x</SelectItem>
                            <SelectItem value="3">3x</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
                <div className="flex-1 space-y-1">
                    <Label className="text-xs">Background</Label>
                    <Select onValueChange={(value: "transparent" | "themed") => updateSettings({ export: { exportBackground: value } })} value={settings.export.exportBackground}>
                        <SelectTrigger className="h-8">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="themed">Themed</SelectItem>
                            <SelectItem value="transparent">Transparent</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>
        </div>
    );
}
