import type { IRoomRotation, IRotation, IRotationSet } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_BADGE, TEXT_KICKER, TEXT_META } from "../shared";
import { AccentKicker, MiniChip, RoomBlock, TEXT_CHIP, TEXT_TINY } from "./parts";

/** The staggered 24/7 rotation: per-room swap order, the shared bench, and the
 *  overlapping sets to cycle through. */
export function RotationSection({ rotation }: { rotation: IRotation }) {
    return (
        <>
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Staggered rotation: keep the main team working, and when you log in swap only the <span className="font-medium">⚡ first operator</span> in each room (the one whose morale runs low soonest) for its backup - never a whole team at once. Times are roughly how long each operator works before it needs rest.
            </p>
            <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                {rotation.rooms.map((room) => (
                    <RotationRoomBlock key={room.slot_id} room={room} />
                ))}
            </div>
            {(rotation.shared_bench?.length ?? 0) > 0 && (
                <div className="flex flex-wrap items-center gap-1.5">
                    <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Shared bench</span>
                    <div className="flex flex-wrap gap-1">
                        {rotation.shared_bench?.map((op) => (
                            <MiniChip key={op.operator_id} op={op} dashed tip={<p>{op.name} is versatile enough to back up several rooms - one bench seat covers them all.</p>} />
                        ))}
                    </div>
                </div>
            )}
            <RotationSets sets={rotation.sets} />
        </>
    );
}

/** One production room's rotation plan as a room block: who to swap first (⚡), how long
 *  each lasts, and the dashed backup. */
function RotationRoomBlock({ room }: { room: IRoomRotation }) {
    return (
        <RoomBlock roomType={room.room_type}>
            <div className="flex flex-wrap items-center gap-0.5">
                {room.members.map((m, i) => (
                    <MiniChip
                        key={m.operator.operator_id}
                        op={m.operator}
                        lead={i === 0 ? <span className={cn("text-amber-500/90", TEXT_CHIP)}>⚡</span> : undefined}
                        suffix={<span className={cn("font-mono text-muted-foreground/65 tabular-nums", TEXT_TINY)}>{m.lasts_hours === null ? "∞" : `~${m.lasts_hours.toFixed(0)}h`}</span>}
                        tip={
                            <p>
                                {i === 0 ? "Swap this one first - " : ""}
                                {m.operator.name}
                                {m.lasts_hours === null ? " drains no morale here - they can work indefinitely." : ` works ~${m.lasts_hours.toFixed(0)}h before its morale runs low.`}
                            </p>
                        }
                    />
                ))}
                {room.backup && (
                    <>
                        <span className={cn("px-0.5", TEXT_CHIP)} style={{ color: "color-mix(in oklch, var(--imp-accent) 55%, var(--muted-foreground))" }}>
                            ⇄
                        </span>
                        <MiniChip op={room.backup} dashed suffix={<span className={cn("font-mono text-muted-foreground/55", TEXT_TINY)}>backup</span>} tip={<p>Rotate {room.backup.name} in when the ⚡ first operator needs rest.</p>} />
                    </>
                )}
            </div>
        </RoomBlock>
    );
}

/** The rotation as a few overlapping staffings to cycle through, so you never swap
 * the whole base at once. Each set rests one operator per room (covered by a backup)
 * and consecutive sets share all but one operator. */
function RotationSets({ sets }: { sets: IRotationSet[] | undefined }) {
    if (!sets || sets.length < 2) return null;
    return (
        <div className="mt-1 flex flex-col gap-2 border-border/30 border-t pt-3">
            <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Overlapping sets to cycle through</span>
            <p className={cn(TEXT_META, "text-muted-foreground")}>Cycle through these {sets.length} sets one swap at a time - each rests only the operators that actually run low on morale (covered by a backup). Consecutive sets share all but one operator per room.</p>
            <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                {sets.map((set, i) => (
                    <RotationSetRow key={set.rooms.map((r) => r.resting?.operator_id ?? r.slot_id).join("-")} set={set} index={i} />
                ))}
            </div>
        </div>
    );
}

function RotationSetRow({ set, index }: { set: IRotationSet; index: number }) {
    const resting = set.rooms.map((r) => r.resting?.name).filter((n): n is string => Boolean(n));
    return (
        <div className="flex flex-col gap-1 rounded-md border border-border/35 bg-muted/15 px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
                <AccentKicker>Set {index + 1}</AccentKicker>
                {resting.length > 0 && <span className={cn(TEXT_BADGE, "truncate text-muted-foreground/60")}>resting: {resting.join(", ")}</span>}
            </div>
            <div className="flex flex-col gap-1">
                {set.rooms.map((r) => (
                    <RoomBlock key={r.slot_id} roomType={r.room_type}>
                        <div className="flex flex-wrap items-center gap-0.5">
                            {r.working.map((op) => (
                                <MiniChip key={op.operator_id} op={op} />
                            ))}
                        </div>
                    </RoomBlock>
                ))}
            </div>
        </div>
    );
}
