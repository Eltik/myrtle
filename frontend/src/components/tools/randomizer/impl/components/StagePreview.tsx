import { mergeProps } from "@base-ui/react/merge-props";
import { Download, Maximize2, RotateCcw, ZoomIn, ZoomOut } from "lucide-react";
import * as React from "react";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { stagePreviewURLs } from "#/lib/api/stages";
import { cn } from "#/lib/utils";
import type { IStage } from "#/types/stages";

interface IStagePreviewProps {
    stage: IStage;
    className?: string;
}

const RESOLVED_PREVIEW = new Map<string, string>();
const FAILED_PREVIEW = new Set<string>();

export function StagePreview({ stage, className }: IStagePreviewProps): React.ReactElement {
    const urls = React.useMemo(() => stagePreviewURLs(stage), [stage]);
    const cached = RESOLVED_PREVIEW.get(stage.stageId) ?? null;
    const [resolved, setResolved] = useState<string | null>(cached);
    const [resolving, setResolving] = useState<boolean>(cached === null);

    useEffect(() => {
        const hit = RESOLVED_PREVIEW.get(stage.stageId);
        if (hit) {
            setResolved(hit);
            setResolving(false);
            return;
        }
        setResolved(null);
        setResolving(true);

        let cancelled = false;
        let i = 0;
        const probe = () => {
            while (i < urls.length && FAILED_PREVIEW.has(urls[i] ?? "")) i++;
            if (cancelled) return;
            if (i >= urls.length) {
                setResolving(false);
                return;
            }
            const url = urls[i] ?? "";
            const img = new Image();
            img.onload = () => {
                if (cancelled) return;
                RESOLVED_PREVIEW.set(stage.stageId, url);
                setResolved(url);
                setResolving(false);
            };
            img.onerror = () => {
                FAILED_PREVIEW.add(url);
                i++;
                probe();
            };
            img.src = url;
        };
        probe();

        return () => {
            cancelled = true;
        };
    }, [stage.stageId, urls]);

    const failed = !resolving && resolved === null;
    const canExpand = resolved !== null;
    const labelName = stage.name ? `${stage.code} - ${stage.name}` : stage.code;

    const thumbnail = (
        <button
            type="button"
            disabled={!canExpand}
            aria-label={canExpand ? `Expand ${labelName} map preview` : `No preview available for ${stage.code}`}
            className={cn(
                "group relative block aspect-video w-full overflow-hidden rounded-lg border border-border/60 bg-muted/40 outline-none transition-shadow",
                canExpand && "cursor-zoom-in focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                !canExpand && "cursor-default",
                className,
            )}
        >
            {resolved && <img src={resolved} alt={`${labelName} map preview`} loading="lazy" decoding="async" className="absolute inset-0 h-full w-full object-cover transition-transform duration-300 ease-out group-hover:scale-[1.02]" />}
            {failed && <PreviewFallback code={stage.code} />}
            <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 bottom-0 h-12 bg-linear-to-t from-black/40 to-transparent dark:from-black/60" />
            {canExpand && (
                <span aria-hidden="true" className="pointer-events-none absolute right-2 top-2 inline-flex size-7 items-center justify-center rounded-md bg-black/55 text-white opacity-0 backdrop-blur-sm transition-opacity duration-200 group-hover:opacity-100 group-focus-visible:opacity-100">
                    <Maximize2 className="size-3.5" />
                </span>
            )}
        </button>
    );

    if (!resolved) {
        return thumbnail;
    }

    return (
        <StageViewerDialog imageSrc={resolved} stageName={labelName}>
            {thumbnail}
        </StageViewerDialog>
    );
}

function PreviewFallback({ code }: { code: string }): React.ReactElement {
    return (
        <div className="absolute inset-0 flex items-center justify-center bg-[radial-gradient(circle_at_30%_20%,var(--lagoon,#4fb8b2)/20,transparent_60%),radial-gradient(circle_at_70%_80%,var(--palm,#2f6a4a)/15,transparent_60%)] text-foreground/40">
            <svg aria-hidden="true" viewBox="0 0 64 64" className="h-10 w-10" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="6" y="14" width="52" height="36" rx="4" />
                <path d="M6 36 L20 28 L32 38 L46 26 L58 34" />
                <circle cx="22" cy="22" r="3" />
            </svg>
            <span className="sr-only">No preview available for {code}</span>
        </div>
    );
}

interface IStageViewerDialogProps {
    imageSrc: string;
    stageName: string;
    children: React.ReactNode;
}

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 5;
const ZOOM_STEP = 0.25;
const ZOOM_WHEEL_SENSITIVITY = 0.0015;
const BASE_SCALE = 1;

interface ITransform {
    zoom: number;
    pan: { x: number; y: number };
}

const INITIAL_TRANSFORM: ITransform = { zoom: 1, pan: { x: 0, y: 0 } };

const clampZoom = (z: number) => Math.min(Math.max(z, MIN_ZOOM), MAX_ZOOM);

