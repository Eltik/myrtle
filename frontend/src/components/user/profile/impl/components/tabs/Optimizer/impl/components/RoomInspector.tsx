import { Lock, Plus, Trash2, X } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IRoomAssignment, IShiftRoom } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { isProduction, roomLabel, seatsOf } from "../layout";
import type { OptimizerApi } from "../use-optimizer";

interface RoomInspectorProps {
    api: OptimizerApi;
    onOpenAssign: () => void;
}

export function RoomInspector({ api, onOpenAssign }: RoomInspectorProps) {
    const room = api.selectedRoom;
    if (!room) {
        return <p className="text-muted-foreground text-sm">Select a room to inspect it.</p>;
    }

    const result = api.evaluation?.assignment.rooms.find((r) => r.slot_id === room.slot_id);
    const seats = seatsOf(room, api.catalog);
    const sustain = api.evaluation?.sustain ?? [];

    // While a shift is on screen the inspector must agree with the board:
    // showing the editable draft crew next to a shift's board cell would be two
    // different answers to the same question.
    const shift = api.shiftRoom(room.slot_id);
    // Only rooms the rotation actually schedules get the read-only shift view;
    // everything else keeps its editable draft crew, which is what it runs in
    // every shift anyway.
    const viewingShift = api.viewShift != null && shift !== undefined;

    return (
        <div className="flex h-full flex-col gap-4 overflow-y-auto">
            <header>
                <div className="flex items-center justify-between gap-2">
                    <h3 className="font-heading font-semibold text-base text-foreground">{roomLabel(room.room_type, api.catalog)}</h3>
                    <Badge variant="outline">L{room.level}</Badge>
                </div>
                <p className="font-mono text-[10px] text-muted-foreground">{room.slot_id}</p>
            </header>

            {viewingShift ? (
                <ShiftCrew api={api} shift={shift} />
            ) : (
                <>
                    {result ? <RoomStats room={room} result={result} /> : null}

                    <LevelPicker api={api} room={room} />

                    {isProduction(room.room_type) && room.room_type === "MANUFACTURE" ? <FormulaPicker api={api} slotId={room.slot_id} current={room.formula_type ?? null} /> : null}

                    <section className="flex flex-col gap-2">
                        <div className="flex items-center justify-between">
                            <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                                Crew {room.operators.length}/{seats}
                            </h4>
                            {room.operators.length > 0 ? (
                                <Button variant="ghost" size="sm" onClick={() => api.clearRoom(room.slot_id)}>
                                    <Trash2 className="h-3.5 w-3.5" /> Clear
                                </Button>
                            ) : null}
                        </div>

                        {room.operators.length === 0 ? <p className="text-muted-foreground text-xs">Nobody stationed here.</p> : null}

                        <ul className="flex flex-col gap-1.5">
                            {room.operators.map((id) => {
                                const entry = sustain.find((s) => s.operator_id === id);
                                const locked = api.lockedIds.includes(id);
                                return (
                                    <li key={id} className="flex items-center gap-2 rounded-md border border-border bg-background p-1.5">
                                        <span className="grid h-7 w-7 shrink-0 place-items-center overflow-hidden rounded-sm bg-muted">
                                            <OperatorAvatar charId={id} name={entry?.name ?? id} />
                                        </span>
                                        <div className="min-w-0 flex-1">
                                            <p className="truncate font-medium text-foreground text-xs">{entry?.name ?? id}</p>
                                            {entry ? <p className="font-mono text-[10px] text-muted-foreground">{entry.lasts_hours == null ? "never tires" : `${entry.lasts_hours.toFixed(1)}h endurance`}</p> : null}
                                        </div>
                                        <Button variant="ghost" size="icon" aria-label={locked ? `Unpin ${entry?.name ?? id}` : `Pin ${entry?.name ?? id} to this room`} aria-pressed={locked} onClick={() => api.toggleLocked(id)}>
                                            <Lock className={cn("h-3.5 w-3.5", locked ? "text-primary" : "text-muted-foreground")} />
                                        </Button>
                                        <Button variant="ghost" size="icon" aria-label={`Remove ${entry?.name ?? id}`} onClick={() => api.unassign(room.slot_id, id)}>
                                            <X className="h-3.5 w-3.5" />
                                        </Button>
                                    </li>
                                );
                            })}
                        </ul>

                        {room.operators.length < seats ? (
                            <Button variant="outline" size="sm" onClick={onOpenAssign}>
                                <Plus className="h-3.5 w-3.5" /> Assign operator
                            </Button>
                        ) : null}
                    </section>
                </>
            )}
        </div>
    );
}

