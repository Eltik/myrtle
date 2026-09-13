import { RoomTile } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/*
 * Fixtures mirror `buildBoard()` in `src/lib/base/board.ts`: one ITile per
 * floorplan slot, joined to the player's draft room. Room definitions are the
 * ones `GET /api/base/catalog` ships (name, phase count, seats per level).
 */
type Kind = "fixed" | "flexible" | "path" | "elevator";
type Facility = "TRADING" | "MANUFACTURE" | "POWER" | "DORMITORY" | "CONTROL" | "MEETING" | "HIRE" | "TRAINING" | "WORKSHOP";
interface ICrew {
    id: string;
    name: string;
    skills: never[];
    change?: "added" | "removed";
}
interface ITile {
    slotId: string;
    kind: Kind;
    facility: Facility | null;
    name: string;
    level: number;
    maxPhase: number;
    built: boolean;
    operators: ICrew[];
    seats: number;
    col: number;
    row: number;
    w: number;
    h: number;
}

const ROOMS: Record<Facility, { name: string; seats: number[] }> = {
    TRADING: { name: "Trading Post", seats: [1, 2, 3] },
    MANUFACTURE: { name: "Factory", seats: [1, 2, 3] },
    POWER: { name: "Power Plant", seats: [1, 1, 1] },
    DORMITORY: { name: "Dormitory", seats: [5, 5, 5, 5, 5] },
    CONTROL: { name: "Control Center", seats: [1, 2, 3, 4, 5] },
    MEETING: { name: "Reception Room", seats: [2, 2, 2] },
    HIRE: { name: "Office", seats: [1, 1, 1] },
    TRAINING: { name: "Training Room", seats: [2, 2, 2] },
    WORKSHOP: { name: "Workshop", seats: [1, 1, 1] },
};

// Output rooms (trading / factory / power) share nine slots between thirteen
// buildable rooms, so the board marks them `flexible`; everything else is `fixed`.
const kindOf = (facility: Facility | null): Kind => (facility === "TRADING" || facility === "MANUFACTURE" || facility === "POWER" ? "flexible" : "fixed");

const op = (id: string, name: string, change?: "added" | "removed"): ICrew => ({ id, name, skills: [], change });

const JAYE = op("char_272_strong", "Jaye");
const TEXAS2 = op("char_1028_texas2", "Texas the Omertosa");
const LAPPLAND = op("char_140_whitew", "Lappland");
const PROVISO = op("char_4032_provs", "Proviso");
const VERMEIL = op("char_190_clour", "Vermeil");
const GUMMY = op("char_196_sunbr", "Gummy");
const PINECONE = op("char_440_pinecn", "Pinecone");
const GREYY = op("char_253_greyy", "Greyy");
const ANSEL = op("char_212_ansel", "Ansel");
const HIBISCUS = op("char_120_hibisc", "Hibiscus");
const ORCHID = op("char_278_orchid", "Orchid");
const PERFUMER = op("char_181_flower", "Perfumer");
const GRAVEL = op("char_237_gravel", "Gravel");
const GNOSIS = op("char_206_gnosis", "Gnosis");
const DOBERMANN = op("char_130_doberm", "Dobermann");
const CHEN = op("char_010_chen", "Ch'en");

interface IRoomSpec {
    slotId: string;
    facility: Facility;
    level: number;
    operators?: ICrew[];
    col: number;
    /** Track span: output rooms are 2 tracks, dorms 3. */
    w?: number;
    built?: boolean;
}

/** A built room: level, seats and crew as the draft has them. */
function room({ slotId, facility, level, operators = [], col, w = 2, built = true }: IRoomSpec): ITile {
    const def = ROOMS[facility];
    return {
        slotId,
        kind: kindOf(facility),
        facility,
        name: def.name,
        level: built ? level : 0,
        maxPhase: def.seats.length,
        built,
        operators: built ? operators : [],
        seats: built ? def.seats[level - 1] : 0,
        col,
        row: 1,
        w,
        h: 1,
    };
}

/** A fixed slot with no room assigned - the board labels it "Empty". */
function emptySlot(slotId: string, col: number, w = 2): ITile {
    return { slotId, kind: "fixed", facility: null, name: "", level: 0, maxPhase: 0, built: false, operators: [], seats: 0, col, row: 1, w, h: 1 };
}

/** The board's stage: `Board.module.css`'s dark surface and half-tile grid, at the desktop unit. */
const Stage = ({ tracks, children }: { tracks: number; children: ReactNode }) => (
    <div style={{ width: "fit-content", borderRadius: 6, background: "#191919", padding: "22px 16px" }}>
        <div style={{ "--riic-unit": "32px", display: "grid", gap: 3, width: "max-content", gridTemplateColumns: Array.from({ length: tracks }, () => "calc(2 * var(--riic-unit))").join(" "), gridAutoRows: "calc(2 * var(--riic-unit))" } as CSSProperties}>{children}</div>
    </div>
);

