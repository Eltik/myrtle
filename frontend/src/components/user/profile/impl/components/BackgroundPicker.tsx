import { Dialog as DialogPrimitive } from "@base-ui/react/dialog";
import { useMutation } from "@tanstack/react-query";
import { GripHorizontalIcon, Trash2Icon, XIcon } from "lucide-react";
import { type KeyboardEvent, type PointerEvent, type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EntityPickerBody } from "#/components/grids/EntityPickerDialog";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogPortal, DialogTitle, DialogViewport } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Slider, SliderPrimitive } from "#/components/ui/slider";
import { Spinner } from "#/components/ui/spinner";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { toastManager } from "#/components/ui/toast";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { entityKey, type ITierEntity } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import { BACKGROUND_KINDS, croppableAxes, type IArtGeometry, isGalleryKind, pickBackground, SCALE_MAX, SCALE_MIN, sameBackground, withFocusX, withFocusY, withScale, zoomOf } from "../background";
import { COMPACT_HEIGHT, defaultRect, type IEdges, type IRect, moveRect, parseRect, resizeRect, WIDE_WIDTH } from "../picker-rect";
import type { messages } from "./BackgroundPicker.messages";
import { GalleryPicker } from "./GalleryPicker";

/** The picker's two sources: the tier-list entity picker over outfits and operators, and the gallery (Archives pictures, story CGs, story scenes). */
type ArtSource = "characters" | "gallery";

interface IBackgroundPickerProps {
    /** The background saved now, `null` for none. */
    saved: ProfileBackground | null;
    /** The unsaved pick the header behind the dialog is previewing, `null` for none. */
    draft: ProfileBackground | null;
    /** The header art's drawn box and file size, `null` before it loads: which crop axes move anything. */
    artGeometry: IArtGeometry | null;
    onDraftChange: (next: ProfileBackground | null) => void;
    /** Closes the picker; the caller drops the draft, so the header shows `saved` again. */
    onClose: () => void;
}

/**
 * Where the picker's rectangle is remembered, per browser. `:v2` since the default grew to
 * 960 x 720 with the Gallery rail: a rectangle remembered at the old size would keep every
 * returning author on the narrow layout, so it is read once more from the default.
 */
const RECT_KEY = "myrtle:profile-background-picker:rect:v2";
/** One arrow-key step of the move and resize handles, in CSS pixels. */
const KEY_STEP = 16;

/** The interactive controls a title-bar drag must leave alone. */
const NO_DRAG = "button, input, a, select, textarea, [role=slider], [role=tab]";

function readRect(): IRect | null {
    try {
        return parseRect(window.localStorage.getItem(RECT_KEY));
    } catch {
        return null;
    }
}

function writeRect(rect: IRect) {
    try {
        window.localStorage.setItem(RECT_KEY, JSON.stringify(rect));
    } catch {
        // Storage blocked or full: the picker still moves, it just opens docked next time.
    }
}

const ARROWS: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };

/**
 * The floating picker's rectangle: opened where the author left it (else docked
 * lower left), kept inside the viewport as the window resizes, and the pointer
 * and keyboard handlers that move and resize it. Saved on each drag's end.
 */
