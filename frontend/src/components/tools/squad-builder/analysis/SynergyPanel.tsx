import { Calculator } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { GameText } from "#/lib/gamedata/GameText";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { Synergy } from "#/types/generated/Synergy";
import type { SynergyGroup } from "#/types/generated/SynergyGroup";
import type { SynergyItem } from "#/types/generated/SynergyItem";
import type { messages } from "./SynergyPanel.messages";
import { type IChipContext, type IPanelBaseProps, type ISeatOperator, OpChip, opName, PanelShell, Pill, SectionHeading, Segmented } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

type SynergyT = TypedT<typeof messages & typeof sharedMessages>;
type View = "source" | "receiver";

const GROUPS: readonly SynergyGroup[] = ["check", "squad", "field", "place"];

interface ISynergyPanelProps extends IPanelBaseProps {
    synergy: Synergy | undefined;
    /** Filled seats by index, support included, as the response has them. */
    seatOps: ReadonlyMap<number, ISeatOperator>;
    onOpenInDps: (seat: number) => void;
    className?: string;
}

export function SynergyPanel({ state, stale, collapsed, onToggle, synergy, seatOps, onOpenInDps, ctx, className }: ISynergyPanelProps): React.ReactElement {
    const t: SynergyT = useT("tools");
    const [view, setView] = React.useState<View>("source");
    return (
        <PanelShell title={t("squad.analysis.synergy.title")} question={t("squad.analysis.synergy.question")} state={state} stale={stale} collapsed={collapsed} onToggle={onToggle} className={className}>
            {synergy && (
                <div className="space-y-4">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <Segmented
                            label={t("squad.analysis.synergy.view")}
                            value={view}
                            options={[
                                { value: "source", label: t("squad.analysis.synergy.view.source") },
                                { value: "receiver", label: t("squad.analysis.synergy.view.receiver") },
                            ]}
                            onChange={setView}
                        />
                        <div className="flex flex-wrap gap-1.5">
                            <Pill tone={synergy.counts.check > 0 ? "accent" : "muted"}>{t("squad.analysis.synergy.count.check", { count: synergy.counts.check })}</Pill>
                            <Pill>{t("squad.analysis.synergy.count.squad", { count: synergy.counts.squad })}</Pill>
                            <Pill>{t("squad.analysis.synergy.count.field", { count: synergy.counts.field })}</Pill>
                            <Pill>{t("squad.analysis.synergy.count.place", { count: synergy.counts.place })}</Pill>
                        </div>
                    </div>
                    {view === "source" ? <BySource items={synergy.items} seatOps={seatOps} ctx={ctx} t={t} /> : <ByReceiver items={synergy.items} seatOps={seatOps} onOpenInDps={onOpenInDps} ctx={ctx} t={t} />}
                </div>
            )}
        </PanelShell>
    );
}

function BySource({ items, seatOps, ctx, t }: { items: SynergyItem[]; seatOps: ReadonlyMap<number, ISeatOperator>; ctx: IChipContext; t: SynergyT }): React.ReactElement {
    return (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            {GROUPS.map((g) => {
                const list = items.filter((i) => i.group === g);
                const head = sectionText(g, t);
                return (
                    <section key={g} className="min-w-0">
                        <SectionHeading title={head.title} hint={head.hint} />
                        {list.length === 0 ? (
                            <p className="text-[13px] text-muted-foreground">{head.empty}</p>
                        ) : (
                            <ul className="space-y-2">
                                {list.map((item) => (
                                    <SourceItem key={itemKey(item)} item={item} seatOps={seatOps} ctx={ctx} t={t} />
                                ))}
                            </ul>
                        )}
                    </section>
                );
            })}
        </div>
    );
}

