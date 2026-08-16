import { Users, Zap } from "lucide-react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IRoomAssignment } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { groupIntoBands, isProduction, ROOM_ACCENT, roomLabel, seatsOf } from "../layout";
import type { OptimizerApi } from "../use-optimizer";

interface BaseBoardProps {
    api: OptimizerApi;
    /** Slots the pending proposal would change, highlighted for review. */
    proposedSlotIds: Set<string>;
}

export function BaseBoard({ api, proposedSlotIds }: BaseBoardProps) {
    const bands = groupIntoBands(api.layout);
    const bySlot = new Map<string, IRoomAssignment>((api.evaluation?.assignment.rooms ?? []).map((r) => [r.slot_id, r]));

    if (api.layout.length === 0) {
        return (
            <div className="rounded-xl border border-border border-dashed bg-card p-8 text-center">
                <p className="text-muted-foreground text-sm">No base data for this profile yet.</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-3">
            {bands.map(({ band, rooms }) => (
                <section key={band.id} className="rounded-xl border border-border bg-card p-3">
                    <h3 className="mb-2 font-mono text-[10px] text-muted-foreground uppercase tracking-widest">{band.label}</h3>
                    <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-2">
                        {rooms.map((room) => (
                            <RoomCell key={room.slot_id} api={api} room={room} result={bySlot.get(room.slot_id)} selected={api.selectedSlotId === room.slot_id} proposed={proposedSlotIds.has(room.slot_id)} />
                        ))}
                    </div>
                </section>
            ))}
        </div>
    );
}

interface RoomCellProps {
    api: OptimizerApi;
    room: OptimizerApi["layout"][number];
    result: IRoomAssignment | undefined;
    selected: boolean;
    proposed: boolean;
}

function RoomCell({ api, room, result, selected, proposed }: RoomCellProps) {
    const seats = seatsOf(room, api.catalog);
    const accent = ROOM_ACCENT[room.room_type];
    const label = roomLabel(room.room_type, api.catalog);
    const yieldPerDay = (result?.yield_lmd_per_day ?? 0) + (result?.yield_exp_per_day ?? 0);
    const power = api.catalog.get(room.room_type)?.phases[room.level - 1]?.electricity ?? 0;

    // Viewing a shift swaps the crew for that shift's, so the board answers
    // "who works here at 20:00" without leaving the board.
    const shift = api.shiftRoom(room.slot_id);
    // A room the rotation never schedules (Workshop, Training Room, dormitories)
    // is not "resting" - it is simply outside the rotation, and its draft crew
    // stands in every shift. Only a room the rotation DOES cover can be dark by
    // decision, and that distinction is the difference between "the solver chose
    // this" and "the solver never considered it".
    const scheduled = api.viewShift != null && shift !== undefined;
    const resting = scheduled && !shift.active;
    const crew = scheduled ? shift.recommended.map((o) => o.operator_id) : room.operators;
    const efficiency = scheduled ? shift.efficiency : result?.total_efficiency;

    return (
        <button
            type="button"
            onClick={() => api.setSelectedSlotId(room.slot_id)}
            title={`${label} · ${room.slot_id}`}
            style={{ borderLeftColor: accent }}
            className={cn("flex flex-col justify-between gap-1.5 rounded-md border border-border border-l-4 bg-background p-2 text-left transition-colors", "hover:border-primary/50", selected && "ring-2 ring-primary", proposed && !selected && "ring-2 ring-[var(--rarity-6)]", resting && "opacity-60")}
        >
            <div className="flex items-start justify-between gap-1">
                <span className="truncate font-semibold text-[11px] text-foreground leading-tight">{label}</span>
                <span className="shrink-0 rounded-sm bg-muted px-1 font-mono text-[9px] text-muted-foreground">{scheduled && shift.team_label ? shift.team_label : `L${room.level}`}</span>
            </div>

            {resting ? (
                // A dark room is a decision, not a hole: the solver would rather
                // rest it than burn a benchwarmer's morale on it.
                <span className="flex min-h-6 items-center text-[10px] text-muted-foreground italic">rests this shift</span>
            ) : (
                <div className="flex min-h-6 flex-wrap items-center gap-1">
                    {crew.map((id) => (
                        <span key={id} className="grid h-6 w-6 place-items-center overflow-hidden rounded-sm bg-muted">
                            <OperatorAvatar charId={id} name={id} />
                        </span>
                    ))}
                    {seats > crew.length ? <span className="text-[9px] text-muted-foreground">+{seats - crew.length}</span> : null}
                </div>
            )}

            <div className="flex items-center justify-between text-[9px] text-muted-foreground">
                <span className="inline-flex items-center gap-0.5">
                    <Users className="h-2.5 w-2.5" />
                    {crew.length}/{seats}
                </span>
                {isProduction(room.room_type) && efficiency != null ? (
                    <span className="inline-flex items-center gap-0.5 font-mono text-foreground">
                        <Zap className="h-2.5 w-2.5" />
                        {Math.round(efficiency)}%
                    </span>
                ) : scheduled ? null : yieldPerDay > 0 ? (
                    <span className="font-mono text-foreground">{Math.round(yieldPerDay).toLocaleString()}/d</span>
                ) : power !== 0 ? (
                    // Support rooms have no yield to show, so surface the one
                    // number that always differs between them: power.
                    <span className={cn("font-mono", power > 0 ? "text-emerald-500" : "text-muted-foreground")}>
                        {power > 0 ? "+" : ""}
                        {power}
                    </span>
                ) : null}
            </div>
        </button>
    );
}