function useFloatingRect() {
    const [rect, setRect] = useState<IRect>(() => {
        const vw = window.innerWidth;
        const vh = window.innerHeight;
        const stored = readRect();
        return stored ? moveRect(stored, 0, 0, vw, vh) : defaultRect(vw, vh);
    });
    const rectRef = useRef(rect);
    rectRef.current = rect;

    useEffect(() => {
        const onResize = () => setRect((r) => moveRect(r, 0, 0, window.innerWidth, window.innerHeight));
        window.addEventListener("resize", onResize);
        return () => window.removeEventListener("resize", onResize);
    }, []);

    /** Starts a drag that maps the pointer's travel through `step`, captured on the pressed element. */
    const track = useCallback((e: PointerEvent<HTMLElement>, step: (start: IRect, dx: number, dy: number) => IRect) => {
        if (e.button !== 0) return;
        e.preventDefault();
        const el = e.currentTarget;
        el.setPointerCapture(e.pointerId);
        const start = rectRef.current;
        const x0 = e.clientX;
        const y0 = e.clientY;
        // The last rectangle this drag made, saved on its end: the ref lags a render behind the final move.
        let last = start;
        const move = (ev: globalThis.PointerEvent) => {
            last = step(start, ev.clientX - x0, ev.clientY - y0);
            setRect(last);
        };
        const end = () => {
            el.removeEventListener("pointermove", move);
            el.removeEventListener("pointerup", end);
            el.removeEventListener("pointercancel", end);
            if (last !== start) writeRect(last);
        };
        el.addEventListener("pointermove", move);
        el.addEventListener("pointerup", end);
        el.addEventListener("pointercancel", end);
    }, []);

    const onMoveStart = (e: PointerEvent<HTMLElement>) => {
        if ((e.target as HTMLElement).closest(NO_DRAG)) return;
        track(e, (start, dx, dy) => moveRect(start, dx, dy, window.innerWidth, window.innerHeight));
    };
    const onResizeStart = (edges: IEdges) => (e: PointerEvent<HTMLElement>) => track(e, (start, dx, dy) => resizeRect(start, edges, dx, dy, window.innerWidth, window.innerHeight));

    /** Arrow keys on a handle: 16 px steps of the move, or of the bottom-right corner. */
    const onKey = (kind: "move" | "resize") => (e: KeyboardEvent<HTMLElement>) => {
        const arrow = ARROWS[e.key];
        if (!arrow) return;
        e.preventDefault();
        const [dx, dy] = [arrow[0] * KEY_STEP, arrow[1] * KEY_STEP];
        const next = kind === "move" ? moveRect(rectRef.current, dx, dy, window.innerWidth, window.innerHeight) : resizeRect(rectRef.current, { right: true, bottom: true }, dx, dy, window.innerWidth, window.innerHeight);
        setRect(next);
        writeRect(next);
    };

    return { rect, onMoveStart, onResizeStart, onKey };
}

/** The resize grips around the floating picker: four edges and four corners, each a few px of the border. */
const EDGE_HANDLES: { edges: IEdges; className: string }[] = [
    { edges: { top: true }, className: "inset-x-3 -top-1 h-2 cursor-ns-resize" },
    { edges: { bottom: true }, className: "inset-x-3 -bottom-1 h-2 cursor-ns-resize" },
    { edges: { left: true }, className: "inset-y-3 -left-1 w-2 cursor-ew-resize" },
    { edges: { right: true }, className: "inset-y-3 -right-1 w-2 cursor-ew-resize" },
    { edges: { top: true, left: true }, className: "-top-1 -left-1 size-3 cursor-nwse-resize" },
    { edges: { top: true, right: true }, className: "-top-1 -right-1 size-3 cursor-nesw-resize" },
    { edges: { bottom: true, left: true }, className: "-bottom-1 -left-1 size-3 cursor-nesw-resize" },
];

/** A single-thumb slider's value, which base-ui may report as a one-entry array. */
function sliderValue(value: number | readonly number[]): number {
    return Array.isArray(value) ? (value[0] ?? 50) : (value as number);
}

/**
 * The scroll region under the source tabs. The picker body flexes to fill it,
 * and its last child (the tile area, in both bodies) keeps at least 224 px: on
 * a short picker the search and filters scroll away above the tiles instead of
 * crushing them. Size containment keeps the tile list's own height out of the
 * body's minimum, or the body would grow to the whole list and the outer region
 * would scroll it instead of the tile grid. `EntityPickerBody` is shared with
 * the grids, so it is styled from here by structure rather than edited.
 */
const BODY_SCROLLER = "flex min-h-0 flex-1 flex-col overflow-y-auto [&>div>div:last-child]:min-h-56 [&>div>div:last-child]:contain-size [&>div]:min-h-auto";

/**
 * The owner's background picker. Every pick, crop, zoom and removal is previewed live
 * by the header behind the dialog, so the dialog has no backdrop and is not modal:
 * the page stays visible and the header's art takes drags (pan) and the wheel (zoom)
 * while it is open. From `sm` up it floats, moved by its title bar and resized by its
 * edges, opening where it was last left (else docked lower left, off the art); on a
 * phone it is a bottom sheet. Nothing is stored until Save, which sends the
 * `background` key alone; the backend keeps the stored tabs and showcase.
 */
