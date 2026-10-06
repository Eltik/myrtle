import { Link } from "@tanstack/react-router";
import { ArrowUpRightIcon, ChevronLeftIcon, ChevronRightIcon, XIcon } from "lucide-react";
import { type KeyboardEvent, type PointerEvent, useRef } from "react";
import { entityAccent } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Dialog, DialogBackdrop, DialogDescription, DialogPortal, DialogPrimitive, DialogTitle } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { CellHeroArt } from "./CellHeroArt";
import { entityLink, neighbourIndex, swipeIntent } from "./cellViewer";
import type { messages } from "./GridCellViewer.messages";
import { cellPosition } from "./shared";
import type { IGridEditCell } from "./state";

// A read-only board's cell, full screen: its art large, its whole label, what
// the pick is and a link to its page. Prev and next (buttons, arrow keys, a
// sideways swipe) step through the board's viewable cells in board order and
// stop at either end; Escape, the X or a swipe down closes it. Dark in both
// themes, like the board it opens from.

type ViewerT = TypedT<typeof messages>;

interface IGridCellViewerProps {
    cells: readonly IGridEditCell[];
    cols: number;
    /** The viewable cells' indices, in board order: what prev and next step through. */
    order: readonly number[];
    /** The cell shown, `null` when closed. */
    index: number | null;
    onIndexChange: (index: number) => void;
    onClose: () => void;
    /** The cell's own button on the board, which takes focus back on close. */
    returnFocus: (index: number) => HTMLElement | null;
}

const NAV_BUTTON = "inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full bg-white/8 text-white transition-colors hover:bg-white/16 focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-2 disabled:cursor-default disabled:opacity-30 disabled:hover:bg-white/8";

export function GridCellViewer({ cells, cols, order, index, onIndexChange, onClose, returnFocus }: IGridCellViewerProps) {
    const t: ViewerT = useT("grids");
    // What stays drawn while the popup animates out after `index` went null.
    const lastShown = useRef<number | null>(null);
    if (index !== null) lastShown.current = index;
    const shown = index ?? lastShown.current;
    const cell = shown === null ? undefined : cells[shown];

    const prev = shown === null ? null : neighbourIndex(order, shown, -1);
    const next = shown === null ? null : neighbourIndex(order, shown, 1);
    const go = (target: number | null) => {
        if (target !== null) onIndexChange(target);
    };

    const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
        if (e.key === "ArrowLeft") {
            e.preventDefault();
            go(prev);
        } else if (e.key === "ArrowRight") {
            e.preventDefault();
            go(next);
        }
    };

    // A finger's swipe, read once it lifts; a mouse drag does nothing.
    const swipeFrom = useRef<{ x: number; y: number; id: number } | null>(null);
    const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
        if (e.pointerType !== "touch" || !e.isPrimary) return;
        swipeFrom.current = { x: e.clientX, y: e.clientY, id: e.pointerId };
    };
    const onPointerUp = (e: PointerEvent<HTMLDivElement>) => {
        const from = swipeFrom.current;
        swipeFrom.current = null;
        if (!from || from.id !== e.pointerId) return;
        const intent = swipeIntent(e.clientX - from.x, e.clientY - from.y);
        if (intent === "prev") go(prev);
        else if (intent === "next") go(next);
        else if (intent === "close") onClose();
    };

    return (
        <Dialog open={index !== null} onOpenChange={(open) => !open && onClose()}>
            <DialogPortal>
                <DialogBackdrop className="bg-black/70" />
                <DialogPrimitive.Popup
                    className="fixed inset-0 z-50 flex flex-col bg-[oklch(0.13_0.004_285)] text-white outline-none transition-[opacity,translate] duration-200 ease-out data-ending-style:translate-y-3 data-starting-style:translate-y-3 data-ending-style:opacity-0 data-starting-style:opacity-0 motion-reduce:transition-none"
                    finalFocus={() => (lastShown.current === null ? true : (returnFocus(lastShown.current) ?? true))}
                    onKeyDown={onKeyDown}
                >
                    {cell && shown !== null && <ViewerContent t={t} cell={cell} index={shown} cols={cols} order={order} prev={prev} next={next} onGo={go} onPointerDown={onPointerDown} onPointerUp={onPointerUp} onPointerCancel={() => (swipeFrom.current = null)} />}
                </DialogPrimitive.Popup>
            </DialogPortal>
        </Dialog>
    );
}

interface IViewerContentProps {
    t: ViewerT;
    cell: IGridEditCell;
    index: number;
    cols: number;
    order: readonly number[];
    prev: number | null;
    next: number | null;
    onGo: (index: number | null) => void;
    onPointerDown: (e: PointerEvent<HTMLDivElement>) => void;
    onPointerUp: (e: PointerEvent<HTMLDivElement>) => void;
    onPointerCancel: () => void;
}

