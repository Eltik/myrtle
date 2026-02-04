"use client";

import { Download, Settings2, X } from "lucide-react";
import { useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Label } from "~/components/ui/shadcn/label";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/shadcn/popover";
import { Progress } from "~/components/ui/shadcn/progress";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Switch } from "~/components/ui/shadcn/switch";
import { DEFAULT_EXPORT_SETTINGS, EXPORT_FPS_OPTIONS, EXPORT_LOOP_OPTIONS, EXPORT_SCALE_OPTIONS, type ExportSettings } from "./constants";
import type { ExportFormat } from "./recorder";

interface DownloadButtonProps {
    isRecording: boolean;
    progress: number;
    disabled: boolean;
    onDownload: (format: ExportFormat, settings?: Partial<ExportSettings>) => void;
    onCancel: () => void;
}

export function DownloadButton({ isRecording, progress, disabled, onDownload, onCancel }: DownloadButtonProps) {
    const [settings, setSettings] = useState<ExportSettings>(DEFAULT_EXPORT_SETTINGS);
    const [open, setOpen] = useState(false);

    const handleDownload = (format: ExportFormat) => {
        setOpen(false);
        onDownload(format, settings);
    };

    const currentResolution = EXPORT_SCALE_OPTIONS.find((o) => o.value === settings.scale)?.label ?? "600x400";

    if (isRecording) {
        return (
            <div className="flex items-center gap-2">
                <div className="flex min-w-[120px] items-center gap-2">
                    <Progress className="h-2 w-full" value={progress} />
                    <span className="text-muted-foreground text-xs tabular-nums">{Math.round(progress)}%</span>
                </div>
                <Button className="h-8 w-8 p-0" onClick={onCancel} size="icon" variant="ghost">
                    <X className="h-3.5 w-3.5" />
                </Button>
            </div>
        );
    }

    return (
        <Popover onOpenChange={setOpen} open={open}>
            <PopoverTrigger asChild>
                <Button className="h-8 gap-1.5 px-2 text-xs" disabled={disabled} size="sm" variant="outline">
                    <Download className="h-3.5 w-3.5" />
                    Export
                </Button>
            </PopoverTrigger>
            <PopoverContent align="end" className="w-64">
                <div className="grid gap-4">
                    <div className="flex items-center gap-2">
                        <Settings2 className="h-4 w-4" />
                        <h4 className="font-medium text-sm">Export Settings</h4>
                    </div>

                    <div className="grid gap-3">
                        <div className="grid gap-1.5">
                            <Label className="text-xs" htmlFor="resolution">
                                Resolution
                            </Label>
                            <Select onValueChange={(v) => setSettings((s) => ({ ...s, scale: Number.parseFloat(v) }))} value={String(settings.scale)}>
                                <SelectTrigger className="h-8 text-xs" id="resolution">
                                    <SelectValue>{currentResolution}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {EXPORT_SCALE_OPTIONS.map((opt) => (
                                        <SelectItem key={opt.value} value={String(opt.value)}>
                                            {opt.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <div className="grid gap-1.5">
                            <Label className="text-xs" htmlFor="fps">
                                Frame Rate
                            </Label>
                            <Select onValueChange={(v) => setSettings((s) => ({ ...s, fps: Number.parseInt(v, 10) }))} value={String(settings.fps)}>
                                <SelectTrigger className="h-8 text-xs" id="fps">
                                    <SelectValue>{settings.fps} FPS</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {EXPORT_FPS_OPTIONS.map((fps) => (
                                        <SelectItem key={fps} value={String(fps)}>
                                            {fps} FPS
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <Separator />

                        <div className="text-muted-foreground text-xs">GIF Options</div>

                        <div className="flex items-center justify-between">
                            <Label className="text-xs" htmlFor="transparent">
                                Transparent Background
                            </Label>
                            <Switch checked={settings.transparentBg} id="transparent" onCheckedChange={(checked) => setSettings((s) => ({ ...s, transparentBg: checked }))} />
                        </div>

                        <Separator />

                        <div className="text-muted-foreground text-xs">MP4 Options</div>

                        <div className="grid gap-1.5">
                            <Label className="text-xs" htmlFor="loops">
                                Loop Count
                            </Label>
                            <Select onValueChange={(v) => setSettings((s) => ({ ...s, loopCount: Number.parseInt(v, 10) }))} value={String(settings.loopCount)}>
                                <SelectTrigger className="h-8 text-xs" id="loops">
                                    <SelectValue>
                                        {settings.loopCount} {settings.loopCount === 1 ? "loop" : "loops"}
                                    </SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {EXPORT_LOOP_OPTIONS.map((count) => (
                                        <SelectItem key={count} value={String(count)}>
                                            {count} {count === 1 ? "loop" : "loops"}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <Separator />

                    <div className="grid grid-cols-2 gap-2">
                        <Button className="h-8 text-xs" onClick={() => handleDownload("gif")} size="sm" variant="outline">
                            Download GIF
                        </Button>
                        <Button className="h-8 text-xs" onClick={() => handleDownload("mp4")} size="sm">
                            Download MP4
                        </Button>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
    );
}
