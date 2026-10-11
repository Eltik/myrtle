import { Minus, Plus } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Cut } from "#/types/generated/Cut";
import type { DpSource } from "#/types/generated/DpSource";
import type { Economy } from "#/types/generated/Economy";
import type { EconomyNote } from "#/types/generated/EconomyNote";
import type { EconomySeat } from "#/types/generated/EconomySeat";
import type { IStageContext } from "../model";
import type { messages } from "./EconomyPanel.messages";
import { baseDpFromLine, clampStartDp, cutAmountText, cutLabelInput, dpFromPointer, fractionOf, ladderScale, respawnRange, START_DP_MAX, START_DP_MIN, squadRespawnCut, startBonusTotal } from "./economyMath";
import { type IChipContext, opName, PanelShell, type PanelState, SectionHeading, StatCell, StatStrip, sourceLabel } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

type EconomyT = TypedT<typeof messages & typeof sharedMessages>;

interface IEconomyPanelProps {
    state: PanelState;
    stale: boolean;
    collapsed: boolean;
    onToggle: () => void;
    economy: Economy | undefined;
    startDp: number;
    onStartDpChange: (startDp: number) => void;
    startDpFrom: { kind: "stage"; code: string } | { kind: "page" };
    stage: IStageContext | null;
    hasSupport: boolean;
    ctx: IChipContext;
}

export function EconomyPanel({ state, stale, collapsed, onToggle, economy, startDp, onStartDpChange, startDpFrom, stage, hasSupport, ctx }: IEconomyPanelProps): React.ReactElement {
    const t: EconomyT = useT("tools");
    return (
        <PanelShell title={t("squad.analysis.economy.title")} question={t("squad.analysis.economy.question")} state={state} stale={stale} collapsed={collapsed} onToggle={onToggle}>
            {economy && <EconomyBody economy={economy} startDp={startDp} onStartDpChange={onStartDpChange} startDpFrom={startDpFrom} stage={stage} hasSupport={hasSupport} ctx={ctx} t={t} />}
        </PanelShell>
    );
}

interface IEconomyBodyProps extends Pick<IEconomyPanelProps, "startDp" | "onStartDpChange" | "startDpFrom" | "stage" | "hasSupport" | "ctx"> {
    economy: Economy;
    t: EconomyT;
}

