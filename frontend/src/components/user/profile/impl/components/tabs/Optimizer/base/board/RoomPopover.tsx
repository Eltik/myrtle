import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { type ITile, vacanciesOf } from "#/lib/base/board";
import { isProduction, powerOf } from "#/lib/base/catalog";
import { useBaseOptimizer } from "../base-context";
import { BaseSkill } from "./BaseSkill";

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
                                    <span className="shrink-0 rounded border border-border px-1 py-px text-[9px] text-muted-foreground uppercase tracking-wider" title="Spare seat: this operator fills a free seat at the lowest opportunity cost. They were not chosen for their skills - any effect that still applies is a bonus.">
                                        Bench
                                    </span>
                                )}
                            </div>
                            {op.skills.length > 0 && (
                                <div className={`ml-3 flex flex-col gap-1.5 border-border border-l pl-2.5 ${benched.has(op.id) ? "opacity-60" : ""}`}>
                                    {op.skills.map((skill) => (
                                        <BaseSkill key={skill.buffId} skill={skill} />
                                    ))}
                                </div>
                            )}
                        </div>
                    ))}
                    {vacancies > 0 && <p className="text-[11px] text-muted-foreground">{vacancies === tile.seats ? "Nobody is working here." : `${vacancies} seat${vacancies === 1 ? "" : "s"} open.`}</p>}
                </section>
            )}

            {!producesOwnOutput && tile.seats > 0 && (
                <p className="border-border border-t pt-2 text-[11px] text-muted-foreground">
                    {api.evaluationError ? "This layout could not be scored." : api.evaluating ? "Scoring…" : unstaffed ? "An empty room produces nothing and buffs nothing." : "Only producing rooms report an efficiency. This crew contributes through the bonuses they cast elsewhere."}
                </p>
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