function ViewerContent({ t, cell, index, cols, order, prev, next, onGo, onPointerDown, onPointerUp, onPointerCancel }: IViewerContentProps) {
    const labels = useEntityLabels();
    const position = cellPosition(index, cols);
    const entity = cell.entity;
    const label = cell.label.trim();
    const link = entityLink(entity);
    const detail = entity ? labels.detail(entity) : [];
    const accent = entity ? entityAccent(entity) : null;
    const place = order.indexOf(index);

    const prevButton = (
        <button type="button" className={NAV_BUTTON} onClick={() => onGo(prev)} disabled={prev === null} aria-label={t("viewer.prev")} title={t("viewer.prev")}>
            <ChevronLeftIcon className="size-5" aria-hidden="true" />
        </button>
    );
    const nextButton = (
        <button type="button" className={NAV_BUTTON} onClick={() => onGo(next)} disabled={next === null} aria-label={t("viewer.next")} title={t("viewer.next")}>
            <ChevronRightIcon className="size-5" aria-hidden="true" />
        </button>
    );

    return (
        <>
            <div className="flex items-center gap-3 px-4 pt-[max(0.75rem,env(safe-area-inset-top,0px))] pb-2 sm:px-6 sm:pt-4">
                <div className="flex min-w-0 flex-1 items-baseline gap-2.5 font-mono text-[11px] text-white/60 uppercase tracking-[0.14em]">
                    <span className="truncate">{t("viewer.position", position)}</span>
                    {place !== -1 && (
                        <>
                            <span aria-hidden="true" className="text-white/30">
                                /
                            </span>
                            <span className="shrink-0 tabular-nums">{t("viewer.count", { n: place + 1, total: order.length })}</span>
                        </>
                    )}
                </div>
                <DialogPrimitive.Close className={NAV_BUTTON} aria-label={t("viewer.close")} title={t("viewer.close")}>
                    <XIcon className="size-5" aria-hidden="true" />
                </DialogPrimitive.Close>
            </div>

            {/* The stage takes the swipes, so the browser must not pan it. */}
            <div className="relative flex min-h-0 flex-1 touch-none items-center justify-center px-4 sm:px-20" onPointerDown={onPointerDown} onPointerUp={onPointerUp} onPointerCancel={onPointerCancel}>
                {accent && <div aria-hidden="true" className="pointer-events-none absolute inset-0 opacity-35" style={{ background: `radial-gradient(ellipse 55% 60% at 50% 55%, ${accent}, transparent 70%)` }} />}
                {entity ? (
                    <div className="relative h-full w-full max-w-[1100px] py-2">
                        <CellHeroArt entity={entity} server={cell.server} size="viewer" />
                    </div>
                ) : (
                    // A label with no pick: the label is the whole cell, so it takes the stage.
                    <p className="m-0 max-w-[22ch] text-balance text-center font-extrabold font-sans text-3xl leading-tight tracking-tight [overflow-wrap:anywhere] sm:text-5xl">{label}</p>
                )}
                <div className="pointer-events-none absolute inset-y-0 right-4 left-4 hidden items-center justify-between sm:flex [&>*]:pointer-events-auto">
                    {prevButton}
                    {nextButton}
                </div>
            </div>

            <div className="mx-auto flex w-full max-w-2xl flex-col items-center gap-2 px-5 pt-3 pb-3 text-center" aria-live="polite">
                <DialogTitle className={cn("m-0 font-extrabold font-sans text-2xl leading-tight tracking-tight [overflow-wrap:anywhere] sm:text-3xl", !entity && "sr-only")}>{label || (entity ? labels.tileLabel(entity) : t("viewer.untitledCell", position))}</DialogTitle>
                <DialogDescription className="sr-only">{t("viewer.hint")}</DialogDescription>
                {entity ? (
                    <div className="flex flex-col items-center gap-1">
                        {entity.resolved ? (
                            <>
                                <span className="font-bold font-mono text-[10.5px] text-white/55 uppercase tracking-[0.16em]">{[labels.singular(entity.kind), ...detail].join(" · ")}</span>
                                {label && <span className="font-sans font-semibold text-base text-white/90 leading-snug sm:text-lg">{entity.name}</span>}
                            </>
                        ) : (
                            <span className="font-sans text-sm text-white/60">
                                {t("viewer.unknown")} <span className="font-mono">{entity.id}</span>
                            </span>
                        )}
                        {link && (
                            <Link
                                to={link.to}
                                params={{ id: link.id }}
                                className="mt-1 inline-flex min-h-11 items-center gap-1.5 rounded-full px-4 font-medium font-sans text-sm text-white underline-offset-4 ring-1 ring-white/20 transition-colors hover:bg-white/10 focus-visible:outline-2 focus-visible:outline-ring sm:min-h-9"
                            >
                                {t("viewer.openPage")}
                                <ArrowUpRightIcon className="size-4" aria-hidden="true" />
                            </Link>
                        )}
                    </div>
                ) : (
                    <span className="font-sans text-sm text-white/50">{t("viewer.noPick")}</span>
                )}
            </div>

            <div className="flex items-center justify-between gap-3 px-4 pt-1 pb-[max(1rem,env(safe-area-inset-bottom,0px))] sm:hidden">
                {prevButton}
                {nextButton}
            </div>
        </>
    );
}