const StageViewerDialog = memo(function StageViewerDialog({ imageSrc, stageName, children }: IStageViewerDialogProps) {
    const [transform, setTransform] = useState<ITransform>(INITIAL_TRANSFORM);
    const [isPanning, setIsPanning] = useState(false);
    const panStartRef = useRef({ x: 0, y: 0 });
    const panOffsetRef = useRef({ x: 0, y: 0 });
    const wheelCleanupRef = useRef<(() => void) | null>(null);

    const reset = useCallback(() => setTransform(INITIAL_TRANSFORM), []);

    const zoomBy = useCallback((delta: number) => {
        setTransform((prev) => {
            const next = clampZoom(prev.zoom + delta);
            return next === prev.zoom ? prev : { ...prev, zoom: next };
        });
    }, []);

    const setContainerRef = useCallback((el: HTMLDivElement | null) => {
        wheelCleanupRef.current?.();
        wheelCleanupRef.current = null;
        if (!el) return;
        const onWheel = (e: WheelEvent) => {
            e.preventDefault();
            const rect = el.getBoundingClientRect();
            const fx = e.clientX - rect.left - rect.width / 2;
            const fy = e.clientY - rect.top - rect.height / 2;
            const factor = Math.exp(-e.deltaY * ZOOM_WHEEL_SENSITIVITY);
            setTransform((prev) => {
                const nextZoom = clampZoom(prev.zoom * factor);
                if (nextZoom === prev.zoom) return prev;
                const ratio = nextZoom / prev.zoom;
                return {
                    zoom: nextZoom,
                    pan: {
                        x: fx + (prev.pan.x - fx) * ratio,
                        y: fy + (prev.pan.y - fy) * ratio,
                    },
                };
            });
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        wheelCleanupRef.current = () => el.removeEventListener("wheel", onWheel);
    }, []);

    const onPointerDown = useCallback(
        (e: React.PointerEvent) => {
            e.preventDefault();
            (e.target as HTMLElement).setPointerCapture(e.pointerId);
            setIsPanning(true);
            panStartRef.current = { x: e.clientX, y: e.clientY };
            panOffsetRef.current = { x: transform.pan.x, y: transform.pan.y };
        },
        [transform.pan],
    );

    const onPointerMove = useCallback(
        (e: React.PointerEvent) => {
            if (!isPanning) return;
            const dx = e.clientX - panStartRef.current.x;
            const dy = e.clientY - panStartRef.current.y;
            setTransform((prev) => ({
                ...prev,
                pan: { x: panOffsetRef.current.x + dx, y: panOffsetRef.current.y + dy },
            }));
        },
        [isPanning],
    );

    const onPointerUp = useCallback(() => setIsPanning(false), []);
    const onDoubleClick = useCallback(() => {
        setTransform((prev) => (prev.zoom === 1 ? { ...prev, zoom: 2 } : INITIAL_TRANSFORM));
    }, []);

    const onDownload = useCallback(async () => {
        try {
            const res = await fetch(imageSrc);
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `${stageName.replace(/[^a-zA-Z0-9\-_]/g, "_")}.png`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (e) {
            console.error("Failed to download stage preview", e);
        }
    }, [imageSrc, stageName]);

    const onOpenChange = useCallback(
        (open: boolean) => {
            if (!open) reset();
        },
        [reset],
    );

    return (
        <Dialog onOpenChange={onOpenChange}>
            <DialogTrigger render={children as React.ReactElement<Record<string, unknown>>} />
            <DialogContent className="flex h-[95vh] max-h-[95vh] w-[95vw] max-w-[95vw] flex-col overflow-hidden p-0 sm:max-w-[95vw]" showCloseButton bottomStickOnMobile={false}>
                <DialogTitle className="sr-only">{stageName}</DialogTitle>

                <div className="absolute left-3 top-3 z-10 flex items-center gap-1 rounded-lg border border-border/50 bg-background/80 p-1 shadow-sm backdrop-blur-sm">
                    <ToolButton onClick={() => zoomBy(-ZOOM_STEP)} disabled={transform.zoom <= MIN_ZOOM} label="Zoom out">
                        <ZoomOut className="h-4 w-4" />
                    </ToolButton>
                    <span className="min-w-12 select-none text-center font-mono text-muted-foreground text-xs">{Math.round(transform.zoom * 100)}%</span>
                    <ToolButton onClick={() => zoomBy(ZOOM_STEP)} disabled={transform.zoom >= MAX_ZOOM} label="Zoom in">
                        <ZoomIn className="h-4 w-4" />
                    </ToolButton>
                    <div className="mx-1 h-4 w-px bg-border" />
                    <ToolButton onClick={reset} label="Reset view">
                        <RotateCcw className="h-3.5 w-3.5" />
                    </ToolButton>
                    <ToolButton onClick={onDownload} label="Download">
                        <Download className="h-3.5 w-3.5" />
                    </ToolButton>
                </div>

                <div ref={setContainerRef} role="application" className={cn("relative h-full w-full cursor-grab select-none overflow-hidden", isPanning && "cursor-grabbing")} onDoubleClick={onDoubleClick} onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={onPointerUp}>
                    <div
                        className={cn("absolute inset-0", !isPanning && "transition-transform duration-150 ease-out")}
                        style={{
                            transform: `scale(${BASE_SCALE * transform.zoom}) translate(${transform.pan.x / (BASE_SCALE * transform.zoom)}px, ${transform.pan.y / (BASE_SCALE * transform.zoom)}px)`,
                        }}
                    >
                        <img alt={stageName} className="h-full w-full object-contain" decoding="async" draggable={false} src={imageSrc} />
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    );
});

function ToolButton({ children, onClick, disabled, label }: { children: React.ReactNode; onClick: () => void; disabled?: boolean; label: string }) {
    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <button
                        {...mergeProps<"button">(props, {
                            type: "button",
                            onClick,
                            disabled,
                            "aria-label": label,
                            className: "inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50",
                        })}
                    >
                        {children}
                    </button>
                )}
            />
            <TooltipPopup sideOffset={6}>{label}</TooltipPopup>
        </Tooltip>
    );
}
