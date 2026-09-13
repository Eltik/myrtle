import { TileHint } from "frontend";
import type { ReactNode } from "react";

/*
 * TileHint is the body of a room tile's hover tooltip: the room's label
 * (`tileLabel` - the facility name, or "Empty" for an unassigned slot) over a
 * Level row. It is shown inside `TileTooltip`'s popup, so each story wraps it
 * in that popup's surface.
 */
type Facility = "TRADING" | "MANUFACTURE" | "POWER" | "DORMITORY" | "CONTROL" | "MEETING";
const NAMES: Record<Facility, string> = { TRADING: "Trading Post", MANUFACTURE: "Factory", POWER: "Power Plant", DORMITORY: "Dormitory", CONTROL: "Control Center", MEETING: "Reception Room" };

const tile = (facility: Facility | null, level: number, maxPhase: number) => ({
    slotId: "slot_5",
    kind: "flexible" as const,
    facility,
    name: facility ? NAMES[facility] : "",
    level,
    maxPhase,
    built: facility !== null,
    operators: [],
    seats: 0,
    col: 1,
    row: 1,
    w: 2,
    h: 1,
});

/** The `TooltipPopup` surface TileTooltip mounts the hint in (`px-2 py-1`, 12px popover text). */
const Popup = ({ children }: { children: ReactNode }) => <div className="w-fit rounded-md border bg-popover px-2 py-1 text-popover-foreground text-xs shadow-md/5">{children}</div>;

/** The hint for a maxed trading post. */
export const RoomHint = () => (
    <Popup>
        <TileHint tile={tile("TRADING", 3, 3)} />
    </Popup>
);

/** Across rooms: the label is the facility name and the level column is mono and right-aligned; an unassigned slot reads "Empty". */
export const Rooms = () => (
    <div className="flex flex-wrap items-start gap-3">
        <Popup>
            <TileHint tile={tile("MANUFACTURE", 1, 3)} />
        </Popup>
        <Popup>
            <TileHint tile={tile("DORMITORY", 5, 5)} />
        </Popup>
        <Popup>
            <TileHint tile={tile("CONTROL", 5, 5)} />
        </Popup>
        <Popup>
            <TileHint tile={tile("MEETING", 2, 3)} />
        </Popup>
        <Popup>
            <TileHint tile={tile(null, 0, 0)} />
        </Popup>
    </div>
);