function EconomyBody({ economy, startDp, onStartDpChange, startDpFrom, stage, hasSupport, ctx, t }: IEconomyBodyProps): React.ReactElement {
    const f = useFormatters();
    // The page's number moves at once; the response's bonus comes from the last answer.
    const bonus = startBonusTotal(economy.start_bonus);
    const lineDp = startDp + bonus;
    const range = respawnRange(economy.seats.map((s) => s.respawn));
    const squadCut = squadRespawnCut(economy.seats);
    const giverOf = (cut: Cut): string | null => {
        if (cut.from_seat === null) return null;
        const seat = economy.seats.find((s) => s.seat === cut.from_seat);
        return seat ? opName(ctx.operators, seat.char_id) : null;
    };
    const cheapestCaption = startDpFrom.kind === "stage" && economy.deploy_limit !== null ? t("squad.analysis.economy.cheapest.stage", { code: startDpFrom.code }) : t("squad.analysis.economy.cheapest.none");
    const notes = [...economy.counted, ...economy.notes];

    return (
        <div className="space-y-5">
            <StatStrip>
                <StatCell
                    label={t("squad.analysis.economy.startDp")}
                    value={<DpStepper value={startDp} onChange={onStartDpChange} t={t} />}
                    caption={startDpFrom.kind === "stage" ? t("squad.analysis.economy.startDp.from", { code: startDpFrom.code }) : t("squad.analysis.economy.startDp.page")}
                    extra={
                        economy.start_bonus.length > 0 && (
                            <div className="mt-0.5 space-y-0.5">
                                {economy.start_bonus.map((b) => (
                                    <div key={`${b.seat}-${b.name}`} className="text-[11px] text-primary leading-snug" title={b.name}>
                                        {t("squad.analysis.economy.startDp.bonus", { amount: f.number(b.amount), name: opName(ctx.operators, b.char_id) })}
                                    </div>
                                ))}
                            </div>
                        )
                    }
                />
                <StatCell label={t("squad.analysis.economy.total", { count: economy.seats.length })} value={f.number(economy.total)} caption={t("squad.analysis.economy.total.caption")} />
                <StatCell label={t("squad.analysis.economy.cheapest", { count: economy.cheapest_count })} value={f.number(economy.cheapest)} caption={cheapestCaption} />
                <StatCell
                    label={t("squad.analysis.economy.redeploy")}
                    value={range === null ? "-" : range.min === range.max ? t("squad.analysis.economy.redeploy.one", { value: f.number(range.min) }) : t("squad.analysis.economy.redeploy.range", { min: f.number(range.min), max: f.number(range.max) })}
                    caption={squadCut ? t("squad.analysis.economy.redeploy.cut", { name: giverOf(squadCut) ?? squadCut.name ?? "", amount: cutAmountText(squadCut) }) : t("squad.analysis.economy.redeploy.noCut")}
                />
            </StatStrip>

            <section>
                <SectionHeading title={t("squad.analysis.economy.ladder")} />
                <CostLadder seats={economy.seats} lineDp={lineDp} bonus={bonus} onStartDpChange={onStartDpChange} giverOf={giverOf} ctx={ctx} t={t} />
            </section>

            <section>
                <SectionHeading title={t("squad.analysis.economy.sources")} hint={t("squad.analysis.economy.sources.hint")} />
                {economy.sources.length === 0 ? (
                    <p className="text-[13px] text-muted-foreground">{t("squad.analysis.economy.sources.empty")}</p>
                ) : (
                    <ul className="divide-y divide-border/60 rounded-xl border border-border bg-card">
                        {economy.sources.map((s) => (
                            <li key={`${s.seat}-${s.source.kind}-${s.source.name ?? ""}`} className="flex items-start gap-3 px-3 py-2" onMouseEnter={() => ctx.onHoverSeat(s.seat)} onMouseLeave={() => ctx.onHoverSeat(null)}>
                                <div className="min-w-0 flex-1">
                                    <div className="font-medium text-[13px] text-foreground">{t("squad.analysis.economy.sources.who", { name: opName(ctx.operators, s.char_id), source: sourceLabel(s.source, t) })}</div>
                                    <div className="text-[12px] text-muted-foreground leading-snug">{sourceSentence(s, t, f.number)}</div>
                                </div>
                                <span className="shrink-0 font-mono text-[13px] text-foreground tabular-nums">{s.per_kill ? t("squad.analysis.economy.sources.perKillShort") : t("squad.analysis.economy.sources.plus", { amount: f.number(s.amount) })}</span>
                            </li>
                        ))}
                    </ul>
                )}
            </section>

            <footer className="space-y-1 border-border border-t pt-3 text-[12px] text-muted-foreground leading-relaxed">
                {notes.map((n) => (
                    <p key={`${n.kind}-${n.seat}-${n.name}`}>{noteSentence(n, opName(ctx.operators, n.char_id), t, f.number)}</p>
                ))}
                <p>{startDpFrom.kind === "stage" || stage ? t("squad.analysis.economy.footer.stage", { code: startDpFrom.kind === "stage" ? startDpFrom.code : (stage?.code ?? "") }) : t("squad.analysis.economy.footer.page")}</p>
                {hasSupport && <p>{t("squad.analysis.economy.footer.support")}</p>}
            </footer>
        </div>
    );
}

// ---------------------------------------------------------------- stepper

function DpStepper({ value, onChange, t }: { value: number; onChange: (n: number) => void; t: EconomyT }): React.ReactElement {
    const [draft, setDraft] = React.useState(String(value));
    React.useEffect(() => setDraft(String(value)), [value]);
    const commit = (raw: string) => {
        const n = Number.parseInt(raw, 10);
        if (Number.isNaN(n)) setDraft(String(value));
        else onChange(clampStartDp(n));
    };
    return (
        <span className="flex items-center gap-1">
            <Button variant="outline" size="icon-xs" aria-label={t("squad.analysis.economy.startDp.less")} disabled={value <= START_DP_MIN} onClick={() => onChange(clampStartDp(value - 1))}>
                <Minus />
            </Button>
            <input
                type="number"
                inputMode="numeric"
                min={START_DP_MIN}
                max={START_DP_MAX}
                value={draft}
                aria-label={t("squad.analysis.economy.startDp.input")}
                onChange={(e) => {
                    setDraft(e.target.value);
                    const n = Number.parseInt(e.target.value, 10);
                    if (!Number.isNaN(n)) onChange(clampStartDp(n));
                }}
                onBlur={(e) => commit(e.target.value)}
                className="h-7 pointer-coarse:h-11 w-11 rounded-md border border-input bg-background px-1 text-center font-mono font-semibold text-[16px] tabular-nums outline-none [appearance:textfield] focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
            />
            <Button variant="outline" size="icon-xs" aria-label={t("squad.analysis.economy.startDp.more")} disabled={value >= START_DP_MAX} onClick={() => onChange(clampStartDp(value + 1))}>
                <Plus />
            </Button>
        </span>
    );
}

