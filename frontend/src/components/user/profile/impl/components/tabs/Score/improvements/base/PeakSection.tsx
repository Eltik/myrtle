import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IRoomAssignment } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_KICKER } from "../shared";
import { MiniChip, RoomBlock, TEXT_CHIP, TEXT_TINY, YieldSummary } from "./parts";
import { roomAccent } from "./roomColors";
import { nonProductionLabel, roomYieldLabel } from "./yield";

const PRODUCTION_OR_CONTROL = new Set(["MANUFACTURE", "TRADING", "POWER", "CONTROL"]);

/**
 * The SINGLE predicate for the "economy" tag (previously two divergent copies in
 * the dialog and the export): a perception-support room - a generator reserved
 * into an Office/dormitory purely to feed the resource pool - carries no
 * efficiency of its own, so it shows its role instead of a misleading +0%. An
 * Office/Reception staffed for its own base skill DOES have a value, so it
 * keeps the percentage.
 */
function isEconomySupportRoom(room: IRoomAssignment): boolean {
    return !PRODUCTION_OR_CONTROL.has(room.room_type) && room.total_efficiency === 0;
}

/** The peak arrangement as a grid of room blocks - the best single-snapshot
 *  staffing per room. Shared by the dialog and the export's no-rotation fallback. */
export function PeakRoomGrid({ optimal, interactive = false, className }: { optimal: IBaseAssignment; interactive?: boolean; className?: string }) {
    return (
        <div className={cn("grid gap-1.5", className)}>
            {optimal.rooms.map((room) => (
                <PeakRoomBlock key={`${room.slot_id}-${room.room_type}`} room={room} interactive={interactive} />
            ))}
        </div>
    );
}

/** Dialog variant: a yield header bar above the room grid. */
export function PeakGrid({ optimal }: { optimal: IBaseAssignment }) {
    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Peak yield</span>
                <YieldSummary assignment={optimal} />
            </div>
            <PeakRoomGrid optimal={optimal} interactive className="grid-cols-1 sm:grid-cols-2" />
        </div>
    );
}

function LockedBadge({ interactive }: { interactive: boolean }) {
    if (!interactive) return <span className="text-amber-500/80">🔒</span>;
    return (
        <Tooltip>
            <TooltipTrigger render={<span className="cursor-help text-amber-500/80">🔒</span>} />
            <TooltipContent sideOffset={4}>
                <p>Fixed synergy squad - these operators depend on each other and can't be swapped without breaking the combo.</p>
            </TooltipContent>
        </Tooltip>
    );
}

function PeakRoomBlock({ room, interactive }: { room: IRoomAssignment; interactive: boolean }) {
    const a = roomAccent(room.room_type);
    const yieldLabel = roomYieldLabel(room);
    const nonProd = nonProductionLabel(room);
    const right = isEconomySupportRoom(room) ? (
        <span className={cn("text-muted-foreground/70", TEXT_TINY)}>economy</span>
    ) : (
        <span className={cn("font-mono font-semibold tabular-nums", TEXT_CHIP)} style={{ color: a.strong }}>
            +{room.total_efficiency.toFixed(0)}%
        </span>
    );
    const badge = room.locked && room.operators.length > 0 ? <LockedBadge interactive={interactive} /> : undefined;
    return (
        <RoomBlock roomType={room.room_type} formula={room.formula_type} badge={badge} right={right}>
            <div className="flex flex-wrap items-center gap-0.5">
                {room.operators.length === 0 && <span className={cn("text-muted-foreground/50", TEXT_CHIP)}>No assignment</span>}
                {room.operators.map((op) => (
                    <MiniChip key={op.operator_id} op={op} tip={interactive ? <p>{op.name}</p> : undefined} />
                ))}
            </div>
            {(room.order_value > 0 || yieldLabel || room.room_type === "CONTROL") && (
                <div className={cn("flex flex-wrap items-center gap-x-1.5 font-mono text-muted-foreground/65", TEXT_TINY)}>
                    {room.room_type === "CONTROL" && <span>global buff</span>}
                    {nonProd && <span>{nonProd}</span>}
                    {room.order_value > 0 && <span className="text-amber-500/85">+{room.order_value.toFixed(0)}% value</span>}
                    {yieldLabel && <span>{yieldLabel}</span>}
                </div>
            )}
        </RoomBlock>
    );
}
