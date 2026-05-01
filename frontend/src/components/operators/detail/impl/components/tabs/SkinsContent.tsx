import { mergeProps } from "@base-ui/react/merge-props";
import { useQuery } from "@tanstack/react-query";
import { Calendar, Download, FileText, Maximize2, MessageCircle, Palette, RotateCcw, Sparkles, ZoomIn, ZoomOut } from "lucide-react";
import { memo, useCallback, useMemo, useRef, useState } from "react";
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Skeleton } from "#/components/ui/skeleton";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { chibisQueryOptions, type IChibiCharacter } from "#/lib/api/chibis";
import { type ISkin, skinsQueryOptions } from "#/lib/api/skins";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { buildOperatorSkinList, chibiSkinKey, type IUISkin } from "../../skins";
import { DynamicChibiViewer } from "../chibi/ChibiViewer.lazy";

interface ISkinsContentProps {
    operator: IOperatorListItem;
}

export const SkinsContent = memo(function SkinsContent({ operator }: ISkinsContentProps) {
    const { data: skinsResponse, isLoading: skinsLoading } = useQuery(skinsQueryOptions());
    const { data: chibis } = useQuery(chibisQueryOptions());

    const skins: IUISkin[] = useMemo(() => {
        const charSkins = skinsResponse?.charSkins ?? {};
        const operatorSkins: ISkin[] = Object.values(charSkins).filter((s) => s.charId === operator.id);
        return buildOperatorSkinList({
            skinsFromBackend: operatorSkins,
            operatorId: operator.id ?? "",
            operatorSkin: operator.skin,
            operatorPortrait: operator.portrait,
            phasesLength: operator.phases.length,
            artistFallback: operator.artists?.[0],
        });
    }, [skinsResponse, operator.id, operator.skin, operator.portrait, operator.phases.length, operator.artists?.[0]]);

    const [selectedId, setSelectedId] = useState<string | null>(null);
    const selected = useMemo(() => skins.find((s) => s.id === selectedId) ?? skins[0] ?? null, [skins, selectedId]);

    const chibiCharacter: IChibiCharacter | null = useMemo(() => chibis?.characters?.find((c) => c.operatorCode === operator.id) ?? null, [chibis, operator.id]);
    const chibiSkin = useMemo(() => {
        if (!chibiCharacter || !selected) return null;
        const key = chibiSkinKey(selected.id);
        return chibiCharacter.skins.find((s) => s.name === key) ?? chibiCharacter.skins[0] ?? null;
    }, [chibiCharacter, selected]);

    if (skinsLoading)
        return (
            <div className="min-w-0 overflow-hidden p-4 md:p-6">
                <div className="mb-6">
                    <Skeleton className="h-7 w-32" />
                    <Skeleton className="mt-2 h-4 w-64" />
                </div>
                <div className="grid gap-5 lg:grid-cols-[1fr,280px]">
                    <div className="flex flex-col gap-4">
                        <Skeleton className="aspect-4/3 w-full rounded-xl md:aspect-16/11" />
                        <Skeleton className="h-32 w-full rounded-xl" />
                    </div>
                    <div className="flex w-full gap-2.5 overflow-x-hidden lg:flex-col">
                        {[0, 1, 2, 3].map((i) => (
                            <Skeleton key={i} className="h-20 w-72 shrink-0 rounded-lg lg:w-full" />
                        ))}
                    </div>
                </div>
            </div>
        );

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Outfits</h2>
                <p className="text-muted-foreground text-sm">Alternate skins, E2 art variants, and collaboration outfits</p>
            </div>
            <div className="grid min-w-0 gap-5 lg:grid-cols-[1fr,280px]">
                <div className="flex flex-col gap-4 min-w-0">
                    {selected && (
                        <div className="relative overflow-hidden rounded-xl border border-border bg-linear-to-b from-secondary/30 via-secondary/10 to-secondary/40 aspect-4/3 md:aspect-16/11">
                            <img alt={selected.name} className="absolute inset-0 h-full w-full object-contain" decoding="async" loading="eager" src={selected.image} />
                            <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/70 via-black/40 to-transparent" />
                            <SkinViewerDialog imageSrc={selected.image} skinName={selected.name}>
                                <button type="button" aria-label="Fullscreen" className="absolute right-3 top-3 inline-flex items-center justify-center rounded-md border border-white/10 bg-white/5 p-1.5 text-white/80 backdrop-blur-md transition-colors hover:bg-white/10 hover:text-white">
                                    <Maximize2 className="h-4 w-4" />
                                </button>
                            </SkinViewerDialog>
                            <div className="absolute inset-x-0 bottom-0 p-4 md:p-5">
                                <div className="font-mono text-[10px] uppercase tracking-wider text-white/70">{selected.kicker}</div>
                                <div className="mt-0.5 font-semibold text-2xl text-white drop-shadow-md">{selected.name}</div>
                                {(selected.displaySkin?.drawerList?.join(", ") ?? null) && <div className="mt-0.5 text-white/70 text-xs">Artist · {selected.displaySkin?.drawerList?.join(", ")}</div>}
                            </div>
                        </div>
                    )}
                </div>
                <div className="flex w-full gap-2.5 overflow-x-auto pb-2 lg:max-h-[calc(100vh-30rem)] lg:flex-col lg:overflow-x-visible lg:overflow-y-auto lg:pb-0 lg:pr-1">
                    {skins.map((skin) => (
                        <button
                            key={skin.id}
                            type="button"
                            onClick={() => setSelectedId(skin.id)}
                            className={cn("flex w-72 shrink-0 items-stretch gap-3 rounded-lg border bg-card p-2.5 text-left transition-colors lg:w-full", selected?.id === skin.id ? "border-primary ring-1 ring-primary/40" : "border-border hover:border-primary/40")}
                        >
                            <div className="h-16 w-16 shrink-0 overflow-hidden rounded-md bg-secondary/30">
                                <img alt={skin.name} className="h-full w-full object-cover" decoding="async" loading="lazy" src={skin.thumbnail} />
                            </div>
                            <div className="min-w-0 flex-1">
                                <div className="font-medium font-mono text-[10px] uppercase tracking-wider text-muted-foreground">{skin.kicker}</div>
                                <div className="mt-0.5 truncate font-semibold text-foreground text-sm">{skin.name}</div>
                                <div className="mt-0.5 truncate text-muted-foreground text-xs">{skin.sub}</div>
                            </div>
                        </button>
                    ))}
                </div>
            </div>
            {selected &&
                (() => {
                    const description = selected.displaySkin?.description ?? (selected.isDefault ? "Standard operator outfit." : selected.kicker);
                    const obtained = selected.displaySkin?.obtainApproach ?? (selected.isDefault || selected.id.endsWith("_e2") ? "Unlocked by default" : null);
                    const dialog = selected.displaySkin?.dialog;
                    const usage = selected.displaySkin?.usage;
                    const colors = (selected.displaySkin?.colorList ?? []).filter((c) => c?.startsWith("#"));

                    return (
                        <div className="mt-5 overflow-hidden rounded-xl border border-border bg-card/60">
                            {description && (
                                <div className="border-b border-border/60 bg-linear-to-br from-secondary/20 via-transparent to-transparent px-5 py-4 md:px-6 md:py-5">
                                    <div className="flex items-center gap-2">
                                        <FileText className="h-3.5 w-3.5 text-muted-foreground" />
                                        <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Description</span>
                                    </div>
                                    <p className="mt-2 text-foreground/90 text-sm italic leading-relaxed">{description}</p>
                                </div>
                            )}
                            <div className="grid gap-x-8 gap-y-5 px-5 py-4 md:grid-cols-2 md:px-6 md:py-5">
                                {obtained && (
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
                                            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Obtained</span>
                                        </div>
                                        <p className="mt-1.5 text-foreground text-sm leading-relaxed">{obtained}</p>
                                    </div>
                                )}
                                {usage && (
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <Sparkles className="h-3.5 w-3.5 text-muted-foreground" />
                                            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Usage</span>
                                        </div>
                                        <p className="mt-1.5 text-foreground text-sm leading-relaxed">{usage}</p>
                                    </div>
                                )}
                                {colors.length > 0 && (
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <Palette className="h-3.5 w-3.5 text-muted-foreground" />
                                            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Colors</span>
                                        </div>
                                        <div className="mt-2 flex flex-wrap gap-2">
                                            {colors.map((color) => (
                                                <Tooltip key={color}>
                                                    <TooltipTrigger render={(props) => <span className="h-7 w-7 cursor-pointer rounded-md border border-border/60 shadow-sm transition-transform hover:scale-110" style={{ backgroundColor: color }} {...props} />} />
                                                    <TooltipPopup>
                                                        <span className="font-mono text-xs">{color.toUpperCase()}</span>
                                                    </TooltipPopup>
                                                </Tooltip>
                                            ))}
                                        </div>
                                    </div>
                                )}
                                {dialog && (
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <MessageCircle className="h-3.5 w-3.5 text-muted-foreground" />
                                            <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-wider">Dialog</span>
                                        </div>
                                        <p className="mt-1.5 text-foreground text-sm leading-relaxed">{dialog}</p>
                                    </div>
                                )}
                            </div>
                        </div>
                    );
                })()}
            {chibiCharacter && <DynamicChibiViewer chibi={chibiCharacter} skin={chibiSkin} />}
        </div>
    );
});

interface ISkinViewerDialogProps {
    imageSrc: string;
    skinName: string;
    children: React.ReactNode;
}

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 5;
const ZOOM_STEP = 0.25;
const ZOOM_WHEEL_SENSITIVITY = 0.0015;
const BASE_SCALE = 1.15;

interface ITransform {
    zoom: number;
    pan: { x: number; y: number };
}

const INITIAL_TRANSFORM: ITransform = { zoom: 1, pan: { x: 0, y: 0 } };

const clampZoom = (z: number) => Math.min(Math.max(z, MIN_ZOOM), MAX_ZOOM);

export const SkinViewerDialog = memo(function SkinViewerDialog({ imageSrc, skinName, children }: ISkinViewerDialogProps) {
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
            a.download = `${skinName.replace(/[^a-zA-Z0-9\-_]/g, "_")}.png`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (e) {
            console.error("Failed to download skin image", e);
        }
    }, [imageSrc, skinName]);

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
                <DialogTitle className="sr-only">{skinName}</DialogTitle>

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
                        <img alt={skinName} className="h-full w-full object-contain" decoding="async" draggable={false} src={imageSrc} />
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
            <TooltipPopup side="bottom">{label}</TooltipPopup>
        </Tooltip>
    );
}