// ----------------------------------------------------------------- ladder

interface ICostLadderProps {
    seats: EconomySeat[];
    lineDp: number;
    bonus: number;
    onStartDpChange: (n: number) => void;
    giverOf: (cut: Cut) => string | null;
    ctx: IChipContext;
    t: EconomyT;
}

function CostLadder({ seats, lineDp, bonus, onStartDpChange, giverOf, ctx, t }: ICostLadderProps): React.ReactElement {
    const f = useFormatters();
    const trackRef = React.useRef<HTMLDivElement>(null);
    const scale = ladderScale(seats.map((s) => s.cost));
    const linePct = fractionOf(lineDp, scale) * 100;

    const moveTo = (clientX: number) => {
        const el = trackRef.current;
        if (!el) return;
        const rect = el.getBoundingClientRect();
        onStartDpChange(baseDpFromLine(dpFromPointer(clientX - rect.left, rect.width, scale), bonus));
    };
    const onKeyDown = (e: React.KeyboardEvent) => {
        const step = e.key === "ArrowRight" || e.key === "ArrowUp" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowDown" ? -1 : 0;
        if (step === 0) return;
        e.preventDefault();
        onStartDpChange(baseDpFromLine(lineDp + step, bonus));
    };

    const n = seats.length;
    // Row 1 holds the line's label, rows 2..n+1 the bars, the last row the handle.
    const gridTemplateRows = `1.25rem repeat(${n}, minmax(1.75rem, auto)) 1.25rem`;
    const labelShift = linePct > 80 ? "-translate-x-full" : linePct < 20 ? "translate-x-0" : "-translate-x-1/2";

    return (
        <div className="grid grid-cols-[minmax(0,8.5rem)_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1" style={{ gridTemplateRows }}>
            {seats.map((s, i) => {
                const name = opName(ctx.operators, s.char_id);
                const open = s.cost <= lineDp;
                const row = i + 2;
                const hover = { onMouseEnter: () => ctx.onHoverSeat(s.seat), onMouseLeave: () => ctx.onHoverSeat(null) };
                return (
                    <React.Fragment key={`${s.seat}-${s.char_id}`}>
                        <div className="flex min-w-0 items-center gap-1.5" style={{ gridRow: row, gridColumn: 1 }} {...hover}>
                            <span aria-hidden="true" className="inline-flex size-6 shrink-0 items-center justify-center overflow-hidden rounded bg-muted text-[9px]">
                                <OperatorAvatar charId={s.char_id} name={name} />
                            </span>
                            <span className="line-clamp-2 text-[12.5px] text-foreground leading-tight" title={name}>
                                {name}
                            </span>
                        </div>
                        <div className="relative h-full" style={{ gridRow: row, gridColumn: 2 }} {...hover}>
                            <Tooltip>
                                <TooltipTrigger
                                    render={(p) => (
                                        <div {...p} className="flex h-full items-center">
                                            <div className={cn("h-3 rounded-sm transition-colors", open ? "bg-primary/75" : "bg-muted-foreground/25")} style={{ width: `${Math.max(fractionOf(s.cost, scale) * 100, 2)}%` }} />
                                        </div>
                                    )}
                                />
                                <TooltipPopup className="max-w-64">
                                    {s.cuts.length === 0 ? (
                                        t("squad.analysis.economy.ladder.noCuts")
                                    ) : (
                                        <ul className="space-y-0.5">
                                            {s.cuts.map((c) => (
                                                <li key={`${c.kind}-${c.name ?? ""}-${c.from_seat ?? ""}-${c.amount}-${c.ratio ?? ""}`}>{cutLabel(c, giverOf(c), t)}</li>
                                            ))}
                                        </ul>
                                    )}
                                </TooltipPopup>
                            </Tooltip>
                        </div>
                        <div className="flex items-baseline justify-end gap-1 font-mono tabular-nums" style={{ gridRow: row, gridColumn: 3 }} {...hover}>
                            {s.base_cost !== s.cost && <span className="text-[11px] text-muted-foreground/70 line-through">{f.number(s.base_cost)}</span>}
                            <span className={cn("text-[13px]", open ? "text-primary" : "text-foreground")}>{f.number(s.cost)}</span>
                        </div>
                    </React.Fragment>
                );
            })}
            <div ref={trackRef} className="pointer-events-none relative z-10 self-stretch" style={{ gridColumn: 2, gridRow: `1 / ${n + 3}` }}>
                <div className="absolute top-5 bottom-3 w-px bg-primary" style={{ left: `${linePct}%` }} />
                <div className={cn("absolute top-0 whitespace-nowrap font-medium text-[11px] text-primary", labelShift)} style={{ left: `${linePct}%` }}>
                    {t("squad.analysis.economy.ladder.line", { dp: f.number(lineDp) })}
                </div>
                <div
                    role="slider"
                    tabIndex={0}
                    aria-label={t("squad.analysis.economy.ladder.drag")}
                    aria-valuemin={bonus}
                    aria-valuemax={START_DP_MAX + bonus}
                    aria-valuenow={lineDp}
                    title={t("squad.analysis.economy.ladder.drag")}
                    onPointerDown={(e) => {
                        e.currentTarget.setPointerCapture(e.pointerId);
                        moveTo(e.clientX);
                    }}
                    onPointerMove={(e) => {
                        if (e.currentTarget.hasPointerCapture(e.pointerId)) moveTo(e.clientX);
                    }}
                    onKeyDown={onKeyDown}
                    className="pointer-events-auto absolute bottom-0 size-4 -translate-x-1/2 cursor-ew-resize touch-none rounded-full border-2 border-background bg-primary shadow-sm outline-none pointer-coarse:after:absolute pointer-coarse:after:-inset-4 pointer-coarse:after:rounded-full focus-visible:ring-2 focus-visible:ring-ring"
                    style={{ left: `${linePct}%` }}
                />
            </div>
        </div>
    );
}

