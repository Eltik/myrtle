"use client";

import { Download, RotateCcw, ZoomIn, ZoomOut } from "lucide-react";
import Image from "next/image";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "~/components/ui/shadcn/dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";

interface SkinViewerDialogProps {
    imageSrc: string;
    skinName: string;
    children: React.ReactNode;
}

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 5;
const ZOOM_STEP = 0.25;
const BASE_SCALE = 1.15;

export const SkinViewerDialog = memo(function SkinViewerDialog({ imageSrc, skinName, children }: SkinViewerDialogProps) {
    const [zoom, setZoom] = useState(1);
    const [pan, setPan] = useState({ x: 0, y: 0 });
    const [isPanning, setIsPanning] = useState(false);
    const panStartRef = useRef({ x: 0, y: 0 });
    const panOffsetRef = useRef({ x: 0, y: 0 });
    const containerRef = useRef<HTMLDivElement>(null);

    const handleZoomIn = useCallback(() => {
        setZoom((prev) => Math.min(prev + ZOOM_STEP, MAX_ZOOM));
    }, []);

    const handleZoomOut = useCallback(() => {
        setZoom((prev) => Math.max(prev - ZOOM_STEP, MIN_ZOOM));
    }, []);

    const handleReset = useCallback(() => {
        setZoom(1);
        setPan({ x: 0, y: 0 });
    }, []);

    // Native wheel event listener for preventDefault support
    useEffect(() => {
        const container = containerRef.current;
        if (!container) return;

        const onWheel = (e: WheelEvent) => {
            e.preventDefault();
            const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
            setZoom((prev) => Math.min(Math.max(prev + delta, MIN_ZOOM), MAX_ZOOM));
        };

        container.addEventListener("wheel", onWheel, { passive: false });
        return () => container.removeEventListener("wheel", onWheel);
    }, []);

    const handlePointerDown = useCallback(
        (e: React.PointerEvent) => {
            e.preventDefault();
            (e.target as HTMLElement).setPointerCapture(e.pointerId);
            setIsPanning(true);
            panStartRef.current = { x: e.clientX, y: e.clientY };
            panOffsetRef.current = { x: pan.x, y: pan.y };
        },
        [pan],
    );

    const handlePointerMove = useCallback(
        (e: React.PointerEvent) => {
            if (!isPanning) return;
            const dx = e.clientX - panStartRef.current.x;
            const dy = e.clientY - panStartRef.current.y;
            setPan({
                x: panOffsetRef.current.x + dx,
                y: panOffsetRef.current.y + dy,
            });
        },
        [isPanning],
    );

    const handlePointerUp = useCallback(() => {
        setIsPanning(false);
    }, []);

    const handleDoubleClick = useCallback(() => {
        if (zoom === 1) {
            setZoom(2);
        } else {
            handleReset();
        }
    }, [zoom, handleReset]);

    const handleDownload = useCallback(async () => {
        try {
            const response = await fetch(imageSrc);
            const blob = await response.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `${skinName.replace(/[^a-zA-Z0-9\-_]/g, "_")}.png`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (err) {
            console.error("Failed to download image:", err);
        }
    }, [imageSrc, skinName]);

    const handleOpenChange = useCallback((open: boolean) => {
        if (!open) {
            setZoom(1);
            setPan({ x: 0, y: 0 });
            setIsPanning(false);
        }
    }, []);

    return (
        <Dialog onOpenChange={handleOpenChange}>
            <DialogTrigger asChild>{children}</DialogTrigger>
            <DialogContent className="flex max-h-[95vh] max-w-[95vw] flex-col overflow-hidden p-0 sm:max-w-[95vw]">
                <DialogTitle className="sr-only">{skinName}</DialogTitle>

                {/* Toolbar */}
                <div className="absolute top-3 left-3 z-10 flex items-center gap-1 rounded-lg border border-border/50 bg-background/80 p-1 shadow-sm backdrop-blur-sm">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button className="h-8 w-8" disabled={zoom <= MIN_ZOOM} onClick={handleZoomOut} size="icon" variant="ghost">
                                <ZoomOut className="h-4 w-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Zoom out</TooltipContent>
                    </Tooltip>

                    <span className="min-w-12 select-none text-center font-mono text-muted-foreground text-xs">{Math.round(zoom * 100)}%</span>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button className="h-8 w-8" disabled={zoom >= MAX_ZOOM} onClick={handleZoomIn} size="icon" variant="ghost">
                                <ZoomIn className="h-4 w-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Zoom in</TooltipContent>
                    </Tooltip>

                    <div className="mx-1 h-4 w-px bg-border" />

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button className="h-8 w-8" onClick={handleReset} size="icon" variant="ghost">
                                <RotateCcw className="h-3.5 w-3.5" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Reset view</TooltipContent>
                    </Tooltip>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button className="h-8 w-8" onClick={handleDownload} size="icon" variant="ghost">
                                <Download className="h-3.5 w-3.5" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Download</TooltipContent>
                    </Tooltip>
                </div>

                {/* Image Container */}
                <div className={cn("relative h-[90vh] w-full cursor-grab select-none overflow-hidden", isPanning && "cursor-grabbing")} onDoubleClick={handleDoubleClick} onPointerDown={handlePointerDown} onPointerMove={handlePointerMove} onPointerUp={handlePointerUp} ref={containerRef} role="application">
                    <div className={cn("absolute inset-0", !isPanning && "transition-transform duration-150 ease-out")} style={{ transform: `scale(${BASE_SCALE * zoom}) translate(${pan.x / (BASE_SCALE * zoom)}px, ${pan.y / (BASE_SCALE * zoom)}px)` }}>
                        <Image alt={skinName} className="object-contain" draggable={false} fill src={imageSrc} unoptimized />
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    );
});
