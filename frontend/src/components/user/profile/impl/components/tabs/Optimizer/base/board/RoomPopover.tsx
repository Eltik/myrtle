import { Sparkles } from "lucide-react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ISkillLine } from "#/lib/api/user";
import { type ITile, vacanciesOf } from "#/lib/base/board";
import { isProduction, powerOf } from "#/lib/base/catalog";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages as panelMessages } from "../BasePanel.messages";
import { useBaseOptimizer } from "../base-context";
import { BaseSkill } from "./BaseSkill";
import type { messages } from "./RoomPopover.messages";
import { TileTooltip } from "./tile/components/TileTooltip";

/** The "Optimizing…" label is shared with the panel's own button. */
type RoomT = TypedT<typeof messages & typeof panelMessages>;

/** A key in `RoomPopover.messages.ts`, resolved through one of the tables below. */
type MessageKey = keyof typeof messages & string;

// Keyed by the API's `disposition` values.
const DISPOSITION_LABEL: Record<string, MessageKey> = {
    inactive: "profile.base.room.disposition.inactive",
    covered: "profile.base.room.disposition.covered",
    per_room: "profile.base.room.disposition.per_room",
    morale: "profile.base.room.disposition.morale",
    capacity: "profile.base.room.disposition.capacity",
    non_production: "profile.base.room.disposition.non_production",
    unmodeled: "profile.base.room.disposition.unmodeled",
};

const DISPOSITION_HINT: Record<string, MessageKey> = {
    inactive: "profile.base.room.disposition.inactive.hint",
    covered: "profile.base.room.disposition.covered.hint",
    per_room: "profile.base.room.disposition.per_room.hint",
    morale: "profile.base.room.disposition.morale.hint",
    capacity: "profile.base.room.disposition.capacity.hint",
    non_production: "profile.base.room.disposition.non_production.hint",
    unmodeled: "profile.base.room.disposition.unmodeled.hint",
};

// The figure a non-producing room reports instead of an efficiency, in its own
// units: a plant its drone recovery, the Reception Room its clue search, the
// Office its HR contact speed.
const EFFICIENCY_LABEL: Record<string, MessageKey> = {
    POWER: "profile.base.room.droneRecovery",
    MEETING: "profile.base.room.clueSearch",
    HIRE: "profile.base.room.hrContact",
};

/** The marginal chip on a skill row: what this line is worth in THIS crew. */
function LedgerChip({ line }: { line: ISkillLine }) {
    const t: RoomT = useT("user");
    if (line.disposition === "contributes") {
        const parts = [Math.abs(line.speed_pct) > 1e-9 ? `${line.speed_pct > 0 ? "+" : ""}${trim(line.speed_pct)}%` : null, line.value_pct && Math.abs(line.value_pct) > 1e-9 ? t("profile.base.room.value", { pct: `${line.value_pct > 0 ? "+" : ""}${trim(line.value_pct)}` }) : null].filter(Boolean);
        const chip = <span className={cn("shrink-0 font-mono font-semibold text-[10px] text-foreground tabular-nums", line.note && "underline decoration-dotted underline-offset-2")}>{parts.join(" · ")}</span>;
        // A count skill's marginal is spread over the skills it counts (its
        // own included), so the chip explains itself rather than reading as a
        // separate flat bonus.
        return line.note ? <TileTooltip label={<span className="block max-w-56">{line.note}</span>}>{chip}</TileTooltip> : chip;
    }
    const labelKey = DISPOSITION_LABEL[line.disposition];
    const label = labelKey ? t(labelKey) : line.disposition;
    const hintKey = DISPOSITION_HINT[line.disposition];
    const hint = hintKey ? t(hintKey) : undefined;
    return (
        <TileTooltip label={<span className="block max-w-56">{hint}</span>}>
            <span className={cn("shrink-0 text-[9px] uppercase tracking-wider", line.disposition === "inactive" ? "text-muted-foreground/60 line-through" : "text-muted-foreground")}>{label}</span>
        </TileTooltip>
    );
}

function trim(v: number): string {
    return Number.isInteger(v) ? String(v) : v.toFixed(1);
}

function Stat({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex flex-col gap-0.5">
            <dt className="text-[10px] text-muted-foreground uppercase tracking-wider">{label}</dt>
            <dd className="font-mono font-semibold text-[12px] tabular-nums">{value}</dd>
        </div>
    );
}

