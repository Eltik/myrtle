import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IImprovementsResponse, IRoomAssignment } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_BADGE, TEXT_BODY, TEXT_KICKER, TEXT_META } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const ROOM_LABELS: Record<string, string> = {
    MANUFACTURE: "Factory",
    TRADING: "Trading Post",
    POWER: "Power Plant",
    DORMITORY: "Dormitory",
    WORKSHOP: "Workshop",
    MEETING: "Reception",
    HIRE: "Office",
    TRAINING: "Training",
    CONTROL: "Control Center",
};

function roomLabel(t: string): string {
    return ROOM_LABELS[t] ?? t.charAt(0) + t.slice(1).toLowerCase();
}

export function BasePanel({ improvements, accent }: IProps) {
    const { base } = improvements;
    if (!base.optimal && !base.rotation) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>No base layout synced yet. Hit refresh once you're in your base.</EmptyHint>
            </div>
        );
    }

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {base.layout.length > 0 && (
                <div className="flex flex-col gap-2">
                    <SectionHeader title="Current layout" accent={accent} />
                    <div className="flex flex-wrap gap-1.5">
                        {base.layout.map((l) => (
                            <span key={l.room_type} className={cn("flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1", TEXT_BODY)}>
                                <span className="font-medium">{roomLabel(l.room_type)}</span>
                                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>×{l.count}</span>
                                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>L{l.levels.join(", L")}</span>
                            </span>
                        ))}
                    </div>
                </div>
            )}

            {base.rotation && (
                <div className="flex flex-col gap-3">
                    <SectionHeader title="Optimal 2-shift rotation" count={`+${base.rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} />
                    <p className={cn(TEXT_META, "text-muted-foreground")}>Average of shift A and shift B. This is what your base score is computed against — run operators on these rotations to hit it.</p>
                    <div className="grid gap-3 lg:grid-cols-2">
                        <ShiftBlock title="Shift A" assignment={base.rotation.shift_a} accent={accent} />
                        <ShiftBlock title="Shift B" assignment={base.rotation.shift_b} accent={accent} />
                    </div>
                </div>
            )}

            {base.optimal && (
                <div className="flex flex-col gap-3">
                    <SectionHeader title="Peak single-shift" count={`+${base.optimal.total_production_efficiency.toFixed(1)}%`} accent={accent} />
                    <p className={cn(TEXT_META, "text-muted-foreground")}>Best possible arrangement at one snapshot in time — useful for set-and-forget AFK runs.</p>
                    <div className="flex flex-col gap-2">
                        {base.optimal.rooms.map((room) => (
                            <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} accent={accent} />
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

function ShiftBlock({ title, assignment, accent }: { title: string; assignment: IBaseAssignment; accent: string }) {
    return (
        <div className="flex flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3">
            <div className="flex items-center justify-between">
                <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                    {title}
                </span>
                <Pill color={accent}>+{assignment.total_production_efficiency.toFixed(1)}%</Pill>
            </div>
            <div className="flex flex-col gap-1.5">
                {assignment.rooms.map((room) => (
                    <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} accent={accent} compact />
                ))}
            </div>
        </div>
    );
}

function RoomRow({ room, accent, compact }: { room: IRoomAssignment; accent: string; compact?: boolean }) {
    const label = roomLabel(room.room_type);
    return (
        <div className={cn("flex items-center gap-2 rounded-md border border-border/35 bg-muted/15", compact ? "px-2 py-1.5" : "px-2.5 py-2")}>
            <div className="flex min-w-28 flex-col">
                <div className="flex items-center gap-1.5">
                    <span className={cn(TEXT_BODY, "font-semibold")}>{label}</span>
                    <span className={cn("rounded-sm border border-border/40 bg-background px-1 py-0.5 text-muted-foreground", TEXT_BADGE)}>L{room.level}</span>
                </div>
                <div className={cn("flex items-center gap-1.5 text-muted-foreground", TEXT_BADGE)}>
                    <span className="opacity-65">{room.slot_id}</span>
                    {room.formula_type && (
                        <Tooltip>
                            <TooltipTrigger render={<span className="opacity-80">{room.formula_type}</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>Production formula assigned to this room.</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
            </div>
            <div className="flex flex-1 flex-wrap gap-1">
                {room.operators.length === 0 && <span className={cn(TEXT_KICKER, "text-muted-foreground/60")}>No assignment</span>}
                {room.operators.map((op) => (
                    <Tooltip key={op.operator_id}>
                        <TooltipTrigger
                            render={
                                <span className="flex items-center gap-1.5 rounded-md border border-border/40 bg-background/65 px-1.5 py-0.5">
                                    <span className="relative size-5 overflow-hidden rounded-sm">
                                        <OperatorAvatar charId={op.operator_id} name={op.name} />
                                    </span>
                                    <span className={cn(TEXT_BODY, "max-w-[11ch] truncate font-medium")}>{op.name}</span>
                                </span>
                            }
                        />
                        <TooltipContent sideOffset={4}>
                            <p>{op.name}</p>
                        </TooltipContent>
                    </Tooltip>
                ))}
            </div>
            <span className={cn("shrink-0 font-semibold", TEXT_BODY, "font-mono tabular-nums")} style={{ color: `color-mix(in oklch, ${accent} 65%, var(--foreground))` }}>
                +{room.total_efficiency.toFixed(1)}%
            </span>
        </div>
    );
}