/** The viewed shift's crew for this room - read-only by design. */
function ShiftCrew({ api, shift }: { api: OptimizerApi; shift: IShiftRoom | undefined }) {
    if (!shift?.active) {
        return <p className="text-muted-foreground text-sm">This room rests during shift {api.viewShift}.</p>;
    }

    return (
        <section className="flex flex-col gap-2">
            <div className="flex items-center justify-between gap-2">
                <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">Shift {api.viewShift}</h4>
                <div className="flex items-center gap-1.5">
                    {shift.team_label ? <Badge variant="outline">{shift.team_label}</Badge> : null}
                    {shift.efficiency != null ? <span className="font-mono text-foreground text-xs">{Math.round(shift.efficiency)}%</span> : null}
                </div>
            </div>
            <ul className="flex flex-col gap-1.5">
                {shift.recommended.map((op) => (
                    <li key={op.operator_id} className="flex items-center gap-2 rounded-md border border-border bg-background p-1.5">
                        <span className="grid h-7 w-7 shrink-0 place-items-center overflow-hidden rounded-sm bg-muted">
                            <OperatorAvatar charId={op.operator_id} name={op.name} />
                        </span>
                        <p className="min-w-0 flex-1 truncate font-medium text-foreground text-xs">{op.name}</p>
                    </li>
                ))}
            </ul>
            <p className="text-[10px] text-muted-foreground">Switch to Draft to edit this room.</p>
        </section>
    );
}

function RoomStats({ room, result }: { room: OptimizerApi["layout"][number]; result: IRoomAssignment }) {
    const stats: Array<{ label: string; value: string }> = [];
    if (isProduction(room.room_type)) {
        stats.push({ label: "Efficiency", value: `${Math.round(result.total_efficiency)}%` });
        if (result.order_value !== 0) stats.push({ label: "Order value", value: `${Math.round(result.order_value)}%` });
    }
    if (result.yield_lmd_per_day > 0) stats.push({ label: "LMD/day", value: Math.round(result.yield_lmd_per_day).toLocaleString() });
    if (result.yield_exp_per_day > 0) stats.push({ label: "EXP/day", value: Math.round(result.yield_exp_per_day).toLocaleString() });
    if (result.yield_gold_per_day > 0) stats.push({ label: "Gold/day", value: Math.round(result.yield_gold_per_day).toLocaleString() });

    // Control Center crews pay out in other facilities' units - clue speed,
    // training speed, HR speed - which never fold into the LMD objective.
    for (const effect of result.non_production) {
        stats.push({ label: `${effect.room_type} speed`, value: `${Math.round(effect.value)}%` });
    }

    if (stats.length === 0) return null;

    return (
        <dl className="grid grid-cols-2 gap-2">
            {stats.map((s) => (
                <div key={s.label} className="rounded-md border border-border bg-background p-2">
                    <dt className="text-[10px] text-muted-foreground uppercase tracking-wide">{s.label}</dt>
                    <dd className="font-mono font-semibold text-foreground text-sm">{s.value}</dd>
                </div>
            ))}
        </dl>
    );
}

const FORMULAS = [
    { id: "F_GOLD", label: "Pure Gold" },
    { id: "F_EXP", label: "Battle Records" },
    { id: "F_DIAMOND", label: "Originium Shard" },
] as const;

function FormulaPicker({ api, slotId, current }: { api: OptimizerApi; slotId: string; current: string | null }) {
    return (
        <section className="flex flex-col gap-1.5">
            <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">Producing</h4>
            <div className="flex flex-wrap gap-1">
                {FORMULAS.map((f) => (
                    <button key={f.id} type="button" onClick={() => api.setFormula(slotId, f.id)} className={cn("rounded-md border px-2 py-1 text-xs transition-colors", current === f.id ? "border-primary bg-primary text-primary-foreground" : "border-border text-muted-foreground hover:text-foreground")}>
                        {f.label}
                    </button>
                ))}
            </div>
        </section>
    );
}

/**
 * Try the room at another level.
 *
 * Levels come from the facility catalogue, so the range is whatever the game
 * actually allows rather than an assumed 1-3. Changing the room TYPE is
 * deliberately not offered: which slots can host which facility is floor-plan
 * knowledge that `building_data` does not publish, and guessing it would let
 * the planner propose a base the player cannot build.
 */
function LevelPicker({ api, room }: { api: OptimizerApi; room: OptimizerApi["layout"][number] }) {
    const phases = api.catalog.get(room.room_type)?.phases ?? [];
    if (phases.length <= 1) return null;

    return (
        <section className="flex flex-col gap-1.5">
            <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">Level</h4>
            <div className="flex flex-wrap gap-1">
                {phases.map((phase) => (
                    <button
                        key={phase.level}
                        type="button"
                        onClick={() => api.setLevel(room.slot_id, phase.level)}
                        title={`${phase.max_stationed} seat${phase.max_stationed === 1 ? "" : "s"} · ${phase.electricity >= 0 ? "+" : ""}${phase.electricity} power`}
                        className={cn("rounded-md border px-2.5 py-1 font-mono text-xs transition-colors", room.level === phase.level ? "border-primary bg-primary text-primary-foreground" : "border-border text-muted-foreground hover:text-foreground")}
                    >
                        L{phase.level}
                    </button>
                ))}
            </div>
            <p className="text-[10px] text-muted-foreground">Planning only — this does not change your real base.</p>
        </section>
    );
}