export function RoomPopover({ tile }: { tile: ITile }) {
    const t: RoomT = useT("user");
    const f = useFormatters();
    const api = useBaseOptimizer();
    const room = api.boardRooms.find((r) => r.slot_id === tile.slotId);
    const scored = api.evaluation?.assignment.rooms.find((r) => r.slot_id === tile.slotId);

    // Spare-seat picks for the crew currently displayed: the shift's rotation cell
    // when a shift tab is active, else the optimized proposal. These operators were
    // parked for zero opportunity cost - their skills are not why they're seated.
    const shiftRoom = api.viewShift != null ? api.shiftRoom(tile.slotId) : undefined;
    const proposalRoom = api.proposal?.proposal.rooms.find((r) => r.slot_id === tile.slotId);
    const benched = new Set((shiftRoom ? shiftRoom.recommended : (proposalRoom?.operators ?? [])).filter((o) => o.bench).map((o) => o.operator_id));
    // The rotation's 24/7 pick: held at full morale by a morale-swap manager
    // (Fiammetta), so never rotated out. A plan fact, shown on every view.
    const sustained = new Set((api.rotation?.rotation.sustained ?? []).map((o) => o.operator_id));

    // The per-skill breakdown for whatever crew is displayed: the shift cell's
    // own ledger when a shift tab is active, else the evaluated draft's.
    const ledger = api.viewShift == null ? (scored?.ledger ?? []) : (shiftRoom?.ledger ?? []);
    const lineFor = (opId: string, buffId: string) => ledger.find((l) => l.operator_id === opId && l.buff_id === buffId && !l.from_control_center);
    const ccLines = ledger.filter((l) => l.from_control_center);

    // The header's efficiency must describe the SAME crew the ledger does: the
    // shift cell's own figure when a shift tab is active, else the evaluated
    // draft's. (Mixing them showed the draft's 45% over a shift crew's +95%.)
    const efficiency = api.viewShift == null ? scored?.total_efficiency : shiftRoom?.efficiency;

    const producesOwnOutput = scored !== undefined;
    const efficiencyLabelKey = EFFICIENCY_LABEL[room?.room_type ?? ""];
    const efficiencyLabel = t(efficiencyLabelKey ?? "profile.base.room.efficiency");
    const unstaffed = tile.seats > 0 && tile.operators.length === 0;

    const formula = room?.formula_type ? api.formulas.find((f) => f.formula_type === room.formula_type) : undefined;
    const power = room ? powerOf(room, api.catalog) : 0;
    const vacancies = vacanciesOf(tile);
    const change = api.proposal?.room_diffs.find((d) => d.slot_id === tile.slotId && d.before.join() !== d.after.join());

    return (
        <div className="flex w-76 flex-col gap-3">
            <header className="flex items-baseline justify-between gap-2">
                <h2 className="font-semibold text-[13px] text-foreground">{tile.name}</h2>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("profile.base.room.level", { level: tile.level, max: tile.maxPhase })}</span>
            </header>

            <dl className="flex flex-wrap gap-x-6 gap-y-2">
                {tile.seats > 0 && <Stat label={t("profile.base.room.staffed")} value={`${tile.operators.length}/${tile.seats}`} />}
                {power !== 0 && <Stat label={power > 0 ? t("profile.base.room.generates") : t("profile.base.room.draws")} value={t("profile.base.room.power", { kw: Math.abs(power) })} />}
                {formula && <Stat label={t("profile.base.room.producing")} value={formula.label} />}
                {efficiency != null && <Stat label={efficiencyLabel} value={`${efficiencyLabelKey === undefined ? "" : "+"}${Math.round(efficiency)}%`} />}
                {scored && isProduction(tile.facility ?? "") && scored.yield_lmd_per_day > 0 && <Stat label={t("profile.base.room.lmd")} value={f.number(Math.round(scored.yield_lmd_per_day))} />}
            </dl>

            {tile.seats > 0 && (
                <section className="flex flex-col gap-2.5 border-border border-t pt-2">
                    {tile.operators.map((op) => (
                        <div key={op.id} className="flex flex-col gap-1">
                            <div className="flex items-center gap-2">
                                <span className="size-6 shrink-0 overflow-hidden rounded-sm bg-muted text-center font-bold text-[9px] leading-6">
                                    <OperatorAvatar charId={op.id} name={op.name} />
                                </span>
                                <span className="min-w-0 flex-1 truncate text-[12px]">{op.name}</span>
                                {sustained.has(op.id) && (
                                    <TileTooltip label={<span className="block max-w-56">{t("profile.base.room.sustained.tooltip")}</span>}>
                                        <span className="shrink-0 rounded border border-emerald-500/50 bg-emerald-500/10 px-1 py-px font-mono text-[9px] text-emerald-500 uppercase tracking-wider">{t("profile.base.room.sustained")}</span>
                                    </TileTooltip>
                                )}
                                {benched.has(op.id) && (
                                    <TileTooltip label={<span className="block max-w-56">{t("profile.base.room.bench.tooltip")}</span>}>
                                        <span className="shrink-0 rounded border border-border px-1 py-px text-[9px] text-muted-foreground uppercase tracking-wider">{t("profile.base.room.bench")}</span>
                                    </TileTooltip>
                                )}
                            </div>
                            {op.skills.length > 0 && (
                                <div className={`ml-3 flex flex-col gap-1.5 border-border border-l pl-2.5 ${benched.has(op.id) ? "opacity-60" : ""}`}>
                                    {op.skills.map((skill) => {
                                        const line = lineFor(op.id, skill.buffId);
                                        // A superseded lower tier: unlocked, but a promotion replaced
                                        // it with a higher tier of the same slot. Read from the kit
                                        // itself, so rooms without a ledger (Reception, dorms) label
                                        // it too.
                                        const replaced = skill.unlocked && !skill.live;
                                        return (
                                            <div className={cn("flex items-start justify-between gap-2", replaced && "opacity-40")} key={skill.buffId}>
                                                <BaseSkill skill={skill} />
                                                {line && <LedgerChip line={line} />}
                                                {replaced && <span className="shrink-0 text-[9px] text-muted-foreground uppercase tracking-wider">{t("profile.base.room.replaced")}</span>}
                                            </div>
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    ))}
                    {vacancies > 0 && <p className="text-[11px] text-muted-foreground">{vacancies === tile.seats ? t("profile.base.room.nobodyWorking") : t("profile.base.room.seatsOpen", { count: vacancies })}</p>}
                    {ledger.length > 0 && <p className="text-[10px] text-muted-foreground/70 leading-snug">{t("profile.base.room.marginalNote")}</p>}
                    {ccLines.length > 0 && (
                        <div className="flex flex-col gap-1 rounded-md border border-border/50 bg-muted/10 px-2 py-1.5">
                            <span className="text-[9px] text-muted-foreground uppercase tracking-wider">{t("profile.base.room.fromControlCenter")}</span>
                            {ccLines.map((l) => (
                                <div className="flex items-baseline justify-between gap-2 text-[11px]" key={`${l.operator_id}:${l.buff_id}`}>
                                    <span className="min-w-0 truncate">
                                        {l.operator_name} · <span className="text-muted-foreground">{l.buff_name}</span>
                                    </span>
                                    <LedgerChip line={l} />
                                </div>
                            ))}
                        </div>
                    )}
                </section>
            )}

            {!producesOwnOutput && tile.seats > 0 && (
                <p className="border-border border-t pt-2 text-[11px] text-muted-foreground">{api.evaluationError ? t("profile.base.room.scoreFailed") : api.evaluating ? t("profile.base.room.scoring") : unstaffed ? t("profile.base.room.emptyRoom") : t("profile.base.room.nonProducing")}</p>
            )}
            {room && ["TRADING", "MANUFACTURE", "POWER", "CONTROL", "MEETING", "HIRE", "DORMITORY"].includes(room.room_type) && (
                <Button className="w-full" disabled={api.optimizing} onClick={() => api.runOptimize([tile.slotId])} size="sm" variant="outline">
                    <Sparkles />
                    {api.optimizing ? t("profile.base.optimizing") : t("profile.base.room.optimizeOne")}
                </Button>
            )}
            {change && (
                <div className="flex items-baseline justify-between gap-2 rounded-md border border-primary/30 bg-primary/5 px-2 py-1.5">
                    <span className="text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.room.optimized")}</span>
                    <span className="font-mono text-[11px] tabular-nums">
                        {Math.round(change.efficiency_before)}% → <span className="font-semibold text-foreground">{Math.round(change.efficiency_after)}%</span>
                    </span>
                </div>
            )}
        </div>
    );
}
