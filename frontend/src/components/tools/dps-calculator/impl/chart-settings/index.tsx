"use client";

import { RotateCcw, Settings } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/shadcn/popover";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";
import { getStoredSettingsTab, setStoredSettingsTab } from "~/lib/dps-chart-settings";
import type { SettingsTab } from "~/types/frontend/impl/tools/dps-chart-settings";
import { DataDisplaySection } from "./data-display-section";
import { ExportSection } from "./export-section";
import { RangeSettingsSection } from "./range-settings-section";
import { VisualSettingsSection } from "./visual-settings-section";

interface ChartSettingsPopoverProps {
    onExportPng: () => void;
    onExportCsv: () => void;
    isExporting?: boolean;
}

export function ChartSettingsPopover({ onExportPng, onExportCsv, isExporting = false }: ChartSettingsPopoverProps) {
    const { isDefault, resetToDefaults } = useDpsChartSettings();
    const [activeTab, setActiveTab] = useState<SettingsTab>("visual");

    // Load saved tab on mount
    useEffect(() => {
        const savedTab = getStoredSettingsTab();
        setActiveTab(savedTab);
    }, []);

    // Save tab when it changes
    const handleTabChange = (tab: string) => {
        const newTab = tab as SettingsTab;
        setActiveTab(newTab);
        setStoredSettingsTab(newTab);
    };

    return (
        <Popover>
            <PopoverTrigger asChild>
                <Button className="h-8 w-8" size="icon" title="Chart settings" variant="ghost">
                    <Settings className="h-4 w-4" />
                    <span className="sr-only">Chart settings</span>
                </Button>
            </PopoverTrigger>
            <PopoverContent align="end" className="w-80 p-0">
                <div className="border-b p-3">
                    <div className="flex items-center justify-between">
                        <h4 className="font-medium text-sm">Chart Settings</h4>
                        {!isDefault && (
                            <Button className="h-6 gap-1 px-2 text-xs" onClick={resetToDefaults} size="sm" variant="ghost">
                                <RotateCcw className="h-3 w-3" />
                                Reset
                            </Button>
                        )}
                    </div>
                </div>

                <Tabs className="w-full" onValueChange={handleTabChange} value={activeTab}>
                    <TabsList className="w-full justify-start rounded-none border-b bg-transparent p-0">
                        <TabsTrigger className="rounded-none border-transparent border-b-2 data-[state=active]:border-primary data-[state=active]:bg-transparent" value="visual">
                            Visual
                        </TabsTrigger>
                        <TabsTrigger className="rounded-none border-transparent border-b-2 data-[state=active]:border-primary data-[state=active]:bg-transparent" value="data">
                            Data
                        </TabsTrigger>
                        <TabsTrigger className="rounded-none border-transparent border-b-2 data-[state=active]:border-primary data-[state=active]:bg-transparent" value="range">
                            Range
                        </TabsTrigger>
                    </TabsList>

                    <div className="max-h-[400px] overflow-y-auto p-3">
                        <TabsContent className="mt-0" value="visual">
                            <VisualSettingsSection />
                        </TabsContent>
                        <TabsContent className="mt-0" value="data">
                            <DataDisplaySection />
                        </TabsContent>
                        <TabsContent className="mt-0" value="range">
                            <RangeSettingsSection />
                        </TabsContent>
                    </div>
                </Tabs>

                <div className="border-t p-3">
                    <ExportSection isExporting={isExporting} onExportCsv={onExportCsv} onExportPng={onExportPng} />
                </div>
            </PopoverContent>
        </Popover>
    );
}
