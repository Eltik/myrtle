import type { IShift, IShiftRotation } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_META } from "../shared";
import { MoraleOverTime } from "./MoraleOverTime";
import { TEXT_MICRO } from "./parts";
import { ShiftRoomBlock } from "./ShiftCell";
import { SustainabilityBanner } from "./SustainabilityBanner";

/**
 * The three shifts side by side - one column per shift, each a stack of
 * colour-accented room blocks. Shared verbatim by the dialog (interactive) and
 * the export poster (static), so the PNG always matches what's on screen.
 */
export function ShiftGrid({ rotation, interactive = false, className }: { rotation: IShiftRotation; interactive?: boolean; className?: string }) {
    const sustained = new Set(rotation.sustained.map((o) => o.operator_id));
    return (
        <div className={cn("grid gap-2.5", className)}>
            {rotation.shifts.map((shift) => (
                <ShiftColumn key={shift.index} shift={shift} interactive={interactive} sustained={sustained} />
            ))}
        </div>
    );
}

function ShiftColumn({ shift, interactive, sustained }: { shift: IShift; interactive: boolean; sustained: Set<string> }) {
    return (
        <div className="flex flex-col gap-1.5 rounded-md border border-border/35 bg-muted/10 p-2">
            <span className="font-semibold text-[11px]">Shift {shift.index}</span>
            {shift.rooms.map((room) => (
                <ShiftRoomBlock key={room.slot_id} room={room} interactive={interactive} sustained={sustained} />
            ))}
        </div>
    );
}

/** The recommended 3-shift rotation as the dialog's primary poster: the reading
 *  legend, then the side-by-side shift grid. Green = add vs your preset, red = remove. */
export function ShiftPoster({ rotation }: { rotation: IShiftRotation }) {
    if (rotation.shifts.length === 0) return null;
    const hasSustained = rotation.sustained.length > 0;
    return (
        <div className="flex flex-col gap-2.5">
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Each production team (Team A/B/C per room pair) works a 24h block - two shifts in a row - then rests one; at every login you swap exactly one team per room group. Power plants, the Office, Reception and the Control Center alternate two squads (Squad 1 / Squad 2) - the swap you make each login.{" "}
                <span className="font-medium text-emerald-500/85">Green</span> = add to your preset; <span className="text-rose-500/85 line-through">red</span> = remove; ≈ yours = your team is close enough to keep (the small % shows what the recommended team would add).
                {hasSustained && (
                    <>
                        {" "}
                        Operators tagged <span className={cn("rounded-sm bg-amber-500/20 px-0.5 font-semibold text-amber-500/90", TEXT_MICRO)}>24/7</span> are held at full morale every shift by a morale-swap manager (Fiammetta).
                    </>
                )}
            </p>
            {rotation.sustainability && <SustainabilityBanner sim={rotation.sustainability} />}
            <ShiftGrid rotation={rotation} interactive className="grid-cols-1 sm:grid-cols-3" />
            {rotation.sustainability && <MoraleOverTime timeline={rotation.sustainability.timeline ?? []} />}
        </div>
    );
}
