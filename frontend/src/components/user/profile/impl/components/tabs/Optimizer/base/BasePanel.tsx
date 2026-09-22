import { Maximize2, Minus, Plus, RotateCcw, Sparkles } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "#/components/ui/dialog";
import { TooltipProvider } from "#/components/ui/tooltip";
import type { IBoard } from "#/lib/base/board";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./BasePanel.messages";
import { useBaseOptimizer } from "./base-context";
import { Board } from "./board/Board";
import styles from "./board/Board.module.css";
import { AccountFacts } from "./controls/AccountFacts";
import { Headline } from "./controls/Headline";
import { PromotionToggle } from "./controls/PromotionToggle";
import { ShiftStrip } from "./controls/ShiftStrip";
import { SustainabilityBadge } from "./controls/SustainabilityBadge";
import { DeepDive } from "./stats/DeepDive";
import { StatsForNerds } from "./stats/StatsForNerds";

type PanelT = TypedT<typeof messages>;

export function BasePanel({ board }: { board: IBoard }) {
    const t: PanelT = useT("user");
    const f = useFormatters();
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
                <p className="rounded-xl border border-amber-500/40 bg-amber-500/10 px-4 py-2 text-[12px] text-foreground">
                    {t("profile.base.wip")}{" "}
                    <a className="underline underline-offset-2" href="/discord" rel="noreferrer" target="_blank">
                        {t("profile.base.wip.link")}
                    </a>
                </p>
                <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
                    <div className="flex flex-wrap gap-6">
                        <Headline hint={t("profile.base.headline.efficiency.hint")} label={t("profile.base.headline.efficiency")} value={totals ? `${Math.round(totals.total_production_efficiency)}%` : "-"} />
                        <Headline label={t("profile.base.headline.lmd")} value={totals ? f.number(Math.round(totals.yield_lmd_per_day)) : "-"} />
                        <Headline label={t("profile.base.headline.exp")} value={totals ? f.number(Math.round(totals.yield_exp_per_day)) : "-"} />
                        <Headline hint={power ? t("profile.base.headline.power.hint", { generated: power.generated, consumed: power.consumed }) : undefined} label={t("profile.base.headline.power")} value={power ? `${power.net > 0 ? "+" : ""}${power.net}` : "-"} />
                    </div>

                    <div className="flex items-center gap-2">
                        {(api.dirty || planned) && (
                            <Button onClick={api.reset} size="sm" variant="ghost">
                                <RotateCcw />
                                {t("profile.base.reset")}
                            </Button>
                        )}
                        <Button disabled={api.optimizing} onClick={() => api.runOptimize([])} size="sm">
                            <Sparkles />
                            {api.optimizing ? t("profile.base.optimizing") : planned ? t("profile.base.reoptimize") : t("profile.base.optimize")}
                        </Button>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="flex flex-wrap items-center gap-4">
                        <ShiftStrip />
                        <PromotionToggle />
                        <AccountFacts />
                    </div>
                    {planned && sustainability && <SustainabilityBadge depletedCount={sustainability.depleted.length} dormOverflow={sustainability.dorm_overflow} horizonHours={sustainability.horizon_hours} verdict={sustainability.verdict} />}
                </div>

                {api.evaluationError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">{t("profile.base.evaluationError", { error: api.evaluationError.message })}</p>}
                {api.optimizeError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">{t("profile.base.optimizeError", { error: api.optimizeError.message })}</p>}

                <div className="relative rounded-xl border border-border bg-card p-3">
                    <Button aria-label={t("profile.base.fullscreen")} className="absolute top-5 right-5 z-10" onClick={() => setBoardFullscreen(true)} size="icon-sm" variant="outline">
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
    const t: PanelT = useT("user");
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
            <DialogTitle className="sr-only">{t("profile.base.boardTitle")}</DialogTitle>
            <div className="relative min-h-0 flex-1">
                <div className="absolute top-3 left-3 z-10 flex items-center gap-1 rounded-lg border border-border/60 bg-background/80 p-1 shadow-sm backdrop-blur-sm">
                    <Button aria-label={t("profile.base.zoomOut")} onClick={() => changeZoom(-0.1)} size="icon-sm" variant="outline">
                        <Minus />
                    </Button>
                    <span className="min-w-12 text-center font-mono text-xs tabular-nums">{Math.round(zoom * 100)}%</span>
                    <Button aria-label={t("profile.base.zoomIn")} onClick={() => changeZoom(0.1)} size="icon-sm" variant="outline">
                        <Plus />
                    </Button>
                    <Button aria-label={t("profile.base.resetView")} onClick={resetView} size="icon-sm" variant="ghost">
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