// ---------------------------------------------------------------- text

function cutLabel(cut: Cut, giver: string | null, t: EconomyT): string {
    const input = cutLabelInput(cut, giver);
    switch (input.key) {
        case "potential":
            return t("squad.analysis.economy.cut.potential", { amount: input.amount });
        case "talent":
            return t("squad.analysis.economy.cut.talent", { name: input.name, amount: input.amount });
        case "module":
            return t("squad.analysis.economy.cut.module", { name: input.name, amount: input.amount });
        case "squad":
            return t("squad.analysis.economy.cut.squad", { giver: input.giver, name: input.name, amount: input.amount });
    }
}

function sourceSentence(s: DpSource, t: EconomyT, num: (n: number) => string): string {
    const amount = num(s.amount);
    if (s.per_kill) return s.refund ? t("squad.analysis.economy.sources.perKillRefund", { amount }) : t("squad.analysis.economy.sources.perKill", { amount });
    if (s.source.kind === "skill") {
        let out = s.duration !== null ? t("squad.analysis.economy.sources.overDuration", { amount, duration: num(s.duration) }) : t("squad.analysis.economy.sources.flat", { amount });
        if (s.first_ready !== null) out += t("squad.analysis.economy.sources.firstReady", { seconds: num(Math.round(s.first_ready)), cost: num(s.sp_cost ?? 0), init: num(s.sp_init ?? 0) });
        else if (s.charges_on_attack) out += t("squad.analysis.economy.sources.onAttack");
        if (s.auto) out += t("squad.analysis.economy.sources.auto");
        return out;
    }
    if (s.interval !== null) return t("squad.analysis.economy.sources.interval", { amount, interval: num(s.interval) });
    return t("squad.analysis.economy.sources.flat", { amount });
}

function noteSentence(n: EconomyNote, name: string, t: EconomyT, num: (n: number) => string): string {
    const talent = n.name;
    const amount = num(n.amount ?? 0);
    switch (n.kind) {
        case "once_deployed":
            return t("squad.analysis.economy.note.once_deployed", { name, talent });
        case "first_deploy":
            return t("squad.analysis.economy.note.first_deploy", { name, talent });
        case "next_operator":
            return t("squad.analysis.economy.note.next_operator", { name, talent });
        case "conditional":
            return t("squad.analysis.economy.note.conditional", { name, talent });
        case "sp_bonus":
            return t("squad.analysis.economy.note.sp_bonus", { name, talent, amount });
        case "redeploy_cut":
            return t("squad.analysis.economy.note.redeploy_cut", { name, talent });
        case "cost_cut":
            return t("squad.analysis.economy.note.cost_cut", { name, talent });
        case "drain":
            return t("squad.analysis.economy.note.drain", { name, amount });
    }
}
