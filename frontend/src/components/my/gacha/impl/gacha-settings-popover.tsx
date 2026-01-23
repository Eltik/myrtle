"use client";

import { Check, Settings2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/motion-primitives/morphing-popover";
import { Button } from "~/components/ui/shadcn/button";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Switch } from "~/components/ui/shadcn/switch";

interface GachaSettingsPopoverProps {
    pageSize: number;
    onPageSizeChange: (size: number) => void;
}

export function GachaSettingsPopover({ pageSize, onPageSizeChange }: GachaSettingsPopoverProps) {
    const [isOpen, setIsOpen] = useState(false);
    const [autoRefresh, setAutoRefresh] = useState(false);
    const [compactView, setCompactView] = useState(false);

    const handlePageSizeChange = (value: string) => {
        const size = Number.parseInt(value, 10);
        onPageSizeChange(size);
        toast.success(`Page size updated to ${size}`);
    };

    return (
        <MorphingPopover onOpenChange={setIsOpen} open={isOpen}>
            <MorphingPopoverTrigger>
                <Button size="sm" variant="outline">
                    <Settings2 className="mr-2 h-4 w-4" />
                    Settings
                </Button>
            </MorphingPopoverTrigger>
            <MorphingPopoverContent className="w-80">
                <div className="space-y-4 p-4">
                    <div className="space-y-1">
                        <h4 className="font-semibold text-sm">Display Settings</h4>
                        <p className="text-muted-foreground text-xs">Customize how your gacha history is displayed</p>
                    </div>

                    <Separator />

                    {/* Page Size Setting */}
                    <div className="space-y-2">
                        <Label className="text-xs" htmlFor="page-size">
                            Items per page
                        </Label>
                        <Select defaultValue={String(pageSize)} onValueChange={handlePageSizeChange}>
                            <SelectTrigger id="page-size">
                                <SelectValue placeholder="Select page size" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="25">25 items</SelectItem>
                                <SelectItem value="50">50 items</SelectItem>
                                <SelectItem value="100">100 items</SelectItem>
                                <SelectItem value="200">200 items</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    {/* Auto Refresh Setting */}
                    <div className="flex items-center justify-between">
                        <div className="space-y-0.5">
                            <Label className="text-xs" htmlFor="auto-refresh">
                                Auto refresh
                            </Label>
                            <p className="text-muted-foreground text-xs">Automatically refresh data periodically</p>
                        </div>
                        <Switch checked={autoRefresh} id="auto-refresh" onCheckedChange={setAutoRefresh} />
                    </div>

                    {/* Compact View Setting */}
                    <div className="flex items-center justify-between">
                        <div className="space-y-0.5">
                            <Label className="text-xs" htmlFor="compact-view">
                                Compact view
                            </Label>
                            <p className="text-muted-foreground text-xs">Show more items with less spacing</p>
                        </div>
                        <Switch checked={compactView} id="compact-view" onCheckedChange={setCompactView} />
                    </div>

                    <Separator />

                    <div className="flex items-center gap-2 text-muted-foreground text-xs">
                        <Check className="h-3 w-3" />
                        <span>Changes are saved automatically</span>
                    </div>
                </div>
            </MorphingPopoverContent>
        </MorphingPopover>
    );
}
