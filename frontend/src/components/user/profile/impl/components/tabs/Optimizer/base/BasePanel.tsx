import { Maximize2, Minus, Plus, RotateCcw, Sparkles } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "#/components/ui/dialog";
import { TooltipProvider } from "#/components/ui/tooltip";
import type { IBoard } from "#/lib/base/board";
import { useBaseOptimizer } from "./base-context";
import { Board } from "./board/Board";
import styles from "./board/Board.module.css";
import { Headline } from "./controls/Headline";
import { PromotionToggle } from "./controls/PromotionToggle";
import { ShiftStrip } from "./controls/ShiftStrip";
import { SustainabilityBadge } from "./controls/SustainabilityBadge";
import { DeepDive } from "./stats/DeepDive";
import { StatsForNerds } from "./stats/StatsForNerds";

export function BasePanel({ board }: { board: IBoard }) {
    const [boardFullscreen, setBoardFullscreen] = useState(false);
    const boardScrollRef = useRef<HTMLDivElement>(null);
    const api = useBaseOptimizer();
    const totals = api.evaluation?.assignment;
    const power = api.evaluation?.power;
    const sustainability = api.rotation?.rotation.sustainability;
    const planned = api.proposal !== null;

    useLayoutEffect(() => {
        const frame = requestAnimationFrame(() => centerBoardScroll(boardScrollRef.current));
        return () => cancelAnimationFrame(frame);
    }, []);

    return (
        <TooltipProvider closeDelay={0} delay={350}>
            <div className="flex flex-col gap-3">
                <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
                    <div className="flex flex-wrap gap-6">
                        <Headline hint="Production rooms" label="Efficiency" value={totals ? `${Math.round(totals.total_production_efficiency)}%` : "-"} />
                        <Headline label="LMD / day" value={totals ? Math.round(totals.yield_lmd_per_day).toLocaleString() : "-"} />
                        <Headline label="EXP / day" value={totals ? Math.round(totals.yield_exp_per_day).toLocaleString() : "-"} />
                        <Headline hint={power ? `${power.generated} generated · ${power.consumed} drawn` : undefined} label="Power" value={power ? `${power.net > 0 ? "+" : ""}${power.net}` : "-"} />
                    </div>

                    <div className="flex items-center gap-2">
                        {(api.dirty || planned) && (
                            <Button onClick={api.reset} size="sm" variant="ghost">
                                <RotateCcw />
                                Reset to my base
                            </Button>
                        )}
                        <Button disabled={api.optimizing} onClick={() => api.runOptimize([])} size="sm">
                            <Sparkles />
                            {api.optimizing ? "Optimizing…" : planned ? "Re-optimize" : "Optimize"}
                        </Button>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="flex flex-wrap items-center gap-4">
                        <ShiftStrip />
                        <PromotionToggle />
                    </div>
                    {planned && sustainability && <SustainabilityBadge depletedCount={sustainability.depleted.length} dormOverflow={sustainability.dorm_overflow} horizonHours={sustainability.horizon_hours} verdict={sustainability.verdict} />}
                </div>

                {api.evaluationError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">This plan could not be scored: {api.evaluationError.message}</p>}
                {api.optimizeError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">The optimizer failed: {api.optimizeError.message}</p>}

                <div className="relative rounded-xl border border-border bg-card p-3">
                    <Button aria-label="Open board in full screen" className="absolute top-5 right-5 z-10" onClick={() => setBoardFullscreen(true)} size="icon-sm" variant="outline">
                        <Maximize2 />
                    </Button>
                    <div className={styles["riic-board-scroll"]} ref={boardScrollRef}>
                        <Board board={board} />
                    </div>
                </div>

                <Dialog open={boardFullscreen} onOpenChange={setBoardFullscreen}>
                    <DialogContent bottomStickOnMobile={false} className="flex h-[95vh] max-h-[95vh] w-[95vw] max-w-[95vw] flex-col overflow-hidden p-0 sm:max-w-[95vw]" showCloseButton>
                        <BoardFullscreen board={board} />
                    </DialogContent>
                </Dialog>

                <DeepDive />

                <StatsForNerds />
            </div>
        </TooltipProvider>
    );
}

function BoardFullscreen({ board }: { board: IBoard }) {
    const scrollRef = useRef<HTMLDivElement>(null);
    const dragRef = useRef({ active: false, x: 0, y: 0 });
    const [zoom, setZoom] = useState(1);

    useLayoutEffect(() => {
        const frame = requestAnimationFrame(() => centerBoardScroll(scrollRef.current));
        return () => cancelAnimationFrame(frame);
    }, []);

    const changeZoom = (delta: number) => setZoom((value) => Math.min(2.5, Math.max(0.5, Number((value + delta).toFixed(2)))));
    const resetView = () => {
        setZoom(1);
        requestAnimationFrame(() => centerBoardScroll(scrollRef.current));
    };

    return (
        <>
            <DialogTitle className="sr-only">RIIC base board</DialogTitle>
            <div className="relative min-h-0 flex-1">
                <div className="absolute top-3 left-3 z-10 flex items-center gap-1 rounded-lg border border-border/60 bg-background/80 p-1 shadow-sm backdrop-blur-sm">
                    <Button aria-label="Zoom out" onClick={() => changeZoom(-0.1)} size="icon-sm" variant="outline">
                        <Minus />
                    </Button>
                    <span className="min-w-12 text-center font-mono text-xs tabular-nums">{Math.round(zoom * 100)}%</span>
                    <Button aria-label="Zoom in" onClick={() => changeZoom(0.1)} size="icon-sm" variant="outline">
                        <Plus />
                    </Button>
                    <Button aria-label="Reset board view" onClick={resetView} size="icon-sm" variant="ghost">
                        <RotateCcw />
                    </Button>
                </div>
                <div
                    className={`${styles["riic-board-scroll"]} size-full cursor-grab touch-none select-none active:cursor-grabbing`}
                    onPointerDown={(event) => {
                        if (event.button !== 0) return;
                        dragRef.current = { active: true, x: event.clientX, y: event.clientY };
                        event.currentTarget.setPointerCapture(event.pointerId);
                    }}
                    onPointerMove={(event) => {
                        if (!dragRef.current.active || !scrollRef.current) return;
                        const dx = dragRef.current.x - event.clientX;
                        const dy = dragRef.current.y - event.clientY;
                        dragRef.current = { active: true, x: event.clientX, y: event.clientY };
                        scrollRef.current.scrollLeft += dx;
                        scrollRef.current.scrollTop += dy;
                    }}
                    onPointerUp={(event) => {
                        dragRef.current.active = false;
                        event.currentTarget.releasePointerCapture(event.pointerId);
                    }}
                    onWheel={(event) => {
                        if (!event.ctrlKey && !event.metaKey) return;
                        event.preventDefault();
                        changeZoom(event.deltaY > 0 ? -0.1 : 0.1);
                    }}
                    ref={scrollRef}
                >
                    <div className={styles["riic-board-pan-canvas"]} style={{ zoom }}>
                        <Board board={board} />
                    </div>
                </div>
            </div>
        </>
    );
}

function centerBoardScroll(element: HTMLDivElement | null) {
    if (!element) return;
    element.scrollTo({
        left: Math.max(0, (element.scrollWidth - element.clientWidth) / 2),
        top: Math.max(0, (element.scrollHeight - element.clientHeight) / 2),
    });

    const controlCenter = element.querySelector<HTMLElement>('[data-facility-type="CONTROL"]');
    if (!controlCenter) return;
    const viewportRect = element.getBoundingClientRect();
    const controlRect = controlCenter.getBoundingClientRect();
    element.scrollLeft = Math.max(0, element.scrollLeft + controlRect.left + controlRect.width / 2 - (viewportRect.left + viewportRect.width / 2));
}
