import { Sparkles } from "lucide-react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ISkillLine } from "#/lib/api/user";
import { type ITile, vacanciesOf } from "#/lib/base/board";
import { isProduction, powerOf } from "#/lib/base/catalog";
import { cn } from "#/lib/utils";
import { useBaseOptimizer } from "../base-context";
import { BaseSkill } from "./BaseSkill";
import { TileTooltip } from "./tile/components/TileTooltip";

/** The marginal chip on a skill row: what this line is worth in THIS crew. */
function LedgerChip({ line }: { line: ISkillLine }) {
    if (line.disposition === "contributes") {
        const parts = [Math.abs(line.speed_pct) > 1e-9 ? `${line.speed_pct > 0 ? "+" : ""}${trim(line.speed_pct)}%` : null, line.value_pct && Math.abs(line.value_pct) > 1e-9 ? `${line.value_pct > 0 ? "+" : ""}${trim(line.value_pct)}% value` : null].filter(Boolean);
        return <span className="shrink-0 font-mono font-semibold text-[10px] text-foreground tabular-nums">{parts.join(" · ")}</span>;
    }
    const label = {
        inactive: "inactive",
        covered: "covered",
        per_room: "per-room",
        morale: "morale",
        capacity: "capacity",
        non_production: "reception / HR",
        unmodeled: "not modeled",
    }[line.disposition];
    const hint = {
        inactive: "This skill's condition isn't met by this crew, so it adds nothing here.",
        covered: "A stronger skill of the same type is already active in this crew - the game only applies the most effective one, so this copy adds nothing on top.",
        per_room: "Active - this skill buffs matching operators in the rooms that satisfy it, so its value is counted inside those rooms' numbers. Open those rooms to see the credit.",
        morale: "This skill changes morale drain or recovery - it shows up in the sustainability simulation, not in this room's efficiency.",
        capacity: "This skill raises the room's order capacity, not its speed - it buys longer gaps between check-ins.",
        non_production: "Non-production value (clues, training, HR) - counted in its own units, never folded into the efficiency number.",
        unmodeled: "The optimizer deliberately prices this at zero rather than guessing.",
    }[line.disposition];
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
    const api = useBaseOptimizer();
    const room = api.boardRooms.find((r) => r.slot_id === tile.slotId);
    const scored = api.evaluation?.assignment.rooms.find((r) => r.slot_id === tile.slotId);

    // Spare-seat picks for the crew currently displayed: the shift's rotation cell
    // when a shift tab is active, else the optimized proposal. These operators were
    // parked for zero opportunity cost - their skills are not why they're seated.
    const shiftRoom = api.viewShift != null ? api.shiftRoom(tile.slotId) : undefined;
    const proposalRoom = api.proposal?.proposal.rooms.find((r) => r.slot_id === tile.slotId);
    const benched = new Set((shiftRoom ? shiftRoom.recommended : (proposalRoom?.operators ?? [])).filter((o) => o.bench).map((o) => o.operator_id));

    // The per-skill breakdown for whatever crew is displayed: the shift cell's
    // own ledger when a shift tab is active, else the evaluated draft's.
    const ledger = api.viewShift == null ? (scored?.ledger ?? []) : (shiftRoom?.ledger ?? []);
    const lineFor = (opId: string, buffId: string) => ledger.find((l) => l.operator_id === opId && l.buff_id === buffId && !l.from_control_center);
    const ccLines = ledger.filter((l) => l.from_control_center);

    const producesOwnOutput = scored !== undefined;
    const unstaffed = tile.seats > 0 && tile.operators.length === 0;

    const formula = room?.formula_type ? api.formulas.find((f) => f.formula_type === room.formula_type) : undefined;
    const power = room ? powerOf(room, api.catalog) : 0;
    const vacancies = vacanciesOf(tile);
    const change = api.proposal?.room_diffs.find((d) => d.slot_id === tile.slotId && d.before.join() !== d.after.join());

    return (
        <div className="flex w-76 flex-col gap-3">
            <header className="flex items-baseline justify-between gap-2">
                <h2 className="font-semibold text-[13px] text-foreground">{tile.name}</h2>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                    Lv {tile.level}/{tile.maxPhase}
                </span>
            </header>

            <dl className="flex flex-wrap gap-x-6 gap-y-2">
                {tile.seats > 0 && <Stat label="Staffed" value={`${tile.operators.length}/${tile.seats}`} />}
                {power !== 0 && <Stat label={power > 0 ? "Generates" : "Draws"} value={`${Math.abs(power)} kW`} />}
                {formula && <Stat label="Producing" value={formula.label} />}
                {scored && <Stat label="Efficiency" value={`${Math.round(scored.total_efficiency)}%`} />}
                {scored && isProduction(tile.facility ?? "") && scored.yield_lmd_per_day > 0 && <Stat label="LMD / day" value={Math.round(scored.yield_lmd_per_day).toLocaleString()} />}
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
                                {benched.has(op.id) && (
                                    <TileTooltip label={<span className="block max-w-56">Spare seat: this operator fills a free seat at the lowest opportunity cost. They were not chosen for their skills - any effect that still applies is a bonus.</span>}>
                                        <span className="shrink-0 rounded border border-border px-1 py-px text-[9px] text-muted-foreground uppercase tracking-wider">Bench</span>
                                    </TileTooltip>
                                )}
                            </div>
                            {op.skills.length > 0 && (
                                <div className={`ml-3 flex flex-col gap-1.5 border-border border-l pl-2.5 ${benched.has(op.id) ? "opacity-60" : ""}`}>
                                    {op.skills.map((skill) => {
                                        const line = lineFor(op.id, skill.buffId);
                                        // A row with no ledger line, for an operator the ledger DOES
                                        // know, is a superseded lower tier: the buff isn't in the
                                        // operator's live kit (a higher-tier skill replaced it).
                                        const replaced = !line && ledger.some((l) => l.operator_id === op.id && !l.from_control_center);
                                        return (
                                            <div className={cn("flex items-start justify-between gap-2", replaced && "opacity-40")} key={skill.buffId}>
                                                <BaseSkill skill={skill} />
                                                {line && <LedgerChip line={line} />}
                                                {replaced && <span className="shrink-0 text-[9px] text-muted-foreground uppercase tracking-wider">replaced</span>}
                                            </div>
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    ))}
                    {vacancies > 0 && <p className="text-[11px] text-muted-foreground">{vacancies === tile.seats ? "Nobody is working here." : `${vacancies} seat${vacancies === 1 ? "" : "s"} open.`}</p>}
                    {ledger.length > 0 && <p className="text-[10px] text-muted-foreground/70 leading-snug">Values are marginals - what this room loses if that one skill is removed. Coupled skills overlap, so they don&rsquo;t sum to the room total.</p>}
                    {ccLines.length > 0 && (
                        <div className="flex flex-col gap-1 rounded-md border border-border/50 bg-muted/10 px-2 py-1.5">
                            <span className="text-[9px] text-muted-foreground uppercase tracking-wider">From the Control Center</span>
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
                <p className="border-border border-t pt-2 text-[11px] text-muted-foreground">
                    {api.evaluationError ? "This layout could not be scored." : api.evaluating ? "Scoring…" : unstaffed ? "An empty room produces nothing and buffs nothing." : "Only producing rooms report an efficiency. This crew contributes through the bonuses they cast elsewhere."}
                </p>
            )}
            {room && ["TRADING", "MANUFACTURE", "POWER", "CONTROL", "MEETING", "HIRE", "DORMITORY"].includes(room.room_type) && (
                <Button className="w-full" disabled={api.optimizing} onClick={() => api.runOptimize([tile.slotId])} size="sm" variant="outline">
                    <Sparkles />
                    {api.optimizing ? "Optimizing…" : "Optimize this room only"}
                </Button>
            )}
            {change && (
                <div className="flex items-baseline justify-between gap-2 rounded-md border border-primary/30 bg-primary/5 px-2 py-1.5">
                    <span className="text-[10px] text-muted-foreground uppercase tracking-wider">Optimized</span>
                    <span className="font-mono text-[11px] tabular-nums">
                        {Math.round(change.efficiency_before)}% → <span className="font-semibold text-foreground">{Math.round(change.efficiency_after)}%</span>
                    </span>
                </div>
            )}
        </div>
    );
}
