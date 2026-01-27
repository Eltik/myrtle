"use client";

import { Download, X } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { Progress } from "~/components/ui/shadcn/progress";
import type { ExportFormat } from "./recorder";

interface DownloadButtonProps {
    isRecording: boolean;
    progress: number;
    disabled: boolean;
    onDownload: (format: ExportFormat) => void;
    onCancel: () => void;
}

export function DownloadButton({ isRecording, progress, disabled, onDownload, onCancel }: DownloadButtonProps) {
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
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className="h-8 gap-1.5 px-2 text-xs" disabled={disabled} size="sm" variant="outline">
                    <Download className="h-3.5 w-3.5" />
                    Export
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => onDownload("gif")}>Download as GIF</DropdownMenuItem>
                <DropdownMenuItem onClick={() => onDownload("mp4")}>Download as MP4</DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