function SourceItem({ item, seatOps, ctx, t }: { item: SynergyItem; seatOps: ReadonlyMap<number, ISeatOperator>; ctx: IChipContext; t: SynergyT }): React.ReactElement {
    const lead = leadSentence(item, t);
    const receivers = item.to.map((seat) => ({ seat, op: seatOps.get(seat) })).filter((r): r is { seat: number; op: ISeatOperator } => r.op !== undefined);
    const othersCount = [...seatOps.keys()].filter((s) => s !== item.seat).length;
    let reaches: React.ReactNode = null;
    if (item.cond === "near" || item.group === "place") reaches = <span className="text-[12px] text-foreground">{t("squad.analysis.synergy.reaches.near")}</span>;
    else if (item.target.kind === "next") reaches = <span className="text-[12px] text-foreground">{t("squad.analysis.synergy.reaches.next")}</span>;
    else if (item.to_all) reaches = <span className="text-[12px] text-foreground">{t("squad.analysis.synergy.reaches.all", { count: othersCount })}</span>;
    else if (receivers.length > 0) reaches = receivers.map((r) => <OpChip key={r.seat} charId={r.op.charId} seat={r.seat} support={r.op.support} ctx={ctx} why={item.name} />);

    return (
        <li className="rounded-xl border border-border bg-card px-3 py-2.5">
            <div className="flex flex-wrap items-center gap-1.5">
                <OpChip charId={item.char_id} seat={item.seat} support={item.support} ctx={ctx} why={item.name} />
                <span className="font-medium text-[13px] text-foreground">{item.name}</span>
                <Pill tone={item.reason === "nobody" ? "accent" : "muted"}>{groupPill(item, t)}</Pill>
            </div>
            {lead && <p className="mt-1.5 text-[12.5px] text-foreground leading-snug">{lead}</p>}
            <p className="mt-1 text-[12px] text-muted-foreground leading-relaxed">
                <GameText text={item.text} palette="game" />
            </p>
            {reaches && (
                <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
                    <span className="text-[11px] text-muted-foreground">{t("squad.analysis.synergy.reaches")}</span>
                    {reaches}
                </div>
            )}
            {item.highest_only && <p className="mt-1 text-[11px] text-muted-foreground italic">{t("squad.analysis.synergy.highest")}</p>}
        </li>
    );
}

/** The items that reach a seat, placement-bound ones left to the footer. */
function itemsReaching(items: ReadonlyArray<SynergyItem>, seat: number): SynergyItem[] {
    return items.filter((i) => i.group !== "place" && i.cond !== "near" && i.seat !== seat && (i.to.includes(seat) || i.to_all));
}

/** One talent can carry several effects, so the text is part of the key. */
function itemKey(item: SynergyItem): string {
    return `${item.seat}|${item.source.kind}|${item.name}|${item.cond}|${item.text}`;
}

function ByReceiver({ items, seatOps, onOpenInDps, ctx, t }: { items: SynergyItem[]; seatOps: ReadonlyMap<number, ISeatOperator>; onOpenInDps: (seat: number) => void; ctx: IChipContext; t: SynergyT }): React.ReactElement {
    const placed = items.filter((i) => i.group === "place" || i.cond === "near");
    const seats = [...seatOps.entries()].sort((a, b) => a[0] - b[0]);
    return (
        <div className="space-y-3">
            <ul className="divide-y divide-border/60 rounded-xl border border-border bg-card">
                {seats.map(([seat, op]) => {
                    const got = itemsReaching(items, seat);
                    return (
                        <li key={seat} className="flex flex-col gap-2 px-3 py-2 sm:flex-row sm:items-center sm:gap-3">
                            <div className="sm:w-40 sm:shrink-0">
                                <OpChip charId={op.charId} seat={seat} support={op.support} ctx={ctx} />
                            </div>
                            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
                                {got.length === 0 ? (
                                    <span className="text-[12px] text-muted-foreground">{t("squad.analysis.synergy.receiver.none")}</span>
                                ) : (
                                    got.map((i) => (
                                        <Pill key={itemKey(i)} tone="muted" className="text-foreground">
                                            {t("squad.analysis.synergy.receiver.giver", { giver: opName(ctx.operators, i.char_id), talent: i.name })}
                                            <span className="font-normal text-[10px] text-muted-foreground">{receiverSuffix(i, t)}</span>
                                        </Pill>
                                    ))
                                )}
                            </div>
                            <Button variant="outline" size="sm" className="self-start sm:self-center" onClick={() => onOpenInDps(seat)}>
                                <Calculator />
                                {t("squad.analysis.synergy.openDps")}
                            </Button>
                        </li>
                    );
                })}
            </ul>
            {placed.length > 0 && (
                <div className="space-y-0.5 text-[12px] text-muted-foreground">
                    {placed.map((i) => (
                        <p key={itemKey(i)}>{t("squad.analysis.synergy.receiver.place", { giver: opName(ctx.operators, i.char_id), talent: i.name })}</p>
                    ))}
                </div>
            )}
        </div>
    );
}