const Caption = ({ children }: { children: ReactNode }) => <p className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.1em]">{children}</p>;

/** The three output rooms at max level, fully staffed - each carries its own accent bar and pip colour. */
export const OutputRooms = () => (
    <Stage tracks={6}>
        <RoomTile tile={room({ slotId: "slot_5", facility: "TRADING", level: 3, operators: [JAYE, TEXAS2, LAPPLAND], col: 1 })} />
        <RoomTile tile={room({ slotId: "slot_6", facility: "MANUFACTURE", level: 3, operators: [VERMEIL, GUMMY, PINECONE], col: 3 })} />
        <RoomTile tile={room({ slotId: "slot_7", facility: "POWER", level: 3, operators: [GREYY], col: 5 })} />
    </Stage>
);

/** Level 1 -> 3: the pips fill and a seat opens per level, so the crew strip grows with the room. */
export const LevelSweep = () => (
    <div className="flex flex-col gap-2">
        <Stage tracks={6}>
            <RoomTile tile={room({ slotId: "slot_24", facility: "MANUFACTURE", level: 1, operators: [VERMEIL], col: 1 })} />
            <RoomTile tile={room({ slotId: "slot_25", facility: "MANUFACTURE", level: 2, operators: [VERMEIL, GUMMY], col: 3 })} />
            <RoomTile tile={room({ slotId: "slot_26", facility: "MANUFACTURE", level: 3, operators: [VERMEIL, GUMMY, PINECONE], col: 5 })} />
        </Stage>
        <Caption>Factory · level 1, 2, 3</Caption>
    </div>
);

/** Fixed slots (dorm, reception, office) get no accent bar; their pips light plain white. A dorm spans three tracks. */
export const FixedRooms = () => (
    <Stage tracks={7}>
        <RoomTile tile={room({ slotId: "slot_9", facility: "DORMITORY", level: 5, operators: [ANSEL, HIBISCUS, ORCHID, PERFUMER, GRAVEL], col: 1, w: 3 })} />
        <RoomTile tile={room({ slotId: "slot_13", facility: "MEETING", level: 3, operators: [GNOSIS, DOBERMANN], col: 4 })} />
        <RoomTile tile={room({ slotId: "slot_23", facility: "HIRE", level: 3, operators: [CHEN], col: 6 })} />
    </Stage>
);

/** Vacancies draw as dashed seat outlines after the crew: a factory with one of three filled, a dorm with two beds free. */
export const OpenSeats = () => (
    <Stage tracks={5}>
        <RoomTile tile={room({ slotId: "slot_15", facility: "MANUFACTURE", level: 3, operators: [PINECONE], col: 1 })} />
        <RoomTile tile={room({ slotId: "slot_20", facility: "DORMITORY", level: 4, operators: [ANSEL, HIBISCUS, ORCHID], col: 3, w: 3 })} />
    </Stage>
);

/** A shift boundary (`crewsForShift`): the incoming crew first, `change: "added"` ringed in the room's accent, then the outgoing `change: "removed"` chips greyed after them. */
export const ShiftChange = () => (
    <div className="flex flex-col gap-2">
        <Stage tracks={4}>
            <RoomTile tile={room({ slotId: "slot_14", facility: "TRADING", level: 3, operators: [TEXAS2, LAPPLAND, op(PROVISO.id, PROVISO.name, "added"), op(JAYE.id, JAYE.name, "removed")], col: 1 })} />
            <RoomTile tile={room({ slotId: "slot_16", facility: "POWER", level: 3, operators: [op("char_277_sqrrel", "Shaw", "added"), op(GREYY.id, GREYY.name, "removed")], col: 3 })} />
        </Stage>
        <Caption>Jaye and Greyy leave · Proviso and Shaw join</Caption>
    </div>
);

/** Unbuilt slots are dashed and inert (no popover); a fixed slot with nothing assigned reads "Empty". */
export const NotBuiltAndEmpty = () => (
    <Stage tracks={7}>
        <RoomTile tile={room({ slotId: "slot_16", facility: "TRADING", level: 1, col: 1, built: false })} />
        <RoomTile tile={emptySlot("slot_32", 3)} />
        <RoomTile tile={room({ slotId: "slot_3", facility: "DORMITORY", level: 1, col: 5, w: 3, built: false })} />
    </Stage>
);