export function BackgroundPicker({ saved, draft, artGeometry, onDraftChange, onClose }: IBackgroundPickerProps) {
    const t: TypedT<typeof messages> = useT("user");
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    const selected = useMemo(() => new Set(draft && (draft.kind === "skin" || draft.kind === "operator") ? [entityKey(draft.kind, draft.id)] : []), [draft]);
    const dirty = !sameBackground(saved, draft);
    // Opens on the source of the art the draft shows.
    const [source, setSource] = useState<ArtSource>(() => (draft && isGalleryKind(draft.kind) ? "gallery" : "characters"));
    const floating = useMediaQuery("sm");
    const { rect, onMoveStart, onResizeStart, onKey } = useFloatingRect();
    const compact = floating && rect.height < COMPACT_HEIGHT;
    const wide = floating && rect.width >= WIDE_WIDTH;
    // An axis the art does not overflow at this zoom has nothing to crop: its slider is disabled with a line saying why, never a control that moves nothing.
    const axes = draft ? croppableAxes(draft.kind, draft.scale, artGeometry) : { x: false, y: false };

    const save = useMutation({
        mutationFn: (background: ProfileBackground | null) => updateUserSettingsFn({ data: { profile_layout: { background } } }),
        onSuccess: async (_ok, background) => {
            await invalidateProfile();
            toastManager.add({ id: `profile-background-${Date.now()}`, title: background ? t("profile.background.saved.title") : t("profile.background.removed.title"), type: "success" });
            onClose();
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `profile-background-err-${Date.now()}`, title: t("profile.background.saveFailed.title"), description: describeError(err), type: "error" });
        },
    });

    const onPick = (entity: ITierEntity) => {
        if (entity.kind === "skin" || entity.kind === "operator") onDraftChange(pickBackground(draft, entity.kind, entity.id));
    };

    return (
        <Dialog open modal={false} disablePointerDismissal onOpenChange={(next) => !next && !save.isPending && onClose()}>
            <DialogPortal>
                <DialogViewport className="pointer-events-none grid-rows-[1fr_auto] max-sm:p-0 max-sm:pt-12">
                    <DialogPrimitive.Popup
                        className="pointer-events-auto relative row-start-2 flex min-h-0 w-full min-w-0 flex-col rounded-2xl border bg-popover not-dark:bg-clip-padding text-popover-foreground shadow-lg/10 outline-none transition-[opacity,translate] duration-200 ease-in-out data-ending-style:opacity-0 data-starting-style:opacity-0 motion-reduce:transition-none max-sm:h-[72dvh] max-sm:max-w-none max-sm:rounded-none max-sm:rounded-t-2xl max-sm:border-x-0 max-sm:border-b-0 max-sm:data-ending-style:translate-y-4 max-sm:data-starting-style:translate-y-4 sm:fixed sm:data-ending-style:translate-y-2 sm:data-starting-style:translate-y-2"
                        style={floating ? { left: rect.x, top: rect.y, width: rect.width, height: rect.height } : undefined}
                        data-slot="dialog-popup"
                    >
                        <DialogHeader className="pb-3 max-sm:px-4 max-sm:pt-4 max-sm:pb-3 sm:cursor-move sm:select-none sm:pe-20" onPointerDown={floating ? onMoveStart : undefined}>
                            <DialogTitle>{t("profile.background.title")}</DialogTitle>
                            <DialogDescription className={compact ? "sr-only" : "max-sm:sr-only"}>{t("profile.background.description")}</DialogDescription>
                        </DialogHeader>
                        <Tabs value={source} onValueChange={(value) => setSource(value === "gallery" ? "gallery" : "characters")} className="mb-3 px-6 max-sm:px-4">
                            <TabsList variant="underline" aria-label={t("profile.background.source")}>
                                <TabsTab value="characters">{t("profile.background.source.characters")}</TabsTab>
                                <TabsTab value="gallery">{t("profile.background.source.gallery")}</TabsTab>
                            </TabsList>
                        </Tabs>
                        <div className={BODY_SCROLLER}>
                            {source === "characters" ? (
                                <EntityPickerBody kinds={BACKGROUND_KINDS} current={null} selected={selected} onPick={onPick} />
                            ) : (
                                <GalleryPicker wide={wide} selected={draft && isGalleryKind(draft.kind) ? { kind: draft.kind, id: draft.id } : null} onPick={(kind, id) => onDraftChange(pickBackground(draft, kind, id))} />
                            )}
                        </div>
                        <div className="flex min-h-14 flex-col gap-2 border-t px-6 py-3 max-sm:px-4">
                            {draft ? (
                                <>
                                    <div className="flex flex-wrap items-start gap-x-4 gap-y-3 [&_[data-slot=slider-control]]:min-w-0">
                                        <PickerSlider label={t("profile.background.zoom")} min={SCALE_MIN} max={SCALE_MAX} value={zoomOf(draft)} onChange={(value) => onDraftChange(withScale(draft, value))}>
                                            <span className="font-sans text-muted-foreground text-xs tabular-nums">{t("profile.background.zoomValue", { scale: zoomOf(draft) })}</span>
                                        </PickerSlider>
                                        <PickerSlider label={t("profile.background.cropX")} min={0} max={100} disabled={!axes.x} value={draft.focus_x ?? 50} onChange={(value) => onDraftChange(withFocusX(draft, value))}>
                                            {!axes.x && <FitsNote title={t("profile.background.cropXFits")}>{t("profile.background.cropFitsShort")}</FitsNote>}
                                        </PickerSlider>
                                        <PickerSlider label={t("profile.background.cropY")} min={0} max={100} disabled={!axes.y} value={draft.focus_y ?? 50} onChange={(value) => onDraftChange(withFocusY(draft, value))}>
                                            {!axes.y && <FitsNote title={t("profile.background.cropYFits")}>{t("profile.background.cropFitsShort")}</FitsNote>}
                                        </PickerSlider>
                                    </div>
                                    {!compact && <p className="m-0 font-sans text-muted-foreground text-xs max-sm:hidden">{t("profile.background.panHint")}</p>}
                                </>
                            ) : (
                                <p className="m-0 font-sans text-muted-foreground text-sm">{t("profile.background.none")}</p>
                            )}
                        </div>
                        <DialogFooter className="flex-row items-center justify-between gap-2 max-sm:px-4 max-sm:py-3 sm:justify-between">
                            <Button type="button" variant="ghost" onClick={() => onDraftChange(null)} disabled={draft === null || save.isPending}>
                                <Trash2Icon />
                                {t("profile.background.remove")}
                            </Button>
                            <div className="flex gap-2">
                                <Button type="button" variant="outline" onClick={onClose} disabled={save.isPending}>
                                    {t("profile.background.cancel")}
                                </Button>
                                <Button type="button" onClick={() => save.mutate(draft)} disabled={!dirty || save.isPending}>
                                    {save.isPending && <Spinner />}
                                    {t("profile.background.save")}
                                </Button>
                            </div>
                        </DialogFooter>
                        {floating && (
                            <button
                                type="button"
                                aria-label={t("profile.background.move")}
                                title={t("profile.background.move")}
                                onPointerDown={onMoveStart}
                                onKeyDown={onKey("move")}
                                className="absolute end-11 top-2 inline-flex size-9 cursor-move items-center justify-center rounded-md text-muted-foreground outline-none hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                            >
                                <GripHorizontalIcon className="size-4" />
                            </button>
                        )}
                        <DialogPrimitive.Close aria-label={t("profile.background.cancel")} className="absolute end-2 top-2" render={<Button size="icon" variant="ghost" />} disabled={save.isPending}>
                            <XIcon />
                        </DialogPrimitive.Close>
                        {floating && (
                            <>
                                {EDGE_HANDLES.map(({ edges, className }) => (
                                    <div key={className} aria-hidden="true" onPointerDown={onResizeStart(edges)} className={`absolute touch-none ${className}`} />
                                ))}
                                <button
                                    type="button"
                                    aria-label={t("profile.background.resize")}
                                    title={t("profile.background.resize")}
                                    onPointerDown={onResizeStart({ right: true, bottom: true })}
                                    onKeyDown={onKey("resize")}
                                    className="absolute end-0 bottom-0 size-5 cursor-nwse-resize touch-none rounded-br-2xl text-muted-foreground/60 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                                >
                                    <svg viewBox="0 0 20 20" className="size-full" aria-hidden="true">
                                        <path d="M15 8 8 15M15 12l-3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none" />
                                    </svg>
                                </button>
                            </>
                        )}
                    </DialogPrimitive.Popup>
                </DialogViewport>
            </DialogPortal>
        </Dialog>
    );
}

interface IPickerSliderProps {
    label: string;
    min: number;
    max: number;
    value: number;
    /** Dimmed and inert: the axis has nothing to crop at this zoom. */
    disabled?: boolean;
    onChange: (value: number) => void;
    /** The right end of the label row: the zoom's value, or why a crop axis is off. */
    children?: ReactNode;
}

/** One of the picker's zoom and crop sliders: its label row above a whole-number track. */
function PickerSlider({ label, min, max, value, disabled, onChange, children }: IPickerSliderProps) {
    return (
        <Slider className={cn("min-w-28 flex-1", disabled && "opacity-55")} min={min} max={max} step={1} disabled={disabled} value={value} onValueChange={(next) => onChange(sliderValue(next))}>
            <div className="mb-2 flex items-baseline justify-between gap-3">
                <SliderPrimitive.Label className="font-medium font-sans text-sm">{label}</SliderPrimitive.Label>
                {children}
            </div>
        </Slider>
    );
}

/** The short line beside a disabled crop slider, its full reason as the tooltip. */
function FitsNote({ title, children }: { title: string; children: ReactNode }) {
    return (
        <span className="truncate font-sans text-muted-foreground text-xs" title={title}>
            {children}
        </span>
    );
}