function sectionText(g: SynergyGroup, t: SynergyT): { title: string; hint: string; empty: string } {
    switch (g) {
        case "check":
            return { title: t("squad.analysis.synergy.section.check"), hint: t("squad.analysis.synergy.section.check.hint"), empty: t("squad.analysis.synergy.empty.check") };
        case "squad":
            return { title: t("squad.analysis.synergy.section.squad"), hint: t("squad.analysis.synergy.section.squad.hint"), empty: t("squad.analysis.synergy.empty.squad") };
        case "field":
            return { title: t("squad.analysis.synergy.section.field"), hint: t("squad.analysis.synergy.section.field.hint"), empty: t("squad.analysis.synergy.empty.field") };
        case "place":
            return { title: t("squad.analysis.synergy.section.place"), hint: t("squad.analysis.synergy.section.place.hint"), empty: t("squad.analysis.synergy.empty.place") };
    }
}

function groupPill(item: SynergyItem, t: SynergyT): string {
    if (item.reason === "nobody") return t("squad.analysis.synergy.pill.nobody");
    if (item.reason === "unread") return t("squad.analysis.synergy.pill.unread");
    switch (item.cond) {
        case "squad":
            return t("squad.analysis.synergy.pill.squad");
        case "field":
            return t("squad.analysis.synergy.pill.field");
        case "near":
            return t("squad.analysis.synergy.pill.near");
        case "skill":
            return t("squad.analysis.synergy.pill.skill");
    }
}

/**
 * The target in words. Kinds the server's own tables name (class, faction,
 * recruitment tag, position) arrive in `label` in the server's language; the
 * rest are worded here from `kind` and `value`, so no English talent wording
 * reaches a reader on another locale. `unknown` keeps the talent's words.
 */
function targetLabel(item: SynergyItem, t: SynergyT): string {
    const { kind, value, label } = item.target;
    switch (kind) {
        case "everyone":
            return t("squad.analysis.synergy.target.everyone");
        case "next":
            return t("squad.analysis.synergy.target.next");
        case "cost_at_most":
            return t("squad.analysis.synergy.target.costAtMost", { cost: value ?? "" });
        case "sp_type":
            if (value === "INCREASE_WHEN_ATTACK") return t("squad.analysis.synergy.target.spOffensive");
            if (value === "INCREASE_WHEN_TAKEN_DAMAGE") return t("squad.analysis.synergy.target.spDefensive");
            if (value === "INCREASE_WITH_TIME") return t("squad.analysis.synergy.target.spAuto");
            return label;
        default:
            return label;
    }
}

function leadSentence(item: SynergyItem, t: SynergyT): string | null {
    const label = targetLabel(item, t);
    switch (item.reason) {
        case "nobody":
            return t("squad.analysis.synergy.lead.nobody", { label });
        case "unknown":
            return t("squad.analysis.synergy.lead.unknown", { label });
        case "needs_count": {
            const need = item.need ?? { count: 0, have: 0 };
            return t("squad.analysis.synergy.lead.needsCount", { count: need.count, label, missing: Math.max(0, need.count - need.have) });
        }
        case "unseen":
            return t("squad.analysis.synergy.lead.unseen");
        case "unread":
            return t("squad.analysis.synergy.lead.unread");
        case null:
            return null;
    }
}

function receiverSuffix(item: SynergyItem, t: SynergyT): string {
    switch (item.cond) {
        case "field":
            return t("squad.analysis.synergy.receiver.field");
        case "skill":
            return t("squad.analysis.synergy.receiver.skill");
        default:
            return t("squad.analysis.synergy.receiver.squad");
    }
}
